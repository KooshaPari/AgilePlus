//! Integration tests for the benchmark harness: event append and replay workflows.
//!
//! These tests exercise the full pipeline that the benchmarks measure:
//! - Event append → SQLite storage → retrieval
//! - Replay from sequence 0 and from snapshot offset
//! - Multi-entity event operations
//! - Aggregate state reconstruction

use agileplus_benchmarks::helpers::*;
#[allow(unused_imports)]
use agileplus_events::{replay_events, replay_events_since};
use agileplus_sqlite::repository::events as event_repo;

// ---------------------------------------------------------------------------
// Event append via SQLite
// ---------------------------------------------------------------------------

#[test]
fn append_single_event_to_sqlite() {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    let ev = make_event(1, 1);
    event_repo::append_event(&conn, &ev).expect("append");
}

#[test]
fn append_sequential_events_single_entity() {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    for seq in 1..=50 {
        let ev = make_event(1, seq);
        event_repo::append_event(&conn, &ev).expect("append");
    }
}

#[test]
fn append_multi_entity_events() {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    let events = make_events_multi_entity(200, 10);
    for ev in &events {
        event_repo::append_event(&conn, ev).expect("append");
    }
}

#[test]
fn append_then_retrieve_single_entity() {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    for seq in 1..=10 {
        let ev = make_event(1, seq);
        event_repo::append_event(&conn, &ev).expect("append");
    }
    let retrieved = event_repo::get_events(&conn, "Feature", 1).expect("get");
    assert_eq!(retrieved.len(), 10);
}

#[test]
fn append_then_retrieve_multi_entity() {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    let events = make_events_multi_entity(100, 5);
    for ev in &events {
        event_repo::append_event(&conn, ev).expect("append");
    }
    // Each entity should have 20 events (100 / 5)
    for entity_id in 1..=5 {
        let retrieved =
            event_repo::get_events(&conn, "Feature", entity_id).expect("get");
        assert_eq!(retrieved.len(), 20);
    }
}

#[test]
fn append_large_batch_events() {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    let events = make_events(1000);
    for ev in &events {
        event_repo::append_event(&conn, ev).expect("append");
    }
    let retrieved = event_repo::get_events(&conn, "Feature", 1).expect("get");
    assert_eq!(retrieved.len(), 1000);
}

// ---------------------------------------------------------------------------
// Event replay from scratch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn replay_empty_events() {
    let mut agg = CountingAggregate::default();
    replay_events(&mut agg, &[]).await.unwrap();
    assert_eq!(agg.events_applied, 0);
    assert_eq!(agg.version, 0);
}

#[tokio::test]
async fn replay_single_event() {
    let mut agg = CountingAggregate::default();
    let events = make_events(1);
    replay_events(&mut agg, &events).await.unwrap();
    assert_eq!(agg.events_applied, 1);
    assert_eq!(agg.version, 1);
    assert_eq!(agg.last_state, "Specified");
}

#[tokio::test]
async fn replay_hundred_events() {
    let mut agg = CountingAggregate::default();
    let events = make_events(100);
    replay_events(&mut agg, &events).await.unwrap();
    assert_eq!(agg.events_applied, 100);
    assert_eq!(agg.version, 100);
}

#[tokio::test]
async fn replay_thousand_events() {
    let mut agg = CountingAggregate::default();
    let events = make_events(1000);
    replay_events(&mut agg, &events).await.unwrap();
    assert_eq!(agg.events_applied, 1000);
    assert_eq!(agg.version, 1000);
}

// ---------------------------------------------------------------------------
// Replay with snapshots
// ---------------------------------------------------------------------------

#[tokio::test]
async fn replay_since_zero_delta() {
    let mut agg = CountingAggregate::default();
    #[allow(unused_variables)]
    let _events = replay_events_since(&mut agg, 0, &[]).await.unwrap();
    // replay_events_since returns the events replayed; empty delta = nothing applied
    assert_eq!(agg.events_applied, 0);
}

