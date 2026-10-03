// SPDX-License-Identifier: MIT OR Apache-2.0
//! Fixtures for the `dag` command integration tests.
//!
//! `run_dag` is unusual among `agileplus-cli` commands: `main.rs` routes the
//! `Dag` arm straight to `run_dag` without opening a database or a VCS port,
//! and `commands/dag.rs` keeps all of its state (work packages + claims) in a
//! process-wide `OnceLock<Mutex<AppState<..>>>`. The fixtures here therefore
//! provide:
//!
//! * `serialized` / `run` — a lock plus a driver so tests in this binary
//!   cannot race that shared static while running `run_dag` in-process,
//! * argument builders for each `DagCmd` variant, mirroring the clap defaults,
//! * real-filesystem helpers (dedup input files, a hand-built git tree) for
//!   the subcommands that read directories and files from disk,
//! * `cli()` — the built `agileplus` binary, so stdout rendering and clap's
//!   argument validation are asserted end-to-end.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use assert_cmd::Command;

use agileplus_cli::commands::dag::{
    AddArgs, ClaimArgs, DagArgs, DagCmd, DedupArgs, DedupExplainArgs, DoneArgs, HeartbeatArgs,
    PickArgs, ReleaseArgs, ScanArgs, TopologyArgs, WhereArgs,
};

pub use super::implement::block_on;

// ── Shared-state discipline ─────────────────────────────────────────────────

