//! Integration tests for `EventQuery` — fluent filter builder.

use agileplus_domain::domain::event::Event;
use agileplus_events::query::EventQuery;
use chrono::{Duration, Utc};

/// Helper to create a test event with specific fields.
fn make_event(seq: i64, entity_type: &str, entity_id: i64, event_type: &str, actor: &str) -> Event {
    Event {
        id: seq,
        entity_type: entity_type.into(),
        entity_id,
        event_type: event_type.into(),
        payload: serde_json::json!({}),
        actor: actor.into(),
        timestamp: Utc::now(),
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        sequence: seq,
    }
}

#[test]
fn new_query_matches_all_events() {
    let events = vec![
        make_event(1, "Feature", 1, "created", "alice"),
        make_event(2, "WorkPackage", 2, "created", "bob"),
    ];
    let result = EventQuery::new().filter(&events);
    assert_eq!(result.len(), 2);
}

#[test]
fn filter_by_entity_type() {
    let events = vec![
        make_event(1, "Feature", 1, "created", "alice"),
        make_event(2, "WorkPackage", 2, "created", "alice"),
        make_event(3, "Feature", 3, "shipped", "bob"),
    ];
    let result = EventQuery::new().entity_type("Feature").filter(&events);
    assert_eq!(result.len(), 2);
    for e in &result {
        assert_eq!(e.entity_type, "Feature");
    }
}

#[test]
fn filter_by_entity_id() {
    let events = vec![
        make_event(1, "Feature", 1, "created", "alice"),
        make_event(2, "Feature", 2, "created", "alice"),
        make_event(3, "Feature", 1, "shipped", "bob"),
    ];
    let result = EventQuery::new().entity_id(1).filter(&events);
    assert_eq!(result.len(), 2);
    for e in &result {
        assert_eq!(e.entity_id, 1);
    }
}

#[test]
fn filter_by_event_type() {
    let events = vec![
        make_event(1, "Feature", 1, "created", "alice"),
        make_event(2, "Feature", 1, "shipped", "alice"),
        make_event(3, "Feature", 2, "created", "bob"),
    ];
    let result = EventQuery::new().event_type("shipped").filter(&events);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].event_type, "shipped");
}

#[test]
fn filter_by_actor() {
    let events = vec![
        make_event(1, "Feature", 1, "created", "alice"),
        make_event(2, "Feature", 1, "shipped", "bob"),
        make_event(3, "Feature", 2, "created", "alice"),
    ];
    let result = EventQuery::new().actor("bob").filter(&events);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].actor, "bob");
}

#[test]
fn filter_by_time_range() {
    let now = Utc::now();
    let mut e1 = make_event(1, "Feature", 1, "created", "a");
    e1.timestamp = now - Duration::hours(3);
    let mut e2 = make_event(2, "Feature", 1, "specified", "a");
    e2.timestamp = now - Duration::hours(1);
    let mut e3 = make_event(3, "Feature", 1, "shipped", "a");
    e3.timestamp = now + Duration::hours(1);

    let events = vec![e1, e2, e3];

    // Filter to only the middle event
    let result = EventQuery::new()
        .start_time(now - Duration::hours(2))
        .end_time(now)
        .filter(&events);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].event_type, "specified");
}

#[test]
fn filter_by_sequence_range() {
    let events = vec![
        make_event(1, "Feature", 1, "created", "a"),
        make_event(2, "Feature", 1, "specified", "a"),
        make_event(3, "Feature", 1, "implemented", "a"),
        make_event(4, "Feature", 1, "shipped", "a"),
    ];
    let result = EventQuery::new()
        .after_sequence(2)
        .end_sequence(3)
        .filter(&events);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].sequence, 2);
    assert_eq!(result[1].sequence, 3);
}

#[test]
fn limit_restricts_results() {
    let events = vec![
        make_event(1, "Feature", 1, "created", "a"),
        make_event(2, "Feature", 1, "specified", "a"),
        make_event(3, "Feature", 1, "shipped", "a"),
    ];
    let result = EventQuery::new().limit(2).filter(&events);
    assert_eq!(result.len(), 2);
}

#[test]
fn combined_filters_narrow_results() {
    let events = vec![
        make_event(1, "Feature", 1, "created", "alice"),
        make_event(2, "Feature", 2, "created", "bob"),
        make_event(3, "WorkPackage", 1, "created", "alice"),
        make_event(4, "Feature", 1, "shipped", "alice"),
    ];
    let result = EventQuery::new()
        .entity_type("Feature")
        .actor("alice")
        .event_type("created")
        .filter(&events);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].entity_id, 1);
}

#[test]
fn filter_with_no_matches_returns_empty() {
    let events = vec![
        make_event(1, "Feature", 1, "created", "alice"),
    ];
    let result = EventQuery::new()
        .entity_type("WorkPackage")
        .filter(&events);
    assert!(result.is_empty());
}

#[test]
fn filter_on_empty_input_returns_empty() {
    let events: Vec<Event> = vec![];
    let result = EventQuery::new().entity_type("Feature").filter(&events);
    assert!(result.is_empty());
}

#[test]
fn limit_of_zero_returns_empty() {
    let events = vec![
        make_event(1, "Feature", 1, "created", "a"),
    ];
    let result = EventQuery::new().limit(0).filter(&events);
    assert!(result.is_empty());
}

#[test]
fn only_start_time_filters_correctly() {
    let now = Utc::now();
    let mut e1 = make_event(1, "F", 1, "created", "a");
    e1.timestamp = now - Duration::hours(5);
    let mut e2 = make_event(2, "F", 1, "created", "a");
    e2.timestamp = now;

    let events = vec![e1, e2];
    let result = EventQuery::new()
        .start_time(now - Duration::hours(2))
        .filter(&events);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].sequence, 2);
}

#[test]
fn only_end_time_filters_correctly() {
    let now = Utc::now();
    let mut e1 = make_event(1, "F", 1, "created", "a");
    e1.timestamp = now;
    let mut e2 = make_event(2, "F", 1, "created", "a");
    e2.timestamp = now + Duration::hours(5);

    let events = vec![e1, e2];
    let result = EventQuery::new()
        .end_time(now + Duration::hours(2))
        .filter(&events);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].sequence, 1);
}
