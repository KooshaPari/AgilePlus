//! Integration tests for the vector_clock module.
//!
//! Tests pure logic of SyncVector: creation, advancing, merging,
//! computing missing events, serde roundtrips, the partial order the vectors
//! induce, and the error path out of `sync_with_peer_vectors`.

use std::collections::BTreeMap;

use agileplus_domain::domain::event::Event;
use agileplus_events::store::{EventError, EventStore};
use agileplus_p2p::discovery::{PeerInfo, PeerStatus};
use agileplus_p2p::error::SyncError;
use agileplus_p2p::vector_clock::{
    SyncResult, SyncVector, compute_missing_locally, sync_with_peer_vectors,
};
use async_trait::async_trait;

// ── SyncVector construction ────────────────────────────────────────────────

#[test]
fn new_vector_is_empty() {
    let v = SyncVector::new("device-1");
    assert_eq!(v.device_id, "device-1");
    assert!(v.entries.is_empty());
}

#[test]
fn new_vector_with_string_ref() {
    let id = String::from("my-device");
    let v = SyncVector::new(&id);
    assert_eq!(v.device_id, "my-device");
}

#[test]
fn default_vector_has_empty_device_id() {
    let v = SyncVector::default();
    assert_eq!(v.device_id, "");
    assert!(v.entries.is_empty());
}

// ── advance ────────────────────────────────────────────────────────────────

#[test]
fn advance_inserts_new_key() {
    let mut v = SyncVector::new("d1");
    v.advance("Feature", "1", 5);
    assert_eq!(v.get("Feature", "1"), 5);
}

#[test]
fn advance_updates_to_higher_sequence() {
    let mut v = SyncVector::new("d1");
    v.advance("Feature", "1", 3);
    v.advance("Feature", "1", 7);
    assert_eq!(v.get("Feature", "1"), 7);
}

#[test]
fn advance_ignores_lower_sequence() {
    let mut v = SyncVector::new("d1");
    v.advance("Feature", "1", 10);
    v.advance("Feature", "1", 5);
    assert_eq!(v.get("Feature", "1"), 10);
}

#[test]
fn advance_ignores_equal_sequence() {
    let mut v = SyncVector::new("d1");
    v.advance("Feature", "1", 5);
    v.advance("Feature", "1", 5);
    assert_eq!(v.get("Feature", "1"), 5);
}

#[test]
fn advance_multiple_entities() {
    let mut v = SyncVector::new("d1");
    v.advance("Feature", "1", 3);
    v.advance("WorkPackage", "42", 10);
    v.advance("Feature", "2", 7);

    assert_eq!(v.get("Feature", "1"), 3);
    assert_eq!(v.get("WorkPackage", "42"), 10);
    assert_eq!(v.get("Feature", "2"), 7);
}

// ── get ────────────────────────────────────────────────────────────────────

#[test]
fn get_returns_zero_for_unknown_key() {
    let v = SyncVector::new("d1");
    assert_eq!(v.get("Feature", "999"), 0);
}

#[test]
fn get_exact_match() {
    let mut v = SyncVector::new("d1");
    v.advance("Feature", "1", 42);
    assert_eq!(v.get("Feature", "1"), 42);
}

#[test]
fn get_different_entity_types_same_id() {
    let mut v = SyncVector::new("d1");
    v.advance("Feature", "1", 5);
    v.advance("WorkPackage", "1", 10);
    assert_eq!(v.get("Feature", "1"), 5);
    assert_eq!(v.get("WorkPackage", "1"), 10);
}

// ── merge ──────────────────────────────────────────────────────────────────

#[test]
fn merge_adds_new_keys() {
    let mut a = SyncVector::new("d1");
    a.advance("Feature", "1", 3);

    let mut b = SyncVector::new("d2");
    b.advance("WorkPackage", "42", 10);

    a.merge(&b);
    assert_eq!(a.get("Feature", "1"), 3);
    assert_eq!(a.get("WorkPackage", "42"), 10);
}

