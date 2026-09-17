//! StoragePort contract: module hierarchy and cycle lifecycle management.
//!
//! Complements `modules_and_cycles.rs` with update/delete/list coverage for the
//! module and cycle aggregates. Every test uses an isolated in-memory SQLite
//! adapter through the public `StoragePort` trait.
//!
//! Traceability: FR-M01/FR-M02/FR-M04/FR-M07, FR-C01/FR-C02/FR-C03

use agileplus_domain::{
    domain::{
        cycle::{Cycle, CycleFeature, CycleState},
        feature::Feature,
        module::{Module, ModuleFeatureTag},
    },
    error::DomainError,
    ports::StoragePort,
};
use agileplus_sqlite::SqliteStorageAdapter;
use chrono::NaiveDate;

fn storage() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory adapter should initialise")
}

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

async fn seed_feature(storage: &SqliteStorageAdapter, slug: &str) -> i64 {
    storage
        .create_feature(&Feature::new(slug, slug, [0u8; 32], None))
        .await
        .expect("create_feature should succeed")
}

async fn seed_module(storage: &SqliteStorageAdapter, name: &str, parent: Option<i64>) -> i64 {
    storage
        .create_module(&Module::new(name, parent))
        .await
        .expect("create_module should succeed")
}

async fn seed_cycle(
    storage: &SqliteStorageAdapter,
    name: &str,
    module_scope_id: Option<i64>,
) -> i64 {
    let cycle = Cycle::new(name, date(2026, 1, 1), date(2026, 3, 31), module_scope_id)
        .expect("cycle construction should succeed");
    storage
        .create_cycle(&cycle)
        .await
        .expect("create_cycle should succeed")
}

/// Assign `features.module_id` directly (StoragePort exposes no setter yet).
fn assign_feature_module_id(
    storage: &SqliteStorageAdapter,
    feature_id: i64,
    module_id: i64,
) -> rusqlite::Result<()> {
    let conn = storage.conn_for_bench().expect("lock should succeed");
    conn.execute(
        "UPDATE features SET module_id = ?1 WHERE id = ?2",
        rusqlite::params![module_id, feature_id],
    )?;
    Ok(())
}

// --- Module lifecycle -----------------------------------------------------

#[tokio::test]
async fn module_get_by_slug_roundtrips() {
    let storage = storage();
    let id = seed_module(&storage, "Core Platform", None).await;

    let module = storage
        .get_module_by_slug("core-platform")
        .await
        .expect("get_module_by_slug should succeed")
        .expect("module should exist");
    assert_eq!(module.id, id);
    assert_eq!(module.friendly_name, "Core Platform");
}

#[tokio::test]
async fn module_get_by_slug_missing_returns_none() {
    let storage = storage();
    let result = storage
        .get_module_by_slug("no-such-module")
        .await
        .expect("lookup should succeed");
    assert!(result.is_none());
}

#[tokio::test]
async fn module_update_re_slugs_and_persists() {
    let storage = storage();
    let id = seed_module(&storage, "Old Name", None).await;

    storage
        .update_module(id, "New Name", Some("fresh description"))
        .await
        .expect("update_module should succeed");

    let module = storage
        .get_module(id)
        .await
        .expect("lookup should succeed")
        .expect("module should exist");
    assert_eq!(module.friendly_name, "New Name");
    assert_eq!(module.slug, "new-name");
    assert_eq!(module.description.as_deref(), Some("fresh description"));
}

#[tokio::test]
async fn module_update_missing_id_errors() {
    let storage = storage();
    let result = storage.update_module(4242, "Ghost", None).await;
    assert!(
        matches!(result, Err(DomainError::ModuleNotFound(_))),
        "expected ModuleNotFound, got: {result:?}"
    );
}

#[tokio::test]
async fn module_delete_leaf_succeeds() {
    let storage = storage();
    let id = seed_module(&storage, "Disposable", None).await;

    storage
        .delete_module(id)
        .await
        .expect("delete_module should succeed");

    let module = storage.get_module(id).await.expect("lookup should succeed");
    assert!(module.is_none());
}

