//! Integration tests for benchmark helper functions.
//!
//! Complements the unit tests in `helpers.rs` with cross-cutting scenarios:
//! edge cases, field preservation, state machine alignment, and composition.

use agileplus_benchmarks::helpers::*;
use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::state_machine::FeatureState;

// ---------------------------------------------------------------------------
// make_event edge cases
// ---------------------------------------------------------------------------

#[test]
fn make_event_zero_ids() {
    let e = make_event(0, 0);
    assert_eq!(e.entity_id, 0);
    assert_eq!(e.sequence, 0);
    assert_eq!(e.entity_type, "Feature");
    assert_eq!(e.event_type, "StateTransitioned");
    assert_eq!(e.actor, "bench-agent");
}

#[test]
fn make_event_large_ids() {
    let e = make_event(i64::MAX, i64::MAX - 1);
    assert_eq!(e.entity_id, i64::MAX);
    assert_eq!(e.sequence, i64::MAX - 1);
}

#[test]
fn make_event_payload_has_state() {
    let e = make_event(42, 7);
    let state = e.payload.get("state").and_then(|v| v.as_str());
    assert_eq!(state, Some("Specified"));
    let seq = e.payload.get("sequence").and_then(|v| v.as_i64());
    assert_eq!(seq, Some(7));
}

#[test]
fn make_event_timestamp_is_recent() {
    use chrono::Utc;
    let before = Utc::now();
    let e = make_event(1, 1);
    let after = Utc::now();
    assert!(e.timestamp >= before && e.timestamp <= after);
}

#[test]
fn make_event_hash_fields_zeroed() {
    let e = make_event(1, 1);
    assert_eq!(e.prev_hash, [0u8; 32]);
    assert_eq!(e.hash, [0u8; 32]);
}

// ---------------------------------------------------------------------------
// make_events sequencing
// ---------------------------------------------------------------------------

#[test]
fn make_events_empty() {
    let events = make_events(0);
    assert!(events.is_empty());
}

#[test]
fn make_events_single() {
    let events = make_events(1);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].sequence, 1);
    assert_eq!(events[0].entity_id, 1);
}

#[test]
fn make_events_sequences_are_sequential() {
    let events = make_events(50);
    for (i, e) in events.iter().enumerate() {
        assert_eq!(e.sequence, (i as i64) + 1);
        assert_eq!(e.entity_id, 1);
    }
}

#[test]
fn make_events_all_same_entity() {
    let events = make_events(200);
    for e in &events {
        assert_eq!(e.entity_id, 1);
        assert_eq!(e.entity_type, "Feature");
    }
}

// ---------------------------------------------------------------------------
// make_events_multi_entity distribution
// ---------------------------------------------------------------------------

#[test]
fn make_events_multi_entity_zero() {
    let events = make_events_multi_entity(0, 5);
    assert!(events.is_empty());
}

#[test]
fn make_events_multi_entity_one_entity() {
    let events = make_events_multi_entity(10, 1);
    assert_eq!(events.len(), 10);
    for e in &events {
        assert_eq!(e.entity_id, 1);
    }
}

#[test]
fn make_events_multi_entity_distribution() {
    let events = make_events_multi_entity(20, 5);
    assert_eq!(events.len(), 20);
    // Each entity should get exactly 4 events (20 / 5 = 4)
    for entity_id in 1..=5 {
        let count = events.iter().filter(|e| e.entity_id == entity_id).count();
        assert_eq!(count, 4, "entity {entity_id} should have 4 events");
    }
}

#[test]
fn make_events_multi_entity_sequences_restart_per_entity() {
    let events = make_events_multi_entity(12, 3);
    // entity 1: sequences 1,2,3,4
    // entity 2: sequences 1,2,3,4
    // entity 3: sequences 1,2,3,4
    for entity_id in 1..=3 {
        let mut seqs: Vec<i64> = events
            .iter()
            .filter(|e| e.entity_id == entity_id)
            .map(|e| e.sequence)
            .collect();
        seqs.sort();
        assert_eq!(seqs, vec![1, 2, 3, 4]);
    }
}

// ---------------------------------------------------------------------------
// make_feature / make_features
// ---------------------------------------------------------------------------

#[test]
fn make_feature_slug_matches_id() {
    let f = make_feature(999);
    assert_eq!(f.slug, "feature-999");
    assert_eq!(f.id, 999);
}

#[test]
fn make_feature_zero() {
    let f = make_feature(0);
    assert_eq!(f.id, 0);
    assert_eq!(f.slug, "feature-0");
}

#[test]
fn make_features_range() {
    let features = make_features(10);
    assert_eq!(features.len(), 10);
    for (i, f) in features.iter().enumerate() {
        assert_eq!(f.id, (i as i64) + 1);
    }
}

#[test]
fn make_features_empty() {
    let features = make_features(0);
    assert!(features.is_empty());
}

// ---------------------------------------------------------------------------
// CountingAggregate
// ---------------------------------------------------------------------------

#[tokio::test]
async fn counting_aggregate_default() {
    let agg = CountingAggregate::default();
    assert_eq!(agg.version, 0);
    assert_eq!(agg.events_applied, 0);
    assert_eq!(agg.last_state, "");
}

#[tokio::test]
async fn counting_aggregate_apply_single() {
    use agileplus_events::Aggregate;

    let mut agg = CountingAggregate::default();
    let e = make_event(1, 1);
    agg.apply(&e).await.unwrap();
    assert_eq!(agg.events_applied, 1);
    assert_eq!(agg.version, 1);
    assert_eq!(agg.last_state, "Specified");
}