#[test]
fn merge_takes_higher_sequence() {
    let mut a = SyncVector::new("d1");
    a.advance("Feature", "1", 3);

    let mut b = SyncVector::new("d2");
    b.advance("Feature", "1", 10);

    a.merge(&b);
    assert_eq!(a.get("Feature", "1"), 10);
}

#[test]
fn merge_preserves_higher_in_self() {
    let mut a = SyncVector::new("d1");
    a.advance("Feature", "1", 15);

    let mut b = SyncVector::new("d2");
    b.advance("Feature", "1", 5);

    a.merge(&b);
    assert_eq!(a.get("Feature", "1"), 15);
}

#[test]
fn merge_complex_multi_key() {
    let mut a = SyncVector::new("d1");
    a.advance("Feature", "1", 3);
    a.advance("Feature", "2", 8);

    let mut b = SyncVector::new("d2");
    b.advance("Feature", "1", 5);
    b.advance("Feature", "2", 4);
    b.advance("Feature", "3", 12);

    a.merge(&b);

    assert_eq!(a.get("Feature", "1"), 5); // b was higher
    assert_eq!(a.get("Feature", "2"), 8); // a was higher
    assert_eq!(a.get("Feature", "3"), 12); // only in b
}

#[test]
fn merge_empty_into_empty() {
    let mut a = SyncVector::new("d1");
    let b = SyncVector::new("d2");
    a.merge(&b);
    assert!(a.entries.is_empty());
}

#[test]
fn merge_empty_into_nonempty() {
    let mut a = SyncVector::new("d1");
    a.advance("Feature", "1", 5);
    let b = SyncVector::new("d2");
    a.merge(&b);
    assert_eq!(a.get("Feature", "1"), 5);
}

#[test]
fn merge_symmetric_result() {
    let mut a = SyncVector::new("d1");
    a.advance("Feature", "1", 3);

    let mut b = SyncVector::new("d2");
    b.advance("Feature", "1", 10);

    let a_clone = a.clone();

    // Merge b into a
    a.merge(&b);
    // Merge a (original) into b
    let mut b2 = SyncVector::new("d2");
    b2.advance("Feature", "1", 10);
    b2.merge(&a_clone);

    // Both should have the same entries after merging
    assert_eq!(a.get("Feature", "1"), b2.get("Feature", "1"));
}

// ── compute_missing_locally ────────────────────────────────────────────────

#[test]
fn compute_missing_empty_vectors() {
    let local = SyncVector::new("d1");
    let peer = SyncVector::new("d2");
    let missing = compute_missing_locally(&local, &peer);
    assert!(missing.is_empty());
}

#[test]
fn compute_missing_local_has_more() {
    let mut local = SyncVector::new("d1");
    local.advance("Feature", "1", 10);

    let peer = SyncVector::new("d2");

    let missing = compute_missing_locally(&local, &peer);
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].0, "Feature");
    assert_eq!(missing[0].1, "1");
    assert_eq!(missing[0].2, 0); // peer_seq
    assert_eq!(missing[0].3, 10); // local_seq
}

#[test]
fn compute_missing_peer_has_more() {
    let local = SyncVector::new("d1");

    let mut peer = SyncVector::new("d2");
    peer.advance("Feature", "1", 10);

    let missing = compute_missing_locally(&local, &peer);
    assert!(missing.is_empty());
}

#[test]
fn compute_missing_equal_sequences() {
    let mut local = SyncVector::new("d1");
    local.advance("Feature", "1", 5);

    let mut peer = SyncVector::new("d2");
    peer.advance("Feature", "1", 5);

    let missing = compute_missing_locally(&local, &peer);
    assert!(missing.is_empty());
}

#[test]
fn compute_missing_mixed_entities() {
    let mut local = SyncVector::new("d1");
    local.advance("Feature", "1", 10); // local ahead
    local.advance("Feature", "2", 3); // local behind
    local.advance("WorkPackage", "1", 7); // equal

    let mut peer = SyncVector::new("d2");
    peer.advance("Feature", "1", 5);
    peer.advance("Feature", "2", 8);
    peer.advance("WorkPackage", "1", 7);

    let missing = compute_missing_locally(&local, &peer);
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].0, "Feature");
    assert_eq!(missing[0].1, "1");
}

