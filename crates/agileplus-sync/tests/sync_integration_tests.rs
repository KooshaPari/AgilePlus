//! Integration tests for SyncError, SyncReport, ResolutionStrategy, and NatsSyncBridge.

use agileplus_sync::conflict::SyncConflict;
use agileplus_sync::error::SyncError;
use agileplus_sync::nats::{
    InboundSyncEvent, OutboundSyncCommand, SUBJECT_INBOUND, SUBJECT_OUTBOUND, STREAM_NAME,
};
use agileplus_sync::report::SyncReport;
use agileplus_sync::resolution::{apply_resolution, FieldSource, ResolutionStrategy};
use serde_json::json;

// ---------------------------------------------------------------------------
// SyncError Display and variant tests
// ---------------------------------------------------------------------------

#[test]
fn sync_error_store_display() {
    let e = SyncError::Store("db down".into());
    assert_eq!(e.to_string(), "Store error: db down");
}

#[test]
fn sync_error_resolution_failed_display() {
    let e = SyncError::ResolutionFailed("merge failed".into());
    assert_eq!(e.to_string(), "Resolution failed: merge failed");
}

#[test]
fn sync_error_conflict_detected_display() {
    let e = SyncError::ConflictDetected {
        entity_type: "feature".into(),
        entity_id: 42,
    };
    assert_eq!(e.to_string(), "Conflict detected for entity feature/42");
}

#[test]
fn sync_error_entity_not_found_display() {
    let e = SyncError::EntityNotFound {
        entity_type: "work_package".into(),
        entity_id: 99,
    };
    assert_eq!(e.to_string(), "Entity not found: work_package/99");
}

#[test]
fn sync_error_from_serde_json_error() {
    let parse_err = serde_json::from_str::<i32>("not a number").unwrap_err();
    let e: SyncError = parse_err.into();
    let msg = e.to_string();
    assert!(msg.contains("Serialization error"));
}

#[test]
fn sync_error_store_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SyncError>();
}

// ---------------------------------------------------------------------------
// SyncReport tests
// ---------------------------------------------------------------------------

#[test]
fn sync_report_new_is_empty() {
    let r = SyncReport::new();
    assert!(r.created.is_empty());
    assert!(r.updated.is_empty());
    assert!(r.skipped.is_empty());
    assert!(r.conflicts.is_empty());
    assert!(r.errors.is_empty());
}

#[test]
fn sync_report_default_is_empty() {
    let r = SyncReport::default();
    assert!(r.created.is_empty());
    assert!(r.total_processed() == 0);
}

#[test]
fn sync_report_total_processed_counts_all() {
    let mut r = SyncReport::new();
    r.created.push(("feature".into(), 1));
    r.created.push(("feature".into(), 2));
    r.updated.push(("work_package".into(), 10));
    r.skipped.push(("feature".into(), 3));

    // Add 2 conflicts and 1 error manually
    let c1 = SyncConflict::new("feature", 1, json!({"a":1}), json!({"a":2}));
    let c2 = SyncConflict::new("feature", 2, json!({"b":1}), json!({"b":2}));
    r.conflicts.push(c1);
    r.conflicts.push(c2);
    r.errors.push(SyncError::Store("x".into()));

    assert_eq!(r.total_processed(), 7);
}

#[test]
fn sync_report_total_processed_empty() {
    let r = SyncReport::new();
    assert_eq!(r.total_processed(), 0);
}

#[test]
fn sync_report_is_clean_true_for_empty() {
    let r = SyncReport::new();
    assert!(r.is_clean());
}

#[test]
fn sync_report_is_clean_with_only_created() {
    let mut r = SyncReport::new();
    r.created.push(("feature".into(), 1));
    r.updated.push(("feature".into(), 2));
    assert!(r.is_clean());
}

#[test]
fn sync_report_is_clean_false_with_conflict() {
    let mut r = SyncReport::new();
    let c = SyncConflict::new("feature", 1, json!({}), json!({}));
    r.conflicts.push(c);
    assert!(!r.is_clean());
}

#[test]
fn sync_report_is_clean_false_with_error() {
    let mut r = SyncReport::new();
    r.errors.push(SyncError::Store("x".into()));
    assert!(!r.is_clean());
}

#[test]
fn sync_report_is_clean_false_with_both() {
    let mut r = SyncReport::new();
    let c = SyncConflict::new("feature", 1, json!({}), json!({}));
    r.conflicts.push(c);
    r.errors.push(SyncError::ResolutionFailed("y".into()));
    assert!(!r.is_clean());
}