#[tokio::test]
async fn module_delete_missing_id_errors() {
    let storage = storage();
    let result = storage.delete_module(9999).await;
    assert!(
        matches!(result, Err(DomainError::ModuleNotFound(_))),
        "expected ModuleNotFound, got: {result:?}"
    );
}

#[tokio::test]
async fn module_create_with_missing_parent_errors() {
    let storage = storage();
    let result = storage
        .create_module(&Module::new("Orphan Child", Some(4242)))
        .await;
    assert!(
        matches!(result, Err(DomainError::ModuleNotFound(_))),
        "expected ModuleNotFound for unknown parent, got: {result:?}"
    );
}

#[tokio::test]
async fn module_get_with_features_includes_owned_and_children() {
    let storage = storage();
    let parent = seed_module(&storage, "Parent", None).await;
    let child = seed_module(&storage, "Child", Some(parent)).await;
    let feature_id = seed_feature(&storage, "owned-feature").await;
    assign_feature_module_id(&storage, feature_id, parent).expect("raw update should succeed");

    let view = storage
        .get_module_with_features(parent)
        .await
        .expect("get_module_with_features should succeed")
        .expect("module should exist");

    assert_eq!(view.module.id, parent);
    assert_eq!(
        view.owned_features.iter().map(|f| f.id).collect::<Vec<_>>(),
        vec![feature_id]
    );
    assert_eq!(
        view.child_modules.iter().map(|m| m.id).collect::<Vec<_>>(),
        vec![child]
    );
}

#[tokio::test]
async fn module_tag_is_idempotent() {
    let storage = storage();
    let module_id = seed_module(&storage, "Tagged", None).await;
    let feature_id = seed_feature(&storage, "tag-idempotent").await;
    let tag = ModuleFeatureTag::new(module_id, feature_id);

    storage
        .tag_feature_to_module(&tag)
        .await
        .expect("first tag should succeed");
    storage
        .tag_feature_to_module(&tag)
        .await
        .expect("second tag should be idempotent");

    let view = storage
        .get_module_with_features(module_id)
        .await
        .expect("view should load")
        .expect("module should exist");
    assert_eq!(view.tagged_features.len(), 1);
}

#[tokio::test]
async fn module_untag_removes_tag() {
    let storage = storage();
    let module_id = seed_module(&storage, "Untagged", None).await;
    let feature_id = seed_feature(&storage, "untag-feature").await;
    storage
        .tag_feature_to_module(&ModuleFeatureTag::new(module_id, feature_id))
        .await
        .expect("tag should succeed");

    storage
        .untag_feature_from_module(module_id, feature_id)
        .await
        .expect("untag should succeed");

    let view = storage
        .get_module_with_features(module_id)
        .await
        .expect("view should load")
        .expect("module should exist");
    assert!(view.tagged_features.is_empty());
}

// --- Cycle lifecycle ------------------------------------------------------

#[tokio::test]
async fn cycle_update_state_persists() {
    let storage = storage();
    let id = seed_cycle(&storage, "State Cycle", None).await;

    storage
        .update_cycle_state(id, CycleState::Active)
        .await
        .expect("update_cycle_state should succeed");

    let cycle = storage
        .get_cycle(id)
        .await
        .expect("get_cycle should succeed")
        .expect("cycle should exist");
    assert_eq!(cycle.state, CycleState::Active);
}

#[tokio::test]
async fn cycle_update_state_missing_id_errors() {
    let storage = storage();
    let result = storage.update_cycle_state(4242, CycleState::Active).await;
    assert!(
        matches!(result, Err(DomainError::CycleNotFound(_))),
        "expected CycleNotFound, got: {result:?}"
    );
}

