//! Event replay engine with Aggregate pattern.

use agileplus_domain::domain::event::Event;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("Aggregate error: {0}")]
    AggregateError(String),
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// Aggregate trait: any entity that can be reconstructed from events.
#[async_trait]
pub trait Aggregate: Send + Sync {
    /// Apply an event to update aggregate state. Must be idempotent for same event sequence.
    async fn apply(&mut self, event: &Event) -> Result<(), ReplayError>;

    /// Current version (latest applied event sequence).
    fn version(&self) -> i64;

    /// Set version after loading from snapshot.
    fn set_version(&mut self, version: i64);
}

/// Replay a sequence of events onto an aggregate.
pub async fn replay_events<A: Aggregate>(
    aggregate: &mut A,
    events: &[Event],
) -> Result<(), ReplayError> {
    if !events.is_empty() {
        let first_id = events[0].entity_id;
        for event in events {
            if event.entity_id != first_id {
                return Err(ReplayError::InvalidState(
                    "Events from different entities in replay".into(),
                ));
            }
        }
    }

    for event in events {
        aggregate.apply(event).await?;
    }

    if let Some(last) = events.last() {
        aggregate.set_version(last.sequence);
    }

    Ok(())
}

/// Replay only events after a snapshot sequence.
pub async fn replay_events_since<A: Aggregate>(
    aggregate: &mut A,
    snapshot_sequence: i64,
    events: &[Event],
) -> Result<(), ReplayError> {
    let filtered: Vec<_> = events
        .iter()
        .filter(|e| e.sequence > snapshot_sequence)
        .cloned()
        .collect();
    replay_events(aggregate, &filtered).await
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAggregate {
        version: i64,
        state: serde_json::Value,
    }

    #[async_trait]
    impl Aggregate for TestAggregate {
        async fn apply(&mut self, event: &Event) -> Result<(), ReplayError> {
            self.state = event.payload.clone();
            self.version = event.sequence;
            Ok(())
        }
        fn version(&self) -> i64 {
            self.version
        }
        fn set_version(&mut self, v: i64) {
            self.version = v;
        }
    }

    fn make_event(seq: i64, entity_id: i64, payload: serde_json::Value) -> Event {
        Event {
            id: seq,
            entity_type: "T".into(),
            entity_id,
            event_type: "U".into(),
            payload,
            actor: "t".into(),
            timestamp: chrono::Utc::now(),
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            sequence: seq,
        }
    }

    #[tokio::test]
    async fn replay_applies_events() {
        let mut agg = TestAggregate {
            version: 0,
            state: serde_json::json!({}),
        };
        let events = vec![
            make_event(1, 1, serde_json::json!({"v": 1})),
            make_event(2, 1, serde_json::json!({"v": 2})),
        ];
        replay_events(&mut agg, &events).await.unwrap();
        assert_eq!(agg.version, 2);
        assert_eq!(agg.state, serde_json::json!({"v": 2}));
    }

    #[tokio::test]
    async fn replay_rejects_mixed_entities() {
        let mut agg = TestAggregate {
            version: 0,
            state: serde_json::json!({}),
        };
        let events = vec![
            make_event(1, 1, serde_json::json!({})),
            make_event(2, 2, serde_json::json!({})),
        ];
        assert!(replay_events(&mut agg, &events).await.is_err());
    }

    #[tokio::test]
    async fn replay_since_filters() {
        let mut agg = TestAggregate {
            version: 0,
            state: serde_json::json!({}),
        };
        let events = vec![
            make_event(1, 1, serde_json::json!({"v": 1})),
            make_event(2, 1, serde_json::json!({"v": 2})),
            make_event(3, 1, serde_json::json!({"v": 3})),
        ];
        replay_events_since(&mut agg, 2, &events).await.unwrap();
        assert_eq!(agg.version, 3);
        assert_eq!(agg.state, serde_json::json!({"v": 3}));
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[derive(Default)]
    struct CountingAggregate {
        version: i64,
        applies: Arc<AtomicUsize>,
        last_payload: Option<serde_json::Value>,
    }

    #[async_trait]
    impl Aggregate for CountingAggregate {
        async fn apply(&mut self, event: &Event) -> Result<(), ReplayError> {
            self.applies.fetch_add(1, Ordering::SeqCst);
            self.last_payload = Some(event.payload.clone());
            self.version = event.sequence;
            Ok(())
        }
        fn version(&self) -> i64 {
            self.version
        }
        fn set_version(&mut self, v: i64) {
            self.version = v;
        }
    }

    fn mk(seq: i64, entity_id: i64, payload: serde_json::Value) -> Event {
        Event {
            id: seq,
            entity_type: "T".into(),
            entity_id,
            event_type: "U".into(),
            payload,
            actor: "t".into(),
            timestamp: chrono::Utc::now(),
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            sequence: seq,
        }
    }

    fn agg() -> CountingAggregate {
        CountingAggregate::default()
    }

    #[test]
    fn replay_error_aggregate_display() {
        assert_eq!(
            ReplayError::AggregateError("x".into()).to_string(),
            "Aggregate error: x"
        );
    }

    #[test]
    fn replay_error_invalid_state_display() {
        assert_eq!(
            ReplayError::InvalidState("bad".into()).to_string(),
            "Invalid state: bad"
        );
    }

    #[tokio::test]
    async fn replay_empty_events_is_ok_and_keeps_version() {
        let mut a = agg();
        a.version = 7;
        replay_events(&mut a, &[]).await.unwrap();
        assert_eq!(a.version(), 7);
        assert_eq!(a.applies.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn replay_single_event_sets_version() {
        let mut a = agg();
        replay_events(&mut a, &[mk(1, 1, serde_json::json!({"v": 1}))]).await.unwrap();
        assert_eq!(a.version(), 1);
        assert_eq!(a.applies.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn replay_multiple_events_applies_in_order() {
        let mut a = agg();
        let events = vec![
            mk(1, 1, serde_json::json!({"v": 1})),
            mk(2, 1, serde_json::json!({"v": 2})),
            mk(3, 1, serde_json::json!({"v": 3})),
        ];
        replay_events(&mut a, &events).await.unwrap();
        assert_eq!(a.applies.load(Ordering::SeqCst), 3);
        assert_eq!(a.version(), 3);
        assert_eq!(a.last_payload, Some(serde_json::json!({"v": 3})));
    }

    #[tokio::test]
    async fn replay_mixed_entities_is_rejected() {
        let mut a = agg();
        let events = vec![mk(1, 1, serde_json::json!({})), mk(2, 2, serde_json::json!({}))];
        assert!(matches!(
            replay_events(&mut a, &events).await,
            Err(ReplayError::InvalidState(_))
        ));
    }

    #[tokio::test]
    async fn replay_same_entity_across_events_ok() {
        let mut a = agg();
        let events = vec![mk(1, 5, serde_json::json!({})), mk(2, 5, serde_json::json!({}))];
        replay_events(&mut a, &events).await.unwrap();
        assert_eq!(a.version(), 2);
    }

    #[tokio::test]
    async fn replay_since_filters_older_events() {
        let mut a = agg();
        let events = vec![
            mk(1, 1, serde_json::json!({"v": 1})),
            mk(2, 1, serde_json::json!({"v": 2})),
            mk(3, 1, serde_json::json!({"v": 3})),
        ];
        replay_events_since(&mut a, 1, &events).await.unwrap();
        assert_eq!(a.applies.load(Ordering::SeqCst), 2);
        assert_eq!(a.version(), 3);
    }

    #[tokio::test]
    async fn replay_since_beyond_all_is_noop() {
        let mut a = agg();
        a.version = 4;
        let events = vec![mk(1, 1, serde_json::json!({})), mk(2, 1, serde_json::json!({}))];
        replay_events_since(&mut a, 10, &events).await.unwrap();
        assert_eq!(a.applies.load(Ordering::SeqCst), 0);
        assert_eq!(a.version(), 4);
    }

    #[tokio::test]
    async fn replay_since_empty_events_is_ok() {
        let mut a = agg();
        replay_events_since(&mut a, 0, &[]).await.unwrap();
        assert_eq!(a.version(), 0);
    }

    #[test]
    fn set_version_overrides() {
        let mut a = agg();
        a.set_version(42);
        assert_eq!(a.version(), 42);
    }

    #[tokio::test]
    async fn replay_since_applies_only_newer_payloads() {
        let mut a = agg();
        let events = vec![
            mk(1, 1, serde_json::json!({"v": 1})),
            mk(2, 1, serde_json::json!({"v": 2})),
            mk(3, 1, serde_json::json!({"v": 3})),
        ];
        replay_events_since(&mut a, 2, &events).await.unwrap();
        assert_eq!(a.applies.load(Ordering::SeqCst), 1);
        assert_eq!(a.last_payload, Some(serde_json::json!({"v": 3})));
    }

    #[tokio::test]
    async fn replay_events_since_excludes_equal_sequence() {
        let mut a = agg();
        let events = vec![mk(5, 1, serde_json::json!({"v": 5}))];
        replay_events_since(&mut a, 5, &events).await.unwrap();
        assert_eq!(a.applies.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn replay_mixed_entities_since_is_rejected() {
        let mut a = agg();
        let events = vec![
            mk(1, 1, serde_json::json!({})),
            mk(2, 2, serde_json::json!({})),
        ];
        assert!(replay_events_since(&mut a, 0, &events).await.is_err());
    }
}