#[test]
fn compute_missing_only_returns_local_ahead() {
    let mut local = SyncVector::new("d1");
    local.advance("Feature", "1", 20);
    local.advance("Feature", "2", 3);

    let mut peer = SyncVector::new("d2");
    peer.advance("Feature", "1", 5);
    peer.advance("Feature", "2", 15);

    let missing = compute_missing_locally(&local, &peer);
    assert_eq!(missing.len(), 1);
    // Only Feature/1 where local=20 > peer=5
    assert_eq!(missing[0].0, "Feature");
    assert_eq!(missing[0].1, "1");
}

// ── SyncVector structural tests ────────────────────────────────────────────

#[test]
fn sync_vector_entries_stored_correctly() {
    let mut v = SyncVector::new("device-abc");
    v.advance("Feature", "1", 5);
    v.advance("WorkPackage", "42", 10);

    // Verify internal map has correct entries
    assert_eq!(v.entries.len(), 2);
    assert_eq!(
        v.entries.get(&("Feature".to_string(), "1".to_string())),
        Some(&5)
    );
    assert_eq!(
        v.entries
            .get(&("WorkPackage".to_string(), "42".to_string())),
        Some(&10)
    );
}

#[test]
fn sync_vector_entries_empty() {
    let v = SyncVector::new("d1");
    assert!(v.entries.is_empty());
}

#[test]
fn sync_vector_default_has_no_entries() {
    let v = SyncVector::default();
    assert!(v.entries.is_empty());
    assert_eq!(v.device_id, "");
}

#[test]
fn sync_vector_clone_produces_independent_copy() {
    let mut a = SyncVector::new("d1");
    a.advance("Feature", "1", 5);
    let mut b = a.clone();
    b.advance("Feature", "1", 10);
    assert_eq!(a.get("Feature", "1"), 5);
    assert_eq!(b.get("Feature", "1"), 10);
}

// ── SyncResult ─────────────────────────────────────────────────────────────

#[test]
fn sync_result_default() {
    let r = SyncResult::default();
    assert_eq!(r.events_sent, 0);
    assert_eq!(r.events_received, 0);
    assert_eq!(r.conflicts_detected, 0);
}

// ── Partial order induced by the vectors ───────────────────────────────────
//
// `SyncVector` has no `Ord` impl; the order peers actually depend on is
// "b already knows everything a knows", which is exactly the negation of
// `compute_missing_locally`. These tests pin that relation directly instead of
// re-asserting one key of one hand-built vector.

/// Build a clock from `(entity_type, entity_id, sequence)` triples.
fn clock(device_id: &str, entries: &[(&str, &str, u64)]) -> SyncVector {
    let mut vector = SyncVector::new(device_id);
    for (entity_type, entity_id, sequence) in entries {
        vector.advance(entity_type, entity_id, *sequence);
    }
    vector
}

/// `a <= b` iff `b` records at least the sequence `a` records for every key
/// `a` knows about.
fn leq(a: &SyncVector, b: &SyncVector) -> bool {
    compute_missing_locally(a, b).is_empty()
}

/// Full entry map, so comparisons cover every key rather than a sample.
fn entries_of(vector: &SyncVector) -> BTreeMap<(String, String), u64> {
    vector
        .entries
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect()
}

fn peer(device_id: &str) -> PeerInfo {
    PeerInfo {
        device_id: device_id.to_string(),
        hostname: format!("{device_id}.tailnet"),
        tailscale_ip: "127.0.0.1".to_string(),
        status: PeerStatus::Online,
    }
}

#[test]
fn ordering_equal_clocks_are_mutually_reachable() {
    let a = clock("dev-a", &[("Feature", "1", 3)]);
    let b = clock("dev-b", &[("Feature", "1", 3)]);

    assert!(leq(&a, &b));
    assert!(leq(&b, &a));
    // The order ignores device identity: these are order-equal but structurally
    // distinct, which is why `device_id` is not part of the comparison.
    assert_ne!(a.device_id, b.device_id);
    assert_eq!(entries_of(&a), entries_of(&b));
}