#[tokio::test]
async fn replay_since_partial() {
    let events = make_events(100);
    let delta = &events[80..]; // sequences 81..100

    let mut agg = CountingAggregate {
        version: 80,
        events_applied: 80,
        last_state: "Specified".to_string(),
    };
    replay_events_since(&mut agg, 80, delta).await.unwrap();
    assert_eq!(agg.version, 100);
    assert_eq!(agg.events_applied, 100);
}

#[tokio::test]
async fn replay_after_snapshot_equals_full_replay() {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    for seq in 1..=200 {
        let ev = make_event(1, seq);
        event_repo::append_event(&conn, &ev).expect("append");
    }

    // Full replay from DB
    let all_events = event_repo::get_events(&conn, "Feature", 1).expect("get_all");
    let mut agg_full = CountingAggregate::default();
    replay_events(&mut agg_full, &all_events).await.unwrap();

    // Snapshot at seq 100, replay delta
    let snapshot = make_snapshot(1, 100);
    let delta = event_repo::get_events_since(&conn, "Feature", 1, snapshot.event_sequence)
        .expect("get_delta");

    let mut agg_partial = CountingAggregate {
        version: snapshot.event_sequence,
        events_applied: snapshot.event_sequence as u64,
        last_state: "Specified".to_string(),
    };
    replay_events_since(&mut agg_partial, snapshot.event_sequence, &delta)
        .await
        .unwrap();

    assert_eq!(agg_full.version, agg_partial.version);
    assert_eq!(agg_full.last_state, agg_partial.last_state);
}

// ---------------------------------------------------------------------------
// Snapshot-assisted replay from SQLite
// ---------------------------------------------------------------------------

#[tokio::test]
async fn snapshot_replay_from_sqlite() {
    let adapter = make_in_memory_adapter();
    let conn = adapter.conn_for_bench().expect("conn");
    for seq in 1..=100 {
        let ev = make_event(1, seq);
        event_repo::append_event(&conn, &ev).expect("append");
    }
    drop(conn);

    let snapshot = make_snapshot(1, 80);
    let conn = adapter.conn_for_bench().expect("conn");
    let delta = event_repo::get_events_since(&conn, "Feature", 1, snapshot.event_sequence)
        .expect("get_since");
    assert_eq!(delta.len(), 20);

    let mut agg = CountingAggregate {
        version: snapshot.event_sequence,
        events_applied: snapshot.event_sequence as u64,
        last_state: "Specified".to_string(),
    };
    replay_events_since(&mut agg, snapshot.event_sequence, &delta)
        .await
        .unwrap();
    assert_eq!(agg.version, 100);
    assert_eq!(agg.events_applied, 100);
}

// ---------------------------------------------------------------------------
// Benchmark config: timing sanity
// ---------------------------------------------------------------------------

#[test]
fn benchmark_config_event_counts_match() {
    // Verify the helper generates the exact counts used in benchmarks
    for &count in &[100_i64, 1_000, 5_000] {
        let events = make_events(count);
        assert_eq!(events.len(), count as usize);
    }
}

#[test]
fn benchmark_config_multi_entity_counts() {
    // Mirror event_append_throughput bench: 1000 events across 100 entities
    let events = make_events_multi_entity(1_000, 100);
    assert_eq!(events.len(), 1000);
    for entity_id in 1..=100 {
        let count = events.iter().filter(|e| e.entity_id == entity_id).count();
        assert_eq!(count, 10, "entity {entity_id} should have exactly 10 events");
    }
}

#[test]
fn benchmark_config_snapshot_offset_matches() {
    // Mirror event_replay bench: snapshot at 900, replay 100 deltas
    let events = make_events(1000);
    let snapshot = make_snapshot(1, 900);
    let delta: Vec<_> = events
        .into_iter()
        .filter(|e| e.sequence > snapshot.event_sequence)
        .collect();
    assert_eq!(delta.len(), 100);
    assert_eq!(delta.first().unwrap().sequence, 901);
    assert_eq!(delta.last().unwrap().sequence, 1000);
}
