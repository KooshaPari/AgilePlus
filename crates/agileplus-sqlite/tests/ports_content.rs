//! Integration tests for the async `ContentStoragePort` implementation.
//!
//! Kept in a separate test binary from `ports_async.rs` because
//! `ContentStoragePort` and `StoragePort` share method names, which would make
//! `adapter.method()` calls ambiguous if both traits were in scope at once.

use agileplus_domain::{
    domain::{
        backlog::{BacklogFilters, BacklogItem, BacklogPriority, BacklogStatus, Intent},
        feature::Feature,
        state_machine::FeatureState,
        work_package::{DependencyType, WpDependency, WpState, WorkPackage},
    },
    ports::ContentStoragePort,
};
use agileplus_sqlite::SqliteStorageAdapter;

fn adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().unwrap()
}

fn feature(slug: &str) -> Feature {
    let now = chrono::Utc::now();
    Feature {
        id: 0,
        slug: slug.into(),
        friendly_name: format!("Feature {slug}"),
        state: FeatureState::Created,
        spec_hash: [0u8; 32],
        target_branch: "main".into(),
        plane_issue_id: None,
        plane_state_id: None,
        labels: vec![],
        module_id: None,
        project_id: None,
        created_at: now,
        updated_at: now,
        created_at_commit: None,
        last_modified_commit: None,
    }
}

fn wp(feature_id: i64, title: &str, seq: i32) -> WorkPackage {
    let now = chrono::Utc::now();
    WorkPackage {
        id: 0,
        feature_id,
        title: title.into(),
        state: WpState::Planned,
        sequence: seq,
        file_scope: vec!["src/lib.rs".into()],
        acceptance_criteria: "tests pass".into(),
        agent_id: None,
        pr_url: None,
        pr_state: None,
        worktree_path: None,
        plane_sub_issue_id: None,
        base_commit: None,
        head_commit: None,
        created_at: now,
        updated_at: now,
    }
}

// ---------------------------------------------------------------------------
// ContentStoragePort
// ---------------------------------------------------------------------------

#[tokio::test]
async fn content_port_feature_crud() {
    let a = adapter();
    let id = a.create_feature(&feature("content-feat")).await.unwrap();
    assert_eq!(
        a.get_feature_by_slug("content-feat").await.unwrap().unwrap().id,
        id
    );
    assert!(a.get_feature_by_id(id).await.unwrap().is_some());
    a.update_feature_state(id, FeatureState::Validated).await.unwrap();
    let mut f = feature("content-feat");
    f.id = id;
    f.friendly_name = "Content Renamed".into();
    f.state = FeatureState::Validated;
    a.update_feature(&f).await.unwrap();
    let got = a.get_feature_by_id(id).await.unwrap().unwrap();
    assert_eq!(got.friendly_name, "Content Renamed");
    assert_eq!(got.state, FeatureState::Validated);
    assert_eq!(a.list_all_features().await.unwrap().len(), 1);
    assert_eq!(
        a.list_features_by_state(FeatureState::Validated).await.unwrap().len(),
        1
    );
}

#[tokio::test]
async fn content_port_feature_missing_returns_none() {
    let a = adapter();
    assert!(a.get_feature_by_id(1).await.unwrap().is_none());
    assert!(a.get_feature_by_slug("ghost").await.unwrap().is_none());
}

#[tokio::test]
async fn content_port_backlog_lifecycle() {
    let a = adapter();
    let item = BacklogItem::from_triage("t".into(), "d".into(), Intent::Bug, "cli".into());
    let id = a.create_backlog_item(&item).await.unwrap();
    assert!(a.get_backlog_item(id).await.unwrap().is_some());

    let filters = BacklogFilters::default();
    assert_eq!(a.list_backlog_items(&filters).await.unwrap().len(), 1);
    assert_eq!(a.list_backlog_items(&filters).await.unwrap()[0].intent, Intent::Bug);

    a.update_backlog_priority(id, BacklogPriority::Critical).await.unwrap();
    a.update_backlog_status(id, BacklogStatus::InProgress).await.unwrap();
    let got = a.get_backlog_item(id).await.unwrap().unwrap();
    assert_eq!(got.priority, BacklogPriority::Critical);
    assert_eq!(got.status, BacklogStatus::InProgress);

    // No 'new' items remain.
    assert!(a.pop_next_backlog_item().await.unwrap().is_none());
}

#[tokio::test]
async fn content_port_backlog_missing_is_none() {
    let a = adapter();
    assert!(a.get_backlog_item(777).await.unwrap().is_none());
}

#[tokio::test]
async fn content_port_pop_next_backlog() {
    let a = adapter();
    let item = BacklogItem::from_triage("pop".into(), "d".into(), Intent::Task, "cli".into());
    let id = a.create_backlog_item(&item).await.unwrap();
    let popped = a.pop_next_backlog_item().await.unwrap().unwrap();
    assert_eq!(popped.id, Some(id));
    assert_eq!(popped.status, BacklogStatus::Triaged);
}

#[tokio::test]
async fn content_port_work_package_ops() {
    let a = adapter();
    let fid = a.create_feature(&feature("cwf")).await.unwrap();
    let wpid = a.create_work_package(&wp(fid, "cwp", 1)).await.unwrap();
    assert!(a.get_work_package(wpid).await.unwrap().is_some());

    let mut changed = wp(fid, "cwp-updated", 1);
    changed.id = wpid;
    changed.state = WpState::Review;
    a.update_work_package(&changed).await.unwrap();
    assert_eq!(a.get_work_package(wpid).await.unwrap().unwrap().title, "cwp-updated");

    a.update_wp_state(wpid, WpState::Done).await.unwrap();
    assert_eq!(a.get_work_package(wpid).await.unwrap().unwrap().state, WpState::Done);

    assert_eq!(a.list_wps_by_feature(fid).await.unwrap().len(), 1);
}

#[tokio::test]
async fn content_port_work_package_missing_is_none() {
    let a = adapter();
    assert!(a.get_work_package(4242).await.unwrap().is_none());
    assert!(a.list_wps_by_feature(4242).await.unwrap().is_empty());
}

#[tokio::test]
async fn content_port_dependencies_and_ready() {
    let a = adapter();
    let fid = a.create_feature(&feature("cpdep")).await.unwrap();
    let w1 = a.create_work_package(&wp(fid, "a", 1)).await.unwrap();
    let w2 = a.create_work_package(&wp(fid, "b", 2)).await.unwrap();
    a.add_wp_dependency(&WpDependency {
        wp_id: w2,
        depends_on: w1,
        dep_type: DependencyType::FileOverlap,
    })
    .await
    .unwrap();
    let deps = a.get_wp_dependencies(w2).await.unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].dep_type, DependencyType::FileOverlap);

    let ready = a.get_ready_wps(fid).await.unwrap();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, w1);
}

#[tokio::test]
async fn content_port_no_dependencies_means_all_ready() {
    let a = adapter();
    let fid = a.create_feature(&feature("cpnodep")).await.unwrap();
    a.create_work_package(&wp(fid, "a", 1)).await.unwrap();
    a.create_work_package(&wp(fid, "b", 2)).await.unwrap();
    assert_eq!(a.get_ready_wps(fid).await.unwrap().len(), 2);
    assert!(a.get_wp_dependencies(1).await.unwrap().is_empty());
}
