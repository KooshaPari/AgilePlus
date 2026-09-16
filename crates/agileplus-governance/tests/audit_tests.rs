//! Integration tests for AuditEvent, AuditFilter, and AuditLogger.
//! Complements the inline unit tests in src/audit.rs.

use agileplus_governance::*;
use agileplus_governance::audit::{AuditEvent, AuditFilter, AuditLogger};
use chrono::Utc;

fn temp_logger() -> (AuditLogger, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let logger = AuditLogger::new(&path, 90).unwrap();
    (logger, dir)
}

// ── AuditEvent builders ──────────────────────────────────────────────

#[test]
fn audit_event_new_sets_defaults() {
    let event = AuditEvent::new("promote", LogLevel::Info, OperationResult::Success);
    assert!(event.id.starts_with("evt_"));
    assert_eq!(event.action, "promote");
    assert_eq!(event.level, LogLevel::Info);
    assert_eq!(event.result, OperationResult::Success);
    assert!(event.message.is_none());
    assert!(event.user_id.is_none());
    assert!(event.category.is_none());
}

#[test]
fn audit_event_success_factory() {
    let event = AuditEvent::success("deploy");
    assert_eq!(event.level, LogLevel::Info);
    assert_eq!(event.result, OperationResult::Success);
    assert_eq!(event.action, "deploy");
}

#[test]
fn audit_event_warn_factory() {
    let event = AuditEvent::warn("slow_query");
    assert_eq!(event.level, LogLevel::Warn);
    assert_eq!(event.result, OperationResult::Success);
}

#[test]
fn audit_event_error_factory() {
    let event = AuditEvent::error("db_insert", "connection timeout");
    assert_eq!(event.level, LogLevel::Error);
    assert_eq!(event.result, OperationResult::Failure);
    assert_eq!(event.message.as_deref(), Some("connection timeout"));
}

#[test]
fn audit_event_builder_chain() {
    let event = AuditEvent::success("test")
        .with_action("override_action")
        .with_message("test message")
        .with_category(ActionCategory::Release)
        .with_user("user-1")
        .with_client_ip("127.0.0.1")
        .with_user_agent("Mozilla/5.0")
        .with_request("POST", "/api/v1/promote", Some("req-123".into()))
        .with_resource("package", Some("pkg-456".into()))
        .with_parameters(serde_json::json!({"key": "value"}))
        .with_duration(150)
        .with_metadata(serde_json::json!({"extra": "data"}))
        .with_error("ERR_001", "something went wrong")
        .with_result(OperationResult::PartialSuccess);

    assert_eq!(event.action, "override_action");
    assert_eq!(event.message.as_deref(), Some("test message"));
    assert_eq!(event.category, Some(ActionCategory::Release));
    assert_eq!(event.user_id.as_deref(), Some("user-1"));
    assert_eq!(event.client_ip.as_deref(), Some("127.0.0.1"));
    assert_eq!(event.user_agent.as_deref(), Some("Mozilla/5.0"));
    assert_eq!(event.method.as_deref(), Some("POST"));
    assert_eq!(event.endpoint.as_deref(), Some("/api/v1/promote"));
    assert_eq!(event.request_id.as_deref(), Some("req-123"));
    assert_eq!(event.resource.as_deref(), Some("package"));
    assert_eq!(event.resource_id.as_deref(), Some("pkg-456"));
    assert!(event.parameters.is_some());
    assert_eq!(event.duration_ms, Some(150));
    assert!(event.metadata.is_some());
    assert_eq!(event.error_code.as_deref(), Some("ERR_001"));
    assert_eq!(event.error_message.as_deref(), Some("something went wrong"));
    assert_eq!(event.result, OperationResult::PartialSuccess);
}

#[test]
fn audit_event_request_generates_id_when_none() {
    let event = AuditEvent::success("test")
        .with_request("GET", "/health", None);
    assert!(event.request_id.is_some());
    assert!(event.request_id.unwrap().starts_with("req_"));
}

// ── AuditEvent serde ─────────────────────────────────────────────────

#[test]
fn audit_event_serde_roundtrip() {
    let event = AuditEvent::success("test")
        .with_message("msg")
        .with_user("user1")
        .with_category(ActionCategory::Policy);
    let json = serde_json::to_string(&event).unwrap();
    let back: AuditEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(back.action, "test");
    assert_eq!(back.message.as_deref(), Some("msg"));
    assert_eq!(back.user_id.as_deref(), Some("user1"));
    assert_eq!(back.category, Some(ActionCategory::Policy));
}

