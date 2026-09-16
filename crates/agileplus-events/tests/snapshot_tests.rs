//! Integration tests for snapshot management — config, store, and LoadedState.

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_events::snapshot::{
    InMemorySnapshotStore, LoadedState, SnapshotConfig, SnapshotStore, should_snapshot,
};
use agileplus_events::store::{EventStore, InMemoryEventStore};
use chrono::{Duration, Utc};

// ── should_snapshot ─────────────────────────────────────────────────────────

#[test]
fn should_snapshot_true_when_event_threshold_reached() {
    let config = SnapshotConfig {
        event_threshold: 10,
        time_threshold_secs: 300,
    };
    assert!(should_snapshot(&config, 10, 0, None));
    assert!(should_snapshot(&config, 50, 40, None));
}

#[test]
fn should_snapshot_false_when_below_event_threshold() {
    let config = SnapshotConfig {
        event_threshold: 100,
        time_threshold_secs: 300,
    };
    assert!(!should_snapshot(&config, 50, 0, None));
    assert!(!should_snapshot(&config, 99, 0, None));
}

#[test]
fn should_snapshot_true_when_time_threshold_exceeded() {
    let config = SnapshotConfig {
        event_threshold: 100,
        time_threshold_secs: 300,
    };
    let old_time = Utc::now() - Duration::seconds(400);
    assert!(should_snapshot(&config, 5, 0, Some(old_time)));
}

#[test]
fn should_snapshot_false_when_time_not_exceeded() {
    let config = SnapshotConfig {
        event_threshold: 100,
        time_threshold_secs: 300,
    };
    let recent = Utc::now() - Duration::seconds(10);
    assert!(!should_snapshot(&config, 5, 0, Some(recent)));
}

#[test]
fn should_snapshot_event_threshold_takes_priority() {
    let config = SnapshotConfig {
        event_threshold: 5,
        time_threshold_secs: 300,
    };
    // Recent time, but event threshold is met
    let recent = Utc::now();
    assert!(should_snapshot(&config, 10, 4, Some(recent)));
}

#[test]
fn should_snapshot_no_time_when_none() {
    let config = SnapshotConfig {
        event_threshold: 100,
        time_threshold_secs: 300,
    };
    // Below event threshold, no time → false
    assert!(!should_snapshot(&config, 50, 0, None));
}

// ── SnapshotConfig defaults ─────────────────────────────────────────────────

#[test]
fn snapshot_config_default_values() {
    let config = SnapshotConfig::default();
    assert_eq!(config.event_threshold, 100);
    assert_eq!(config.time_threshold_secs, 300);
}

#[test]
fn snapshot_config_clone() {
    let config = SnapshotConfig {
        event_threshold: 50,
        time_threshold_secs: 60,
    };
    let cloned = config.clone();
    assert_eq!(cloned.event_threshold, 50);
    assert_eq!(cloned.time_threshold_secs, 60);
}

// ── InMemorySnapshotStore ──────────────────────────────────────────────────

fn make_snapshot(entity_type: &str, entity_id: i64, sequence: i64) -> Snapshot {
    Snapshot::new(
        entity_type,
        entity_id,
        serde_json::json!({"state": "test", "seq": sequence}),
        sequence,
    )
}

#[tokio::test]
async fn snapshot_store_save_and_load_latest() {
    let store = InMemorySnapshotStore::new();
    store.save(&make_snapshot("Feature", 1, 10)).await.unwrap();
    store.save(&make_snapshot("Feature", 1, 20)).await.unwrap();
    store.save(&make_snapshot("Feature", 1, 30)).await.unwrap();

    let loaded = store.load("Feature", 1).await.unwrap().unwrap();
    assert_eq!(loaded.event_sequence, 30);
}

#[tokio::test]
async fn snapshot_store_load_returns_none_for_unknown() {
    let store = InMemorySnapshotStore::new();
    assert!(store.load("Feature", 999).await.unwrap().is_none());
}

#[tokio::test]
async fn snapshot_store_load_returns_none_for_empty() {
    let store = InMemorySnapshotStore::new();
    store.save(&make_snapshot("Feature", 1, 10)).await.unwrap();
    assert!(store.load("WorkPackage", 1).await.unwrap().is_none());
}

#[tokio::test]
async fn snapshot_store_delete_before_removes_old() {
    let store = InMemorySnapshotStore::new();
    store.save(&make_snapshot("Feature", 1, 10)).await.unwrap();
    store.save(&make_snapshot("Feature", 1, 20)).await.unwrap();
    store.save(&make_snapshot("Feature", 1, 30)).await.unwrap();

    store.delete_before("Feature", 1, 20).await.unwrap();

    let loaded = store.load("Feature", 1).await.unwrap().unwrap();
    assert_eq!(loaded.event_sequence, 30);
}

