//! T112 — `agileplus-events` ↔ `agileplus-sqlite` `EventStore` trait contract.
//!
//! Verifies that `SqliteStorageAdapter` (which implements `EventStore`)
//! satisfies the same observable contract as `InMemoryEventStore`.
//! This guards against the SQLite port silently diverging from the
//! canonical in-memory reference implementation.

use agileplus_domain::domain::event::Event;
use agileplus_events::{EventStore, InMemoryEventStore};
use agileplus_sqlite::SqliteStorageAdapter;

fn make_event(entity_type: &str, entity_id: i64, event_type: &str, actor: &str) -> Event {
    Event::new(
        entity_type,
        entity_id,
        event_type,
        serde_json::json!({"contract": "events_sqlite"}),
        actor,
    )
}

/// Property: append returns strictly monotonic per-(entity_type, entity_id) sequence.
#[tokio::test]
async fn sqlite_append_assigns_per_entity_sequence() {
    let db = SqliteStorageAdapter::in_memory().expect("in-memory sqlite");
    let store: &dyn EventStore = &db;

    let s1 = store
        .append(&make_event("Feature", 1, "created", "a"))
        .await
        .unwrap();
    let s2 = store
        .append(&make_event("Feature", 1, "updated", "a"))
        .await
        .unwrap();
    let s3 = store
        .append(&make_event("Feature", 2, "created", "a"))
        .await
        .unwrap();

    assert!(s1 >= 1);
    assert!(s2 > s1, "second append on same entity must be greater: {s1} -> {s2}");
    assert_eq!(s3, 1, "different entity_id must start a new sequence at 1");

    assert_eq!(store.get_latest_sequence("Feature", 1).await.unwrap(), s2);
    assert_eq!(store.get_latest_sequence("Feature", 2).await.unwrap(), 1);
}

/// Property: get_events returns ascending order, scoped to (entity_type, entity_id).
#[tokio::test]
async fn sqlite_get_events_scoped_and_ordered() {
    let db = SqliteStorageAdapter::in_memory().expect("in-memory sqlite");
    let store: &dyn EventStore = &db;

    store
        .append(&make_event("Feature", 1, "created", "a"))
        .await
        .unwrap();
    store
        .append(&make_event("Feature", 1, "updated", "a"))
        .await
        .unwrap();
    store
        .append(&make_event("Feature", 2, "created", "a"))
        .await
        .unwrap();

    let events = store.get_events("Feature", 1).await.unwrap();
    assert_eq!(events.len(), 2);
    let seqs: Vec<i64> = events.iter().map(|e| e.sequence).collect();
    assert_eq!(seqs, vec![1, 2], "events must be returned in ascending sequence order");

    let other = store.get_events("Feature", 2).await.unwrap();
    assert_eq!(other.len(), 1);
    assert_eq!(other[0].entity_id, 2);
}

/// Property: get_events_since filters with strict-greater-than semantics.
#[tokio::test]
async fn sqlite_get_events_since_strictly_greater() {
    let db = SqliteStorageAdapter::in_memory().expect("in-memory sqlite");
    let store: &dyn EventStore = &db;

    store
        .append(&make_event("Feature", 1, "created", "a"))
        .await
        .unwrap();
    store
        .append(&make_event("Feature", 1, "updated", "a"))
        .await
        .unwrap();
    store
        .append(&make_event("Feature", 1, "shipped", "a"))
        .await
        .unwrap();

    let after = store.get_events_since("Feature", 1, 1).await.unwrap();
    assert_eq!(after.len(), 2);
    assert!(after.iter().all(|e| e.sequence > 1));
}

/// Property: get_events_by_range includes the full inclusive time range.
#[tokio::test]
async fn sqlite_get_events_by_range_inclusive() {
    let db = SqliteStorageAdapter::in_memory().expect("in-memory sqlite");
    let store: &dyn EventStore = &db;

    store
        .append(&make_event("Feature", 1, "created", "a"))
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let mid = chrono::Utc::now();
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    store
        .append(&make_event("Feature", 1, "updated", "a"))
        .await
        .unwrap();

    let range = store
        .get_events_by_range("Feature", 1, mid - chrono::Duration::seconds(1), mid + chrono::Duration::seconds(1))
        .await
        .unwrap();
    // The exact count depends on which side of `mid` each event lands on,
    // but at minimum the `updated` event must be present.
    assert!(!range.is_empty(), "range query must return at least the post-mid event");
    assert!(range.iter().any(|e| e.event_type == "updated"));
}

/// Cross-implementation equivalence: InMemory and Sqlite agree on append/sequence.
#[tokio::test]
async fn sqlite_matches_in_memory_contract() {
    let mem = InMemoryEventStore::new();
    let db = SqliteStorageAdapter::in_memory().expect("in-memory sqlite");
    let sql: &dyn EventStore = &db;

    // Append 3 events to both stores.
    for (i, et) in ["created", "updated", "shipped"].iter().enumerate() {
        mem.append(&make_event("Feature", 1, et, "a")).await.unwrap();
        sql.append(&make_event("Feature", 1, et, "a")).await.unwrap();
        let _ = i;
    }

    let mem_events = mem.get_events("Feature", 1).await.unwrap();
    let sql_events = sql.get_events("Feature", 1).await.unwrap();
    assert_eq!(mem_events.len(), sql_events.len());
    assert_eq!(mem_events.len(), 3);

    for (m, s) in mem_events.iter().zip(sql_events.iter()) {
        assert_eq!(m.entity_type, s.entity_type);
        assert_eq!(m.entity_id, s.entity_id);
        assert_eq!(m.event_type, s.event_type);
        assert_eq!(m.sequence, s.sequence);
    }

    assert_eq!(
        mem.get_latest_sequence("Feature", 1).await.unwrap(),
        sql.get_latest_sequence("Feature", 1).await.unwrap(),
    );
}

/// Contract: latest_sequence is 0 (not an error) when no events exist.
#[tokio::test]
async fn sqlite_latest_sequence_zero_when_empty() {
    let db = SqliteStorageAdapter::in_memory().expect("in-memory sqlite");
    let store: &dyn EventStore = &db;

    assert_eq!(store.get_latest_sequence("Feature", 999).await.unwrap(), 0);
    assert!(store.get_events("Feature", 999).await.unwrap().is_empty());
}
