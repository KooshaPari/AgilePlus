//! Integration tests: event store repository and sync mappings.

use agileplus_domain::domain::{
    event::Event,
    sync_mapping::{SyncDirection, SyncMapping},
};
use agileplus_sqlite::{
    repository::{events, sync_mappings},
    SqliteStorageAdapter,
};

fn adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().unwrap()
}

fn event(entity_type: &str, entity_id: i64, event_type: &str, sequence: i64) -> Event {
    Event {
        id: 0,
        entity_type: entity_type.into(),
        entity_id,
        event_type: event_type.into(),
        payload: serde_json::json!({"seq": sequence}),
        actor: "tester".into(),
        timestamp: chrono::Utc::now(),
        prev_hash: [0u8; 32],
        hash: [sequence as u8; 32],
        sequence,
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[test]
fn event_append_returns_positive_id() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = events::append_event(&conn, &event("Feature", 1, "created", 1)).unwrap();
    assert!(id > 0);
}

#[test]
fn event_get_orders_by_sequence() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    events::append_event(&conn, &event("Feature", 1, "created", 2)).unwrap();
    events::append_event(&conn, &event("Feature", 1, "created", 1)).unwrap();

    let got = events::get_events(&conn, "Feature", 1).unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].sequence, 1);
    assert_eq!(got[1].sequence, 2);
}

#[test]
fn event_scoped_to_entity() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    events::append_event(&conn, &event("Feature", 1, "created", 1)).unwrap();
    events::append_event(&conn, &event("Feature", 2, "created", 1)).unwrap();
    events::append_event(&conn, &event("WorkPackage", 1, "created", 1)).unwrap();

    assert_eq!(events::get_events(&conn, "Feature", 1).unwrap().len(), 1);
    assert_eq!(events::get_events(&conn, "Feature", 2).unwrap().len(), 1);
    assert_eq!(events::get_events(&conn, "WorkPackage", 1).unwrap().len(), 1);
    assert!(events::get_events(&conn, "Feature", 99).unwrap().is_empty());
}

#[test]
fn event_since_filters_strictly_greater() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    for seq in 1..=3 {
        events::append_event(&conn, &event("Feature", 1, "tick", seq)).unwrap();
    }
    let got = events::get_events_since(&conn, "Feature", 1, 1).unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].sequence, 2);
    assert_eq!(got[1].sequence, 3);
    // Since the latest returns nothing.
    assert!(events::get_events_since(&conn, "Feature", 1, 3).unwrap().is_empty());
}

#[test]
fn event_by_range_inclusive_bounds() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let base = chrono::Utc::now();
    for seq in 1..=5 {
        let mut e = event("Feature", 1, "tick", seq);
        e.timestamp = base + chrono::Duration::seconds(seq);
        events::append_event(&conn, &e).unwrap();
    }
    let from = (base + chrono::Duration::seconds(2)).to_rfc3339();
    let to = (base + chrono::Duration::seconds(4)).to_rfc3339();
    let got = events::get_events_by_range(&conn, "Feature", 1, &from, &to).unwrap();
    assert_eq!(got.len(), 3);
    assert_eq!(got.first().unwrap().sequence, 2);
    assert_eq!(got.last().unwrap().sequence, 4);
}

#[test]
fn event_by_range_empty_when_outside() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut e = event("Feature", 1, "tick", 1);
    e.timestamp = chrono::Utc::now();
    events::append_event(&conn, &e).unwrap();

    let from = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339();
    let to = (chrono::Utc::now() + chrono::Duration::days(2)).to_rfc3339();
    assert!(events::get_events_by_range(&conn, "Feature", 1, &from, &to)
        .unwrap()
        .is_empty());
}

#[test]
fn event_latest_sequence_zero_when_empty() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert_eq!(events::get_latest_sequence(&conn, "Feature", 1).unwrap(), 0);
}

#[test]
fn event_latest_sequence_returns_max() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    events::append_event(&conn, &event("Feature", 1, "tick", 4)).unwrap();
    events::append_event(&conn, &event("Feature", 1, "tick", 9)).unwrap();
    events::append_event(&conn, &event("Feature", 1, "tick", 2)).unwrap();
    assert_eq!(events::get_latest_sequence(&conn, "Feature", 1).unwrap(), 9);
}

#[test]
fn event_payload_roundtrips() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut e = event("Feature", 1, "created", 1);
    e.payload = serde_json::json!({"name": "auth", "nested": {"k": [1, 2, 3]}});
    events::append_event(&conn, &e).unwrap();
    let got = events::get_events(&conn, "Feature", 1).unwrap();
    assert_eq!(got[0].payload["name"], "auth");
    assert_eq!(got[0].payload["nested"]["k"][1], 2);
}

#[test]
fn event_hashes_roundtrip_as_bytes() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut e = event("Feature", 1, "created", 1);
    e.prev_hash = [7u8; 32];
    e.hash = [9u8; 32];
    events::append_event(&conn, &e).unwrap();
    let got = events::get_events(&conn, "Feature", 1).unwrap();
    assert_eq!(got[0].prev_hash, [7u8; 32]);
    assert_eq!(got[0].hash, [9u8; 32]);
}

