// SPDX-License-Identifier: MIT OR Apache-2.0
//! End-to-end database flow tests for the `worklog` command module
//! (`emit`, `show` and the `worklog_entries` table they share).
//!
//! These run as an integration target, so they link the crate's **rlib** and
//! exercise the production compilation units that the campaign's
//! production-only coverage basis reports over (see
//! `scripts/coverage-complete.sh`: only rlib objects carry production counts).
//!
//! The module's own `#[cfg(test)]` suite compiles into the test harness
//! instead, so those tests contribute nothing to that basis. This file drives
//! the primary flows through the public rlib API rather than re-asserting
//! every edge case the in-file suite already pins down.
//!
//! Everything here is real: real temp directories, real JSON files on disk,
//! and a real SQLite database created by the production migration runner.

use std::fs;

use tempfile::TempDir;

use agileplus_cli::commands::worklog::{insert_entry, open_db, run_emit, run_show};

mod support;
use support::worklog::{
    count_rows, db_in, emit_args, parse, payload_value, unfiltered, write_worklog,
};

// ── Connection management ────────────────────────────────────────────────────

#[test]
fn open_db_creates_worklog_schema_and_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let db = db_in(&tmp);

    let conn = open_db(&db).expect("fresh database opens and applies migrations");
    let tables: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='worklog_entries'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(tables, 1, "migration 022 must create worklog_entries");
    drop(conn);

    // Re-opening runs every migration again; already-applied ones are skipped.
    open_db(&db).expect("existing database re-opens without re-applying migrations");
    assert_eq!(count_rows(&db), 0, "schema bootstrap must not seed rows");
}

// ── emit ─────────────────────────────────────────────────────────────────────

#[test]
fn emit_from_directory_ingests_valid_files_and_skips_invalid() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let db = db_in(&tmp);

    write_worklog(
        dir,
        "worklog-good.json",
        payload_value("T-100", "completed"),
    );

    let mut bad = payload_value("T-200", "completed");
    bad["status"] = serde_json::json!("not-a-status");
    write_worklog(dir, "worklog-bad.json", bad);

    fs::write(dir.join("notes.txt"), "not a worklog").unwrap();

    let report = run_emit(&emit_args(dir.to_path_buf(), false), &db).expect("emit runs");

    assert_eq!(report.files_seen, 2, "only *.json files are collected");
    assert_eq!(report.files_loaded, 1);
    assert_eq!(
        report.files_skipped, 1,
        "the invalid status file is skipped"
    );
    assert_eq!(report.rows_inserted, 1);
    assert_eq!(report.validation_errors.len(), 1);

    let (path, msg) = &report.validation_errors[0];
    assert!(
        path.ends_with("worklog-bad.json"),
        "error must name the rejected file, got {}",
        path.display()
    );
    assert!(msg.contains("invalid status"), "unexpected error: {msg}");

    let entries = run_show(&unfiltered(), &db).unwrap();
    assert_eq!(entries.len(), 1, "only the valid file reaches the table");
    assert_eq!(entries[0].task_id, "T-100");
}

#[test]
fn emit_from_single_file_ingests_only_that_file() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let db = db_in(&tmp);

    write_worklog(dir, "worklog-one.json", payload_value("T-100", "completed"));
    write_worklog(dir, "worklog-two.json", payload_value("T-200", "pending"));

    let report = run_emit(&emit_args(dir.join("worklog-one.json"), false), &db).unwrap();

    assert_eq!(report.files_seen, 1);
    assert_eq!(report.files_loaded, 1);
    let entries = run_show(&unfiltered(), &db).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].task_id, "T-100");
}

#[test]
fn emit_is_idempotent_and_replace_keeps_exactly_one_row() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let db = db_in(&tmp);
    let file = write_worklog(
        dir,
        "worklog-once.json",
        payload_value("T-100", "completed"),
    );

    let first = run_emit(&emit_args(file.clone(), false), &db).unwrap();
    assert_eq!(first.rows_inserted, 1);

    // Re-emitting the same (task_id, source_path) hits UNIQUE and is ignored.
    let second = run_emit(&emit_args(file.clone(), false), &db).unwrap();
    assert_eq!(
        second.files_loaded, 1,
        "the file still parses and validates"
    );
    assert_eq!(second.rows_inserted, 0, "duplicate key must not insert");
    assert_eq!(count_rows(&db), 1);

    // `replace` deletes the prior row before re-inserting, so the table keeps
    // exactly one row and the insert is reported as a fresh insert. Note the
    // `EmitReport::rows_replaced` counter cannot move on this path: its branch
    // needs `insert_entry` to return `false`, but a successful DELETE always
    // lets the following INSERT land. That counter is dead in production.
    let replaced = run_emit(&emit_args(file.clone(), true), &db).unwrap();
    assert_eq!(replaced.files_loaded, 1);
    assert_eq!(replaced.rows_inserted, 1);
    assert_eq!(
        replaced.rows_replaced, 0,
        "the replace counter never advances"
    );
    assert_eq!(count_rows(&db), 1, "replace must not duplicate the row");
}

#[test]
fn emit_reports_missing_path_as_error() {
    let tmp = TempDir::new().unwrap();
    let db = db_in(&tmp);

    let err = run_emit(&emit_args(tmp.path().join("does-not-exist"), false), &db).unwrap_err();

    assert!(
        format!("{err:#}").contains("--from path does not exist"),
        "unexpected error: {err:#}"
    );
}

