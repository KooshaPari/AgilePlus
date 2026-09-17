//! EventStore trait — async append-only event storage.

use agileplus_domain::domain::event::Event;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event not found: {0}")]
    NotFound(String),
    #[error("Duplicate sequence: {0}")]
    DuplicateSequence(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Invalid hash: {0}")]
    InvalidHash(String),
    #[error("Sequence gap: expected {expected}, got {actual}")]
    SequenceGap { expected: i64, actual: i64 },
}

#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append a new event; returns the assigned sequence number.
    async fn append(&self, event: &Event) -> Result<i64, EventError>;

    /// All events for an entity, ascending by sequence.
    async fn get_events(&self, entity_type: &str, entity_id: i64)
    -> Result<Vec<Event>, EventError>;

    /// Events from a specific sequence onward (exclusive).
    async fn get_events_since(
        &self,
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
    ) -> Result<Vec<Event>, EventError>;

    /// Events within a time range.
    async fn get_events_by_range(
        &self,
        entity_type: &str,
        entity_id: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Event>, EventError>;

    /// Latest event sequence number for an entity (0 if none).
    async fn get_latest_sequence(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<i64, EventError>;

    /// Events by event type, ascending by sequence.
    async fn get_events_by_type(&self, event_type: &str) -> Result<Vec<Event>, EventError> {
        Err(EventError::StorageError(format!(
            "get_events_by_type is not implemented for {event_type}"
        )))
    }
}

#[derive(Debug, Default)]
pub struct InMemoryEventStore {
    events: RwLock<Vec<Event>>,
    sequences: RwLock<HashMap<(String, i64), i64>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append(&self, event: &Event) -> Result<i64, EventError> {
        let key = (event.entity_type.clone(), event.entity_id);
        let sequence = {
            let mut sequences = self.sequences.write().await;
            let next = sequences.get(&key).copied().unwrap_or(0) + 1;
            sequences.insert(key, next);
            next
        };

        let mut stored = event.clone();
        stored.sequence = sequence;
        self.events.write().await.push(stored);
        Ok(sequence)
    }

    async fn get_events(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Vec<Event>, EventError> {
        let mut events: Vec<_> = self
            .events
            .read()
            .await
            .iter()
            .filter(|event| event.entity_type == entity_type && event.entity_id == entity_id)
            .cloned()
            .collect();
        events.sort_by_key(|event| event.sequence);
        Ok(events)
    }

    async fn get_events_since(
        &self,
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
    ) -> Result<Vec<Event>, EventError> {
        Ok(self
            .get_events(entity_type, entity_id)
            .await?
            .into_iter()
            .filter(|event| event.sequence > sequence)
            .collect())
    }

    async fn get_events_by_range(
        &self,
        entity_type: &str,
        entity_id: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Event>, EventError> {
        Ok(self
            .get_events(entity_type, entity_id)
            .await?
            .into_iter()
            .filter(|event| event.timestamp >= from && event.timestamp <= to)
            .collect())
    }

    async fn get_latest_sequence(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<i64, EventError> {
        Ok(self
            .sequences
            .read()
            .await
            .get(&(entity_type.to_string(), entity_id))
            .copied()
            .unwrap_or(0))
    }

    async fn get_events_by_type(&self, event_type: &str) -> Result<Vec<Event>, EventError> {
        let mut events: Vec<_> = self
            .events
            .read()
            .await
            .iter()
            .filter(|event| event.event_type == event_type)
            .cloned()
            .collect();
        events.sort_by_key(|event| event.sequence);
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(entity_type: &str, entity_id: i64, event_type: &str) -> Event {
        Event::new(
            entity_type,
            entity_id,
            event_type,
            serde_json::json!({}),
            "test",
        )
    }

    #[tokio::test]
    async fn in_memory_append_assigns_per_entity_sequence() {
        let store = InMemoryEventStore::new();

        assert_eq!(
            store.append(&event("Feature", 1, "created")).await.unwrap(),
            1
        );
        assert_eq!(
            store.append(&event("Feature", 1, "updated")).await.unwrap(),
            2
        );
        assert_eq!(
            store.append(&event("Feature", 2, "created")).await.unwrap(),
            1
        );

        assert_eq!(store.get_latest_sequence("Feature", 1).await.unwrap(), 2);
        assert_eq!(store.get_latest_sequence("Feature", 2).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn in_memory_get_events_since_filters_sequence() {
        let store = InMemoryEventStore::new();
        store.append(&event("Feature", 1, "created")).await.unwrap();
        store.append(&event("Feature", 1, "updated")).await.unwrap();

        let events = store.get_events_since("Feature", 1, 1).await.unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "updated");
    }

    #[tokio::test]
    async fn in_memory_get_events_by_type_filters_type() {
        let store = InMemoryEventStore::new();
        store.append(&event("Feature", 1, "created")).await.unwrap();
        store
            .append(&event("WorkPackage", 1, "created"))
            .await
            .unwrap();
        store.append(&event("Feature", 1, "updated")).await.unwrap();

        let events = store.get_events_by_type("created").await.unwrap();

        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|event| event.event_type == "created"));
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use chrono::Duration;

    fn ev(entity_type: &str, entity_id: i64, event_type: &str) -> Event {
        Event::new(entity_type, entity_id, event_type, serde_json::json!({}), "tester")
    }

    #[test]
    fn event_error_not_found_display() {
        assert_eq!(EventError::NotFound("x".into()).to_string(), "Event not found: x");
    }

    #[test]
    fn event_error_duplicate_sequence_display() {
        assert_eq!(EventError::DuplicateSequence("5".into()).to_string(), "Duplicate sequence: 5");
    }

    #[test]
    fn event_error_storage_display() {
        assert_eq!(EventError::StorageError("boom".into()).to_string(), "Storage error: boom");
    }

    #[test]
    fn event_error_invalid_hash_display() {
        assert_eq!(EventError::InvalidHash("bad".into()).to_string(), "Invalid hash: bad");
    }

    #[test]
    fn event_error_sequence_gap_display() {
        let e = EventError::SequenceGap { expected: 2, actual: 4 };
        assert_eq!(e.to_string(), "Sequence gap: expected 2, got 4");
    }

    #[tokio::test]
    async fn append_assigns_sequence_per_entity() {
        let store = InMemoryEventStore::new();
        assert_eq!(store.append(&ev("F", 1, "a")).await.unwrap(), 1);
        assert_eq!(store.append(&ev("F", 1, "b")).await.unwrap(), 2);
        assert_eq!(store.append(&ev("F", 2, "a")).await.unwrap(), 1);
        assert_eq!(store.append(&ev("G", 1, "a")).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn append_does_not_mutate_input_sequence() {
        let store = InMemoryEventStore::new();
        let event = ev("F", 1, "a");
        store.append(&event).await.unwrap();
        assert_eq!(event.sequence, 0, "caller's event must be untouched");
    }

    #[tokio::test]
    async fn get_events_empty_for_unknown_entity() {
        let store = InMemoryEventStore::new();
        assert!(store.get_events("F", 99).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn get_events_returns_ascending_sequence() {
        let store = InMemoryEventStore::new();
        store.append(&ev("F", 1, "a")).await.unwrap();
        store.append(&ev("F", 1, "b")).await.unwrap();
        store.append(&ev("F", 1, "c")).await.unwrap();
        let events = store.get_events("F", 1).await.unwrap();
        assert_eq!(events.len(), 3);
        assert!(events.windows(2).all(|w| w[0].sequence < w[1].sequence));
    }

    #[tokio::test]
    async fn get_events_filters_by_entity() {
        let store = InMemoryEventStore::new();
        store.append(&ev("F", 1, "a")).await.unwrap();
        store.append(&ev("F", 2, "a")).await.unwrap();
        assert_eq!(store.get_events("F", 1).await.unwrap().len(), 1);
        assert_eq!(store.get_events("F", 2).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn get_events_since_is_exclusive() {
        let store = InMemoryEventStore::new();
        store.append(&ev("F", 1, "a")).await.unwrap();
        store.append(&ev("F", 1, "b")).await.unwrap();
        let after = store.get_events_since("F", 1, 1).await.unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].event_type, "b");
    }

    #[tokio::test]
    async fn get_events_since_beyond_latest_is_empty() {
        let store = InMemoryEventStore::new();
        store.append(&ev("F", 1, "a")).await.unwrap();
        assert!(store.get_events_since("F", 1, 10).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn get_events_by_range_includes_matching_timestamps() {
        let store = InMemoryEventStore::new();
        let mut old = ev("F", 1, "old");
        old.timestamp = Utc::now() - Duration::days(10);
        let mut mid = ev("F", 1, "mid");
        mid.timestamp = Utc::now() - Duration::days(5);
        store.append(&old).await.unwrap();
        store.append(&mid).await.unwrap();

        let from = Utc::now() - Duration::days(7);
        let to = Utc::now();
        let got = store.get_events_by_range("F", 1, from, to).await.unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].event_type, "mid");
    }

    #[tokio::test]
    async fn get_events_by_range_excludes_outside() {
        let store = InMemoryEventStore::new();
        let mut old = ev("F", 1, "old");
        old.timestamp = Utc::now() - Duration::days(30);
        store.append(&old).await.unwrap();
        let from = Utc::now() - Duration::days(1);
        let got = store.get_events_by_range("F", 1, from, Utc::now()).await.unwrap();
        assert!(got.is_empty());
    }

    #[tokio::test]
    async fn get_latest_sequence_zero_when_none() {
        let store = InMemoryEventStore::new();
        assert_eq!(store.get_latest_sequence("F", 1).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn get_latest_sequence_tracks_appends() {
        let store = InMemoryEventStore::new();
        store.append(&ev("F", 1, "a")).await.unwrap();
        store.append(&ev("F", 1, "b")).await.unwrap();
        assert_eq!(store.get_latest_sequence("F", 1).await.unwrap(), 2);
        assert_eq!(store.get_latest_sequence("F", 2).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn get_events_by_type_filters_and_sorts() {
        let store = InMemoryEventStore::new();
        store.append(&ev("F", 1, "created")).await.unwrap();
        store.append(&ev("F", 1, "updated")).await.unwrap();
        store.append(&ev("W", 2, "created")).await.unwrap();
        let created = store.get_events_by_type("created").await.unwrap();
        assert_eq!(created.len(), 2);
        assert!(created.iter().all(|e| e.event_type == "created"));
        assert!(created.windows(2).all(|w| w[0].sequence <= w[1].sequence));
    }

    struct DummyStore;

    #[async_trait::async_trait]
    impl EventStore for DummyStore {
        async fn append(&self, _event: &Event) -> Result<i64, EventError> {
            Ok(0)
        }
        async fn get_events(&self, _e: &str, _i: i64) -> Result<Vec<Event>, EventError> {
            Ok(vec![])
        }
        async fn get_events_since(&self, _e: &str, _i: i64, _s: i64) -> Result<Vec<Event>, EventError> {
            Ok(vec![])
        }
        async fn get_events_by_range(
            &self,
            _e: &str,
            _i: i64,
            _f: DateTime<Utc>,
            _t: DateTime<Utc>,
        ) -> Result<Vec<Event>, EventError> {
            Ok(vec![])
        }
        async fn get_latest_sequence(&self, _e: &str, _i: i64) -> Result<i64, EventError> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn default_get_events_by_type_returns_storage_error() {
        let err = DummyStore.get_events_by_type("created").await.unwrap_err();
        assert!(matches!(err, EventError::StorageError(_)));
    }

    #[tokio::test]
    async fn new_store_is_empty() {
        let store = InMemoryEventStore::new();
        assert!(store.get_events("F", 1).await.unwrap().is_empty());
        assert_eq!(store.get_latest_sequence("F", 1).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn default_new_matches_new_constructor() {
        let default_store = InMemoryEventStore::default();
        let new_store = InMemoryEventStore::new();
        assert!(default_store.get_events("X", 1).await.unwrap().is_empty());
        assert!(new_store.get_events("X", 1).await.unwrap().is_empty());
    }
}
