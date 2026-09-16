//! Integration tests for event replay — `replay_events`, `replay_events_since`, Aggregate.

use agileplus_domain::domain::event::Event;
use agileplus_events::replay::{replay_events, replay_events_since, Aggregate, ReplayError};
use async_trait::async_trait;

// ── Test aggregate ──────────────────────────────────────────────────────────

struct CounterAggregate {
    version: i64,
    count: i64,
    last_payload: serde_json::Value,
}

impl CounterAggregate {
    fn new() -> Self {
        Self {
            version: 0,
            count: 0,
            last_payload: serde_json::json!({}),
        }
    }
}

#[async_trait]
impl Aggregate for CounterAggregate {
    async fn apply(&mut self, event: &Event) -> Result<(), ReplayError> {
        self.count += 1;
        self.last_payload = event.payload.clone();
        Ok(())
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn set_version(&mut self, version: i64) {
        self.version = version;
    }
}

/// An aggregate that fails on specific events.
struct FailingAggregate {
    version: i64,
    fail_on_sequence: i64,
}

#[async_trait]
impl Aggregate for FailingAggregate {
    async fn apply(&mut self, event: &Event) -> Result<(), ReplayError> {
        if event.sequence == self.fail_on_sequence {
            return Err(ReplayError::AggregateError("forced failure".into()));
        }
        self.version = event.sequence;
        Ok(())
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn set_version(&mut self, version: i64) {
        self.version = version;
    }
}

fn make_event(seq: i64, entity_id: i64, payload: serde_json::Value) -> Event {
    Event {
        id: seq,
        entity_type: "T".into(),
        entity_id,
        event_type: "U".into(),
        payload,
        actor: "test".into(),
        timestamp: chrono::Utc::now(),
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        sequence: seq,
    }
}

// ── replay_events ───────────────────────────────────────────────────────────

#[tokio::test]
async fn replay_empty_events_is_noop() {
    let mut agg = CounterAggregate::new();
    replay_events(&mut agg, &[]).await.unwrap();
    assert_eq!(agg.version, 0);
    assert_eq!(agg.count, 0);
}

#[tokio::test]
async fn replay_applies_all_events_and_sets_version() {
    let mut agg = CounterAggregate::new();
    let events = vec![
        make_event(1, 1, serde_json::json!({"step": 1})),
        make_event(2, 1, serde_json::json!({"step": 2})),
        make_event(3, 1, serde_json::json!({"step": 3})),
    ];
    replay_events(&mut agg, &events).await.unwrap();

    assert_eq!(agg.version, 3);
    assert_eq!(agg.count, 3);
    assert_eq!(agg.last_payload, serde_json::json!({"step": 3}));
}

#[tokio::test]
async fn replay_rejects_mixed_entity_ids() {
    let mut agg = CounterAggregate::new();
    let events = vec![
        make_event(1, 1, serde_json::json!({})),
        make_event(2, 2, serde_json::json!({})),
    ];
    let result = replay_events(&mut agg, &events).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ReplayError::InvalidState(msg) => {
            assert!(msg.contains("different entities"));
        }
        other => panic!("expected InvalidState, got {other:?}"),
    }
}

#[tokio::test]
async fn replay_propagates_aggregate_errors() {
    let mut agg = FailingAggregate {
        version: 0,
        fail_on_sequence: 2,
    };
    let events = vec![
        make_event(1, 1, serde_json::json!({})),
        make_event(2, 1, serde_json::json!({})),
    ];
    let result = replay_events(&mut agg, &events).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ReplayError::AggregateError(msg) => assert_eq!(msg, "forced failure"),
        other => panic!("expected AggregateError, got {other:?}"),
    }
    // Version should not be updated for event 2 since it failed
    assert_eq!(agg.version, 1);
}

#[tokio::test]
async fn replay_with_single_event_sets_version() {
    let mut agg = CounterAggregate::new();
    let events = vec![make_event(5, 10, serde_json::json!({"x": 1}))];
    replay_events(&mut agg, &events).await.unwrap();
    assert_eq!(agg.version, 5);
}

// ── replay_events_since ─────────────────────────────────────────────────────

#[tokio::test]
async fn replay_since_filters_out_older_events() {
    let mut agg = CounterAggregate::new();
    let events = vec![
        make_event(1, 1, serde_json::json!({"v": 1})),
        make_event(2, 1, serde_json::json!({"v": 2})),
        make_event(3, 1, serde_json::json!({"v": 3})),
        make_event(4, 1, serde_json::json!({"v": 4})),
    ];
    replay_events_since(&mut agg, 2, &events).await.unwrap();

    assert_eq!(agg.version, 4);
    assert_eq!(agg.count, 2); // Only events 3 and 4 were applied
    assert_eq!(agg.last_payload, serde_json::json!({"v": 4}));
}

#[tokio::test]
async fn replay_since_with_zero_applies_all() {
    let mut agg = CounterAggregate::new();
    let events = vec![
        make_event(1, 1, serde_json::json!({})),
        make_event(2, 1, serde_json::json!({})),
    ];
    replay_events_since(&mut agg, 0, &events).await.unwrap();
    assert_eq!(agg.count, 2);
}

#[tokio::test]
async fn replay_since_beyond_all_events_applies_none() {
    let mut agg = CounterAggregate::new();
    let events = vec![
        make_event(1, 1, serde_json::json!({})),
        make_event(2, 1, serde_json::json!({})),
    ];
    replay_events_since(&mut agg, 100, &events).await.unwrap();
    assert_eq!(agg.count, 0);
    assert_eq!(agg.version, 0);
}

#[tokio::test]
async fn replay_since_with_empty_events() {
    let mut agg = CounterAggregate::new();
    replay_events_since(&mut agg, 5, &[]).await.unwrap();
    assert_eq!(agg.version, 0);
    assert_eq!(agg.count, 0);
}

// ── ReplayError Display ─────────────────────────────────────────────────────

#[test]
fn replay_error_display_messages() {
    let e1 = ReplayError::AggregateError("bad".into());
    assert!(e1.to_string().contains("bad"));

    let e2 = ReplayError::InvalidState("mixed".into());
    assert!(e2.to_string().contains("mixed"));
}
