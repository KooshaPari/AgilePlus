//! Integration tests for measurement collection and data generation.
//!
//! Tests the helper functions that produce measured data for benchmarks:
//! event generation, feature creation, sync payloads, and aggregate counting.

use agileplus_benchmarks::helpers::*;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_events::replay_events;
use agileplus_sqlite::repository::events as event_repo;

// ---------------------------------------------------------------------------
// Event measurement collection
// ---------------------------------------------------------------------------

#[test]
fn event_measurement_sequential_access_pattern() {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    let count = 500_i64;

    // Append events
    for seq in 1..=count {
        let ev = make_event(1, seq);
        event_repo::append_event(&conn, &ev).expect("append");
    }

    // Read them back sequentially (mirrors benchmark measurement)
    let retrieved = event_repo::get_events(&conn, "Feature", 1).expect("get");
    assert_eq!(retrieved.len(), count as usize);

    // Verify sequence order
    for (i, ev) in retrieved.iter().enumerate() {
        assert_eq!(ev.sequence, (i as i64) + 1);
    }
}

#[test]
fn event_measurement_multi_entity_access_pattern() {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    let entity_count = 20_i64;
    let events_per_entity = 25_i64;

    // Append events spread across entities
    for seq in 1..=events_per_entity {
        for entity_id in 1..=entity_count {
            let ev = make_event(entity_id, seq);
            event_repo::append_event(&conn, &ev).expect("append");
        }
    }

    // Measure retrieval per entity
    for entity_id in 1..=entity_count {
        let retrieved =
            event_repo::get_events(&conn, "Feature", entity_id).expect("get");
        assert_eq!(retrieved.len(), events_per_entity as usize);
    }
}

#[test]
fn event_measurement_since_cutoff() {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");

    for seq in 1..=100 {
        let ev = make_event(1, seq);
        event_repo::append_event(&conn, &ev).expect("append");
    }

    // Measure delta events since various cutoffs
    for &cutoff in &[0, 25, 50, 75, 90, 99] {
        let delta = event_repo::get_events_since(&conn, "Feature", 1, cutoff)
            .expect("get_since");
        let expected = 100 - cutoff;
        assert_eq!(
            delta.len(),
            expected as usize,
            "cutoff={cutoff} expected {expected} events"
        );
    }
}

// ---------------------------------------------------------------------------
// Feature measurement collection
// ---------------------------------------------------------------------------

#[test]
fn feature_measurement_list_all() {
    use agileplus_sqlite::repository::features as feat_repo;

    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");

    for i in 1..=50 {
        let f = make_feature(i);
        feat_repo::create_feature(&conn, &f).expect("create");
    }

    let features = feat_repo::list_all_features(&conn).expect("list");
    assert_eq!(features.len(), 50);
}

#[test]
fn feature_measurement_get_by_id() {
    use agileplus_sqlite::repository::features as feat_repo;

    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");

    for i in 1..=20 {
        let f = make_feature(i);
        feat_repo::create_feature(&conn, &f).expect("create");
    }

    // Measure individual lookups
    for id in 1..=20 {
        let f = feat_repo::get_feature_by_id(&conn, id).expect("get");
        assert!(f.is_some());
        assert_eq!(f.unwrap().id, id);
    }
}

#[test]
fn feature_measurement_state_transitions() {
    use agileplus_sqlite::repository::features as feat_repo;

    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    let f = make_feature(1);
    feat_repo::create_feature(&conn, &f).expect("create");

    // Measure state transitions
    feat_repo::update_feature_state(&conn, 1, FeatureState::Specified).expect("transition");
    let f = feat_repo::get_feature_by_id(&conn, 1).expect("get").unwrap();
    assert_eq!(f.state, FeatureState::Specified);

    // Reset for next measurement
    feat_repo::update_feature_state(&conn, 1, FeatureState::Created).expect("reset");
    let f = feat_repo::get_feature_by_id(&conn, 1).expect("get").unwrap();
    assert_eq!(f.state, FeatureState::Created);
}

// ---------------------------------------------------------------------------
// Aggregate replay measurement
// ---------------------------------------------------------------------------

#[tokio::test]
async fn replay_measurement_count_accuracy() {
    let mut agg = CountingAggregate::default();
    let events = make_events(500);

    replay_events(&mut agg, &events).await.unwrap();

    // Verify exact count
    assert_eq!(agg.events_applied, 500);
    assert_eq!(agg.version, 500);
}

#[tokio::test]
async fn replay_measurement_state_tracking() {
    use agileplus_domain::domain::event::Event;

    let mut agg = CountingAggregate::default();

    // Create events with alternating states
    let events: Vec<Event> = (1..=10)
        .map(|seq| {
            let state = if seq % 2 == 0 { "Shipped" } else { "Created" };
            Event {
                id: 0,
                entity_type: "Feature".to_string(),
                entity_id: 1,
                event_type: "StateTransitioned".to_string(),
                payload: serde_json::json!({"state": state}),
                actor: "test".to_string(),
                timestamp: chrono::Utc::now(),
                prev_hash: [0u8; 32],
                hash: [0u8; 32],
                sequence: seq,
            }
        })
        .collect();

    replay_events(&mut agg, &events).await.unwrap();

    assert_eq!(agg.events_applied, 10);
    assert_eq!(agg.version, 10);
    // Last event has seq=10 (even) → "Shipped"
    assert_eq!(agg.last_state, "Shipped");
}

// ---------------------------------------------------------------------------
// Snapshot measurement helpers
// ---------------------------------------------------------------------------

#[test]
fn snapshot_measurement_various_sequences() {
    for &seq in &[0, 1, 100, 500, 1000, 10_000] {
        let snap = make_snapshot(1, seq);
        assert_eq!(snap.event_sequence, seq);
        assert_eq!(snap.entity_id, 1);
    }
}

#[test]
fn snapshot_measurement_various_entities() {
    for &eid in &[1, 5, 10, 100] {
        let snap = make_snapshot(eid, 1);
        assert_eq!(snap.entity_id, eid);
    }
}

// ---------------------------------------------------------------------------
// Sync payload measurement collection
// ---------------------------------------------------------------------------

#[test]
fn sync_measurement_payload_size() {
    let p = SyncPayload::new(1);
    // Description is 256 bytes, plus slug and state
    assert!(p.description.len() >= 256);
}

#[test]
fn sync_measurement_batch_sizes() {
    for &n in &[1_i64, 10, 50, 100, 200] {
        let payloads = make_sync_payloads(n);
        assert_eq!(payloads.len(), n as usize);
        // All should have consistent payload size
        for p in &payloads {
            assert_eq!(p.description.len(), 256);
        }
    }
}

#[test]
fn sync_measurement_roundtrip_json_size() {
    let p = SyncPayload::new(1);
    let json = serde_json::json!({
        "id":          p.id,
        "slug":        p.slug,
        "state":       format!("{:?}", p.state),
        "description": p.description,
    });
    let json_str = json.to_string();
    // JSON should be larger than raw fields due to escaping and keys
    assert!(json_str.len() > p.description.len());
}
