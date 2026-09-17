//! Integration tests: module hierarchy/tagging and cycle lifecycle repositories.

use agileplus_domain::domain::{
    cycle::{Cycle, CycleFeature, CycleState},
    feature::Feature,
    module::{Module, ModuleFeatureTag},
    state_machine::FeatureState,
};
use agileplus_sqlite::{
    repository::{cycles, features, modules},
    SqliteStorageAdapter,
};
use rusqlite::Connection;

fn adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().unwrap()
}

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn module(name: &str, parent: Option<i64>) -> Module {
    Module::new(name, parent)
}

fn sample_feature(slug: &str) -> Feature {
    let now = chrono::Utc::now();
    Feature {
        id: 0,
        slug: slug.into(),
        friendly_name: slug.into(),
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

fn seed_feature(conn: &Connection, slug: &str) -> i64 {
    features::create_feature(conn, &sample_feature(slug)).unwrap()
}

fn cycle(name: &str, scope: Option<i64>) -> Cycle {
    Cycle::new(name, date(2026, 1, 1), date(2026, 2, 1), scope).unwrap()
}

// ---------------------------------------------------------------------------
// Modules
// ---------------------------------------------------------------------------

#[test]
fn module_create_and_get_by_id() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = modules::create_module(&conn, &module("Auth", None)).unwrap();
    let got = modules::get_module(&conn, id).unwrap().unwrap();
    assert_eq!(got.friendly_name, "Auth");
    assert_eq!(got.slug, "auth");
    assert!(got.parent_module_id.is_none());
}

#[test]
fn module_get_by_id_missing() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(modules::get_module(&conn, 10).unwrap().is_none());
}

#[test]
fn module_get_by_slug_roundtrips() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    modules::create_module(&conn, &module("OAuth Providers", None)).unwrap();
    let got = modules::get_module_by_slug(&conn, "oauth-providers")
        .unwrap()
        .unwrap();
    assert_eq!(got.friendly_name, "OAuth Providers");
    assert!(modules::get_module_by_slug(&conn, "nope").unwrap().is_none());
}

#[test]
fn module_create_with_missing_parent_is_error() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let err = modules::create_module(&conn, &module("Child", Some(999))).unwrap_err();
    assert!(matches!(
        err,
        agileplus_domain::error::DomainError::ModuleNotFound(_)
    ));
}

#[test]
fn module_child_parent_relationship() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let root = modules::create_module(&conn, &module("Root", None)).unwrap();
    let child = modules::create_module(&conn, &module("Child", Some(root))).unwrap();

    let roots = modules::list_root_modules(&conn).unwrap();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].id, root);

    let children = modules::list_child_modules(&conn, root).unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id, child);
    assert!(modules::list_child_modules(&conn, child).unwrap().is_empty());
}

#[test]
fn module_duplicate_slug_under_same_parent_rejected() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let parent = modules::create_module(&conn, &module("Parent", None)).unwrap();
    modules::create_module(&conn, &module("Twin", Some(parent))).unwrap();
    assert!(modules::create_module(&conn, &module("Twin", Some(parent))).is_err());
}

#[test]
fn module_duplicate_root_slug_allowed_by_null_parent_unique_semantics() {
    // SQLite treats NULLs as distinct in UNIQUE(parent_module_id, slug), so two
    // roots with the same slug are permitted at the storage layer.
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    modules::create_module(&conn, &module("Root", None)).unwrap();
    modules::create_module(&conn, &module("Root", None)).unwrap();
    assert_eq!(modules::list_root_modules(&conn).unwrap().len(), 2);
}

#[test]
fn module_same_slug_under_different_parents_allowed() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let p1 = modules::create_module(&conn, &module("P1", None)).unwrap();
    let p2 = modules::create_module(&conn, &module("P2", None)).unwrap();
    modules::create_module(&conn, &module("Shared", Some(p1))).unwrap();
    modules::create_module(&conn, &module("Shared", Some(p2))).unwrap();
    assert_eq!(modules::list_child_modules(&conn, p1).unwrap().len(), 1);
    assert_eq!(modules::list_child_modules(&conn, p2).unwrap().len(), 1);
}

#[test]
fn module_update_changes_name_slug_description() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = modules::create_module(&conn, &module("Old Name", None)).unwrap();
    modules::update_module(&conn, id, "New Name", Some("a description")).unwrap();

    let got = modules::get_module(&conn, id).unwrap().unwrap();
    assert_eq!(got.friendly_name, "New Name");
    assert_eq!(got.slug, "new-name");
    assert_eq!(got.description.as_deref(), Some("a description"));
}

