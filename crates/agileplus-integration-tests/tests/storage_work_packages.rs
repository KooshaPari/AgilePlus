//! StoragePort contract: work package lifecycle, dependencies, and readiness.
//!
//! Every test uses an isolated in-memory SQLite adapter through the public
//! `StoragePort` trait — no file I/O and no network required.
//!
//! Traceability: WP19-T108 (work package scheduling surface)

use agileplus_domain::{
    domain::{
        feature::Feature,
        work_package::{DependencyType, PrState, WorkPackage, WpDependency, WpState},
    },
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

fn work_package(feature_id: i64, title: &str, sequence: i32) -> WorkPackage {
    WorkPackage::new(feature_id, title, sequence, "acceptance criteria")
}

async fn create_wp(storage: &SqliteStorageAdapter, feature_id: i64, title: &str, seq: i32) -> i64 {
    storage
        .create_work_package(&work_package(feature_id, title, seq))
        .await
        .expect("create_work_package should succeed")
}

#[tokio::test]
async fn wp_create_and_get_roundtrips() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "wp-feature").await;
    let id = create_wp(&storage, feature_id, "Implement store", 1).await;

    let wp = storage
        .get_work_package(id)
        .await
        .expect("get_work_package should succeed")
        .expect("work package should exist");

    assert_eq!(wp.feature_id, feature_id);
    assert_eq!(wp.title, "Implement store");
    assert_eq!(wp.sequence, 1);
    assert_eq!(wp.state, WpState::Planned);
    assert_eq!(wp.acceptance_criteria, "acceptance criteria");
}

#[tokio::test]
async fn wp_get_missing_returns_none() {
    let storage = storage();
    let result = storage
        .get_work_package(9999)
        .await
        .expect("lookup should succeed");
    assert!(result.is_none());
}

#[tokio::test]
async fn wp_create_requires_existing_feature() {
    let storage = storage();
    let result = storage
        .create_work_package(&work_package(777, "Orphan WP", 1))
        .await;

    assert!(
        matches!(result, Err(DomainError::Storage(_))),
        "FK violation should surface a storage error, got: {result:?}"
    );
}

#[tokio::test]
async fn wp_list_by_feature_orders_by_sequence() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "ordered-wps").await;
    create_wp(&storage, feature_id, "third", 3).await;
    create_wp(&storage, feature_id, "first", 1).await;
    create_wp(&storage, feature_id, "second", 2).await;

    let titles: Vec<String> = storage
        .list_wps_by_feature(feature_id)
        .await
        .expect("list_wps_by_feature should succeed")
        .into_iter()
        .map(|wp| wp.title)
        .collect();

    assert_eq!(titles, vec!["first", "second", "third"]);
}

#[tokio::test]
async fn wp_list_by_feature_scopes_to_feature() {
    let storage = storage();
    let feature_a = seed_feature(&storage, "scope-a").await;
    let feature_b = seed_feature(&storage, "scope-b").await;
    create_wp(&storage, feature_a, "a-wp", 1).await;
    create_wp(&storage, feature_b, "b-wp", 1).await;

    let a_wps = storage
        .list_wps_by_feature(feature_a)
        .await
        .expect("list should succeed");
    assert_eq!(a_wps.len(), 1);
    assert_eq!(a_wps[0].title, "a-wp");
}

#[tokio::test]
async fn wp_list_by_feature_empty_for_unknown_feature() {
    let storage = storage();
    let wps = storage
        .list_wps_by_feature(12345)
        .await
        .expect("list should succeed");
    assert!(wps.is_empty());
}

#[tokio::test]
async fn wp_update_state_persists() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "wp-state").await;
    let id = create_wp(&storage, feature_id, "stateful", 1).await;

    storage
        .update_wp_state(id, WpState::Doing)
        .await
        .expect("update_wp_state should succeed");

    let wp = storage
        .get_work_package(id)
        .await
        .expect("lookup should succeed")
        .expect("work package should exist");
    assert_eq!(wp.state, WpState::Doing);
}

#[tokio::test]
async fn wp_update_state_supports_every_state() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "wp-all-states").await;
    let id = create_wp(&storage, feature_id, "all-states", 1).await;

    for state in [
        WpState::Doing,
        WpState::Review,
        WpState::Done,
        WpState::Blocked,
        WpState::Planned,
    ] {
        storage
            .update_wp_state(id, state)
            .await
            .expect("update_wp_state should succeed");
        let wp = storage
            .get_work_package(id)
            .await
            .expect("lookup should succeed")
            .expect("work package should exist");
        assert_eq!(wp.state, state);
    }
}

#[tokio::test]
async fn wp_file_scope_roundtrips() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "wp-file-scope").await;
    let mut wp = work_package(feature_id, "scoped", 1);
    wp.file_scope = vec!["src/lib.rs".to_string(), "src/store.rs".to_string()];

    let id = storage
        .create_work_package(&wp)
        .await
        .expect("create should succeed");

    let stored = storage
        .get_work_package(id)
        .await
        .expect("lookup should succeed")
        .expect("work package should exist");
    assert_eq!(stored.file_scope, vec!["src/lib.rs", "src/store.rs"]);
}

#[tokio::test]
async fn wp_pr_state_roundtrips() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "wp-pr-state").await;
    let mut wp = work_package(feature_id, "pr", 1);
    wp.pr_url = Some("https://example.invalid/pr/1".to_string());
    wp.pr_state = Some(PrState::Approved);

    let id = storage
        .create_work_package(&wp)
        .await
        .expect("create should succeed");

    let stored = storage
        .get_work_package(id)
        .await
        .expect("lookup should succeed")
        .expect("work package should exist");
    assert_eq!(
        stored.pr_url.as_deref(),
        Some("https://example.invalid/pr/1")
    );
    assert_eq!(stored.pr_state, Some(PrState::Approved));
}

