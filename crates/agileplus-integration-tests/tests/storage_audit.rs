//! StoragePort contract: tamper-evident audit chain.
//!
//! Every test uses an isolated in-memory SQLite adapter through the public
//! `StoragePort` trait — no file I/O and no network required.
//!
//! Traceability: WP19-T108 (audit hash-chain surface)

use agileplus_domain::{
    domain::{
        audit::{AuditEntry, EvidenceRef},
        feature::Feature,
    },
    error::DomainError,
    ports::StoragePort,
};
use agileplus_sqlite::SqliteStorageAdapter;
use chrono::Utc;

fn storage() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory adapter should initialise")
}

async fn seed_feature(storage: &SqliteStorageAdapter, slug: &str) -> i64 {
    storage
        .create_feature(&Feature::new(slug, slug, [0u8; 32], None))
        .await
        .expect("create_feature should succeed")
}

fn audit_entry(feature_id: i64, prev_hash: [u8; 32], hash: [u8; 32]) -> AuditEntry {
    AuditEntry {
        id: 0,
        feature_id,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "tester".to_string(),
        transition: "created->specified".to_string(),
        evidence_refs: vec![],
        prev_hash,
        hash,
        event_id: None,
        archived_to: None,
    }
}

#[tokio::test]
async fn audit_append_first_entry_succeeds() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "audit-first").await;

    let id = storage
        .append_audit_entry(&audit_entry(feature_id, [0u8; 32], [1u8; 32]))
        .await
        .expect("append_audit_entry should succeed");
    assert!(id > 0);
}

#[tokio::test]
async fn audit_first_entry_with_nonzero_prev_hash_is_rejected() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "audit-bad-first").await;

    let result = storage
        .append_audit_entry(&audit_entry(feature_id, [9u8; 32], [1u8; 32]))
        .await;
    assert!(
        matches!(result, Err(DomainError::Storage(_))),
        "first entry must have an all-zero prev_hash"
    );
}

#[tokio::test]
async fn audit_chain_links_entries() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "audit-chain").await;
    storage
        .append_audit_entry(&audit_entry(feature_id, [0u8; 32], [1u8; 32]))
        .await
        .expect("first append should succeed");
    storage
        .append_audit_entry(&audit_entry(feature_id, [1u8; 32], [2u8; 32]))
        .await
        .expect("second append should succeed");

    let trail = storage
        .get_audit_trail(feature_id)
        .await
        .expect("get_audit_trail should succeed");
    assert_eq!(trail.len(), 2);
    assert_eq!(trail[0].prev_hash, [0u8; 32]);
    assert_eq!(trail[1].prev_hash, trail[0].hash);
}

#[tokio::test]
async fn audit_wrong_prev_hash_is_rejected() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "audit-wrong-prev").await;
    storage
        .append_audit_entry(&audit_entry(feature_id, [0u8; 32], [1u8; 32]))
        .await
        .expect("first append should succeed");

    let result = storage
        .append_audit_entry(&audit_entry(feature_id, [7u8; 32], [2u8; 32]))
        .await;
    assert!(
        matches!(result, Err(DomainError::Storage(_))),
        "broken chain link must be rejected"
    );
}

#[tokio::test]
async fn audit_trail_preserves_insertion_order() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "audit-order").await;
    let hashes = [[1u8; 32], [2u8; 32], [3u8; 32]];
    let mut prev = [0u8; 32];
    for hash in hashes {
        storage
            .append_audit_entry(&audit_entry(feature_id, prev, hash))
            .await
            .expect("append should succeed");
        prev = hash;
    }

    let trail = storage
        .get_audit_trail(feature_id)
        .await
        .expect("get_audit_trail should succeed");
    let observed: Vec<[u8; 32]> = trail.iter().map(|e| e.hash).collect();
    assert_eq!(observed, hashes.to_vec());
}

#[tokio::test]
async fn audit_latest_entry_returns_last() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "audit-latest").await;
    storage
        .append_audit_entry(&audit_entry(feature_id, [0u8; 32], [1u8; 32]))
        .await
        .expect("append should succeed");
    storage
        .append_audit_entry(&audit_entry(feature_id, [1u8; 32], [2u8; 32]))
        .await
        .expect("append should succeed");

    let latest = storage
        .get_latest_audit_entry(feature_id)
        .await
        .expect("get_latest_audit_entry should succeed")
        .expect("latest entry should exist");
    assert_eq!(latest.hash, [2u8; 32]);
}

#[tokio::test]
async fn audit_latest_missing_returns_none() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "audit-none").await;

    let latest = storage
        .get_latest_audit_entry(feature_id)
        .await
        .expect("get_latest_audit_entry should succeed");
    assert!(latest.is_none());
}

#[tokio::test]
async fn audit_trail_missing_returns_empty() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "audit-empty").await;

    let trail = storage
        .get_audit_trail(feature_id)
        .await
        .expect("get_audit_trail should succeed");
    assert!(trail.is_empty());
}

#[tokio::test]
async fn audit_trails_are_isolated_per_feature() {
    let storage = storage();
    let feature_a = seed_feature(&storage, "audit-iso-a").await;
    let feature_b = seed_feature(&storage, "audit-iso-b").await;
    storage
        .append_audit_entry(&audit_entry(feature_a, [0u8; 32], [1u8; 32]))
        .await
        .expect("append should succeed");
    storage
        .append_audit_entry(&audit_entry(feature_b, [0u8; 32], [2u8; 32]))
        .await
        .expect("append should succeed");

    let trail_a = storage
        .get_audit_trail(feature_a)
        .await
        .expect("trail should load");
    let trail_b = storage
        .get_audit_trail(feature_b)
        .await
        .expect("trail should load");
    assert_eq!(trail_a.len(), 1);
    assert_eq!(trail_b.len(), 1);
    assert_eq!(trail_a[0].hash, [1u8; 32]);
    assert_eq!(trail_b[0].hash, [2u8; 32]);
}

#[tokio::test]
async fn audit_evidence_refs_roundtrip() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "audit-refs").await;
    let mut entry = audit_entry(feature_id, [0u8; 32], [1u8; 32]);
    entry.evidence_refs = vec![EvidenceRef {
        evidence_id: 7,
        fr_id: "FR-CACHE".to_string(),
    }];

    storage
        .append_audit_entry(&entry)
        .await
        .expect("append should succeed");

    let trail = storage
        .get_audit_trail(feature_id)
        .await
        .expect("get_audit_trail should succeed");
    assert_eq!(trail[0].evidence_refs.len(), 1);
    assert_eq!(trail[0].evidence_refs[0].fr_id, "FR-CACHE");
    assert_eq!(trail[0].evidence_refs[0].evidence_id, 7);
}

#[tokio::test]
async fn audit_actor_and_transition_roundtrip() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "audit-fields").await;
    let mut entry = audit_entry(feature_id, [0u8; 32], [1u8; 32]);
    entry.actor = "agent:jcode".to_string();
    entry.transition = "planned->implementing".to_string();

    storage
        .append_audit_entry(&entry)
        .await
        .expect("append should succeed");

    let trail = storage
        .get_audit_trail(feature_id)
        .await
        .expect("get_audit_trail should succeed");
    assert_eq!(trail[0].actor, "agent:jcode");
    assert_eq!(trail[0].transition, "planned->implementing");
}