#[tokio::test]
async fn cycle_list_by_state_filters() {
    let storage = storage();
    let draft = seed_cycle(&storage, "Draft Cycle", None).await;
    let active = seed_cycle(&storage, "Active Cycle", None).await;
    storage
        .update_cycle_state(active, CycleState::Active)
        .await
        .expect("update should succeed");

    let drafts = storage
        .list_cycles_by_state(CycleState::Draft)
        .await
        .expect("list_cycles_by_state should succeed");
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].id, draft);

    let actives = storage
        .list_cycles_by_state(CycleState::Active)
        .await
        .expect("list_cycles_by_state should succeed");
    assert_eq!(actives.len(), 1);
    assert_eq!(actives[0].id, active);
}

#[tokio::test]
async fn cycle_list_by_module_scopes_to_module() {
    let storage = storage();
    let module_id = seed_module(&storage, "Scoped Cycle Module", None).await;
    let scoped = seed_cycle(&storage, "Scoped Cycle", Some(module_id)).await;
    seed_cycle(&storage, "Global Cycle", None).await;

    let cycles = storage
        .list_cycles_by_module(module_id)
        .await
        .expect("list_cycles_by_module should succeed");
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].id, scoped);
    assert_eq!(cycles[0].module_scope_id, Some(module_id));
}

#[tokio::test]
async fn cycle_list_all_returns_every_cycle() {
    let storage = storage();
    seed_cycle(&storage, "Cycle A", None).await;
    seed_cycle(&storage, "Cycle B", None).await;

    let cycles = storage
        .list_all_cycles()
        .await
        .expect("list_all_cycles should succeed");
    assert_eq!(cycles.len(), 2);
}

#[tokio::test]
async fn cycle_duplicate_name_is_rejected() {
    let storage = storage();
    seed_cycle(&storage, "Unique Name", None).await;

    let duplicate = Cycle::new("Unique Name", date(2026, 1, 1), date(2026, 2, 1), None)
        .expect("cycle construction should succeed");
    let result = storage.create_cycle(&duplicate).await;
    assert!(
        matches!(result, Err(DomainError::Storage(_))),
        "cycle name is unique"
    );
}

#[tokio::test]
async fn cycle_remove_feature_unlinks_without_changing_state() {
    let storage = storage();
    let cycle_id = seed_cycle(&storage, "Unlink Cycle", None).await;
    let feature_id = seed_feature(&storage, "unlink-feature").await;
    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("add should succeed");

    storage
        .remove_feature_from_cycle(cycle_id, feature_id)
        .await
        .expect("remove should succeed");

    let view = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");
    assert!(view.features.is_empty());

    let feature = storage
        .get_feature_by_id(feature_id)
        .await
        .expect("lookup should succeed")
        .expect("feature should exist");
    assert_eq!(
        feature.state,
        agileplus_domain::domain::state_machine::FeatureState::Created
    );
}

#[tokio::test]
async fn cycle_get_with_features_empty_progress_is_zero() {
    let storage = storage();
    let cycle_id = seed_cycle(&storage, "Empty Cycle", None).await;

    let view = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");
    assert_eq!(view.wp_progress.total, 0);
    assert!(view.is_shippable(), "an empty cycle is vacuously shippable");
}

#[tokio::test]
async fn cycle_add_feature_is_idempotent() {
    let storage = storage();
    let cycle_id = seed_cycle(&storage, "Idempotent Cycle", None).await;
    let feature_id = seed_feature(&storage, "idempotent-feature").await;
    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("first add should succeed");
    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("second add should be idempotent");

    let view = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("view should load")
        .expect("cycle should exist");
    assert_eq!(view.features.len(), 1);
}

#[tokio::test]
async fn cycle_add_feature_to_missing_cycle_errors() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "orphan-cycle-feature").await;
    let result = storage
        .add_feature_to_cycle(&CycleFeature::new(4242, feature_id))
        .await;
    assert!(
        matches!(result, Err(DomainError::CycleNotFound(_))),
        "expected CycleNotFound, got: {result:?}"
    );
}
