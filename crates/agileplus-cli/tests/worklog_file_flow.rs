// SPDX-License-Identifier: MIT OR Apache-2.0
//! File-level flow tests for the `worklog` command module: payload
//! validation and the `validate` / `convert` / `list` / `schema` subcommands
//! dispatched through `run_with_db`.
//!
//! These run as an integration target, so they link the crate's **rlib** and
//! exercise the production compilation units the campaign's production-only
//! coverage basis reports over. The module's own `#[cfg(test)]` suite compiles
//! into the test harness instead and contributes nothing to that basis.
//!
//! All file effects are asserted on real documents in a real temp directory.

use std::fs;

use tempfile::TempDir;

use agileplus_cli::commands::worklog::{
    ShowArgs, WorklogAction, WorklogArgs, run_with_db, validate_payload,
};

mod support;
use support::worklog::{
    count_rows, db_in, emit_args, legacy_value, parse, payload_value, write_worklog,
};

// ── validation ───────────────────────────────────────────────────────────────

#[test]
fn validate_payload_accepts_the_canonical_shape() {
    let payload = parse(payload_value("T-100", "completed"));
    validate_payload(&payload).expect("a canonical payload validates");
}

/// A rejection case: the case label, how to mangle a canonical payload, and
/// the error fragment the production validator must report.
type BrokenCase = (&'static str, fn(&mut serde_json::Value), &'static str);

#[test]
fn validate_payload_rejects_each_broken_field() {
    let cases: Vec<BrokenCase> = vec![
        (
            "unknown status",
            |v| v["status"] = serde_json::json!("archived"),
            "invalid status",
        ),
        (
            "blank task_id",
            |v| v["task_id"] = serde_json::json!("   "),
            "task_id must be a non-empty string",
        ),
        (
            "blank agent_id",
            |v| v["agent_id"] = serde_json::json!(""),
            "agent_id must be a non-empty string",
        ),
        (
            "uppercase sha",
            |v| v["commit_sha"] = serde_json::json!("ABCDEF1"),
            "is not a 7-40 char hex string",
        ),
        (
            "short sha",
            |v| v["commit_sha"] = serde_json::json!("abc"),
            "is not a 7-40 char hex string",
        ),
        (
            "unknown verification status",
            |v| v["verification_result"]["status"] = serde_json::json!("skipped"),
            "verification_result.status",
        ),
        (
            "commands present while not_run",
            |v| {
                v["verification_result"]["status"] = serde_json::json!("not_run");
                v["verification_result"]["commands"] = serde_json::json!(["cargo test"]);
            },
            "must be empty when status is 'not_run'",
        ),
        (
            "empty command entry",
            |v| v["verification_result"]["commands"] = serde_json::json!(["  "]),
            "contains an empty entry",
        ),
        (
            "malformed started_at",
            |v| v["started_at"] = serde_json::json!("yesterday"),
            "is not an ISO-8601-like string",
        ),
        (
            "malformed completed_at",
            |v| v["completed_at"] = serde_json::json!("01/02/2026"),
            "is not an ISO-8601-like string",
        ),
        (
            "empty file entry",
            |v| v["files_changed"] = serde_json::json!(["src/lib.rs", "  "]),
            "files_changed contains an empty entry",
        ),
        (
            "duplicate file entry",
            |v| v["files_changed"] = serde_json::json!(["src/lib.rs", "src/lib.rs"]),
            "files_changed contains duplicate entry",
        ),
    ];

    for (label, mutate, expected) in cases {
        let mut value = payload_value("T-100", "completed");
        mutate(&mut value);
        let err = validate_payload(&parse(value))
            .expect_err("case must be rejected")
            .to_string();
        assert!(
            err.contains(expected),
            "{label}: expected error containing {expected:?}, got {err:?}"
        );
    }
}

// ── convert ──────────────────────────────────────────────────────────────────

