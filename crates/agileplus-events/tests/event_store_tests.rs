//! Integration tests for `InMemoryEventStore` — append, query, and sequence logic.

use agileplus_domain::domain::event::Event;
use agileplus_events::store::{EventError, EventStore, InMemoryEventStore};
use chrono::{Duration, Utc};

/// Helper to create a minimal event for testing.
fn make_event(entity_type: &str, entity_id: i64, event_type: &str, actor: &str) -> Event {
    Event::new(entity_type, entity_id, event_type, serde_json::json!({}), actor)
}

// ── append ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn append_returns_incrementing_sequence() {
    let store = InMemoryEventStore::new();
    let e1 = make_event("Feature", 1, "created", "alice");
    let e2 = make_event("Feature", 1, "transitioned", "alice");
    let e3 = make_event("Feature", 1, "shipped", "bob");

    let seq1 = store.append(&e1).await.unwrap();
    let seq2 = store.append(&e2).await.unwrap();
    let seq3 = store.append(&e3).await.unwrap();

    assert_eq!(seq1, 1);
    assert_eq!(seq2, 2);
    assert_eq!(seq3, 3);
}

#[tokio::test]
async fn append_independent_entities_have_separate_sequences() {
    let store = InMemoryEventStore::new();
    let e1 = make_event("Feature", 1, "created", "a");
    let e2 = make_event("Feature", 2, "created", "a");
    let e3 = make_event("WorkPackage", 1, "created", "a");

    let seq1 = store.append(&e1).await.unwrap();
    let seq2 = store.append(&e2).await.unwrap();
    let seq3 = store.append(&e3).await.unwrap();

    assert_eq!(seq1, 1);
    assert_eq!(seq2, 1);
    assert_eq!(seq3, 1);
}

// ── get_events ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_events_returns_all_for_entity_sorted_by_sequence() {
    let store = InMemoryEventStore::new();
    store.append(&make_event("Feature", 1, "created", "a")).await.unwrap();
    store.append(&make_event("Feature", 1, "transitioned", "a")).await.unwrap();
    store.append(&make_event("Feature", 2, "created", "a")).await.unwrap();

    let events = store.get_events("Feature", 1).await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_type, "created");
    assert_eq!(events[1].event_type, "transitioned");
}

#[tokio::test]
async fn get_events_returns_empty_for_unknown_entity() {
    let store = InMemoryEventStore::new();
    let events = store.get_events("Feature", 999).await.unwrap();
    assert!(events.is_empty());
}

#[tokio::test]
async fn get_events_does_not_mix_entity_types() {
    let store = InMemoryEventStore::new();
    store.append(&make_event("Feature", 1, "created", "a")).await.unwrap();
    store.append(&make_event("WorkPackage", 1, "created", "a")).await.unwrap();

    let feature_events = store.get_events("Feature", 1).await.unwrap();
    let wp_events = store.get_events("WorkPackage", 1).await.unwrap();

    assert_eq!(feature_events.len(), 1);
    assert_eq!(wp_events.len(), 1);
}

// ── get_events_since ────────────────────────────────────────────────────────

#[tokio::test]
async fn get_events_since_filters_by_sequence() {
    let store = InMemoryEventStore::new();
    store.append(&make_event("Feature", 1, "created", "a")).await.unwrap();
    store.append(&make_event("Feature", 1, "specified", "a")).await.unwrap();
    store.append(&make_event("Feature", 1, "implemented", "a")).await.unwrap();
    store.append(&make_event("Feature", 1, "shipped", "a")).await.unwrap();

    let events = store.get_events_since("Feature", 1, 2).await.unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_type, "implemented");
    assert_eq!(events[1].event_type, "shipped");
}

#[tokio::test]
async fn get_events_since_with_zero_returns_all() {
    let store = InMemoryEventStore::new();
    store.append(&make_event("Feature", 1, "created", "a")).await.unwrap();
    store.append(&make_event("Feature", 1, "shipped", "a")).await.unwrap();

    let events = store.get_events_since("Feature", 1, 0).await.unwrap();
    assert_eq!(events.len(), 2);
}

#[tokio::test]
async fn get_events_since_beyond_last_returns_empty() {
    let store = InMemoryEventStore::new();
    store.append(&make_event("Feature", 1, "created", "a")).await.unwrap();

    let events = store.get_events_since("Feature", 1, 1).await.unwrap();
    assert!(events.is_empty());
}

// ── get_events_by_range ─────────────────────────────────────────────────────

