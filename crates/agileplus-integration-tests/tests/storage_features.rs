//! StoragePort contract: feature CRUD, state transitions, and labels.
//!
//! Every test uses an isolated in-memory SQLite adapter through the public
//! `StoragePort` trait — no file I/O and no network required.
//!
//! Traceability: WP19-T107/T108 (storage contract surface)

use agileplus_domain::{
    domain::{feature::Feature, state_machine::FeatureState},
    error::DomainError,
    ports::StoragePort,
};
use agileplus_sqlite::SqliteStorageAdapter;

fn storage() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory adapter should initialise")
}

async fn seed_feature(storage: &SqliteStorageAdapter, slug: &str) -> i64 {
    storage
        .create_feature(&Feature::new(slug, slug, [0u8; 32], None))
        .await
        .expect("create_feature should succeed")
}

#[tokio::test]
async fn feature_create_returns_positive_id() {
    let storage = storage();
    let id = seed_feature(&storage, "feat-alpha").await;
    assert!(id > 0, "assigned feature id must be positive");
}

#[tokio::test]
async fn feature_create_and_read_by_id_roundtrips() {
    let storage = storage();
    let id = seed_feature(&storage, "feat-beta").await;

    let feature = storage
        .get_feature_by_id(id)
        .await
        .expect("get_feature_by_id should succeed")
        .expect("feature should exist");

    assert_eq!(feature.slug, "feat-beta");
    assert_eq!(feature.friendly_name, "feat-beta");
    assert_eq!(feature.state, FeatureState::Created);
    assert_eq!(feature.target_branch, "main");
}

#[tokio::test]
async fn feature_get_by_slug_roundtrips() {
    let storage = storage();
    let id = seed_feature(&storage, "slug-feature").await;

    let feature = storage
        .get_feature_by_slug("slug-feature")
        .await
        .expect("get_feature_by_slug should succeed")
        .expect("feature should exist");

    assert_eq!(feature.id, id);
}

#[tokio::test]
async fn feature_get_by_slug_missing_returns_none() {
    let storage = storage();
    let result = storage
        .get_feature_by_slug("does-not-exist")
        .await
        .expect("lookup should succeed");
    assert!(result.is_none());
}

#[tokio::test]
async fn feature_get_by_id_missing_returns_none() {
    let storage = storage();
    let result = storage
        .get_feature_by_id(4242)
        .await
        .expect("lookup should succeed");
    assert!(result.is_none());
}

#[tokio::test]
async fn feature_duplicate_slug_is_rejected() {
    let storage = storage();
    seed_feature(&storage, "dup-slug").await;

    let result = storage
        .create_feature(&Feature::new("dup-slug", "Dup", [0u8; 32], None))
        .await;

    assert!(
        matches!(result, Err(DomainError::Storage(_))),
        "duplicate slug should surface a storage error, got: {result:?}"
    );
}

#[tokio::test]
async fn feature_list_all_includes_created_features() {
    let storage = storage();
    let a = seed_feature(&storage, "list-a").await;
    let b = seed_feature(&storage, "list-b").await;

    let ids: Vec<i64> = storage
        .list_all_features()
        .await
        .expect("list_all_features should succeed")
        .into_iter()
        .map(|f| f.id)
        .collect();

    assert!(ids.contains(&a) && ids.contains(&b));
}

#[tokio::test]
async fn feature_list_by_state_filters_correctly() {
    let storage = storage();
    let created = seed_feature(&storage, "state-created").await;
    let moved = seed_feature(&storage, "state-moved").await;
    storage
        .update_feature_state(moved, FeatureState::Specified)
        .await
        .expect("transition should succeed");

    let specified = storage
        .list_features_by_state(FeatureState::Specified)
        .await
        .expect("list_features_by_state should succeed");

    assert_eq!(specified.len(), 1);
    assert_eq!(specified[0].id, moved);

    let created_list = storage
        .list_features_by_state(FeatureState::Created)
        .await
        .expect("list_features_by_state should succeed");
    assert_eq!(created_list.len(), 1);
    assert_eq!(created_list[0].id, created);
}

#[tokio::test]
async fn feature_state_update_persists() {
    let storage = storage();
    let id = seed_feature(&storage, "persist-state").await;

    for state in [
        FeatureState::Specified,
        FeatureState::Researched,
        FeatureState::Planned,
    ] {
        storage
            .update_feature_state(id, state)
            .await
            .expect("transition should succeed");
        let feature = storage
            .get_feature_by_id(id)
            .await
            .expect("lookup should succeed")
            .expect("feature should exist");
        assert_eq!(feature.state, state);
    }
}

#[tokio::test]
async fn feature_labels_roundtrip_through_storage() {
    let storage = storage();
    let mut feature = Feature::new("labelled", "Labelled", [0x11u8; 32], Some("develop"));
    feature.labels = vec!["backend".to_string(), "security".to_string()];

    let id = storage
        .create_feature(&feature)
        .await
        .expect("create_feature should succeed");

    let stored = storage
        .get_feature_by_id(id)
        .await
        .expect("lookup should succeed")
        .expect("feature should exist");
    assert_eq!(stored.labels, vec!["backend", "security"]);
    assert_eq!(stored.target_branch, "develop");
    assert_eq!(stored.spec_hash, [0x11u8; 32]);
}

#[tokio::test]
async fn feature_spec_hash_and_branch_persist_verbatim() {
    let storage = storage();
    let feature = Feature::new("verbatim", "Verbatim", [0xABu8; 32], Some("release/1.2"));

    let id = storage
        .create_feature(&feature)
        .await
        .expect("create_feature should succeed");

    let stored = storage
        .get_feature_by_id(id)
        .await
        .expect("lookup should succeed")
        .expect("feature should exist");
    assert_eq!(stored.spec_hash, [0xABu8; 32]);
    assert_eq!(stored.target_branch, "release/1.2");
}

#[tokio::test]
async fn feature_list_all_empty_on_fresh_database() {
    let storage = storage();
    let features = storage
        .list_all_features()
        .await
        .expect("list_all_features should succeed");
    assert!(features.is_empty());
}