#[test]
fn convert_writes_a_canonical_sibling_and_applies_field_fallbacks() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let db = db_in(&tmp);
    let source = write_worklog(dir, "worklog-legacy.json", legacy_value());
    let before = fs::read_to_string(&source).unwrap();

    run_with_db(
        &WorklogArgs {
            dir: dir.to_path_buf(),
            action: WorklogAction::Convert { in_place: false },
        },
        &db,
    )
    .expect("convert succeeds");

    assert_eq!(
        fs::read_to_string(&source).unwrap(),
        before,
        "a non-in-place convert leaves the source untouched"
    );

    let canonical = dir.join("worklog-legacy-canonical.json");
    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&canonical).unwrap()).unwrap();
    let obj = written.as_object().expect("canonical output is an object");

    for field in [
        "status",
        "task_id",
        "agent_id",
        "files_changed",
        "commit_sha",
        "verification_result",
        "started_at",
        "completed_at",
    ] {
        assert!(obj.contains_key(field), "canonical output missing {field}");
    }

    // Legacy keys are folded into the canonical ones.
    assert_eq!(obj["task_id"], "T-LEGACY", "`task` maps to `task_id`");
    assert_eq!(
        obj["commit_sha"], "branch-42",
        "`branch` maps to `commit_sha`"
    );
    assert_eq!(obj["files_changed"], serde_json::json!(["a.rs"]));
    assert_eq!(
        obj["completed_at"], "2026-01-01",
        "`date` maps to `completed_at`"
    );
    assert_eq!(obj["agent_id"], "codex-exec", "absent agent_id falls back");
    assert_eq!(obj["status"], "completed");
    assert_eq!(
        obj["verification_result"]["status"], "passed",
        "the legacy `verification` key is read too"
    );
}

#[test]
fn convert_in_place_rewrites_the_source_file() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let db = db_in(&tmp);
    let source = write_worklog(dir, "worklog-legacy.json", legacy_value());

    run_with_db(
        &WorklogArgs {
            dir: dir.to_path_buf(),
            action: WorklogAction::Convert { in_place: true },
        },
        &db,
    )
    .expect("in-place convert succeeds");

    let rewritten: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&source).unwrap()).unwrap();
    assert_eq!(rewritten["task_id"], "T-LEGACY", "the source is rewritten");
    assert!(
        !dir.join("worklog-legacy-canonical.json").exists(),
        "no sibling file is produced in-place"
    );
}

// ── subcommand dispatch ──────────────────────────────────────────────────────

#[test]
fn run_with_db_validates_and_lists_canonical_files() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let db = db_in(&tmp);

    write_worklog(dir, "worklog-ok.json", payload_value("T-100", "completed"));
    fs::write(dir.join("worklog-ok-canonical.json"), "{}").unwrap();

    // `validate` exits non-zero on any failure, so this directory is
    // deliberately all-valid; the assertion is that the whole flow resolves.
    run_with_db(
        &WorklogArgs {
            dir: dir.to_path_buf(),
            action: WorklogAction::Validate,
        },
        &db,
    )
    .expect("every file is canonical, so validate must succeed");

    run_with_db(
        &WorklogArgs {
            dir: dir.to_path_buf(),
            action: WorklogAction::List,
        },
        &db,
    )
    .expect("list resolves");

    run_with_db(
        &WorklogArgs {
            dir: dir.to_path_buf(),
            action: WorklogAction::Schema,
        },
        &db,
    )
    .expect("schema resolves");
}

#[test]
fn run_with_db_dispatches_emit_and_show_against_the_given_database() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let db = db_in(&tmp);
    let file = write_worklog(
        dir,
        "worklog-dispatch.json",
        payload_value("T-900", "completed"),
    );

    run_with_db(
        &WorklogArgs {
            dir: dir.to_path_buf(),
            action: WorklogAction::Emit(emit_args(file, false)),
        },
        &db,
    )
    .expect("emit dispatch resolves");

    // `json: true` takes the pretty-printed projection branch.
    run_with_db(
        &WorklogArgs {
            dir: dir.to_path_buf(),
            action: WorklogAction::Show(ShowArgs {
                task: Some("T-900".to_string()),
                status: Some("completed".to_string()),
                limit: 10,
                json: true,
            }),
        },
        &db,
    )
    .expect("json show dispatch resolves");

    // `json: false` takes the human-readable table branch.
    run_with_db(
        &WorklogArgs {
            dir: dir.to_path_buf(),
            action: WorklogAction::Show(ShowArgs {
                task: None,
                status: None,
                limit: 50,
                json: false,
            }),
        },
        &db,
    )
    .expect("table show dispatch resolves");

    assert_eq!(count_rows(&db), 1, "emit landed the row in the given db");
}