#[test]
fn sync_report_display_contains_counts() {
    let mut r = SyncReport::new();
    r.created.push(("feature".into(), 1));
    r.updated.push(("work_package".into(), 2));
    r.skipped.push(("feature".into(), 3));

    let display = format!("{}", r);
    assert!(display.contains("AgilePlus Sync Report"));
    assert!(display.contains("Created"));
    assert!(display.contains("Updated"));
    assert!(display.contains("Skipped"));
    assert!(display.contains("Conflicts"));
    assert!(display.contains("Errors"));
    assert!(display.contains("Duration"));
}

#[test]
fn sync_report_display_with_conflicts_and_errors() {
    let mut r = SyncReport::new();
    let c = SyncConflict::new("feature", 1, json!({}), json!({}));
    r.conflicts.push(c);
    r.errors.push(SyncError::ResolutionFailed("test".into()));

    let display = format!("{}", r);
    assert!(display.contains("0"));
}

#[test]
fn sync_report_display_clean_report() {
    let r = SyncReport::new();
    let display = format!("{}", r);
    assert!(display.contains("0"));
}

#[test]
fn sync_report_display_created_count() {
    let mut r = SyncReport::new();
    r.created.push(("a".into(), 1));
    r.created.push(("b".into(), 2));
    let display = format!("{}", r);
    assert!(display.contains("Created") && display.contains("2"));
}

#[test]
fn sync_report_display_updated_count() {
    let mut r = SyncReport::new();
    r.updated.push(("a".into(), 1));
    let display = format!("{}", r);
    assert!(display.contains("Updated") && display.contains("1"));
}

#[test]
fn sync_report_duration_is_settable() {
    let mut r = SyncReport::new();
    r.duration = std::time::Duration::from_secs(42);
    assert_eq!(r.duration, std::time::Duration::from_secs(42));
}

// ---------------------------------------------------------------------------
// ResolutionStrategy / FieldSource tests
// ---------------------------------------------------------------------------

#[test]
fn local_wins_resolution() {
    let conflict = SyncConflict::new("feature", 1, json!({"name": "local_name"}), json!({"name": "remote_name"}));
    let result = apply_resolution(&conflict, &ResolutionStrategy::LocalWins).unwrap();

    assert_eq!(result.resolved_value, json!({"name": "local_name"}));
    assert_eq!(result.strategy_label, "local_wins");
    assert!(!result.resolved_hash.is_empty());
}

#[test]
fn remote_wins_resolution() {
    let conflict = SyncConflict::new("feature", 1, json!({"name": "local_name"}), json!({"name": "remote_name"}));
    let result = apply_resolution(&conflict, &ResolutionStrategy::RemoteWins).unwrap();

    assert_eq!(result.resolved_value, json!({"name": "remote_name"}));
    assert_eq!(result.strategy_label, "remote_wins");
}

#[test]
fn manual_resolution() {
    let conflict = SyncConflict::new("feature", 1, json!({"name": "local_name"}), json!({"name": "remote_name"}));
    let strategy = ResolutionStrategy::Manual(json!({"name": "merged_name"}));
    let result = apply_resolution(&conflict, &strategy).unwrap();

    assert_eq!(result.resolved_value, json!({"name": "merged_name"}));
    assert_eq!(result.strategy_label, "manual");
}

#[test]
fn field_level_resolution_local_source() {
    let conflict = SyncConflict::new("feature", 1, json!({"a": "local_a", "b": "local_b"}), json!({"a": "remote_a", "b": "remote_b"}));

    let mut sources = std::collections::HashMap::new();
    sources.insert("a".to_string(), FieldSource::Local);
    sources.insert("b".to_string(), FieldSource::Remote);

    let strategy = ResolutionStrategy::FieldLevel(sources);
    let result = apply_resolution(&conflict, &strategy).unwrap();

    assert_eq!(result.resolved_value, json!({"a": "local_a", "b": "remote_b"}));
    assert_eq!(result.strategy_label, "field_level");
}

#[test]
fn field_level_resolution_falls_back_to_remote() {
    let conflict = SyncConflict::new("feature", 1, json!({"a": "local_a"}), json!({"a": "remote_a", "c": "remote_c"}));

    let mut sources = std::collections::HashMap::new();
    sources.insert("a".to_string(), FieldSource::Local);

    let strategy = ResolutionStrategy::FieldLevel(sources);
    let result = apply_resolution(&conflict, &strategy).unwrap();

    // "a" from local, "c" falls back to remote
    assert_eq!(result.resolved_value, json!({"a": "local_a", "c": "remote_c"}));
}