#[test]
fn event_duplicate_sequence_rejected() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    events::append_event(&conn, &event("Feature", 1, "created", 1)).unwrap();
    assert!(events::append_event(&conn, &event("Feature", 1, "other", 1)).is_err());
}

#[test]
fn event_actor_roundtrips() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut e = event("Feature", 1, "created", 1);
    e.actor = "agent-42".into();
    events::append_event(&conn, &e).unwrap();
    assert_eq!(events::get_events(&conn, "Feature", 1).unwrap()[0].actor, "agent-42");
}

// ---------------------------------------------------------------------------
// Sync mappings
// ---------------------------------------------------------------------------

fn mapping(entity_type: &str, entity_id: i64, plane: &str) -> SyncMapping {
    SyncMapping::new(entity_type, entity_id, plane, "hash-abc")
}

#[test]
fn sync_mapping_upsert_and_get() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    sync_mappings::upsert_sync_mapping(&conn, &mapping("feature", 1, "plane-1")).unwrap();

    let got = sync_mappings::get_sync_mapping(&conn, "feature", 1)
        .unwrap()
        .unwrap();
    assert_eq!(got.entity_type, "feature");
    assert_eq!(got.entity_id, 1);
    assert_eq!(got.plane_issue_id, "plane-1");
    assert_eq!(got.content_hash, "hash-abc");
    assert_eq!(got.sync_direction, SyncDirection::Bidirectional);
    assert_eq!(got.conflict_count, 0);
}

#[test]
fn sync_mapping_get_missing() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(sync_mappings::get_sync_mapping(&conn, "feature", 1).unwrap().is_none());
}

#[test]
fn sync_mapping_upsert_updates_existing() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    sync_mappings::upsert_sync_mapping(&conn, &mapping("feature", 1, "plane-1")).unwrap();

    let mut updated = mapping("feature", 1, "plane-1");
    updated.content_hash = "hash-xyz".into();
    updated.sync_direction = SyncDirection::Push;
    updated.conflict_count = 3;
    sync_mappings::upsert_sync_mapping(&conn, &updated).unwrap();

    let got = sync_mappings::get_sync_mapping(&conn, "feature", 1)
        .unwrap()
        .unwrap();
    assert_eq!(got.content_hash, "hash-xyz");
    assert_eq!(got.sync_direction, SyncDirection::Push);
    assert_eq!(got.conflict_count, 3);
}

#[test]
fn sync_mapping_get_by_plane_id() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    sync_mappings::upsert_sync_mapping(&conn, &mapping("feature", 42, "plane-99")).unwrap();
    let got = sync_mappings::get_sync_mapping_by_plane_id(&conn, "feature", "plane-99")
        .unwrap()
        .unwrap();
    assert_eq!(got.entity_id, 42);
    assert!(sync_mappings::get_sync_mapping_by_plane_id(&conn, "feature", "nope")
        .unwrap()
        .is_none());
}

#[test]
fn sync_mapping_all_directions_roundtrip() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    for (i, dir) in [
        SyncDirection::Push,
        SyncDirection::Pull,
        SyncDirection::Bidirectional,
    ]
    .into_iter()
    .enumerate()
    {
        let mut m = mapping("feature", i as i64, &format!("plane-{i}"));
        m.sync_direction = dir;
        sync_mappings::upsert_sync_mapping(&conn, &m).unwrap();
        let got = sync_mappings::get_sync_mapping(&conn, "feature", i as i64)
            .unwrap()
            .unwrap();
        assert_eq!(got.sync_direction, dir);
    }
}

#[test]
fn sync_mapping_delete_removes_row() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    sync_mappings::upsert_sync_mapping(&conn, &mapping("feature", 1, "plane-1")).unwrap();
    sync_mappings::delete_sync_mapping(&conn, "feature", 1).unwrap();
    assert!(sync_mappings::get_sync_mapping(&conn, "feature", 1).unwrap().is_none());
}

#[test]
fn sync_mapping_delete_missing_is_ok() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    sync_mappings::delete_sync_mapping(&conn, "feature", 999).unwrap();
}

#[test]
fn sync_mapping_plane_issue_id_is_unique() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    sync_mappings::upsert_sync_mapping(&conn, &mapping("feature", 1, "plane-shared")).unwrap();
    // Different entity but same plane id -> UNIQUE violation.
    assert!(sync_mappings::upsert_sync_mapping(&conn, &mapping("feature", 2, "plane-shared")).is_err());
}

#[test]
fn sync_mapping_separate_entity_types_coexist() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    sync_mappings::upsert_sync_mapping(&conn, &mapping("feature", 1, "plane-f1")).unwrap();
    sync_mappings::upsert_sync_mapping(&conn, &mapping("work_package", 1, "plane-w1")).unwrap();
    assert!(sync_mappings::get_sync_mapping(&conn, "feature", 1).unwrap().is_some());
    assert!(sync_mappings::get_sync_mapping(&conn, "work_package", 1).unwrap().is_some());
}