// ── AuditFilter ──────────────────────────────────────────────────────

#[test]
fn audit_filter_default() {
    let filter = AuditFilter::default();
    assert!(filter.action.is_none());
    assert!(filter.category.is_none());
    assert!(filter.level.is_none());
    assert!(filter.result.is_none());
    assert!(filter.user_id.is_none());
    assert!(filter.unsynced_only == false);
}

#[test]
fn audit_filter_new_sets_default_limits() {
    let filter = AuditFilter::new();
    assert_eq!(filter.limit, 100);
    assert_eq!(filter.offset, 0);
}

#[test]
fn audit_filter_builder_chain() {
    let start = Utc::now();
    let end = Utc::now();
    let filter = AuditFilter::new()
        .action("promote")
        .user("user-1")
        .time_range(start, end)
        .unsynced()
        .limit(50);

    assert_eq!(filter.action.as_deref(), Some("promote"));
    assert_eq!(filter.user_id.as_deref(), Some("user-1"));
    assert!(filter.start_time.is_some());
    assert!(filter.end_time.is_some());
    assert!(filter.unsynced_only);
    assert_eq!(filter.limit, 50);
}

// ── AuditLogger CRUD ─────────────────────────────────────────────────

#[test]
fn audit_logger_create_and_query() {
    let (logger, _dir) = temp_logger();
    let event = AuditEvent::success("test_action");
    logger.log(&event).unwrap();

    let results = logger.query(&AuditFilter::new()).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].action, "test_action");
}

#[test]
fn audit_logger_multiple_events() {
    let (logger, _dir) = temp_logger();
    logger.log(&AuditEvent::success("action1")).unwrap();
    logger.log(&AuditEvent::success("action2")).unwrap();
    logger.log(&AuditEvent::error("action3", "err")).unwrap();

    let results = logger.query(&AuditFilter::new()).unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
fn audit_logger_query_filter_by_action() {
    let (logger, _dir) = temp_logger();
    logger.log(&AuditEvent::success("deploy")).unwrap();
    logger.log(&AuditEvent::success("promote")).unwrap();
    logger.log(&AuditEvent::success("deploy")).unwrap();

    let filter = AuditFilter::new().action("deploy");
    let results = logger.query(&filter).unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|e| e.action == "deploy"));
}

#[test]
fn audit_logger_query_filter_by_user() {
    let (logger, _dir) = temp_logger();
    logger
        .log(&AuditEvent::success("a").with_user("alice"))
        .unwrap();
    logger
        .log(&AuditEvent::success("b").with_user("bob"))
        .unwrap();
    logger
        .log(&AuditEvent::success("c").with_user("alice"))
        .unwrap();

    let filter = AuditFilter::new().user("alice");
    let results = logger.query(&filter).unwrap();
    assert_eq!(results.len(), 2);
    assert!(results
        .iter()
        .all(|e| e.user_id.as_deref() == Some("alice")));
}

#[test]
fn audit_logger_query_with_limit() {
    let (logger, _dir) = temp_logger();
    for i in 0..10 {
        logger
            .log(&AuditEvent::success(format!("action_{i}")))
            .unwrap();
    }

    let filter = AuditFilter::new().limit(3);
    let results = logger.query(&filter).unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
fn audit_logger_query_empty_on_fresh_db() {
    let (logger, _dir) = temp_logger();
    let results = logger.query(&AuditFilter::new()).unwrap();
    assert!(results.is_empty());
}

#[test]
fn audit_logger_file_based() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test_audit.db");
    let logger = AuditLogger::new(&db_path, 90).unwrap();
    logger.log(&AuditEvent::success("file_event")).unwrap();

    let results = logger.query(&AuditFilter::new()).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].action, "file_event");
}

#[test]
fn audit_logger_with_retention() {
    let (logger, _dir) = temp_logger();
    // Just verify it doesn't panic with a custom retention
    logger.log(&AuditEvent::success("retention_test")).unwrap();
    let results = logger.query(&AuditFilter::new()).unwrap();
    assert_eq!(results.len(), 1);
}

// ── AuditEvent timestamps ────────────────────────────────────────────

#[test]
fn audit_event_timestamps_are_set() {
    let before = Utc::now();
    let event = AuditEvent::success("ts_test");
    let after = Utc::now();

    assert!(event.timestamp >= before);
    assert!(event.timestamp <= after);
    assert!(event.created_at >= before);
    assert!(event.created_at <= after);
}

#[test]
fn audit_event_synced_at_initially_none() {
    let event = AuditEvent::success("sync_test");
    assert!(event.synced_at.is_none());
}
