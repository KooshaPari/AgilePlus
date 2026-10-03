// SPDX-License-Identifier: MIT OR Apache-2.0
//! End-to-end flow tests for the `dag` command (`run_dag`).
//!
//! Like the other flow suites this links the crate's rlib, so it exercises
//! the production compilation units the coverage basis reports over.
//!
//! `dag` has no storage or VCS port: `main.rs` routes the `Dag` arm straight
//! to `run_dag`, which keeps claims and work packages in a process-wide
//! in-memory `AppState`. Behavior is asserted at two levels — in-process
//! `run_dag` calls drive real state transitions (claim exclusivity, release,
//! add → done) through the `Result` surface like `queue_command_flow.rs`;
//! spawned `agileplus dag ...` invocations assert clap validation and the
//! rendered stdout contract against real dedup files and directory trees.

use std::process::Output;

use tempfile::TempDir;

mod support;
use support::dag::{
    add, claim, claim_kind, cli, dedup_file, done, make_git_repo, release, run, unique,
    write_dedup_file,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Spawn `agileplus <args>` and capture output; state is process-local.
fn cli_run(args: &[&str]) -> Output {
    cli()
        .args(args)
        .output()
        .expect("agileplus binary should start")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

// ── In-process state transitions ────────────────────────────────────────────

#[test]
fn claim_is_exclusive_until_released_and_idempotent_for_one_id() {
    let res = unique("res-excl");

    // First holder takes the resource; the default claim id is `<agent>:<res>`.
    run(claim("agent-a", &res)).expect("first claim succeeds");
    // Re-claiming under the SAME id is an idempotent refresh, not an error:
    // a keeper re-issuing its own claim must not be treated as a conflict.
    run(claim("agent-a", &res)).expect("same-id re-claim refreshes");

    // A different claim id on the same resource is refused while active.
    let err = run(claim("agent-b", &res)).expect_err("resource is already held");
    let msg = err.to_string();
    assert!(msg.contains("resource already claimed"), "got: {msg}");

    // Releasing the holder frees the resource for a different agent.
    let holder = format!("agent-a:{res}");
    run(release(&holder)).expect("release succeeds");
    run(claim("agent-b", &res)).expect("resource free after release");
}

#[test]
fn claim_validates_kind_and_accepts_the_four_supported_kinds() {
    let res = unique("res-kind");

    // Unknown kind: rejected up front with the offending value in the message.
    let err = run(claim_kind("agent-a", &res, "vehicle")).expect_err("unknown kind must fail");
    let msg = err.to_string();
    assert!(msg.contains("unknown claim kind: vehicle"), "got: {msg}");

    // The four clap-supported kinds, each on its own resource.
    for kind in ["repo", "branch", "worktree", "subproject"] {
        let r = format!("{res}-{kind}");
        run(claim_kind("agent-a", &r, kind))
            .unwrap_or_else(|e| panic!("--kind {kind} must be accepted, got: {e}"));
    }

    // Kind parsing is case-insensitive: "Worktree" is the same kind.
    run(claim_kind("agent-a", &format!("{res}-upper"), "Worktree"))
        .expect("--kind Worktree is accepted");
}

#[test]
fn done_marks_an_added_work_package_and_releases_its_claim() {
    let wp = unique("wp-done");
    let res = unique("res-done");

    // Seed a work package and a claim held by the worker finishing it.
    run(add(&wp, "Ship the widget", "")).expect("add succeeds");
    let holder = format!("bot:{res}");
    run(claim("bot", &res)).expect("claim held while the WP runs");

    // Completing the WP releases the claim it was done under. The next claim
    // only succeeds if `done` removed the resource binding — otherwise
    // `claim` would refuse with "resource already claimed".
    run(done("bot", &wp, &holder)).expect("done succeeds");
    run(claim("other-bot", &res)).expect("done released the claim");
}

#[test]
fn done_on_an_unknown_task_errors_but_still_releases_the_claim() {
    let res = unique("res-ghost");
    let holder = format!("bot:{res}");
    run(claim("bot", &res)).expect("claim succeeds");

    // The work package does not exist: `mark_done` fails and `done` propagates.
    let err = run(done("bot", "no-such-wp", &holder)).expect_err("unknown work package must fail");
    let msg = err.to_string();
    assert!(msg.contains("unknown wp_id: no-such-wp"), "got: {msg}");

    // Documented current behavior: `AppState::done` releases the claim BEFORE
    // looking up the work package, so the resource is already free even
    // though `done` reported an error.
    run(claim("rescuer", &res)).expect("claim was already released despite the error");
}

#[test]
fn dedup_with_a_missing_input_file_is_an_error() {
    let tmp = TempDir::new().expect("tempdir");
    let missing = tmp.path().join("absent.tsv");

    // The failure surfaces as an io error through the command's Result.
    let err = run(dedup_file(&missing)).expect_err("reading a missing file must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("No such file or directory"),
        "the io error propagates through run_dag, got: {msg}"
    );
}

// ── Spawned-binary contract: validation, rendering, on-disk input ───────────

#[test]
fn invalid_input_is_rejected_with_guidance_at_parse_and_run_time() {
    // Unknown subcommand: clap rejects it with usage guidance, exit code 2.
    let out = cli_run(&["dag", "bogus"]);
    assert_eq!(out.status.code(), Some(2), "clap usage errors exit 2");
    let err = stderr(&out);
    assert!(err.contains("unrecognized subcommand"), "stderr: {err}");

    // Missing required argument: clap names the missing flag.
    let out = cli_run(&["dag", "pick"]);
    assert_eq!(out.status.code(), Some(2));
    let err = stderr(&out);
    assert!(
        err.contains("required arguments were not provided"),
        "stderr: {err}"
    );
    assert!(err.contains("--agent"), "stderr: {err}");

    // Help exits 0 and enumerates the surface: every subcommand is either
    // listed here or directly spawned by another test in this file.
    let out = cli_run(&["dag", "--help"]);
    assert!(out.status.success(), "help ok, stderr: {}", stderr(&out));
    let text = stdout(&out);
    for sub in ["pick", "claim", "release", "done", "add", "dedup"] {
        assert!(text.contains(sub), "help must list `{sub}`:\n{text}");
    }

    // Parses fine (kind is a free string) but fails at run time: exit 1 with
    // the anyhow message on stderr.
    let out = cli_run(&[
        "dag",
        "claim",
        "--agent",
        "a",
        "--resource",
        "r",
        "--kind",
        "vehicle",
    ]);
    assert_eq!(out.status.code(), Some(1), "runtime errors exit 1");
    let err = stderr(&out);
    assert!(err.contains("unknown claim kind: vehicle"), "stderr: {err}");
}

#[test]
fn dedup_scores_a_real_file_and_thresholds_change_the_result() {
    let tmp = TempDir::new().expect("tempdir");
    let input = write_dedup_file(
        tmp.path(),
        "backlog.tsv",
        &[
            "# comment lines are skipped",
            "wp-same\tFix the login redirect loop",
            "wp-copy\tFix the login redirect loop",
            "wp-other\tRewrite the invoice export pipeline",
        ]
        .join("\n"),
    );
    let from = input.to_str().expect("utf-8 temp path");

    // Default threshold (0.75): the identical pair scores hybrid=1.000 and
    // the unrelated item is not a candidate.
    let out = cli_run(&["dag", "dedup", "--from", from]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("wp-same"), "stdout: {text}");
    assert!(text.contains("wp-copy"), "stdout: {text}");
    assert!(text.contains("hybrid=1.000"), "stdout: {text}");
    assert!(text.contains("jaccard=1.000"), "stdout: {text}");
    assert!(
        !text.contains("wp-other"),
        "not a candidate at 0.75: {text}"
    );

    // Threshold above any possible score: nothing qualifies.
    let out = cli_run(&["dag", "dedup", "--from", from, "--threshold", "1.5"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("(no duplicate groups @ threshold 1.5)"),
        "stdout: {text}"
    );

    // Threshold zero: every pair qualifies, including the unrelated one.
    let out = cli_run(&["dag", "dedup", "--from", from, "--threshold", "0"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("hybrid="), "stdout: {text}");
    assert!(text.contains("wp-other"), "threshold 0 admits all: {text}");

    // dedup-explain reports the same scorer's breakdown for an identical pair.
    let phrase = "Fix the login redirect loop";
    let out = cli_run(&["dag", "dedup-explain", "--a", phrase, "--b", phrase]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("token_jaccard    = 1.0000"), "stdout: {text}");
    assert!(text.contains("hybrid_score     = 1.0000"), "stdout: {text}");
    assert!(text.contains("candidate_pair   = a vs b"), "stdout: {text}");
}

#[test]
fn scan_classifies_real_directories_on_disk() {
    let tmp = TempDir::new().expect("tempdir");
    let git_repo = tmp.path().join("repo");
    let with_src = tmp.path().join("with-src");
    let empty = tmp.path().join("empty");
    make_git_repo(&git_repo);
    std::fs::create_dir_all(with_src.join("src")).expect("source marker dir");
    std::fs::create_dir_all(&empty).expect("empty dir");
    let missing = tmp.path().join("not-there");
    let git = git_repo.to_str().expect("utf-8");
    let src = with_src.to_str().expect("utf-8");
    let emp = empty.to_str().expect("utf-8");
    let mis = missing.to_str().expect("utf-8");

    let out = cli_run(&["dag", "scan", git, src, emp, mis]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);

    // The fake git tree reads as a healthy repo: branch, remote, both refs.
    assert!(text.contains("state=Git"), "stdout: {text}");
    assert!(text.contains("branch=Some(\"main\")"), "stdout: {text}");
    assert!(text.contains("branches=2"), "stdout: {text}");
    assert!(text.contains("hygiene=100"), "stdout: {text}");

    // Non-git directories: hygiene 70 with a source marker, 30 without.
    assert!(text.contains("hygiene=70"), "stdout: {text}");
    assert!(text.contains("hygiene=30"), "stdout: {text}");

    // Roots that are not directories are silently skipped, not errors.
    assert!(!text.contains("not-there"), "root skipped: {text}");
}

#[test]
fn where_reports_context_for_the_requested_cwd() {
    let tmp = TempDir::new().expect("tempdir");
    let dir = tmp.path().join("proj");
    std::fs::create_dir_all(dir.join("src")).expect("source marker dir");

    let out = cli_run(&["dag", "where", "--cwd", dir.to_str().expect("utf-8")]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains(&format!("repo={}", dir.display())),
        "stdout: {text}"
    );
    assert!(text.contains("state=NoGit"), "stdout: {text}");
    assert!(text.contains("branch=None"), "stdout: {text}");
    assert!(text.contains("hygiene=70"), "stdout: {text}");

    // A cwd that does not exist yields no repo line and still exits 0:
    // inspect_repo returns None and scan skips the root.
    let gone = tmp.path().join("gone");
    let out = cli_run(&["dag", "where", "--cwd", gone.to_str().expect("utf-8")]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(text.is_empty(), "empty for missing cwd: {text}");
}

#[test]
fn fresh_process_renders_the_empty_store_and_parsed_defaults() {
    // Every spawn starts from an empty dag store: pick/topology render that.
    let out = cli_run(&[
        "dag", "pick", "--agent", "b", "--limit", "3", "--lane", "fast",
    ]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("(no pickable items)"), "stdout: {text}");

    let out = cli_run(&["dag", "topology"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("topo order: 0 nodes"), "stdout: {text}");

    // release/heartbeat on an unknown claim report `not found` without failing.
    let out = cli_run(&["dag", "release", "--claim-id", "ghost"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("release(ghost): not found"), "stdout: {text}");

    let out = cli_run(&["dag", "heartbeat", "--claim-id", "ghost"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("heartbeat(ghost): not found"),
        "stdout: {text}"
    );

    // Claim renders the parsed defaults: derived id, kind, ttl 3600.
    let out = cli_run(&["dag", "claim", "--agent", "bot", "--resource", "fresh-res"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains(
            "claimed: id=bot:fresh-res kind=Subproject resource=fresh-res agent=bot ttl=3600s"
        ),
        "stdout: {text}"
    );

    // `add` renders the parsed dependency list: comma-split, trimmed, empty
    // entries dropped — 2 deps from "a, b ,".
    let out = cli_run(&["dag", "add", "w", "--title", "T", "--depends", "a, b ,"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("added: w (2 deps)"), "stdout: {text}");
}