#[tokio::test]
async fn snapshot_store_delete_before_keeps_all_when_none_old() {
    let store = InMemorySnapshotStore::new();
    store.save(&make_snapshot("Feature", 1, 10)).await.unwrap();
    store.save(&make_snapshot("Feature", 1, 20)).await.unwrap();

    store.delete_before("Feature", 1, 5).await.unwrap();

    let loaded = store.load("Feature", 1).await.unwrap().unwrap();
    assert_eq!(loaded.event_sequence, 20);
}

#[tokio::test]
async fn snapshot_store_delete_before_no_panic_on_unknown() {
    let store = InMemorySnapshotStore::new();
    store.delete_before("Feature", 999, 10).await.unwrap();
}

#[tokio::test]
async fn snapshot_store_independent_entities() {
    let store = InMemorySnapshotStore::new();
    store.save(&make_snapshot("Feature", 1, 10)).await.unwrap();
    store.save(&make_snapshot("WorkPackage", 1, 20)).await.unwrap();

    let f = store.load("Feature", 1).await.unwrap().unwrap();
    let w = store.load("WorkPackage", 1).await.unwrap().unwrap();

    assert_eq!(f.event_sequence, 10);
    assert_eq!(w.event_sequence, 20);
}

#[tokio::test]
async fn snapshot_store_delete_all_removes_entity() {
    let store = InMemorySnapshotStore::new();
    store.save(&make_snapshot("Feature", 1, 10)).await.unwrap();
    store.save(&make_snapshot("Feature", 1, 20)).await.unwrap();

    store.delete_before("Feature", 1, 1).await.unwrap();

    // After deleting everything with seq >= 1 (all of them), should be None
    // Actually delete_before removes snapshots where event_sequence < sequence
    // So delete_before with sequence=100 removes everything with seq < 100
    store.delete_before("Feature", 1, 100).await.unwrap();
    assert!(store.load("Feature", 1).await.unwrap().is_none());
}

// ── LoadedState ─────────────────────────────────────────────────────────────

fn make_event(entity_type: &str, entity_id: i64, seq: i64) -> Event {
    Event {
        id: seq,
        entity_type: entity_type.into(),
        entity_id,
        event_type: "created".into(),
        payload: serde_json::json!({"seq": seq}),
        actor: "test".into(),
        timestamp: Utc::now(),
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        sequence: seq,
    }
}

#[tokio::test]
async fn loaded_state_without_snapshot_returns_all_events() {
    let snap_store = InMemorySnapshotStore::new();
    let event_store = InMemoryEventStore::new();

    event_store.append(&make_event("Feature", 1, 1)).await.unwrap();
    event_store.append(&make_event("Feature", 1, 2)).await.unwrap();
    event_store.append(&make_event("Feature", 1, 3)).await.unwrap();

    let state = LoadedState::load(&snap_store, &event_store, "Feature", 1).await.unwrap();

    assert!(state.snapshot.is_none());
    assert_eq!(state.events_to_replay.len(), 3);
}

#[tokio::test]
async fn loaded_state_with_snapshot_returns_only_newer_events() {
    let snap_store = InMemorySnapshotStore::new();
    let event_store = InMemoryEventStore::new();

    // Store events 1-5
    for i in 1..=5 {
        event_store.append(&make_event("Feature", 1, i)).await.unwrap();
    }

    // Create snapshot at sequence 3
    snap_store
        .save(&make_snapshot("Feature", 1, 3))
        .await
        .unwrap();

    let state = LoadedState::load(&snap_store, &event_store, "Feature", 1).await.unwrap();

    assert!(state.snapshot.is_some());
    assert_eq!(state.snapshot.as_ref().unwrap().event_sequence, 3);
    // Should only get events after sequence 3 (events 4 and 5)
    assert_eq!(state.events_to_replay.len(), 2);
    assert_eq!(state.events_to_replay[0].sequence, 4);
    assert_eq!(state.events_to_replay[1].sequence, 5);
}

#[tokio::test]
async fn loaded_state_with_snapshot_no_events_since() {
    let snap_store = InMemorySnapshotStore::new();
    let event_store = InMemoryEventStore::new();

    event_store.append(&make_event("Feature", 1, 1)).await.unwrap();

    snap_store
        .save(&make_snapshot("Feature", 1, 1))
        .await
        .unwrap();

    let state = LoadedState::load(&snap_store, &event_store, "Feature", 1).await.unwrap();

    assert!(state.snapshot.is_some());
    assert!(state.events_to_replay.is_empty());
}

#[tokio::test]
async fn loaded_state_no_events_no_snapshot() {
    let snap_store = InMemorySnapshotStore::new();
    let event_store = InMemoryEventStore::new();

    let state = LoadedState::load(&snap_store, &event_store, "Feature", 1).await.unwrap();

    assert!(state.snapshot.is_none());
    assert!(state.events_to_replay.is_empty());
}
