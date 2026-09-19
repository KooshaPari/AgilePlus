//! Integration tests for `domain::audit` — the tamper-evident hash chain as a
//! consumer verifies it, including the exact failure diagnostics.

use agileplus_domain::domain::audit::{AuditChain, AuditEntry, EvidenceRef, hash_entry};

fn entry(id: i64, prev_hash: [u8; 32]) -> AuditEntry {
    let mut entry = AuditEntry {
        id,
        feature_id: 1,
        wp_id: None,
        timestamp: chrono::DateTime::from_timestamp(1_700_000_000 + id, 0).unwrap(),
        actor: "tester".to_string(),
        transition: "draft->active".to_string(),
        evidence_refs: Vec::new(),
        prev_hash,
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    entry.hash = hash_entry(&entry);
    entry
}

fn chain(len: i64) -> Vec<AuditEntry> {
    let mut entries = Vec::new();
    let mut prev = [0u8; 32];
    for id in 1..=len {
        let next = entry(id, prev);
        prev = next.hash;
        entries.push(next);
    }
    entries
}

#[test]
fn a_long_linked_chain_verifies() {
    let entries = chain(32);
    let chain = AuditChain { entries };
    assert!(chain.verify_chain().is_ok());
}

#[test]
fn an_empty_chain_fails_with_a_stable_message() {
    let chain = AuditChain {
        entries: Vec::new(),
    };
    assert_eq!(chain.verify_chain().unwrap_err(), "empty audit chain");
}

#[test]
fn a_tampered_hash_is_reported_with_its_index_and_entry_id() {
    let mut entries = chain(3);
    entries[1].hash = [0xFF; 32];
    let error = AuditChain { entries }.verify_chain().unwrap_err();
    assert!(
        error.contains("hash mismatch at entry index 1"),
        "got {error:?}"
    );
    assert!(error.contains("id=2"), "got {error:?}");
}

#[test]
fn a_broken_link_reports_the_neighbouring_entry_ids_and_indices() {
    let mut entries = chain(3);
    // Re-point the third entry at the first entry's hash, skipping entry 2.
    entries[2].prev_hash = entries[0].hash;
    entries[2].hash = hash_entry(&entries[2]);
    let error = AuditChain { entries }.verify_chain().unwrap_err();
    assert!(
        error.contains("chain break between entries 2 and 3"),
        "got {error:?}"
    );
    assert!(error.contains("index 1-2"), "got {error:?}");
}

#[test]
fn a_reordered_chain_is_rejected() {
    let mut entries = chain(3);
    entries.swap(0, 1);
    assert!(AuditChain { entries }.verify_chain().is_err());
}

#[test]
fn a_nonzero_genesis_prev_hash_is_accepted() {
    // The first entry has nothing to link against, so any prev_hash is valid.
    let genesis = entry(1, [0x7A; 32]);
    let chain = AuditChain {
        entries: vec![genesis],
    };
    assert!(chain.verify_chain().is_ok());
}

#[test]
fn the_hash_covers_actor_transition_timestamp_feature_wp_and_prev_hash() {
    let base = entry(1, [1u8; 32]);
    let original = hash_entry(&base);

    let mut actor = base.clone();
    actor.actor = "someone-else".to_string();
    assert_ne!(hash_entry(&actor), original, "actor");

    let mut transition = base.clone();
    transition.transition = "active->review".to_string();
    assert_ne!(hash_entry(&transition), original, "transition");

    let mut timestamp = base.clone();
    timestamp.timestamp = chrono::DateTime::from_timestamp(1, 0).unwrap();
    assert_ne!(hash_entry(&timestamp), original, "timestamp");

    let mut feature = base.clone();
    feature.feature_id = 2;
    assert_ne!(hash_entry(&feature), original, "feature_id");

    let mut wp = base.clone();
    wp.wp_id = Some(9);
    assert_ne!(hash_entry(&wp), original, "wp_id");

    let mut prev = base.clone();
    prev.prev_hash = [2u8; 32];
    assert_ne!(hash_entry(&prev), original, "prev_hash");
}

#[test]
fn the_hash_ignores_bookkeeping_fields() {
    let base = entry(1, [1u8; 32]);
    let original = hash_entry(&base);

    let mut mutated = base.clone();
    mutated.id = 999;
    mutated.event_id = Some(42);
    mutated.archived_to = Some("archive/2026".to_string());
    mutated.evidence_refs = vec![EvidenceRef {
        evidence_id: 7,
        fr_id: "FR-7".to_string(),
    }];
    assert_eq!(hash_entry(&mutated), original);
}

#[test]
fn hashing_is_deterministic_and_entry_round_trips_through_json() {
    let with_evidence = {
        let mut e = entry(4, [9u8; 32]);
        e.wp_id = Some(3);
        e.event_id = Some(5);
        e.evidence_refs = vec![EvidenceRef {
            evidence_id: 1,
            fr_id: "FR-1".to_string(),
        }];
        e
    };
    assert_eq!(hash_entry(&with_evidence), hash_entry(&with_evidence));

    let back: AuditEntry =
        serde_json::from_str(&serde_json::to_string(&with_evidence).unwrap()).unwrap();
    assert_eq!(back.id, 4);
    assert_eq!(back.wp_id, Some(3));
    assert_eq!(back.hash, with_evidence.hash);
    assert_eq!(back.evidence_refs.len(), 1);
    assert_eq!(back.evidence_refs[0].fr_id, "FR-1");
}