#[test]
fn field_level_resolution_all_local() {
    let conflict = SyncConflict::new("feature", 1, json!({"x": 1, "y": 2}), json!({"x": 10, "y": 20}));

    let mut sources = std::collections::HashMap::new();
    sources.insert("x".to_string(), FieldSource::Local);
    sources.insert("y".to_string(), FieldSource::Local);

    let strategy = ResolutionStrategy::FieldLevel(sources);
    let result = apply_resolution(&conflict, &strategy).unwrap();

    assert_eq!(result.resolved_value, json!({"x": 1, "y": 2}));
}

#[test]
fn resolution_result_has_valid_hash() {
    let conflict = SyncConflict::new("feature", 1, json!({"a": 1}), json!({"a": 2}));
    let result = apply_resolution(&conflict, &ResolutionStrategy::LocalWins).unwrap();

    // resolved_hash should match hash of resolved_value
    use agileplus_sync::conflict::hash_value;
    assert_eq!(result.resolved_hash, hash_value(&result.resolved_value));
}

#[test]
fn resolution_strategy_serde_roundtrip() {
    let strategies = [
        ("local_wins", ResolutionStrategy::LocalWins),
        ("remote_wins", ResolutionStrategy::RemoteWins),
        ("manual", ResolutionStrategy::Manual(json!({"key": "value"}))),
        ("field_level", ResolutionStrategy::FieldLevel({
            let mut m = std::collections::HashMap::new();
            m.insert("f".to_string(), FieldSource::Local);
            m
        })),
    ];
    for (tag, strategy) in &strategies {
        let json = serde_json::to_string(strategy).unwrap();
        let restored: ResolutionStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(
            *tag,
            serde_json::from_str::<serde_json::Value>(&serde_json::to_string(&restored).unwrap())
                .unwrap()["strategy"]
                .as_str()
                .unwrap()
        );
    }
}

#[test]
fn field_source_serde_roundtrip() {
    for source in [FieldSource::Local, FieldSource::Remote] {
        let json = serde_json::to_string(&source).unwrap();
        let restored: FieldSource = serde_json::from_str(&json).unwrap();
        assert_eq!(source, restored);
    }
}

// ---------------------------------------------------------------------------
// NatsSyncBridge constants tests
// ---------------------------------------------------------------------------

#[test]
fn subject_inbound_is_correct() {
    assert_eq!(SUBJECT_INBOUND, "agileplus.sync.plane.inbound");
}

#[test]
fn subject_outbound_is_correct() {
    assert_eq!(SUBJECT_OUTBOUND, "agileplus.sync.plane.outbound");
}

#[test]
fn stream_name_is_correct() {
    assert_eq!(STREAM_NAME, "AGILEPLUS_SYNC");
}

// ---------------------------------------------------------------------------
// OutboundSyncCommand / InboundSyncEvent serialization
// ---------------------------------------------------------------------------

#[test]
fn outbound_command_serialization() {
    let cmd = OutboundSyncCommand {
        entity_type: "feature".into(),
        entity_id: 42,
        operation: "update".into(),
        payload: json!({"name": "test"}),
    };
    let json = serde_json::to_string(&cmd).unwrap();
    let restored: OutboundSyncCommand = serde_json::from_str(&json).unwrap();

    assert_eq!(cmd.entity_type, restored.entity_type);
    assert_eq!(cmd.entity_id, restored.entity_id);
    assert_eq!(cmd.operation, restored.operation);
    assert_eq!(cmd.payload, restored.payload);
}

#[test]
fn inbound_event_serialization() {
    let evt = InboundSyncEvent {
        plane_issue_id: "PLN-123".into(),
        event_type: "issue.updated".into(),
        payload: json!({"title": "Test"}),
    };
    let json = serde_json::to_string(&evt).unwrap();
    let restored: InboundSyncEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(evt.plane_issue_id, restored.plane_issue_id);
    assert_eq!(evt.event_type, restored.event_type);
    assert_eq!(evt.payload, restored.payload);
}

#[test]
fn outbound_command_with_empty_payload() {
    let cmd = OutboundSyncCommand {
        entity_type: "feature".into(),
        entity_id: 1,
        operation: "create".into(),
        payload: json!(null),
    };
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("feature"));
    assert!(json.contains("create"));
}