#[test]
fn module_update_unknown_id_is_module_not_found() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let err = modules::update_module(&conn, 55, "X", None).unwrap_err();
    assert!(matches!(
        err,
        agileplus_domain::error::DomainError::ModuleNotFound(_)
    ));
}

#[test]
fn module_delete_leaf_succeeds() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = modules::create_module(&conn, &module("Solo", None)).unwrap();
    modules::delete_module(&conn, id).unwrap();
    assert!(modules::get_module(&conn, id).unwrap().is_none());
}

#[test]
fn module_delete_missing_is_module_not_found() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let err = modules::delete_module(&conn, 77).unwrap_err();
    assert!(matches!(
        err,
        agileplus_domain::error::DomainError::ModuleNotFound(_)
    ));
}

#[test]
fn module_delete_with_children_is_blocked() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let root = modules::create_module(&conn, &module("Root", None)).unwrap();
    modules::create_module(&conn, &module("Child", Some(root))).unwrap();
    let err = modules::delete_module(&conn, root).unwrap_err();
    assert!(matches!(
        err,
        agileplus_domain::error::DomainError::ModuleHasDependents(_)
    ));
}

#[test]
fn module_delete_with_owned_feature_is_blocked() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = modules::create_module(&conn, &module("Owner", None)).unwrap();
    let feature_id = seed_feature(&conn, "owned-feature");
    conn.execute(
        "UPDATE features SET module_id = ?1 WHERE id = ?2",
        rusqlite::params![id, feature_id],
    )
    .unwrap();
    let err = modules::delete_module(&conn, id).unwrap_err();
    assert!(matches!(
        err,
        agileplus_domain::error::DomainError::ModuleHasDependents(_)
    ));
}

#[test]
fn module_circular_ref_detection_true_for_ancestor() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let root = modules::create_module(&conn, &module("Root", None)).unwrap();
    let child = modules::create_module(&conn, &module("Child", Some(root))).unwrap();
    // Making root's parent the child would create a cycle.
    assert!(modules::would_create_circular_ref(&conn, root, child).unwrap());
    // Making child's parent root is fine.
    assert!(!modules::would_create_circular_ref(&conn, child, root).unwrap());
}

#[test]
fn module_circular_ref_self_is_true() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = modules::create_module(&conn, &module("Self", None)).unwrap();
    assert!(modules::would_create_circular_ref(&conn, id, id).unwrap());
}

#[test]
fn module_with_features_reports_owned() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = modules::create_module(&conn, &module("Owning", None)).unwrap();
    let feature_id = seed_feature(&conn, "f-owned");
    conn.execute(
        "UPDATE features SET module_id = ?1 WHERE id = ?2",
        rusqlite::params![id, feature_id],
    )
    .unwrap();

    let mwf = modules::get_module_with_features(&conn, id).unwrap().unwrap();
    assert_eq!(mwf.module.id, id);
    assert_eq!(mwf.owned_features.len(), 1);
    assert_eq!(mwf.owned_features[0].slug, "f-owned");
    assert!(mwf.tagged_features.is_empty());
}

#[test]
fn module_with_features_reports_tagged() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = modules::create_module(&conn, &module("Tagging", None)).unwrap();
    let feature_id = seed_feature(&conn, "f-tagged");
    modules::tag_feature_to_module(&conn, &ModuleFeatureTag::new(id, feature_id)).unwrap();

    let mwf = modules::get_module_with_features(&conn, id).unwrap().unwrap();
    assert!(mwf.owned_features.is_empty());
    assert_eq!(mwf.tagged_features.len(), 1);
    assert_eq!(mwf.tagged_features[0].slug, "f-tagged");
}

#[test]
fn module_with_features_reports_children() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let root = modules::create_module(&conn, &module("Root", None)).unwrap();
    modules::create_module(&conn, &module("Child", Some(root))).unwrap();
    let mwf = modules::get_module_with_features(&conn, root).unwrap().unwrap();
    assert_eq!(mwf.child_modules.len(), 1);
}

#[test]
fn module_with_features_missing_is_none() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(modules::get_module_with_features(&conn, 5).unwrap().is_none());
}

#[test]
fn module_tag_is_idempotent_and_untag_removes() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mid = modules::create_module(&conn, &module("M", None)).unwrap();
    let fid = seed_feature(&conn, "f");
    let tag = ModuleFeatureTag::new(mid, fid);
    modules::tag_feature_to_module(&conn, &tag).unwrap();
    modules::tag_feature_to_module(&conn, &tag).unwrap(); // idempotent

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM module_feature_tags WHERE module_id = ?1 AND feature_id = ?2",
            rusqlite::params![mid, fid],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    modules::untag_feature_from_module(&conn, mid, fid).unwrap();
    let after: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM module_feature_tags WHERE module_id = ?1 AND feature_id = ?2",
            rusqlite::params![mid, fid],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(after, 0);
}

