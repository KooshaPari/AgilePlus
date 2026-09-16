//! Integration tests for the audit log.
//!
//! Covers: append-only JSONL persistence, read-back, serialization, directory creation.

#![cfg(feature = "audit")]

use agileplus_subcmds::AuditLog;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Basic write/read cycle
// ---------------------------------------------------------------------------

#[test]
fn write_pre_then_post_and_read_back() {
    let tmp = TempDir::new().unwrap();
    let log = AuditLog::new(tmp.path().join("audit.jsonl"));

    log.log_pre_dispatch(
        "triage:classify",
        Some("claude"),
        Some("my-feature"),
        serde_json::json!({"input": "fix this bug"}),
    )
    .unwrap();

    log.log_post_dispatch(
        "triage:classify",
        Some("claude"),
        Some("my-feature"),
        true,
        "classified as bug",
        42,
    )
    .unwrap();

    let entries = log.read_all().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].command, "triage:classify");
    assert_eq!(entries[0].agent.as_deref(), Some("claude"));
    assert_eq!(entries[0].feature_slug.as_deref(), Some("my-feature"));
    assert_eq!(
        entries[0].args,
        serde_json::json!({"input": "fix this bug"})
    );

    let post = &entries[1];
    assert!(post.result.as_ref().unwrap().success);
    assert_eq!(post.result.as_ref().unwrap().message, "classified as bug");
    assert_eq!(post.result.as_ref().unwrap().duration_ms, 42);
}

// ---------------------------------------------------------------------------
// Empty log
// ---------------------------------------------------------------------------

#[test]
fn read_nonexistent_file_returns_empty() {
    let tmp = TempDir::new().unwrap();
    let log = AuditLog::new(tmp.path().join("nope.jsonl"));
    let entries = log.read_all().unwrap();
    assert!(entries.is_empty());
}

// ---------------------------------------------------------------------------
// Append-only ordering
// ---------------------------------------------------------------------------

#[test]
fn multiple_entries_preserve_order() {
    let tmp = TempDir::new().unwrap();
    let log = AuditLog::new(tmp.path().join("audit.jsonl"));

    for i in 0..10 {
        log.log_pre_dispatch(
            &format!("cmd:{i}"),
            None,
            None,
            serde_json::Value::Null,
        )
        .unwrap();
    }

    let entries = log.read_all().unwrap();
    assert_eq!(entries.len(), 10);
    for (i, entry) in entries.iter().enumerate() {
        assert_eq!(entry.command, format!("cmd:{i}"));
    }
}

// ---------------------------------------------------------------------------
// Default path: write/read works via default_path constructor
// ---------------------------------------------------------------------------

#[test]
fn default_path_write_read_works() {
    let tmp = TempDir::new().unwrap();
    // Simulate project root with .agileplus directory.
    let log = AuditLog::default_path(tmp.path());

    log.log_pre_dispatch("test:cmd", None, None, serde_json::Value::Null)
        .unwrap();

    let entries = log.read_all().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].command, "test:cmd");
}

// ---------------------------------------------------------------------------
// Parent directory auto-creation
// ---------------------------------------------------------------------------

#[test]
fn creates_parent_directories_on_write() {
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("deeply").join("nested");
    let log = AuditLog::new(nested.join("audit.jsonl"));

    log.log_pre_dispatch("test:cmd", None, None, serde_json::Value::Null)
        .unwrap();

    assert!(nested.join("audit.jsonl").exists());
    let entries = log.read_all().unwrap();
    assert_eq!(entries.len(), 1);
}

// ---------------------------------------------------------------------------
// Serialization round-trip via JSON value
// ---------------------------------------------------------------------------

#[test]
fn entry_serialization_round_trip_via_json() {
    let tmp = TempDir::new().unwrap();
    let log = AuditLog::new(tmp.path().join("audit.jsonl"));

    log.log_pre_dispatch(
        "governance:check-gates",
        Some("spec-kitty"),
        Some("auth-flow"),
        serde_json::json!({"feature": "auth-flow"}),
    )
    .unwrap();

    let entries = log.read_all().unwrap();
    let entry = &entries[0];

    let json_str = serde_json::to_string(entry).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(v["command"], "governance:check-gates");
    assert_eq!(v["agent"], "spec-kitty");
    assert_eq!(v["feature_slug"], "auth-flow");
    assert_eq!(v["phase"], "predispatch");
    assert!(v["args"]["feature"].is_string());
}

// ---------------------------------------------------------------------------
// Post-dispatch with failure
// ---------------------------------------------------------------------------

#[test]
fn post_dispatch_failure_recorded() {
    let tmp = TempDir::new().unwrap();
    let log = AuditLog::new(tmp.path().join("audit.jsonl"));

    log.log_post_dispatch(
        "sync:push-plane",
        Some("sync-oracle"),
        Some("api-design"),
        false,
        "Plane.so API timeout",
        5000,
    )
    .unwrap();

    let entries = log.read_all().unwrap();
    assert_eq!(entries.len(), 1);
    let result = entries[0].result.as_ref().unwrap();
    assert!(!result.success);
    assert_eq!(result.message, "Plane.so API timeout");
    assert_eq!(result.duration_ms, 5000);
}

// ---------------------------------------------------------------------------
// Interleaved pre/post entries
// ---------------------------------------------------------------------------

#[test]
fn interleaved_pre_and_post_entries() {
    let tmp = TempDir::new().unwrap();
    let log = AuditLog::new(tmp.path().join("audit.jsonl"));

    log.log_pre_dispatch("cmd:a", None, None, serde_json::Value::Null)
        .unwrap();
    log.log_post_dispatch("cmd:a", None, None, true, "ok", 10)
        .unwrap();

    log.log_pre_dispatch("cmd:b", None, None, serde_json::Value::Null)
        .unwrap();
    log.log_post_dispatch("cmd:b", None, None, false, "fail", 20)
        .unwrap();

    let entries = log.read_all().unwrap();
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].command, "cmd:a");
    assert_eq!(entries[1].command, "cmd:a");
    assert_eq!(entries[2].command, "cmd:b");
    assert_eq!(entries[3].command, "cmd:b");
    assert!(entries[1].result.as_ref().unwrap().success);
    assert!(!entries[3].result.as_ref().unwrap().success);
}

// ---------------------------------------------------------------------------
// Optional fields are None when omitted
// ---------------------------------------------------------------------------

#[test]
fn optional_fields_are_none_when_omitted() {
    let tmp = TempDir::new().unwrap();
    let log = AuditLog::new(tmp.path().join("audit.jsonl"));

    log.log_pre_dispatch("cmd:x", None, None, serde_json::Value::Null)
        .unwrap();

    let entries = log.read_all().unwrap();
    let entry = &entries[0];
    assert!(entry.agent.is_none());
    assert!(entry.feature_slug.is_none());
    assert!(entry.result.is_none());
}
