// SPDX-License-Identifier: MIT OR Apache-2.0
//! Fixtures for the `dashboard` command integration tests.
//!
//! `dashboard` is a read-only window over one SQLite file: `run` resolves
//! `--db`, opens the database (creating and migrating it when absent),
//! aggregates four sections and prints either the ASCII table or `--json`.
//! Every helper here therefore works against real files under a caller-owned
//! temp directory — no mocks, no shared state, no process-wide locks:
//!
//! * [`args`] / [`json_args`] build `DashboardArgs` with clap's defaults,
//! * [`migrated_db`] creates a fully-migrated database file,
//! * [`seed_dashboard`] writes real rows into every section's table,
//! * [`seed_events`] writes N events for the `--limit` clamp tests,
//! * [`drop_optional_tables`] removes the two tables `run` tolerates missing,
//! * [`cli`] yields the built `agileplus` binary for stdout/clap assertions.
#![allow(dead_code)]

use std::path::Path;

use assert_cmd::Command;
use rusqlite::{Connection, params};

use agileplus_cli::commands::dashboard::DashboardArgs;
use agileplus_sqlite::migrations::MigrationRunner;

// ── Binary ──────────────────────────────────────────────────────────────────

/// The built `agileplus` binary for end-to-end stdout/clap assertions.
pub fn cli() -> Command {
    Command::cargo_bin("agileplus").expect("agileplus binary should be built")
}

// ── Argument builders ───────────────────────────────────────────────────────

/// `dashboard` args with clap's defaults (limit 5, table output) pointed at
/// `db`, mirroring what the binary parses for `agileplus dashboard --db <db>`.
pub fn args(db: &Path) -> DashboardArgs {
    DashboardArgs {
        limit: 5,
        db: Some(db.to_path_buf()),
        json: false,
        no_color: false,
    }
}

/// Same as [`args`] with `--json` set.
pub fn json_args(db: &Path) -> DashboardArgs {
    DashboardArgs {
        json: true,
        ..args(db)
    }
}

// ── Real-filesystem database fixtures ───────────────────────────────────────

/// Open (creating if missing) a fully-migrated database at `db` — the same
/// connection contract `dashboard::run` gets from `trace::open_db`.
pub fn migrated_db(db: &Path) -> Connection {
    if let Some(parent) = db.parent() {
        std::fs::create_dir_all(parent).expect("create database parent directory");
    }
    let conn = Connection::open(db).expect("create or open the sqlite file");
    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .expect("foreign_keys pragma");
    MigrationRunner::new(&conn)
        .run_all()
        .expect("migrations apply");
    conn
}

/// Seed every dashboard section at `db`: one feature, 6 work packages
/// (2 planned + one each doing/review/done/blocked), 3 worklog entries
/// (ids 1..=3, newest `wt-3`), 3 events (newest `shipped`) and 4 trace
/// links across 3 link types (implements x2, blocks, verifies).
pub fn seed_dashboard(db: &Path) {
    let conn = migrated_db(db);

    conn.execute(
        "INSERT INTO features (id, slug, friendly_name, spec_hash, state, created_at, updated_at) \
         VALUES (1, 'feat-dash', 'Dash', x'00', 'specified', '2026-06-11T00:00:00Z', '2026-06-11T00:00:00Z')",
        (),
    )
    .expect("seed feature");

    for (i, state) in ["planned", "planned", "doing", "review", "done", "blocked"]
        .into_iter()
        .enumerate()
    {
        let id = i as i64 + 1;
        conn.execute(
            "INSERT INTO work_packages (id, feature_id, title, state, sequence, file_scope, \
             acceptance_criteria, agent_id, pr_url, pr_state, worktree_path, created_at, updated_at) \
             VALUES (?1, 1, ?2, ?3, ?4, '[]', '', NULL, NULL, NULL, NULL, \
                     '2026-06-11T00:00:00Z', '2026-06-11T00:00:00Z')",
            params![id, format!("wp-{id}"), state, id],
        )
        .expect("seed work package");
    }

    for (task, agent, status, verify, completed) in [
        (
            "wt-1",
            "agent-one",
            "completed",
            "passed",
            Some("2026-06-11T00:30:00Z"),
        ),
        ("wt-2", "agent-two", "running", "not_run", None),
        (
            "wt-3",
            "agent-three",
            "failed",
            "failed",
            Some("2026-06-11T02:00:00Z"),
        ),
    ] {
        conn.execute(
            "INSERT INTO worklog_entries (task_id, agent_id, status, commit_sha, files_changed_json, \
             verification_status, verification_notes, verification_cmds, started_at, completed_at, ingested_at) \
             VALUES (?1, ?2, ?3, NULL, '[]', ?4, '', '[]', '2026-06-11T00:00:00Z', ?5, '2026-06-11T03:00:00Z')",
            params![task, agent, status, verify, completed],
        )
        .expect("seed worklog entry");
    }

    for (i, (etype, actor)) in [
        ("created", "alice"),
        ("updated", "bob"),
        ("shipped", "carol"),
    ]
    .into_iter()
    .enumerate()
    {
        let id = i as i64 + 1;
        conn.execute(
            "INSERT INTO events (entity_type, entity_id, event_type, payload, actor, timestamp, \
             prev_hash, hash, sequence) \
             VALUES ('work_package', ?1, ?2, '{}', ?3, '2026-06-11T00:00:00Z', x'00', x'00', ?1)",
            params![id, etype, actor],
        )
        .expect("seed event");
    }

    for (from, link) in [
        ("1", "implements"),
        ("2", "implements"),
        ("3", "verifies"),
        ("4", "blocks"),
    ] {
        conn.execute(
            "INSERT INTO trace_links (from_kind, from_id, to_kind, to_id, link_type, note, \
             created_by, created_at) \
             VALUES ('work_package', ?1, 'feature', '1', ?2, '', 'tester', '2026-06-11T00:00:00Z')",
            params![from, link],
        )
        .expect("seed trace link");
    }
}

/// Seed `n` events with distinct (entity_id, sequence) pairs so the
/// `events` UNIQUE(entity_type, entity_id, sequence) constraint holds.
pub fn seed_events(db: &Path, n: i64) {
    let conn = migrated_db(db);
    for i in 1..=n {
        conn.execute(
            "INSERT INTO events (entity_type, entity_id, event_type, payload, actor, timestamp, \
             prev_hash, hash, sequence) \
             VALUES ('work_package', ?1, 'evt', '{}', 'bot', '2026-06-11T00:00:00Z', x'00', x'00', ?1)",
            params![i],
        )
        .expect("seed event");
    }
}

/// Drop the two tables `dashboard::run` explicitly tolerates missing — the
/// migration tracker keeps them "applied", so reopening does not recreate them.
pub fn drop_optional_tables(db: &Path) {
    let conn = Connection::open(db).expect("open db");
    conn.execute_batch("DROP TABLE IF EXISTS worklog_entries; DROP TABLE IF EXISTS trace_links;")
        .expect("drop optional tables");
}