#[test]
fn module_tag_missing_feature_errors() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mid = modules::create_module(&conn, &module("M", None)).unwrap();
    assert!(modules::tag_feature_to_module(&conn, &ModuleFeatureTag::new(mid, 9999)).is_err());
}

// ---------------------------------------------------------------------------
// Cycles
// ---------------------------------------------------------------------------

#[test]
fn cycle_create_and_get() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = cycles::create_cycle(&conn, &cycle("Sprint 1", None)).unwrap();
    let got = cycles::get_cycle(&conn, id).unwrap().unwrap();
    assert_eq!(got.name, "Sprint 1");
    assert_eq!(got.state, CycleState::Draft);
    assert_eq!(got.start_date, date(2026, 1, 1));
    assert_eq!(got.end_date, date(2026, 2, 1));
    assert!(got.module_scope_id.is_none());
}

#[test]
fn cycle_get_missing_is_none() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(cycles::get_cycle(&conn, 3).unwrap().is_none());
}

#[test]
fn cycle_name_is_unique() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    cycles::create_cycle(&conn, &cycle("Dup", None)).unwrap();
    assert!(cycles::create_cycle(&conn, &cycle("Dup", None)).is_err());
}

#[test]
fn cycle_update_state_persists() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = cycles::create_cycle(&conn, &cycle("S", None)).unwrap();
    cycles::update_cycle_state(&conn, id, CycleState::Active).unwrap();
    assert_eq!(cycles::get_cycle(&conn, id).unwrap().unwrap().state, CycleState::Active);
}

#[test]
fn cycle_update_state_unknown_id_is_cycle_not_found() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let err = cycles::update_cycle_state(&conn, 808, CycleState::Active).unwrap_err();
    assert!(matches!(
        err,
        agileplus_domain::error::DomainError::CycleNotFound(_)
    ));
}

#[test]
fn cycle_list_by_state_and_all() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let c1 = cycles::create_cycle(&conn, &cycle("A", None)).unwrap();
    let c2 = cycles::create_cycle(&conn, &cycle("B", None)).unwrap();
    cycles::update_cycle_state(&conn, c2, CycleState::Active).unwrap();

    assert_eq!(cycles::list_all_cycles(&conn).unwrap().len(), 2);
    let drafts = cycles::list_cycles_by_state(&conn, CycleState::Draft).unwrap();
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].id, c1);
    let active = cycles::list_cycles_by_state(&conn, CycleState::Active).unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, c2);
    assert!(cycles::list_cycles_by_state(&conn, CycleState::Shipped).unwrap().is_empty());
}

#[test]
fn cycle_list_by_module_scope() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mid = modules::create_module(&conn, &module("Scope", None)).unwrap();
    cycles::create_cycle(&conn, &cycle("Scoped", Some(mid))).unwrap();
    cycles::create_cycle(&conn, &cycle("Unscoped", None)).unwrap();

    let scoped = cycles::list_cycles_by_module(&conn, mid).unwrap();
    assert_eq!(scoped.len(), 1);
    assert_eq!(scoped[0].name, "Scoped");
    assert!(cycles::list_cycles_by_module(&conn, 1234).unwrap().is_empty());
}

#[test]
fn cycle_add_and_remove_feature() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let cid = cycles::create_cycle(&conn, &cycle("C", None)).unwrap();
    let fid = seed_feature(&conn, "in-cycle");

    cycles::add_feature_to_cycle(&conn, &CycleFeature::new(cid, fid)).unwrap();
    let cwf = cycles::get_cycle_with_features(&conn, cid).unwrap().unwrap();
    assert_eq!(cwf.features.len(), 1);
    assert_eq!(cwf.features[0].slug, "in-cycle");

    cycles::remove_feature_from_cycle(&conn, cid, fid).unwrap();
    let cwf = cycles::get_cycle_with_features(&conn, cid).unwrap().unwrap();
    assert!(cwf.features.is_empty());
}

#[test]
fn cycle_add_feature_is_idempotent() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let cid = cycles::create_cycle(&conn, &cycle("C", None)).unwrap();
    let fid = seed_feature(&conn, "dup");
    cycles::add_feature_to_cycle(&conn, &CycleFeature::new(cid, fid)).unwrap();
    cycles::add_feature_to_cycle(&conn, &CycleFeature::new(cid, fid)).unwrap();
    assert_eq!(
        cycles::get_cycle_with_features(&conn, cid).unwrap().unwrap().features.len(),
        1
    );
}