#[test]
fn ordering_distinguishes_before_from_after() {
    let behind = clock("dev-a", &[("Feature", "1", 2)]);
    let ahead = clock("dev-b", &[("Feature", "1", 5)]);

    assert!(leq(&behind, &ahead));
    assert!(!leq(&ahead, &behind));
    assert!(compute_missing_locally(&behind, &ahead).is_empty());
    assert_eq!(
        compute_missing_locally(&ahead, &behind),
        vec![("Feature".to_string(), "1".to_string(), 2, 5)]
    );
}

#[test]
fn ordering_concurrent_clocks_conflict_in_both_directions() {
    let a = clock("dev-a", &[("Feature", "1", 4), ("Epic", "2", 1)]);
    let b = clock("dev-b", &[("Feature", "1", 1), ("Epic", "2", 7)]);

    assert!(!leq(&a, &b), "a has Feature/1 events b lacks");
    assert!(!leq(&b, &a), "b has Epic/2 events a lacks");

    assert_eq!(
        compute_missing_locally(&a, &b),
        vec![("Feature".to_string(), "1".to_string(), 1, 4)]
    );
    assert_eq!(
        compute_missing_locally(&b, &a),
        vec![("Epic".to_string(), "2".to_string(), 1, 7)]
    );
}

#[test]
fn ordering_ignores_keys_the_other_side_never_saw() {
    let narrow = clock("dev-a", &[("Feature", "1", 4)]);
    let wide = clock("dev-b", &[("Feature", "1", 4), ("Story", "9", 1)]);

    assert!(leq(&narrow, &wide), "an extra key is still an advance");
    assert!(!leq(&wide, &narrow), "the extra key is missing locally");
}

#[test]
fn ordering_treats_unknown_key_and_explicit_zero_alike() {
    let absent = clock("dev-a", &[("Feature", "1", 1)]);
    let explicit_zero = clock("dev-b", &[("Feature", "1", 1), ("Epic", "9", 0)]);

    assert!(
        explicit_zero
            .entries
            .contains_key(&("Epic".into(), "9".into()))
    );
    // A zero watermark carries no information, so it cannot make either side
    // strictly ahead of the other.
    assert!(leq(&absent, &explicit_zero));
    assert!(leq(&explicit_zero, &absent));
}

#[test]
fn ordering_is_transitive_across_three_clocks() {
    let a = clock("dev-a", &[("Feature", "1", 1)]);
    let b = clock("dev-b", &[("Feature", "1", 2)]);
    let c = clock("dev-c", &[("Feature", "1", 3), ("Epic", "2", 1)]);

    assert!(leq(&a, &b));
    assert!(leq(&b, &c));
    assert!(leq(&a, &c));
}

#[test]
fn three_clock_ordering_matrix() {
    // `a` is the common ancestor; `b` and `c` each advance a different stream.
    let a = clock("dev-a", &[("Feature", "1", 1)]);
    let b = clock("dev-b", &[("Feature", "1", 2)]);
    let c = clock("dev-c", &[("Feature", "1", 1), ("Epic", "2", 1)]);

    let clocks = [&a, &b, &c];
    let names = ["a", "b", "c"];
    // rows = subject, cols = object; true means "subject <= object".
    let expected = [
        [true, true, true],
        [false, true, false],
        [false, false, true],
    ];

    for (i, row) in expected.iter().enumerate() {
        for (j, wanted) in row.iter().enumerate() {
            assert_eq!(
                leq(clocks[i], clocks[j]),
                *wanted,
                "expected {} <= {} to be {}",
                names[i],
                names[j],
                wanted
            );
        }
    }

    // b and c are the concurrent pair, so a sync between them carries events
    // in both directions and must be resolved as a conflict.
    assert!(!leq(&b, &c));
    assert!(!leq(&c, &b));
    assert!(!compute_missing_locally(&b, &c).is_empty());
    assert!(!compute_missing_locally(&c, &b).is_empty());
}

// ── Merge laws over multi-key clocks ───────────────────────────────────────