#[tokio::test]
async fn get_events_by_range_filters_by_timestamp() {
    let store = InMemoryEventStore::new();
    let before = Utc::now() - Duration::hours(2);
    let middle = Utc::now() - Duration::hours(1);
    let after = Utc::now() + Duration::hours(1);

    let mut e1 = make_event("Feature", 1, "created", "a");
    e1.timestamp = before;
    store.append(&e1).await.unwrap();

    let mut e2 = make_event("Feature", 1, "specified", "a");
    e2.timestamp = middle;
    store.append(&e2).await.unwrap();

    let events = store
        .get_events_by_range("Feature", 1, before - Duration::minutes(1), middle + Duration::minutes(1))
        .await
        .unwrap();
    assert_eq!(events.len(), 2);

    let events = store
        .get_events_by_range("Feature", 1, middle - Duration::minutes(1), after)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "specified");
}

// ── get_latest_sequence ─────────────────────────────────────────────────────

#[tokio::test]
async fn get_latest_sequence_returns_zero_for_empty() {
    let store = InMemoryEventStore::new();
    let seq = store.get_latest_sequence("Feature", 1).await.unwrap();
    assert_eq!(seq, 0);
}

#[tokio::test]
async fn get_latest_sequence_returns_last_appended() {
    let store = InMemoryEventStore::new();
    store.append(&make_event("Feature", 1, "created", "a")).await.unwrap();
    store.append(&make_event("Feature", 1, "shipped", "a")).await.unwrap();

    let seq = store.get_latest_sequence("Feature", 1).await.unwrap();
    assert_eq!(seq, 2);
}

// ── get_events_by_type ──────────────────────────────────────────────────────

#[tokio::test]
async fn get_events_by_type_filters_correctly() {
    let store = InMemoryEventStore::new();
    store.append(&make_event("Feature", 1, "created", "a")).await.unwrap();
    store.append(&make_event("Feature", 1, "transitioned", "a")).await.unwrap();
    store.append(&make_event("Feature", 2, "created", "a")).await.unwrap();

    let events = store.get_events_by_type("created").await.unwrap();
    assert_eq!(events.len(), 2);
    for e in &events {
        assert_eq!(e.event_type, "created");
    }
}

#[tokio::test]
async fn get_events_by_type_returns_empty_for_nonexistent() {
    let store = InMemoryEventStore::new();
    store.append(&make_event("Feature", 1, "created", "a")).await.unwrap();

    let events = store.get_events_by_type("nonexistent").await.unwrap();
    assert!(events.is_empty());
}

// ── new() ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn new_store_is_empty() {
    let store = InMemoryEventStore::new();
    let events = store.get_events("Feature", 1).await.unwrap();
    assert!(events.is_empty());
    let seq = store.get_latest_sequence("Feature", 1).await.unwrap();
    assert_eq!(seq, 0);
}

// ── EventError surface ──────────────────────────────────────────────────────

#[test]
fn event_error_variants_display_expected_messages() {
    assert_eq!(EventError::NotFound("e".into()).to_string(), "Event not found: e");
    assert_eq!(
        EventError::DuplicateSequence("2".into()).to_string(),
        "Duplicate sequence: 2"
    );
    assert_eq!(
        EventError::StorageError("disk".into()).to_string(),
        "Storage error: disk"
    );
    assert_eq!(EventError::InvalidHash("h".into()).to_string(), "Invalid hash: h");
    assert_eq!(
        EventError::SequenceGap { expected: 1, actual: 3 }.to_string(),
        "Sequence gap: expected 1, got 3"
    );
}

#[tokio::test]
async fn append_returns_increasing_sequences_for_same_entity() {
    let store = InMemoryEventStore::new();
    let a = store.append(&make_event("Feature", 7, "created", "x")).await.unwrap();
    let b = store.append(&make_event("Feature", 7, "updated", "x")).await.unwrap();
    assert_eq!(a, 1);
    assert_eq!(b, 2);
}

#[tokio::test]
async fn events_for_distinct_entities_are_isolated() {
    let store = InMemoryEventStore::new();
    store.append(&make_event("Feature", 1, "created", "x")).await.unwrap();
    store.append(&make_event("Feature", 2, "created", "y")).await.unwrap();
    let one = store.get_events("Feature", 1).await.unwrap();
    let two = store.get_events("Feature", 2).await.unwrap();
    assert_eq!(one.len(), 1);
    assert_eq!(two.len(), 1);
    assert_eq!(one[0].actor, "x");
    assert_eq!(two[0].actor, "y");
}