#[test]
fn emit_reports_json_parse_errors_without_inserting() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let db = db_in(&tmp);
    fs::write(dir.join("worklog-broken.json"), "{not json").unwrap();

    let report = run_emit(&emit_args(dir.to_path_buf(), false), &db).unwrap();

    assert_eq!(report.files_seen, 1);
    assert_eq!(report.files_loaded, 0);
    assert_eq!(report.files_skipped, 1);
    assert!(report.validation_errors[0].1.contains("json parse error"));
    assert_eq!(count_rows(&db), 0);
}

// ── show ─────────────────────────────────────────────────────────────────────

#[test]
fn show_filters_by_task_and_status_and_orders_newest_first() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let db = db_in(&tmp);

    write_worklog(dir, "worklog-a.json", payload_value("T-100", "completed"));
    write_worklog(dir, "worklog-b.json", payload_value("T-200", "failed"));
    write_worklog(dir, "worklog-c.json", payload_value("T-300", "failed"));
    run_emit(&emit_args(dir.to_path_buf(), false), &db).unwrap();

    assert_eq!(count_rows(&db), 3);

    let mut by_task = unfiltered();
    by_task.task = Some("T-200".to_string());
    let hits = run_show(&by_task, &db).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].task_id, "T-200");

    let mut by_status = unfiltered();
    by_status.status = Some("failed".to_string());
    assert_eq!(run_show(&by_status, &db).unwrap().len(), 2);

    // Both clauses are combined with AND.
    let mut both = unfiltered();
    both.task = Some("T-300".to_string());
    both.status = Some("failed".to_string());
    assert_eq!(run_show(&both, &db).unwrap().len(), 1);

    let mut mismatched = unfiltered();
    mismatched.task = Some("T-300".to_string());
    mismatched.status = Some("completed".to_string());
    assert_eq!(run_show(&mismatched, &db).unwrap().len(), 0);

    // ORDER BY id DESC, so the newest row comes back under a limit of 1.
    let mut limited = unfiltered();
    limited.limit = 1;
    let capped = run_show(&limited, &db).unwrap();
    assert_eq!(capped.len(), 1);
    assert_eq!(capped[0].task_id, "T-300");
}

#[test]
fn show_rejects_non_positive_limit() {
    let tmp = TempDir::new().unwrap();
    let db = db_in(&tmp);

    let mut args = unfiltered();
    args.limit = 0;
    let err = run_show(&args, &db).unwrap_err();

    assert!(
        format!("{err:#}").contains("--limit must be a positive integer"),
        "unexpected error: {err:#}"
    );
    assert!(
        !db.exists(),
        "the limit guard fires before the database opens"
    );
}

#[test]
fn emitted_payload_round_trips_through_show() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let db = db_in(&tmp);
    let file = write_worklog(
        dir,
        "worklog-round.json",
        payload_value("T-4242", "completed"),
    );

    run_emit(&emit_args(file, false), &db).unwrap();

    let entries = run_show(&unfiltered(), &db).unwrap();
    assert_eq!(entries.len(), 1);
    let row = &entries[0];

    assert_eq!(row.status, "completed");
    assert_eq!(row.task_id, "T-4242");
    assert_eq!(row.agent_id, "jcode");
    assert_eq!(row.files_changed, vec!["src/lib.rs", "src/main.rs"]);
    assert_eq!(row.commit_sha.as_deref(), Some("abcdef1"));
    assert_eq!(row.started_at, "2026-10-01T10:00:00Z");
    assert_eq!(row.completed_at.as_deref(), Some("2026-10-01T10:05:00Z"));
    assert!(
        row.source_path.ends_with("worklog-round.json"),
        "source_path records the ingested file, got {}",
        row.source_path
    );
    assert!(!row.ingested_at.is_empty(), "ingest timestamp is stamped");

    // `VerificationResult` keeps its fields private, so read them back through
    // the serialisation the production code itself relies on.
    let verification = serde_json::to_value(&row.verification).unwrap();
    assert_eq!(verification["status"], "passed");
    assert_eq!(verification["notes"], "all green");
    assert_eq!(
        verification["commands"],
        serde_json::json!(["cargo test -p agileplus-cli"])
    );
}

// ── insert_entry ─────────────────────────────────────────────────────────────

#[test]
fn insert_entry_is_keyed_on_task_and_source_path() {
    let tmp = TempDir::new().unwrap();
    let db = db_in(&tmp);
    let conn = open_db(&db).unwrap();
    let payload = parse(payload_value("T-100", "completed"));

    assert!(
        insert_entry(&conn, &payload, "/tmp/worklog-a.json", "{}", false).unwrap(),
        "the first insert is new"
    );
    assert!(
        !insert_entry(&conn, &payload, "/tmp/worklog-a.json", "{}", false).unwrap(),
        "the same (task_id, source_path) is ignored"
    );
    assert!(
        insert_entry(&conn, &payload, "/tmp/worklog-b.json", "{}", false).unwrap(),
        "a different source_path is a distinct row"
    );
    assert!(
        insert_entry(&conn, &payload, "/tmp/worklog-b.json", "{}", true).unwrap(),
        "replace deletes the prior row before inserting"
    );
    drop(conn);

    assert_eq!(count_rows(&db), 2, "two source paths, one row each");
}
