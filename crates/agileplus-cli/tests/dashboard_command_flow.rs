// SPDX-License-Identifier: MIT OR Apache-2.0
//! End-to-end flow tests for the `dashboard` command
//! (`agileplus_cli::commands::dashboard::run`).
//!
//! Like the other flow suites this links the crate's rlib and spawns the
//! built `agileplus` binary, so both the `Result` surface and the rendered
//! stdout contract are asserted over a real SQLite file under a temp dir.
//! Nothing is mocked: every count, row and section header below is read
//! back from a database the test itself created and seeded.
//!
//! CLI reachability note: `main.rs` rejects the global `--db` outright and
//! the subcommand's own `--db` parses into that same global, so a spawned
//! `agileplus dashboard` can only be pointed at a database through repo
//! discovery (`--repo <git worktree>` → `<root>/.agileplus/agileplus.db`).
//! The explicit `--db` path is therefore exercised in-process instead.

use std::path::{Path, PathBuf};
use std::process::Output;

use tempfile::TempDir;

use agileplus_cli::commands::dashboard::run as run_dashboard;

mod support;
use support::dashboard::{
    args, cli, drop_optional_tables, json_args, migrated_db, seed_dashboard, seed_events,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Spawn `agileplus <args>` and capture output.
fn cli_run(argv: &[&str]) -> Output {
    cli()
        .args(argv)
        .output()
        .expect("agileplus binary should start")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// The one database file an in-process test owns, under its own temp dir.
fn fresh_db(tmp: &TempDir) -> PathBuf {
    tmp.path().join("agileplus-dashboard-test.db")
}

/// A real, canonical git worktree the spawned CLI can discover, plus the
/// repository-local database path `main.rs` derives from it.
fn repo(tmp: &TempDir) -> (PathBuf, PathBuf) {
    let root = tmp.path().canonicalize().expect("canonicalize temp dir");
    let status = std::process::Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&root)
        .status()
        .expect("git init should run");
    assert!(status.success(), "git init should succeed");
    let db = root.join(".agileplus").join("agileplus.db");
    (root, db)
}

/// `--repo` argument value for [`cli_run`].
fn root_str(root: &Path) -> &str {
    root.to_str().expect("utf-8 repo path")
}

// ── Result surface and file side effects ────────────────────────────────────

#[test]
fn in_process_run_creates_and_migrates_a_missing_database_file() {
    let tmp = TempDir::new().unwrap();
    let db = fresh_db(&tmp);
    assert!(!db.exists(), "fixture starts without the file");

    run_dashboard(&args(&db)).expect("run on a missing file creates and migrates it");

    assert!(db.exists(), "run created the database file");
    let conn = migrated_db(&db); // idempotent reopen
    let tables: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' \
             AND name IN ('work_packages', 'worklog_entries', 'events', 'trace_links')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(tables, 4, "open_db ran the migrations");

    // Re-running over the migrated file is a read-only no-op, both renderers.
    run_dashboard(&args(&db)).expect("second run succeeds");
    run_dashboard(&json_args(&db)).expect("json run succeeds");
}

#[test]
fn missing_parent_directory_surfaces_the_open_context_error() {
    let tmp = TempDir::new().unwrap();
    let db = tmp.path().join("no-such-dir").join("x.db");

    let err = run_dashboard(&args(&db)).expect_err("a missing parent dir must fail");
    let msg = err.to_string();
    assert!(msg.contains("opening sqlite db at"), "got: {msg}");
    assert!(!db.exists(), "nothing was created");
}

// ── Rendered stdout: empty database ─────────────────────────────────────────

#[test]
fn fresh_database_renders_every_section_empty() {
    let tmp = TempDir::new().unwrap();
    let (root, db) = repo(&tmp);
    let db_str = db.to_str().unwrap();
    assert!(!db.exists(), "fixture starts without the file");

    let out = cli_run(&["dashboard", "--repo", root_str(&root)]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    assert!(db.exists(), "spawned run created the repository-local file");
    let text = stdout(&out);

    assert!(text.contains("agileplus dashboard"), "stdout: {text}");
    assert!(text.contains(&format!("db: {db_str}")), "stdout: {text}");
    assert!(
        text.contains("[ Work packages by state ]  total = 0"),
        "stdout: {text}"
    );
    assert!(
        text.contains("<no worklog entries ingested yet>"),
        "stdout: {text}"
    );
    assert!(text.contains("<no events recorded>"), "stdout: {text}");
    assert!(text.contains("<no trace links recorded>"), "stdout: {text}");
}

// ── JSON contract over a seeded database ────────────────────────────────────

#[test]
fn json_snapshot_reflects_real_seeded_rows() {
    let tmp = TempDir::new().unwrap();
    let (root, db) = repo(&tmp);
    seed_dashboard(&db);
    let db_str = db.to_str().unwrap();

    let out = cli_run(&["dashboard", "--repo", root_str(&root), "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value =
        serde_json::from_str(&stdout(&out)).expect("stdout is a JSON document");

    // Path echo and timestamp are part of the document.
    assert_eq!(v["db_path"].as_str(), Some(db_str));
    let generated = v["generated_at"].as_str().expect("generated_at");
    generated
        .parse::<chrono::DateTime<chrono::FixedOffset>>()
        .expect("generated_at is RFC 3339");

    // Work-package breakdown: the exact rows written by the fixture.
    let wp = &v["work_packages"];
    assert_eq!(wp["total"].as_i64(), Some(6), "{wp}");
    assert_eq!(wp["planned"].as_i64(), Some(2), "{wp}");
    assert_eq!(wp["doing"].as_i64(), Some(1), "{wp}");
    assert_eq!(wp["review"].as_i64(), Some(1), "{wp}");
    assert_eq!(wp["done"].as_i64(), Some(1), "{wp}");
    assert_eq!(wp["blocked"].as_i64(), Some(1), "{wp}");

    // The rows behind those counts are physically in the file.
    let conn = migrated_db(&db);
    let rows: i64 = conn
        .query_row("SELECT COUNT(*) FROM work_packages", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 6, "fixture rows really exist");

    // Recent worklog entries: newest first, full row shape.
    let wl = v["recent_worklog_entries"].as_array().expect("array");
    assert_eq!(wl.len(), 3, "{wl:?}");
    assert_eq!(wl[0]["task_id"].as_str(), Some("wt-3"));
    assert_eq!(wl[0]["agent_id"].as_str(), Some("agent-three"));
    assert_eq!(wl[0]["status"].as_str(), Some("failed"));
    assert_eq!(wl[0]["verification_status"].as_str(), Some("failed"));
    assert!(wl[0]["completed_at"].is_string(), "{wl:?}");
    assert_eq!(wl[1]["task_id"].as_str(), Some("wt-2"));
    assert!(
        wl[1]["completed_at"].is_null(),
        "running row has no completion"
    );

    // Recent events: newest first with distinct types and actors.
    let ev = v["recent_events"].as_array().expect("array");
    assert_eq!(ev.len(), 3, "{ev:?}");
    assert_eq!(ev[0]["event_type"].as_str(), Some("shipped"));
    assert_eq!(ev[0]["actor"].as_str(), Some("carol"));
    assert_eq!(ev[1]["event_type"].as_str(), Some("updated"));

    // Trace links grouped: implements x2 first, then ties by name.
    let pairs: Vec<(String, i64)> = v["trace_link_summary"]
        .as_array()
        .expect("array")
        .iter()
        .map(|r| {
            (
                r["link_type"].as_str().expect("link_type").to_string(),
                r["count"].as_i64().expect("count"),
            )
        })
        .collect();
    assert_eq!(
        pairs,
        vec![
            ("implements".to_string(), 2),
            ("blocks".to_string(), 1),
            ("verifies".to_string(), 1),
        ],
        "ordered by count DESC then link_type ASC"
    );
}

#[test]
fn recent_sections_honor_limit_and_newest_first_order() {
    let tmp = TempDir::new().unwrap();
    let (root, db) = repo(&tmp);
    seed_dashboard(&db);

    let out = cli_run(&[
        "dashboard",
        "--repo",
        root_str(&root),
        "--json",
        "--limit",
        "2",
    ]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value =
        serde_json::from_str(&stdout(&out)).expect("stdout is a JSON document");

    let wl = v["recent_worklog_entries"].as_array().expect("array");
    assert_eq!(wl.len(), 2, "limit 2 caps the section");
    assert_eq!(wl[0]["task_id"].as_str(), Some("wt-3"), "newest first");
    assert_eq!(wl[1]["task_id"].as_str(), Some("wt-2"));

    let ev = v["recent_events"].as_array().expect("array");
    assert_eq!(ev.len(), 2, "limit 2 caps the section");
    assert_eq!(
        ev[0]["event_type"].as_str(),
        Some("shipped"),
        "newest first"
    );
    assert_eq!(ev[1]["event_type"].as_str(), Some("updated"));

    // The trace summary is an aggregate, not a "recent" section: unaffected.
    assert_eq!(v["trace_link_summary"].as_array().map(Vec::len), Some(3));
}

#[test]
fn limit_zero_clamps_to_a_single_row() {
    // `limit.clamp(1, 100)`: a plain SQL `LIMIT 0` would return nothing.
    let tmp = TempDir::new().unwrap();
    let (root, db) = repo(&tmp);
    seed_dashboard(&db);

    let out = cli_run(&[
        "dashboard",
        "--repo",
        root_str(&root),
        "--json",
        "--limit",
        "0",
    ]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value =
        serde_json::from_str(&stdout(&out)).expect("stdout is a JSON document");

    assert_eq!(
        v["recent_worklog_entries"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(v["recent_events"].as_array().map(Vec::len), Some(1));
}

#[test]
fn limit_above_100_clamps_to_100() {
    let tmp = TempDir::new().unwrap();
    let (root, db) = repo(&tmp);
    seed_events(&db, 105);

    let out = cli_run(&[
        "dashboard",
        "--repo",
        root_str(&root),
        "--json",
        "--limit",
        "500",
    ]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let v: serde_json::Value =
        serde_json::from_str(&stdout(&out)).expect("stdout is a JSON document");

    let ev = v["recent_events"].as_array().expect("array");
    assert_eq!(ev.len(), 100, "clamp(1, 100) caps an oversized --limit");
    assert_eq!(
        ev[0]["id"].as_i64(),
        Some(105),
        "still newest-first from the top"
    );
}

// ── Tolerance and rendering ─────────────────────────────────────────────────

#[test]
fn missing_optional_tables_still_render_the_other_sections() {
    let tmp = TempDir::new().unwrap();
    let (root, db) = repo(&tmp);
    seed_dashboard(&db);
    drop_optional_tables(&db);

    let out = cli_run(&["dashboard", "--repo", root_str(&root)]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);

    assert!(
        text.contains("<no worklog entries ingested yet>"),
        "stdout: {text}"
    );
    assert!(text.contains("<no trace links recorded>"), "stdout: {text}");
    assert!(
        text.contains("[ Recent events ]  (3 shown)"),
        "stdout: {text}"
    );
    assert!(
        text.contains("[ Work packages by state ]  total = 6"),
        "stdout: {text}"
    );
}

#[test]
fn ascii_table_renders_seeded_sections_with_counts_and_rows() {
    let tmp = TempDir::new().unwrap();
    let (root, db) = repo(&tmp);
    seed_dashboard(&db);
    let db_str = db.to_str().unwrap();

    let out = cli_run(&["dashboard", "--repo", root_str(&root)]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);

    // Header carries the generation time and the database actually read.
    assert!(text.contains("agileplus dashboard"), "stdout: {text}");
    assert!(text.contains(&format!("db: {db_str}")), "stdout: {text}");

    // WP breakdown: total plus every state row of the kanban table.
    assert!(
        text.contains("[ Work packages by state ]  total = 6"),
        "stdout: {text}"
    );
    for state in ["planned", "doing", "review", "done", "blocked"] {
        assert!(text.contains(state), "state row `{state}` missing: {text}");
    }

    // Recent sections show the seeded rows newest-first under the limit.
    assert!(
        text.contains("[ Recent worklog entries ]  (3 shown)"),
        "stdout: {text}"
    );
    assert!(text.contains("wt-3"), "stdout: {text}");
    assert!(text.contains("agent-three"), "stdout: {text}");
    assert!(
        text.contains("[ Recent events ]  (3 shown)"),
        "stdout: {text}"
    );
    assert!(text.contains("shipped"), "stdout: {text}");

    // Trace summary totals every link.
    assert!(
        text.contains("[ Trace links by type ]  total = 4"),
        "stdout: {text}"
    );
    assert!(text.contains("implements"), "stdout: {text}");
    assert!(text.contains("blocks"), "stdout: {text}");
    assert!(text.contains("verifies"), "stdout: {text}");
}

#[test]
fn no_color_output_carries_no_ansi_escapes() {
    let tmp = TempDir::new().unwrap();
    let (root, db) = repo(&tmp);
    seed_dashboard(&db);

    let out = cli_run(&["dashboard", "--repo", root_str(&root), "--no-color"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(
        !text.contains('\u{1b}'),
        "--no-color must strip ANSI: {text:?}"
    );
}

// ── Clap surface ────────────────────────────────────────────────────────────

#[test]
fn help_lists_the_surface_and_bad_limit_exits_2() {
    let out = cli_run(&["dashboard", "--help"]);
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    for flag in ["--limit", "--db", "--json", "--no-color"] {
        assert!(text.contains(flag), "help must list `{flag}`:\n{text}");
    }

    let out = cli_run(&["dashboard", "--limit", "not-a-number"]);
    assert_eq!(out.status.code(), Some(2), "clap usage errors exit 2");
    let err = stderr(&out);
    assert!(err.contains("invalid value"), "stderr: {err}");
}
