// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared fixtures for the `agileplus worklog` integration tests.
//!
//! Test-side only; nothing here is compiled into the production binary.
//! The payloads are written as real JSON documents so the command layer runs
//! its own serde and validation paths rather than being handed pre-built
//! structs.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use agileplus_cli::commands::worklog::{EmitArgs, ShowArgs, WorklogPayload, run_show};

// ── payload fixtures ─────────────────────────────────────────────────────────

/// A payload shaped exactly like the documented 8-field canonical schema.
pub fn payload_value(task_id: &str, status: &str) -> serde_json::Value {
    serde_json::json!({
        "status": status,
        "task_id": task_id,
        "agent_id": "jcode",
        "files_changed": ["src/lib.rs", "src/main.rs"],
        "commit_sha": "abcdef1",
        "verification_result": {
            "status": "passed",
            "commands": ["cargo test -p agileplus-cli"],
            "notes": "all green",
        },
        "started_at": "2026-10-01T10:00:00Z",
        "completed_at": "2026-10-01T10:05:00Z",
    })
}

/// A pre-canonical document using the legacy key names, to drive `convert`.
pub fn legacy_value() -> serde_json::Value {
    serde_json::json!({
        "status": "completed",
        "task": "T-LEGACY",
        "branch": "branch-42",
        "files": ["a.rs"],
        "date": "2026-01-01",
        "verification": { "status": "passed", "commands": ["cargo test"], "notes": "" },
    })
}

/// Deserialize a JSON document through the production payload type.
pub fn parse(value: serde_json::Value) -> WorklogPayload {
    serde_json::from_value(value).expect("canonical payload must deserialize")
}

// ── filesystem / database helpers ────────────────────────────────────────────

/// Write a JSON document into `dir` under a worklog-shaped file name.
pub fn write_worklog(dir: &Path, name: &str, value: serde_json::Value) -> PathBuf {
    let path = dir.join(name);
    let body = serde_json::to_string_pretty(&value).expect("payload serializes");
    fs::write(&path, body).expect("worklog file written");
    path
}

/// Database path inside a temp dir, so no test touches a shared file.
pub fn db_in(dir: &TempDir) -> PathBuf {
    dir.path().join("worklog.db")
}

/// Number of rows currently visible to the `show` subcommand.
pub fn count_rows(db: &Path) -> usize {
    run_show(&unfiltered(), db)
        .expect("worklog_entries is queryable")
        .len()
}

// ── argument builders ────────────────────────────────────────────────────────

pub fn emit_args(from: PathBuf, replace: bool) -> EmitArgs {
    EmitArgs {
        from,
        verbose: false,
        replace,
    }
}

pub fn unfiltered() -> ShowArgs {
    ShowArgs {
        task: None,
        status: None,
        limit: 50,
        json: false,
    }
}
