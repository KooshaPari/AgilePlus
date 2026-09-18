//! Snapshot management for fast aggregate loading.

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::store::EventStore;

#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("Snapshot not found for {entity_type}:{entity_id}")]
    NotFound { entity_type: String, entity_id: i64 },
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Invalid snapshot: {0}")]
    Invalid(String),
}

#[derive(Clone, Debug)]
pub struct SnapshotConfig {
    /// Create snapshot after this many events since the last snapshot.
    pub event_threshold: i64,
    /// Create snapshot after this many seconds since the last snapshot.
    pub time_threshold_secs: u64,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            event_threshold: 100,
            time_threshold_secs: 300,
        }
    }
}

#[async_trait]
pub trait SnapshotStore: Send + Sync {
    /// Save a snapshot.
    async fn save(&self, snapshot: &Snapshot) -> Result<(), SnapshotError>;

    /// Load the most recent snapshot for an entity.
    async fn load(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Option<Snapshot>, SnapshotError>;

    /// Delete snapshots older than a given sequence.
    async fn delete_before(
        &self,
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
    ) -> Result<(), SnapshotError>;
}

/// Determine whether a new snapshot should be created.
pub fn should_snapshot(
    config: &SnapshotConfig,
    current_sequence: i64,
    last_snapshot_sequence: i64,
    last_snapshot_time: Option<chrono::DateTime<Utc>>,
) -> bool {
    if current_sequence - last_snapshot_sequence >= config.event_threshold {
        return true;
    }
    if let Some(last_time) = last_snapshot_time {
        let elapsed = Utc::now().signed_duration_since(last_time);
        if elapsed > TimeDelta::seconds(config.time_threshold_secs as i64) {
            return true;
        }
    }
    false
}

/// State loaded from a snapshot plus events to replay.
pub struct LoadedState {
    pub snapshot: Option<Snapshot>,
    pub events_to_replay: Vec<Event>,
}

impl LoadedState {
    /// Load the latest snapshot and any events since it.
    pub async fn load<SS: SnapshotStore, ES: EventStore>(
        snapshot_store: &SS,
        event_store: &ES,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Self, SnapshotError> {
        let snapshot = snapshot_store.load(entity_type, entity_id).await?;

        let events_to_replay = if let Some(ref snap) = snapshot {
            event_store
                .get_events_since(entity_type, entity_id, snap.event_sequence)
                .await
                .map_err(|e| SnapshotError::StorageError(e.to_string()))?
        } else {
            event_store
                .get_events(entity_type, entity_id)
                .await
                .map_err(|e| SnapshotError::StorageError(e.to_string()))?
        };

        Ok(Self {
            snapshot,
            events_to_replay,
        })
    }
}

#[derive(Debug, Default)]
pub struct InMemorySnapshotStore {
    snapshots: RwLock<HashMap<(String, i64), Vec<Snapshot>>>,
}

impl InMemorySnapshotStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SnapshotStore for InMemorySnapshotStore {
    async fn save(&self, snapshot: &Snapshot) -> Result<(), SnapshotError> {
        self.snapshots
            .write()
            .await
            .entry((snapshot.entity_type.clone(), snapshot.entity_id))
            .or_default()
            .push(snapshot.clone());
        Ok(())
    }