#[tokio::test]
async fn wp_pr_state_none_roundtrips() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "wp-no-pr").await;
    let id = create_wp(&storage, feature_id, "no-pr", 1).await;

    let stored = storage
        .get_work_package(id)
        .await
        .expect("lookup should succeed")
        .expect("work package should exist");
    assert!(stored.pr_state.is_none());
    assert!(stored.pr_url.is_none());
}

#[tokio::test]
async fn wp_dependency_add_and_get() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "dep-feature").await;
    let first = create_wp(&storage, feature_id, "first", 1).await;
    let second = create_wp(&storage, feature_id, "second", 2).await;

    storage
        .add_wp_dependency(&WpDependency {
            wp_id: second,
            depends_on: first,
            dep_type: DependencyType::Explicit,
        })
        .await
        .expect("add_wp_dependency should succeed");

    let deps = storage
        .get_wp_dependencies(second)
        .await
        .expect("get_wp_dependencies should succeed");
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].depends_on, first);
    assert_eq!(deps[0].dep_type, DependencyType::Explicit);
}

#[tokio::test]
async fn wp_dependency_types_all_roundtrip() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "dep-types").await;
    let target = create_wp(&storage, feature_id, "target", 1).await;
    let mut expected = Vec::new();

    for (index, dep_type) in [
        DependencyType::Explicit,
        DependencyType::FileOverlap,
        DependencyType::Data,
    ]
    .into_iter()
    .enumerate()
    {
        let source = create_wp(
            &storage,
            feature_id,
            &format!("src-{index}"),
            10 + index as i32,
        )
        .await;
        storage
            .add_wp_dependency(&WpDependency {
                wp_id: target,
                depends_on: source,
                dep_type,
            })
            .await
            .expect("add_wp_dependency should succeed");
        expected.push(dep_type);
    }

    let deps = storage
        .get_wp_dependencies(target)
        .await
        .expect("get_wp_dependencies should succeed");
    let types: Vec<DependencyType> = deps.iter().map(|d| d.dep_type).collect();
    for dep_type in expected {
        assert!(
            types.contains(&dep_type),
            "missing dependency type {dep_type:?}"
        );
    }
}

#[tokio::test]
async fn wp_dependencies_empty_for_isolated_package() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "iso-feature").await;
    let id = create_wp(&storage, feature_id, "isolated", 1).await;

    let deps = storage
        .get_wp_dependencies(id)
        .await
        .expect("get_wp_dependencies should succeed");
    assert!(deps.is_empty());
}

#[tokio::test]
async fn wp_ready_excludes_package_blocked_by_unfinished_dependency() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "ready-blocked").await;
    let first = create_wp(&storage, feature_id, "first", 1).await;
    let second = create_wp(&storage, feature_id, "second", 2).await;
    storage
        .add_wp_dependency(&WpDependency {
            wp_id: second,
            depends_on: first,
            dep_type: DependencyType::Explicit,
        })
        .await
        .expect("add_wp_dependency should succeed");

    let ready: Vec<i64> = storage
        .get_ready_wps(feature_id)
        .await
        .expect("get_ready_wps should succeed")
        .into_iter()
        .map(|wp| wp.id)
        .collect();

    assert!(ready.contains(&first), "unblocked package should be ready");
    assert!(
        !ready.contains(&second),
        "blocked package must not be ready while dependency is unfinished"
    );
}

#[tokio::test]
async fn wp_ready_includes_package_once_dependency_is_done() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "ready-done").await;
    let first = create_wp(&storage, feature_id, "first", 1).await;
    let second = create_wp(&storage, feature_id, "second", 2).await;
    storage
        .add_wp_dependency(&WpDependency {
            wp_id: second,
            depends_on: first,
            dep_type: DependencyType::Explicit,
        })
        .await
        .expect("add_wp_dependency should succeed");

    storage
        .update_wp_state(first, WpState::Done)
        .await
        .expect("update_wp_state should succeed");

    let ready: Vec<i64> = storage
        .get_ready_wps(feature_id)
        .await
        .expect("get_ready_wps should succeed")
        .into_iter()
        .map(|wp| wp.id)
        .collect();

    assert!(
        ready.contains(&second),
        "blocked package becomes ready once its dependency is done"
    );
}

#[tokio::test]
async fn wp_ready_excludes_non_planned_packages() {
    let storage = storage();
    let feature_id = seed_feature(&storage, "ready-nonplanned").await;
    let id = create_wp(&storage, feature_id, "in-flight", 1).await;
    storage
        .update_wp_state(id, WpState::Doing)
        .await
        .expect("update_wp_state should succeed");

    let ready = storage
        .get_ready_wps(feature_id)
        .await
        .expect("get_ready_wps should succeed");
    assert!(
        ready.iter().all(|wp| wp.id != id),
        "only planned packages are considered ready"
    );
}

#[tokio::test]
async fn wp_ready_is_scoped_to_feature() {
    let storage = storage();
    let feature_a = seed_feature(&storage, "ready-scope-a").await;
    let feature_b = seed_feature(&storage, "ready-scope-b").await;
    create_wp(&storage, feature_a, "a-ready", 1).await;
    create_wp(&storage, feature_b, "b-ready", 1).await;

    let ready = storage
        .get_ready_wps(feature_a)
        .await
        .expect("get_ready_wps should succeed");
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].title, "a-ready");
}