/// Serializes in-process `run_dag` calls.
///
/// `commands/dag.rs` keeps one `Mutex<AppState<..>>` for the whole process,
/// so every test in this binary shares claims and work packages. Tests go
/// through [`run`], which holds this lock for the duration of a command so no
/// test can observe another test's mid-mutation state. A poisoned lock is
/// recovered from: one panicking test must not cascade into the rest.
pub fn serialized() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Process-unique fragment for wp ids / claim resources so fixtures cannot
/// collide across tests, even if a future test forgets [`serialized`].
pub fn unique(prefix: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!(
        "{prefix}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

/// Run one `dag` subcommand in-process, serialized against sibling tests.
///
/// `run_dag` is async but performs no I/O that needs a tokio reactor, so the
/// same `tokio_test::block_on` the other flow suites use drives it.
pub fn run(args: DagArgs) -> anyhow::Result<()> {
    let _guard = serialized();
    block_on(agileplus_cli::commands::dag::run_dag(args))
}

// ── Argument builders ───────────────────────────────────────────────────────
//
// Each builder mirrors the clap defaults in `commands/dag.rs`, so a test that
// does not care about a flag still exercises the values the binary would pass.

/// `dag pick --agent <agent>` with clap's default limit of 5.
pub fn pick(agent: &str) -> DagArgs {
    DagArgs {
        cmd: DagCmd::Pick(PickArgs {
            agent: agent.to_string(),
            limit: 5,
            lane: None,
            category: None,
        }),
    }
}

/// `dag claim` with clap's defaults: `--kind subproject --ttl 3600` and the
/// derived claim id `<agent>:<resource>`.
pub fn claim(agent: &str, resource: &str) -> DagArgs {
    claim_kind(agent, resource, "subproject")
}

/// `dag claim --kind <kind>` with the derived claim id.
pub fn claim_kind(agent: &str, resource: &str, kind: &str) -> DagArgs {
    claim_as(agent, resource, kind, None)
}

/// `dag claim` with an optional explicit `--claim-id`.
pub fn claim_as(agent: &str, resource: &str, kind: &str, claim_id: Option<&str>) -> DagArgs {
    DagArgs {
        cmd: DagCmd::Claim(ClaimArgs {
            agent: agent.to_string(),
            resource: resource.to_string(),
            kind: kind.to_string(),
            ttl: 3600,
            reason: None,
            claim_id: claim_id.map(str::to_string),
        }),
    }
}

/// `dag release --claim-id <id>`.
pub fn release(claim_id: &str) -> DagArgs {
    DagArgs {
        cmd: DagCmd::Release(ReleaseArgs {
            claim_id: claim_id.to_string(),
        }),
    }
}

/// `dag heartbeat --claim-id <id>`.
pub fn heartbeat(claim_id: &str) -> DagArgs {
    DagArgs {
        cmd: DagCmd::Heartbeat(HeartbeatArgs {
            claim_id: claim_id.to_string(),
        }),
    }
}

/// `dag done --agent <agent> --task <wp> --claim-id <id>`.
pub fn done(agent: &str, task: &str, claim_id: &str) -> DagArgs {
    DagArgs {
        cmd: DagCmd::Done(DoneArgs {
            agent: agent.to_string(),
            task: task.to_string(),
            claim_id: claim_id.to_string(),
            result: None,
        }),
    }
}

/// `dag add <wp_id> --title <title> --depends <depends>` (`--state ready`).
pub fn add(wp_id: &str, title: &str, depends: &str) -> DagArgs {
    DagArgs {
        cmd: DagCmd::Add(AddArgs {
            wp_id: wp_id.to_string(),
            title: title.to_string(),
            state: "ready".to_string(),
            depends: depends.to_string(),
        }),
    }
}

/// `dag dedup --from <path>` with clap's default threshold of 0.75.
pub fn dedup_file(path: &Path) -> DagArgs {
    dedup_at(path, 0.75)
}

/// `dag dedup --from <path> --threshold <t>`.
pub fn dedup_at(path: &Path, threshold: f64) -> DagArgs {
    DagArgs {
        cmd: DagCmd::Dedup(DedupArgs {
            from: path.to_string_lossy().into_owned(),
            threshold,
        }),
    }
}

/// `dag dedup-explain --a <a> --b <b>`.
pub fn explain(a: &str, b: &str) -> DagArgs {
    DagArgs {
        cmd: DagCmd::DedupExplain(DedupExplainArgs {
            a: a.to_string(),
            b: b.to_string(),
        }),
    }
}

/// `dag scan <roots...>`; roots that are not directories are skipped by the
/// production `AppState::scan`.
pub fn scan(roots: &[PathBuf]) -> DagArgs {
    DagArgs {
        cmd: DagCmd::Scan(ScanArgs {
            roots: roots
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect(),
        }),
    }
}

/// `dag where --cwd <dir>`.
pub fn where_cwd(dir: &Path) -> DagArgs {
    DagArgs {
        cmd: DagCmd::Where(WhereArgs {
            cwd: Some(dir.to_string_lossy().into_owned()),
        }),
    }
}

/// `dag topology` (no `--root`; the in-memory port ignores it).
pub fn topology() -> DagArgs {
    DagArgs {
        cmd: DagCmd::Topology(TopologyArgs { root: None }),
    }
}

// ── Real-filesystem fixtures ────────────────────────────────────────────────

/// Write a `dag dedup` input file under `dir` and return its path.
pub fn write_dedup_file(dir: &Path, name: &str, contents: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, contents).expect("writing dedup input file");
    path
}

/// Build a structurally real git directory at `dir` — `.git/HEAD`, two branch
/// refs and a remote — which `dag scan` / `dag where` classify as `Git` with
/// `branch=Some("main")`, `branches=2`, `hygiene=100`.
pub fn make_git_repo(dir: &Path) {
    let git = dir.join(".git");
    let heads = git.join("refs").join("heads");
    std::fs::create_dir_all(&heads).expect("creating git refs directory");
    std::fs::write(git.join("HEAD"), "ref: refs/heads/main\n").expect("writing HEAD");
    std::fs::write(heads.join("main"), "abc123\n").expect("writing main ref");
    std::fs::write(heads.join("feat-x"), "def456\n").expect("writing feat-x ref");
    std::fs::write(
        git.join("config"),
        "[remote \"origin\"]\n\turl = https://example.invalid/origin.git\n",
    )
    .expect("writing git config");
}

// ── Binary ──────────────────────────────────────────────────────────────────

/// The built `agileplus` binary for end-to-end argument/stdout assertions.
/// Every spawn starts from an empty dag store: state is process-local.
pub fn cli() -> Command {
    Command::cargo_bin("agileplus").expect("agileplus binary should be built")
}