#[test]
fn inbound_event_with_complex_payload() {
    let evt = InboundSyncEvent {
        plane_issue_id: "PLN-999".into(),
        event_type: "issue.created".into(),
        payload: json!({
            "title": "Test Issue",
            "labels": ["bug", "high-priority"],
            "metadata": {"created_by": "agent"}
        }),
    };
    let json = serde_json::to_string(&evt).unwrap();
    let restored: InboundSyncEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(evt.payload, restored.payload);
}

// ---------------------------------------------------------------------------
// SyncReport and ResolutionResult integration
// ---------------------------------------------------------------------------

#[test]
fn sync_report_with_resolution_results() {
    let mut r = SyncReport::new();
    let conflict = SyncConflict::new("feature", 1, json!({"a": 1}), json!({"a": 2}));
    r.conflicts.push(conflict);

    let result = apply_resolution(
        r.conflicts.first().unwrap(),
        &ResolutionStrategy::RemoteWins,
    ).unwrap();

    assert_eq!(result.resolved_value, json!({"a": 2}));
    assert_eq!(r.total_processed(), 1);
    assert!(!r.is_clean());
}

#[test]
fn resolution_result_full_structure() {
    let conflict = SyncConflict::new("feature", 1, json!({"name": "local"}), json!({"name": "remote"}));
    let result = apply_resolution(&conflict, &ResolutionStrategy::RemoteWins).unwrap();

    assert!(!result.resolved_value.is_null());
    assert!(!result.resolved_hash.is_empty());
    assert_eq!(result.resolved_hash.len(), 64);
    assert!(!result.strategy_label.is_empty());
}

// ---------------------------------------------------------------------------
// Edge case / stress tests
// ---------------------------------------------------------------------------

#[test]
fn sync_report_many_entities() {
    let mut r = SyncReport::new();
    for i in 0..100 {
        r.created.push((format!("feature"), i));
        r.updated.push((format!("work_package"), i));
        r.skipped.push((format!("feature"), i + 100));
    }
    assert_eq!(r.total_processed(), 300);
}

#[test]
fn sync_report_mixed_operations() {
    let mut r = SyncReport::new();
    r.created.push(("a".into(), 1));
    r.updated.push(("b".into(), 2));
    r.skipped.push(("c".into(), 3));
    r.errors.push(SyncError::Store("e1".into()));
    let c = SyncConflict::new("d", 4, json!({}), json!({}));
    r.conflicts.push(c);

    assert!(!r.is_clean());
    assert_eq!(r.total_processed(), 5);
}

#[test]
fn resolution_field_level_empty_map() {
    let conflict = SyncConflict::new("feature", 1, json!({"a": "local"}), json!({"a": "remote"}));
    let strategy = ResolutionStrategy::FieldLevel(std::collections::HashMap::new());
    let result = apply_resolution(&conflict, &strategy).unwrap();

    // Empty map → all fields fall back to remote
    assert_eq!(result.resolved_value, json!({"a": "remote"}));
}

#[test]
fn sync_error_from_serde_json_error_with_detail() {
    let err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
    let sync_err: SyncError = err.into();
    let msg = sync_err.to_string();
    assert!(msg.starts_with("Serialization error"));
}

#[test]
fn sync_report_display_no_panic_with_large_numbers() {
    let mut r = SyncReport::new();
    for i in 0..1000 {
        r.created.push((format!("feature"), i));
    }
    // Should not panic
    let _ = format!("{}", r);
}

#[test]
fn resolution_strategy_label_matches_variant() {
    let conflict = SyncConflict::new("feature", 1, json!({"k": "l"}), json!({"k": "r"}));

    let lw = apply_resolution(&conflict, &ResolutionStrategy::LocalWins).unwrap();
    assert_eq!(lw.strategy_label, "local_wins");

    let rw = apply_resolution(&conflict, &ResolutionStrategy::RemoteWins).unwrap();
    assert_eq!(rw.strategy_label, "remote_wins");

    let mw = apply_resolution(&conflict, &ResolutionStrategy::Manual(json!("x"))).unwrap();
    assert_eq!(mw.strategy_label, "manual");

    let fl = apply_resolution(&conflict, &ResolutionStrategy::FieldLevel({
        let mut m = std::collections::HashMap::new();
        m.insert("k".to_string(), FieldSource::Local);
        m
    })).unwrap();
    assert_eq!(fl.strategy_label, "field_level");
}