#[test]
fn merge_of_concurrent_clocks_is_the_least_upper_bound() {
    let a = clock("dev-a", &[("Feature", "1", 1), ("Epic", "2", 5)]);
    let b = clock("dev-b", &[("Feature", "1", 9), ("Story", "3", 2)]);

    let mut join = a.clone();
    join.merge(&b);

    // Upper bound: the join dominates both inputs.
    assert!(leq(&a, &join));
    assert!(leq(&b, &join));
    // Least: it invents no sequence beyond the per-key maximum.
    assert_eq!(join.get("Feature", "1"), 9);
    assert_eq!(join.get("Epic", "2"), 5);
    assert_eq!(join.get("Story", "3"), 2);
    assert_eq!(entries_of(&join).len(), 3);

    // Every other common upper bound is dominated by the join.
    let upper = clock(
        "dev-u",
        &[("Feature", "1", 40), ("Epic", "2", 5), ("Story", "3", 2)],
    );
    assert!(leq(&a, &upper));
    assert!(leq(&b, &upper));
    assert!(leq(&join, &upper));
}

#[test]
fn merge_is_commutative_associative_and_idempotent_across_keys() {
    let a = clock("dev-a", &[("Feature", "1", 1), ("Epic", "2", 5)]);
    let b = clock("dev-b", &[("Feature", "1", 9), ("Story", "3", 2)]);
    let c = clock(
        "dev-c",
        &[("Epic", "2", 7), ("Story", "3", 1), ("Bug", "4", 6)],
    );

    let mut a_then_b = a.clone();
    a_then_b.merge(&b);
    let mut b_then_a = b.clone();
    b_then_a.merge(&a);
    assert_eq!(
        entries_of(&a_then_b),
        entries_of(&b_then_a),
        "merge must not depend on argument order"
    );

    let mut left = a.clone();
    left.merge(&b);
    left.merge(&c);

    let mut b_merged_c = b.clone();
    b_merged_c.merge(&c);
    let mut right = a.clone();
    right.merge(&b_merged_c);
    assert_eq!(
        entries_of(&left),
        entries_of(&right),
        "merge must not depend on grouping"
    );

    let mut merged_twice = a.clone();
    merged_twice.merge(&b);
    merged_twice.merge(&b);
    assert_eq!(entries_of(&merged_twice), entries_of(&a_then_b));

    // Re-merging the join with an operand it already absorbed changes nothing.
    let mut join_again = a_then_b.clone();
    join_again.merge(&a);
    join_again.merge(&b);
    assert_eq!(entries_of(&join_again), entries_of(&a_then_b));
}

#[test]
fn mutually_reachable_clocks_agree_on_every_key() {
    let a = clock("dev-a", &[("Feature", "1", 4), ("Epic", "2", 0)]);
    let b = clock("dev-b", &[("Feature", "1", 4), ("Epic", "2", 0)]);

    assert!(leq(&a, &b));
    assert!(leq(&b, &a));
    assert_eq!(entries_of(&a), entries_of(&b));
}

// ── Serde behaviour ────────────────────────────────────────────────────────

#[test]
fn sync_vector_serde_roundtrips_when_entries_are_empty() {
    let vector = SyncVector::new("dev-1");
    let json = serde_json::to_string(&vector).expect("an empty entry map is JSON-encodable");
    let back: SyncVector = serde_json::from_str(&json).unwrap();
    assert_eq!(back.device_id, "dev-1");
    assert!(back.entries.is_empty());
}

#[test]
fn sync_vector_serde_rejects_populated_entries() {
    // `entries` is keyed by `(String, String)`, which serde_json cannot address
    // as an object key. Both directions must fail loudly rather than silently
    // dropping watermarks, which is why the export path projects the vector
    // through `serde_json::Value` by hand.
    let mut vector = SyncVector::new("dev-1");
    vector.advance("Feature", "1", 3);

    assert!(serde_json::to_string(&vector).is_err());
    assert!(serde_json::to_value(&vector).is_err());

    let with_entries = r#"{"device_id":"dev-1","entries":{"Feature/1":3}}"#;
    let decoded = serde_json::from_str::<SyncVector>(with_entries);
    assert!(
        decoded.is_err(),
        "a string-keyed entries map must not decode into tuple keys"
    );
}