#[test]
fn cycle_add_feature_unknown_cycle_is_error() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let fid = seed_feature(&conn, "f");
    let err = cycles::add_feature_to_cycle(&conn, &CycleFeature::new(999, fid)).unwrap_err();
    assert!(matches!(
        err,
        agileplus_domain::error::DomainError::CycleNotFound(_)
    ));
}

#[test]
fn cycle_scope_rejects_out_of_scope_feature() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mid = modules::create_module(&conn, &module("Scoped", None)).unwrap();
    let cid = cycles::create_cycle(&conn, &cycle("Scoped Cycle", Some(mid))).unwrap();
    let fid = seed_feature(&conn, "outside");

    let err = cycles::add_feature_to_cycle(&conn, &CycleFeature::new(cid, fid)).unwrap_err();
    assert!(matches!(
        err,
        agileplus_domain::error::DomainError::FeatureNotInModuleScope { .. }
    ));
}

#[test]
fn cycle_scope_accepts_tagged_feature() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mid = modules::create_module(&conn, &module("Scoped", None)).unwrap();
    let cid = cycles::create_cycle(&conn, &cycle("Scoped Cycle", Some(mid))).unwrap();
    let fid = seed_feature(&conn, "tagged-in");
    modules::tag_feature_to_module(&conn, &ModuleFeatureTag::new(mid, fid)).unwrap();

    cycles::add_feature_to_cycle(&conn, &CycleFeature::new(cid, fid)).unwrap();
    assert_eq!(
        cycles::get_cycle_with_features(&conn, cid).unwrap().unwrap().features.len(),
        1
    );
}

#[test]
fn cycle_scope_accepts_owned_feature() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mid = modules::create_module(&conn, &module("Scoped", None)).unwrap();
    let cid = cycles::create_cycle(&conn, &cycle("Scoped Cycle", Some(mid))).unwrap();
    let fid = seed_feature(&conn, "owned-in");
    conn.execute(
        "UPDATE features SET module_id = ?1 WHERE id = ?2",
        rusqlite::params![mid, fid],
    )
    .unwrap();

    cycles::add_feature_to_cycle(&conn, &CycleFeature::new(cid, fid)).unwrap();
    assert_eq!(
        cycles::get_cycle_with_features(&conn, cid).unwrap().unwrap().features.len(),
        1
    );
}

#[test]
fn cycle_with_features_missing_is_none() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(cycles::get_cycle_with_features(&conn, 42).unwrap().is_none());
}

#[test]
fn cycle_wp_progress_counts_states() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let cid = cycles::create_cycle(&conn, &cycle("Progress", None)).unwrap();
    let fid = seed_feature(&conn, "with-wps");
    cycles::add_feature_to_cycle(&conn, &CycleFeature::new(cid, fid)).unwrap();

    // Two work packages: one planned, one done.
    conn.execute(
        "INSERT INTO work_packages (feature_id, title, state, sequence, file_scope, acceptance_criteria, created_at, updated_at)
         VALUES (?1, 'WP1', 'planned', 1, '[]', '', ?2, ?2)",
        rusqlite::params![fid, chrono::Utc::now().to_rfc3339()],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO work_packages (feature_id, title, state, sequence, file_scope, acceptance_criteria, created_at, updated_at)
         VALUES (?1, 'WP2', 'done', 2, '[]', '', ?2, ?2)",
        rusqlite::params![fid, chrono::Utc::now().to_rfc3339()],
    )
    .unwrap();

    let cwf = cycles::get_cycle_with_features(&conn, cid).unwrap().unwrap();
    assert_eq!(cwf.wp_progress.total, 2);
    assert_eq!(cwf.wp_progress.planned, 1);
    assert_eq!(cwf.wp_progress.done, 1);
}

#[test]
fn cycle_remove_nonexistent_feature_is_ok() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let cid = cycles::create_cycle(&conn, &cycle("C", None)).unwrap();
    cycles::remove_feature_from_cycle(&conn, cid, 1234).unwrap();
}

#[test]
fn cycle_create_rejects_inverted_dates_at_domain_level() {
    // Domain guard: end_date must be after start_date.
    let err = Cycle::new("Bad", date(2026, 2, 1), date(2026, 1, 1), None).unwrap_err();
    assert!(matches!(err, agileplus_domain::error::DomainError::Other(_)));
}
