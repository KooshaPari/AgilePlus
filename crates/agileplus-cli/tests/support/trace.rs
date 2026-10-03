// SPDX-License-Identifier: MIT OR Apache-2.0
//! Fixtures for the `trace` command integration tests.
//!
//! The `trace` subcommands do real work against a real SQLite file: `link`
//! inserts into `trace_links`, while `list`/`show` read it back. The binary's
//! top-level command enum does not route `trace` yet, so these fixtures drive
//! the public `commands::trace` entry points in-process, against real temp
//! databases, and read the rows straight out of SQLite so a broken command
//! fails the assertions rather than a double.
//!
//! stdout is observable too: [`capture_probe`] re-execs this test binary and
//! runs the command in the child, so `println!` output is captured without
//! reaching for a stdout-mocking dependency.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

use rusqlite::Connection;

/// Name of the re-exec entry point in `trace_command_flow.rs`.
pub const PROBE_TEST: &str = "__trace_stdout_probe";

/// Process-unique fragment so parallel tests never share temp paths.
pub fn unique(prefix: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!(
        "{prefix}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

/// Initialize a real git repository at `dir`.
///
/// `trace link` calls `ProjectContext::discover`, so it must run inside a git
/// work tree even when `--db` points somewhere else.
pub fn init_git_repo(dir: &Path) {
    let status = Command::new("git")
        .args(["init", "--quiet"])
        .arg(dir)
        .status()
        .expect("git must be installed to build the fixture repo");
    assert!(status.success(), "git init failed for {}", dir.display());
}

/// A database path inside `dir`.
pub fn db_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(name)
}

/// Re-exec this test binary with `TRACE_PROBE=<probe>` and return its stdout.
///
/// The child runs exactly [`PROBE_TEST`] with output uncaptured, so the command
/// under test prints to the child's real stdout. `env` entries are extra
/// `TRACE_*` values the probe reads; `cwd` overrides the child's working
/// directory (used to give `link` a temporary git repository).
pub fn capture_probe(probe: &str, env: &[(&str, &str)], cwd: Option<&Path>) -> String {
    let exe = std::env::current_exe().expect("current test binary path");
    let mut cmd = Command::new(exe);
    cmd.args(["--exact", PROBE_TEST, "--nocapture", "--test-threads=1"])
        .env("TRACE_PROBE", probe)
        .env_remove("AGILEPLUS_DB");
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("probe child should run");
    assert!(
        out.status.success(),
        "probe `{probe}` failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// One persisted `trace_links` row, read straight from SQLite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRow {
    pub id: i64,
    pub from_kind: String,
    pub from_id: String,
    pub to_kind: String,
    pub to_id: String,
    pub link_type: String,
    pub note: String,
    pub created_by: String,
    pub created_at: String,
}

/// Connection to a CLI-written database; panics if the file is absent.
pub fn open_readonly(path: &Path) -> Connection {
    assert!(
        path.exists(),
        "expected the command to create {}",
        path.display()
    );
    Connection::open(path).expect("opening the sqlite database")
}

/// Number of rows in `trace_links`.
pub fn count_links(path: &Path) -> i64 {
    open_readonly(path)
        .query_row("SELECT COUNT(*) FROM trace_links", [], |r| r.get(0))
        .expect("counting trace_links rows")
}

/// Every row in `trace_links`, oldest first.
pub fn fetch_links(path: &Path) -> Vec<LinkRow> {
    let conn = open_readonly(path);
    let mut stmt = conn
        .prepare(
            "SELECT id, from_kind, from_id, to_kind, to_id, link_type, note, created_by, created_at \
             FROM trace_links ORDER BY id ASC",
        )
        .expect("preparing trace_links SELECT");
    stmt.query_map([], |r| {
        Ok(LinkRow {
            id: r.get(0)?,
            from_kind: r.get(1)?,
            from_id: r.get(2)?,
            to_kind: r.get(3)?,
            to_id: r.get(4)?,
            link_type: r.get(5)?,
            note: r.get(6)?,
            created_by: r.get(7)?,
            created_at: r.get(8)?,
        })
    })
    .expect("querying trace_links")
    .collect::<rusqlite::Result<Vec<_>>>()
    .expect("collecting trace_links")
}

/// Find a file named `name` anywhere under `root`; proves the repository-local
/// default database actually landed on disk without hard-coding its path.
pub fn find_file(root: &Path, name: &str) -> Option<PathBuf> {
    fn walk(dir: &Path, name: &str, found: &mut Option<PathBuf>) {
        if found.is_some() {
            return;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // The git object store can never hold our database.
                if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                    continue;
                }
                walk(&path, name, found);
            } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
                *found = Some(path);
                return;
            }
        }
    }
    let mut found = None;
    walk(root, name, &mut found);
    found
}