#[tokio::test]
async fn counting_aggregate_apply_sequence() {
    use agileplus_events::Aggregate;

    let mut agg = CountingAggregate::default();
    for seq in 1..=100 {
        let e = make_event(1, seq);
        agg.apply(&e).await.unwrap();
    }
    assert_eq!(agg.events_applied, 100);
    assert_eq!(agg.version, 100);
}

#[tokio::test]
async fn counting_aggregate_version_trait() {
    use agileplus_events::Aggregate;

    let mut agg = CountingAggregate::default();
    assert_eq!(agg.version(), 0);

    agg.set_version(42);
    assert_eq!(agg.version(), 42);
    assert_eq!(agg.version, 42);
}

#[tokio::test]
async fn counting_aggregate_multiple_entity_payloads() {
    use agileplus_events::Aggregate;

    let mut agg = CountingAggregate::default();

    let e1 = Event {
        id: 0,
        entity_type: "Feature".to_string(),
        entity_id: 1,
        event_type: "StateTransitioned".to_string(),
        payload: serde_json::json!({"state": "Created"}),
        actor: "test".to_string(),
        timestamp: chrono::Utc::now(),
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        sequence: 1,
    };
    agg.apply(&e1).await.unwrap();
    assert_eq!(agg.last_state, "Created");

    let e2 = Event {
        id: 0,
        entity_type: "Feature".to_string(),
        entity_id: 1,
        event_type: "StateTransitioned".to_string(),
        payload: serde_json::json!({"state": "Shipped"}),
        actor: "test".to_string(),
        timestamp: chrono::Utc::now(),
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        sequence: 2,
    };
    agg.apply(&e2).await.unwrap();
    assert_eq!(agg.last_state, "Shipped");
    assert_eq!(agg.version, 2);
}

// ---------------------------------------------------------------------------
// Snapshot construction
// ---------------------------------------------------------------------------

#[test]
fn make_snapshot_fields() {
    let snap = make_snapshot(5, 100);
    assert_eq!(snap.entity_id, 5);
    assert_eq!(snap.event_sequence, 100);
    assert_eq!(snap.entity_type, "Feature");
}

#[test]
fn make_snapshot_state_json() {
    let snap = make_snapshot(1, 50);
    let state = &snap.state;
    assert_eq!(state["events_applied"], 50);
    assert_eq!(state["last_state"], "Specified");
}

#[test]
fn make_snapshot_zero_sequence() {
    let snap = make_snapshot(1, 0);
    assert_eq!(snap.event_sequence, 0);
}

// ---------------------------------------------------------------------------
// SyncPayload construction and roundtrip
// ---------------------------------------------------------------------------

#[test]
fn sync_payload_new_fields() {
    let p = SyncPayload::new(42);
    assert_eq!(p.id, 42);
    assert_eq!(p.slug, "feature-42");
    assert_eq!(p.state, FeatureState::Specified);
    assert_eq!(p.description.len(), 256);
}

#[test]
fn sync_payload_zero_id() {
    let p = SyncPayload::new(0);
    assert_eq!(p.id, 0);
    assert_eq!(p.slug, "feature-0");
}

#[test]
fn sync_payload_clone() {
    let p = SyncPayload::new(10);
    let cloned = p.clone();
    assert_eq!(cloned.id, p.id);
    assert_eq!(cloned.slug, p.slug);
    assert_eq!(cloned.description, p.description);
}

#[test]
fn make_sync_payloads_count_range() {
    let payloads = make_sync_payloads(0);
    assert!(payloads.is_empty());

    let payloads = make_sync_payloads(1);
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0].id, 1);

    let payloads = make_sync_payloads(200);
    assert_eq!(payloads.len(), 200);
    assert_eq!(payloads.last().unwrap().id, 200);
}

#[test]
fn simulate_roundtrip_idempotent() {
    let p = SyncPayload::new(7);
    let out1 = simulate_sync_roundtrip(&p);
    let out2 = simulate_sync_roundtrip(&out1);
    assert_eq!(out1.id, out2.id);
    assert_eq!(out1.slug, out2.slug);
    assert_eq!(out1.description, out2.description);
}

#[test]
fn simulate_roundtrip_large_payload() {
    let mut p = SyncPayload::new(1);
    p.description = "x".repeat(10_000);
    let out = simulate_sync_roundtrip(&p);
    assert_eq!(out.description.len(), 10_000);
    assert_eq!(out.description, p.description);
}

#[test]
fn simulate_roundtrip_all_fields_preserved() {
    let p = SyncPayload::new(99);
    let out = simulate_sync_roundtrip(&p);
    assert_eq!(out.id, 99);
    assert_eq!(out.slug, "feature-99");
    assert_eq!(out.state, FeatureState::Specified);
    assert_eq!(out.description, p.description);
}

// ---------------------------------------------------------------------------
// in-memory adapter
// ---------------------------------------------------------------------------

#[test]
fn in_memory_adapter_creates_valid_connection() {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    let result: i64 = conn.query_row("SELECT 42", [], |row| row.get(0)).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn in_memory_adapter_sequential_connections() {
    let adapter = make_in_memory_adapter();
    // First connection: write + read
    {
        let conn = adapter.conn_for_bench().expect("conn1");
        conn.execute("CREATE TABLE t (x INTEGER)", []).unwrap();
        conn.execute("INSERT INTO t VALUES (1)", []).unwrap();
    }
    // Second connection: read back the data
    {
        let conn = adapter.conn_for_bench().expect("conn2");
        let result: i64 = conn.query_row("SELECT x FROM t", [], |row| row.get(0)).unwrap();
        assert_eq!(result, 1);
    }
}