#[test]
fn sync_vector_deserializes_from_json_with_empty_entries() {
    let decoded: SyncVector =
        serde_json::from_str(r#"{"device_id":"dev-9","entries":{}}"#).unwrap();
    assert_eq!(decoded.device_id, "dev-9");
    assert_eq!(decoded.get("Feature", "1"), 0);
}

// ── sync_with_peer_vectors error path ──────────────────────────────────────

/// Event store that fails every read, so the sync aborts before it can reach
/// the network.
#[derive(Debug, Default)]
struct FailingEventStore;

fn store_failure() -> EventError {
    EventError::StorageError("event store down".to_string())
}

#[async_trait]
impl EventStore for FailingEventStore {
    async fn append(&self, _event: &Event) -> Result<i64, EventError> {
        Err(store_failure())
    }

    async fn get_events(
        &self,
        _entity_type: &str,
        _entity_id: i64,
    ) -> Result<Vec<Event>, EventError> {
        Err(store_failure())
    }

    async fn get_events_since(
        &self,
        _entity_type: &str,
        _entity_id: i64,
        _sequence: i64,
    ) -> Result<Vec<Event>, EventError> {
        Err(store_failure())
    }

    async fn get_events_by_range(
        &self,
        _entity_type: &str,
        _entity_id: i64,
        _from: chrono::DateTime<chrono::Utc>,
        _to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Event>, EventError> {
        Err(store_failure())
    }

    async fn get_latest_sequence(
        &self,
        _entity_type: &str,
        _entity_id: i64,
    ) -> Result<i64, EventError> {
        Err(store_failure())
    }
}

#[tokio::test]
async fn sync_surfaces_event_store_failure_as_sync_error() {
    let local = clock("dev-local", &[("Feature", "1", 5)]);
    let remote = clock("dev-peer", &[("Feature", "1", 1)]);

    let error = sync_with_peer_vectors(
        "dev-local",
        &peer("dev-peer"),
        &local,
        &remote,
        &FailingEventStore,
    )
    .await
    .unwrap_err();

    match error {
        SyncError::EventStore(message) => {
            assert!(
                message.contains("event store down"),
                "underlying cause should survive: {message}"
            );
        }
        other => panic!("expected EventStore, got {other:?}"),
    }
}

#[tokio::test]
async fn sync_with_equal_vectors_needs_no_reads() {
    // Both sides agree, so nothing is pending: the store must not be consulted
    // for any entity stream.
    #[derive(Default)]
    struct PanickingStore;

    #[async_trait]
    impl EventStore for PanickingStore {
        async fn append(&self, _event: &Event) -> Result<i64, EventError> {
            panic!("append must not be called when vectors agree");
        }
        async fn get_events(
            &self,
            _entity_type: &str,
            _entity_id: i64,
        ) -> Result<Vec<Event>, EventError> {
            panic!("get_events must not be called when vectors agree");
        }
        async fn get_events_since(
            &self,
            _entity_type: &str,
            _entity_id: i64,
            _sequence: i64,
        ) -> Result<Vec<Event>, EventError> {
            panic!("get_events_since must not be called when vectors agree");
        }
        async fn get_events_by_range(
            &self,
            _entity_type: &str,
            _entity_id: i64,
            _from: chrono::DateTime<chrono::Utc>,
            _to: chrono::DateTime<chrono::Utc>,
        ) -> Result<Vec<Event>, EventError> {
            panic!("get_events_by_range must not be called when vectors agree");
        }
        async fn get_latest_sequence(
            &self,
            _entity_type: &str,
            _entity_id: i64,
        ) -> Result<i64, EventError> {
            panic!("get_latest_sequence must not be called when vectors agree");
        }
    }

    let shared = clock("dev-local", &[("Feature", "1", 4), ("Epic", "2", 9)]);
    let result = sync_with_peer_vectors(
        "dev-local",
        &peer("dev-peer"),
        &shared,
        &shared.clone(),
        &PanickingStore,
    )
    .await
    .unwrap();

    assert_eq!(entries_of(&result.updated_vector), entries_of(&shared));
}
