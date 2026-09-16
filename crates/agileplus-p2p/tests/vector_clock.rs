//! Integration tests for the vector_clock module.
//!
//! Tests pure logic of SyncVector: creation, advancing, merging,
//! computing missing events, and serde roundtrips.

use agileplus_p2p::vector_clock::{SyncResult, SyncVector, compute_missing_locally};

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
        v.entries.get(&("WorkPackage".to_string(), "42".to_string())),
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