    async fn load(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Option<Snapshot>, SnapshotError> {
        Ok(self
            .snapshots
            .read()
            .await
            .get(&(entity_type.to_string(), entity_id))
            .and_then(|snapshots| {
                snapshots
                    .iter()
                    .max_by_key(|snapshot| snapshot.event_sequence)
                    .cloned()
            }))
    }

    async fn delete_before(
        &self,
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
    ) -> Result<(), SnapshotError> {
        if let Some(snapshots) = self
            .snapshots
            .write()
            .await
            .get_mut(&(entity_type.to_string(), entity_id))
        {
            snapshots.retain(|snapshot| snapshot.event_sequence >= sequence);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(entity_type: &str, entity_id: i64, event_sequence: i64) -> Snapshot {
        Snapshot::new(
            entity_type,
            entity_id,
            serde_json::json!({ "state": "test" }),
            event_sequence,
        )
    }

    #[test]
    fn should_snapshot_event_threshold() {
        let config = SnapshotConfig {
            event_threshold: 100,
            time_threshold_secs: 300,
        };
        assert!(should_snapshot(&config, 100, 0, None));
        assert!(!should_snapshot(&config, 50, 0, None));
    }

    #[test]
    fn should_snapshot_time_threshold() {
        let config = SnapshotConfig {
            event_threshold: 100,
            time_threshold_secs: 300,
        };
        let old = Utc::now() - TimeDelta::seconds(400);
        assert!(should_snapshot(&config, 50, 0, Some(old)));
        assert!(!should_snapshot(&config, 50, 0, Some(Utc::now())));
    }

    #[tokio::test]
    async fn in_memory_snapshot_loads_latest() {
        let store = InMemorySnapshotStore::new();
        store.save(&make_snapshot("Feature", 1, 50)).await.unwrap();
        store.save(&make_snapshot("Feature", 1, 100)).await.unwrap();

        let loaded = store.load("Feature", 1).await.unwrap().unwrap();

        assert_eq!(loaded.event_sequence, 100);
    }

    #[tokio::test]
    async fn in_memory_snapshot_delete_before_keeps_newer_snapshots() {
        let store = InMemorySnapshotStore::new();
        store.save(&make_snapshot("Feature", 1, 50)).await.unwrap();
        store.save(&make_snapshot("Feature", 1, 100)).await.unwrap();
        store.save(&make_snapshot("Feature", 1, 150)).await.unwrap();

        store.delete_before("Feature", 1, 100).await.unwrap();
        let loaded = store.load("Feature", 1).await.unwrap().unwrap();

        assert_eq!(loaded.event_sequence, 150);
    }

    #[tokio::test]
    async fn in_memory_snapshot_returns_none_for_unknown_entity() {
        let store = InMemorySnapshotStore::new();

        assert!(store.load("Feature", 99).await.unwrap().is_none());
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::store::{EventError, InMemoryEventStore};

    fn snap(entity_type: &str, entity_id: i64, seq: i64) -> Snapshot {
        Snapshot::new(entity_type, entity_id, serde_json::json!({"v": seq}), seq)
    }

    fn ev(entity_type: &str, entity_id: i64, event_type: &str) -> Event {
        Event::new(entity_type, entity_id, event_type, serde_json::json!({}), "t")
    }

    #[test]
    fn snapshot_error_not_found_display() {
        let e = SnapshotError::NotFound { entity_type: "Feature".into(), entity_id: 3 };
        assert_eq!(e.to_string(), "Snapshot not found for Feature:3");
    }

    #[test]
    fn snapshot_error_storage_display() {
        assert_eq!(
            SnapshotError::StorageError("io".into()).to_string(),
            "Storage error: io"
        );
    }

    #[test]
    fn snapshot_error_invalid_display() {
        assert_eq!(SnapshotError::Invalid("bad".into()).to_string(), "Invalid snapshot: bad");
    }

    #[test]
    fn config_default_values() {
        let c = SnapshotConfig::default();
        assert_eq!(c.event_threshold, 100);
        assert_eq!(c.time_threshold_secs, 300);
    }

    #[test]
    fn config_clone_preserves_fields() {
        let c = SnapshotConfig { event_threshold: 5, time_threshold_secs: 9 };
        let d = c.clone();
        assert_eq!(d.event_threshold, 5);
        assert_eq!(d.time_threshold_secs, 9);
    }

    #[test]
    fn should_snapshot_event_threshold_true() {
        let c = SnapshotConfig { event_threshold: 10, time_threshold_secs: 300 };
        assert!(should_snapshot(&c, 10, 0, None));
    }

    #[test]
    fn should_snapshot_event_threshold_false() {
        let c = SnapshotConfig { event_threshold: 10, time_threshold_secs: 300 };
        assert!(!should_snapshot(&c, 9, 0, None));
    }

    #[test]
    fn should_snapshot_time_threshold_true() {
        let c = SnapshotConfig { event_threshold: 100, time_threshold_secs: 60 };
        let old = Utc::now() - TimeDelta::seconds(120);
        assert!(should_snapshot(&c, 1, 0, Some(old)));
    }

    #[test]
    fn should_snapshot_time_threshold_false() {
        let c = SnapshotConfig { event_threshold: 100, time_threshold_secs: 600 };
        assert!(!should_snapshot(&c, 1, 0, Some(Utc::now())));
    }

    #[test]
    fn should_snapshot_with_none_time_ignores_time_rule() {
        let c = SnapshotConfig { event_threshold: 100, time_threshold_secs: 0 };
        assert!(!should_snapshot(&c, 1, 0, None));
    }

    #[test]
    fn should_snapshot_event_threshold_short_circuits() {
        let c = SnapshotConfig { event_threshold: 1, time_threshold_secs: 999_999 };
        assert!(should_snapshot(&c, 5, 4, None));
    }

    #[tokio::test]
    async fn save_and_load_latest_by_sequence() {
        let store = InMemorySnapshotStore::new();
        store.save(&snap("F", 1, 10)).await.unwrap();
        store.save(&snap("F", 1, 30)).await.unwrap();
        store.save(&snap("F", 1, 20)).await.unwrap();
        let loaded = store.load("F", 1).await.unwrap().unwrap();
        assert_eq!(loaded.event_sequence, 30);
    }

    #[tokio::test]
    async fn load_unknown_returns_none() {
        let store = InMemorySnapshotStore::new();
        assert!(store.load("F", 9).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn delete_before_removes_older_keeps_newer() {
        let store = InMemorySnapshotStore::new();
        store.save(&snap("F", 1, 10)).await.unwrap();
        store.save(&snap("F", 1, 20)).await.unwrap();
        store.delete_before("F", 1, 20).await.unwrap();
        let loaded = store.load("F", 1).await.unwrap().unwrap();
        assert_eq!(loaded.event_sequence, 20);
    }

    #[tokio::test]
    async fn delete_before_unknown_is_noop() {
        let store = InMemorySnapshotStore::new();
        store.delete_before("F", 404, 1).await.unwrap();
        assert!(store.load("F", 404).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn entities_are_independent() {
        let store = InMemorySnapshotStore::new();
        store.save(&snap("F", 1, 5)).await.unwrap();
        store.save(&snap("F", 2, 7)).await.unwrap();
        assert_eq!(store.load("F", 1).await.unwrap().unwrap().event_sequence, 5);
        assert_eq!(store.load("F", 2).await.unwrap().unwrap().event_sequence, 7);
    }

    #[tokio::test]
    async fn new_store_is_empty() {
        let store = InMemorySnapshotStore::new();
        assert!(store.load("X", 1).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn default_store_matches_new() {
        let store = InMemorySnapshotStore::default();
        assert!(store.load("X", 1).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn loaded_state_without_snapshot_returns_all_events() {
        let snaps = InMemorySnapshotStore::new();
        let events = InMemoryEventStore::new();
        events.append(&ev("F", 1, "a")).await.unwrap();
        events.append(&ev("F", 1, "b")).await.unwrap();

        let state = LoadedState::load(&snaps, &events, "F", 1).await.unwrap();
        assert!(state.snapshot.is_none());
        assert_eq!(state.events_to_replay.len(), 2);
    }

    #[tokio::test]
    async fn loaded_state_with_snapshot_returns_newer_events_only() {
        let snaps = InMemorySnapshotStore::new();
        let events = InMemoryEventStore::new();
        events.append(&ev("F", 1, "a")).await.unwrap();
        events.append(&ev("F", 1, "b")).await.unwrap();
        events.append(&ev("F", 1, "c")).await.unwrap();
        snaps.save(&snap("F", 1, 1)).await.unwrap();

        let state = LoadedState::load(&snaps, &events, "F", 1).await.unwrap();
        assert!(state.snapshot.is_some());
        assert_eq!(state.events_to_replay.len(), 2);
        assert!(state.events_to_replay.iter().all(|e| e.sequence > 1));
    }

    #[tokio::test]
    async fn loaded_state_empty_when_nothing_stored() {
        let snaps = InMemorySnapshotStore::new();
        let events = InMemoryEventStore::new();
        let state = LoadedState::load(&snaps, &events, "F", 1).await.unwrap();
        assert!(state.snapshot.is_none());
        assert!(state.events_to_replay.is_empty());
    }

    /// Snapshot store whose every operation fails, used to exercise the
    /// error arms of `LoadedState::load` that the in-memory store never hits.
    struct FailingSnapshotStore;

    #[async_trait]
    impl SnapshotStore for FailingSnapshotStore {
        async fn save(&self, _snapshot: &Snapshot) -> Result<(), SnapshotError> {
            Err(SnapshotError::StorageError("snapshot write failed".into()))
        }

        async fn load(
            &self,
            entity_type: &str,
            entity_id: i64,
        ) -> Result<Option<Snapshot>, SnapshotError> {
            Err(SnapshotError::StorageError(format!(
                "snapshot read failed for {entity_type}:{entity_id}"
            )))
        }

        async fn delete_before(
            &self,
            _entity_type: &str,
            _entity_id: i64,
            _sequence: i64,
        ) -> Result<(), SnapshotError> {
            Err(SnapshotError::StorageError("snapshot delete failed".into()))
        }
    }

    /// Event store whose reads always fail.
    struct FailingEventStore;

    #[async_trait]
    impl EventStore for FailingEventStore {
        async fn append(&self, _event: &Event) -> Result<i64, EventError> {
            Err(EventError::StorageError("append failed".into()))
        }

        async fn get_events(
            &self,
            entity_type: &str,
            entity_id: i64,
        ) -> Result<Vec<Event>, EventError> {
            Err(EventError::StorageError(format!(
                "event read failed for {entity_type}:{entity_id}"
            )))
        }

        async fn get_events_since(
            &self,
            entity_type: &str,
            entity_id: i64,
            _sequence: i64,
        ) -> Result<Vec<Event>, EventError> {
            Err(EventError::StorageError(format!(
                "event read-since failed for {entity_type}:{entity_id}"
            )))
        }

        async fn get_events_by_range(
            &self,
            entity_type: &str,
            entity_id: i64,
            _from: chrono::DateTime<Utc>,
            _to: chrono::DateTime<Utc>,
        ) -> Result<Vec<Event>, EventError> {
            Err(EventError::StorageError(format!(
                "event range read failed for {entity_type}:{entity_id}"
            )))
        }

        async fn get_latest_sequence(
            &self,
            entity_type: &str,
            entity_id: i64,
        ) -> Result<i64, EventError> {
            Err(EventError::StorageError(format!(
                "event latest failed for {entity_type}:{entity_id}"
            )))
        }
    }

    #[tokio::test]
    async fn loaded_state_propagates_snapshot_store_error() {
        let events = InMemoryEventStore::new();
        let err = load_error(&FailingSnapshotStore, &events, "F", 1).await;
        match err {
            SnapshotError::StorageError(msg) => {
                assert_eq!(msg, "snapshot read failed for F:1");
            }
            other => panic!("expected StorageError, got {other:?}"),
        }
    }

    async fn load_error(
        snaps: &impl SnapshotStore,
        events: &impl EventStore,
        entity_type: &str,
        entity_id: i64,
    ) -> SnapshotError {
        match LoadedState::load(snaps, events, entity_type, entity_id).await {
            Ok(_) => panic!("expected LoadedState::load to fail"),
            Err(err) => err,
        }
    }

    #[tokio::test]
    async fn loaded_state_wraps_event_store_error_when_no_snapshot() {
        let snaps = InMemorySnapshotStore::new();
        let err = load_error(&snaps, &FailingEventStore, "F", 1).await;
        match err {
            SnapshotError::StorageError(msg) => {
                assert_eq!(msg, "Storage error: event read failed for F:1");
            }
            other => panic!("expected StorageError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn loaded_state_wraps_event_store_error_when_snapshot_present() {
        let snaps = InMemorySnapshotStore::new();
        snaps.save(&snap("F", 2, 7)).await.unwrap();

        let err = load_error(&snaps, &FailingEventStore, "F", 2).await;
        match err {
            SnapshotError::StorageError(msg) => {
                assert_eq!(msg, "Storage error: event read-since failed for F:2");
            }
            other => panic!("expected StorageError, got {other:?}"),
        }
    }

    #[test]
    fn should_snapshot_false_when_sequence_did_not_advance_enough() {
        let c = SnapshotConfig {
            event_threshold: 10,
            time_threshold_secs: 300,
        };
        assert!(!should_snapshot(&c, 5, 5, None), "no movement");
        assert!(!should_snapshot(&c, 4, 10, None), "sequence moved backwards");
        assert!(should_snapshot(&c, 15, 5, None), "moved by exactly the threshold");
    }

    #[test]
    fn should_snapshot_false_when_last_snapshot_time_is_in_the_future() {
        let c = SnapshotConfig {
            event_threshold: 100,
            time_threshold_secs: 60,
        };
        let future = Utc::now() + TimeDelta::seconds(3600);
        assert!(!should_snapshot(&c, 1, 0, Some(future)));
    }
}
