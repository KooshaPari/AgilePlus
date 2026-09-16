//! Integration tests for feature CRUD operations via the repository layer.

use agileplus_domain::domain::{
    feature::Feature,
    state_machine::FeatureState,
};
use agileplus_sqlite::{
    repository::features::{self, *},
    SqliteStorageAdapter,
};

fn make_adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().unwrap()
}

fn sample_feature(slug: &str) -> Feature {
    let now = chrono::Utc::now();
    Feature {
        id: 0,
        slug: slug.to_string(),
        friendly_name: format!("Feature {slug}"),
        state: FeatureState::Created,
        spec_hash: [0u8; 32],
        target_branch: "main".to_string(),
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

// --- Create ---

#[test]
fn create_feature_returns_positive_id() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    let f = sample_feature("feat-1");
    let id = features::create_feature(&conn, &f).unwrap();
    assert!(id > 0, "create should return positive id, got {id}");
}

#[test]
fn create_feature_persists_all_fields() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    let mut f = sample_feature("auth-module");
    f.friendly_name = "Auth Module".to_string();
    f.state = FeatureState::Specified;
    f.target_branch = "feature/auth".to_string();
    f.spec_hash = [1u8; 32];
    f.labels = vec!["security".into(), "backend".into()];

    let id = features::create_feature(&conn, &f).unwrap();
    let got = features::get_feature_by_id(&conn, id).unwrap().unwrap();

    assert_eq!(got.slug, "auth-module");
    assert_eq!(got.friendly_name, "Auth Module");
    assert_eq!(got.state, FeatureState::Specified);
    assert_eq!(got.target_branch, "feature/auth");
    assert_eq!(got.spec_hash, [1u8; 32]);
    assert_eq!(got.labels, vec!["security", "backend"]);
}

// --- Read ---

#[test]
fn get_feature_by_slug_returns_none_for_missing() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    assert!(features::get_feature_by_slug(&conn, "nonexistent").unwrap().is_none());
}

#[test]
fn get_feature_by_id_returns_none_for_missing() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    assert!(features::get_feature_by_id(&conn, 99999).unwrap().is_none());
}

#[test]
fn get_feature_by_slug_matches_id() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    let f = sample_feature("match-test");
    let id = features::create_feature(&conn, &f).unwrap();
    let by_slug = features::get_feature_by_slug(&conn, "match-test").unwrap().unwrap();
    let by_id = features::get_feature_by_id(&conn, id).unwrap().unwrap();
    assert_eq!(by_slug.id, by_id.id);
}

// --- Update ---

#[test]
fn update_feature_state_transitions() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    let f = sample_feature("state-test");
    let id = features::create_feature(&conn, &f).unwrap();

    let states = [
        FeatureState::Specified,
        FeatureState::Researched,
        FeatureState::Planned,
        FeatureState::Implementing,
        FeatureState::Validated,
        FeatureState::Shipped,
        FeatureState::Retrospected,
    ];

    for state in &states {
        features::update_feature_state(&conn, id, *state).unwrap();
        let got = features::get_feature_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(got.state, *state, "state should be {state:?}");
    }
}

#[test]
fn update_feature_modifies_all_fields() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    let f = sample_feature("old-slug");
    let id = features::create_feature(&conn, &f).unwrap();

    let mut updated = sample_feature("new-slug");
    updated.id = id;
    updated.friendly_name = "New Name".to_string();
    updated.state = FeatureState::Implementing;
    updated.target_branch = "feature/new".to_string();
    updated.labels = vec!["updated".into()];

    features::update_feature(&conn, &updated).unwrap();
    let got = features::get_feature_by_id(&conn, id).unwrap().unwrap();

    assert_eq!(got.slug, "new-slug");
    assert_eq!(got.friendly_name, "New Name");
    assert_eq!(got.state, FeatureState::Implementing);
    assert_eq!(got.target_branch, "feature/new");
    assert_eq!(got.labels, vec!["updated"]);
}

#[test]
fn update_feature_preserves_created_at() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    let f = sample_feature("ts-test");
    let id = features::create_feature(&conn, &f).unwrap();
    let original = features::get_feature_by_id(&conn, id).unwrap().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    let mut updated = sample_feature("ts-test");
    updated.id = id;
    updated.friendly_name = "Updated".to_string();
    features::update_feature(&conn, &updated).unwrap();

    let got = features::get_feature_by_id(&conn, id).unwrap().unwrap();
    assert_eq!(got.created_at, original.created_at, "created_at should not change");
}

// --- List ---

#[test]
fn list_features_by_state_filters_correctly() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();

    let f1 = sample_feature("s1");
    let f2 = sample_feature("s2");
    let f3 = sample_feature("s3");
    let id1 = features::create_feature(&conn, &f1).unwrap();
    let _id2 = features::create_feature(&conn, &f2).unwrap();
    let id3 = features::create_feature(&conn, &f3).unwrap();

    features::update_feature_state(&conn, id1, FeatureState::Implementing).unwrap();
    features::update_feature_state(&conn, id3, FeatureState::Implementing).unwrap();

    let implementing = features::list_features_by_state(&conn, FeatureState::Implementing).unwrap();
    assert_eq!(implementing.len(), 2);

    let created = features::list_features_by_state(&conn, FeatureState::Created).unwrap();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].slug, "s2");
}

#[test]
fn list_all_features_returns_all() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();

    features::create_feature(&conn, &sample_feature("a")).unwrap();
    features::create_feature(&conn, &sample_feature("b")).unwrap();
    features::create_feature(&conn, &sample_feature("c")).unwrap();

    let all = features::list_all_features(&conn).unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn list_features_by_label_filters() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();

    let mut f1 = sample_feature("labeled");
    f1.labels = vec!["bug".into(), "critical".into()];
    features::create_feature(&conn, &f1).unwrap();

    let mut f2 = sample_feature("unlabeled");
    f2.labels = vec![];
    features::create_feature(&conn, &f2).unwrap();

    let bugs = features::list_features_by_label(&conn, "bug").unwrap();
    assert_eq!(bugs.len(), 1);
    assert_eq!(bugs[0].slug, "labeled");

    let critical = features::list_features_by_label(&conn, "critical").unwrap();
    assert_eq!(critical.len(), 1);

    let no_match = features::list_features_by_label(&conn, "feature").unwrap();
    assert!(no_match.is_empty());
}

#[test]
fn list_features_by_state_empty_when_none_match() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    features::create_feature(&conn, &sample_feature("x")).unwrap();

    let shipped = features::list_features_by_state(&conn, FeatureState::Shipped).unwrap();
    assert!(shipped.is_empty());
}

#[test]
fn list_all_features_empty_on_fresh_db() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    let all = features::list_all_features(&conn).unwrap();
    assert!(all.is_empty());
}

#[test]
fn create_feature_duplicate_slug_fails() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    features::create_feature(&conn, &sample_feature("dup")).unwrap();
    let result = features::create_feature(&conn, &sample_feature("dup"));
    assert!(result.is_err(), "duplicate slug should produce an error");
}

#[test]
fn feature_timestamps_are_set_on_create() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    let f = sample_feature("ts-create");
    let id = features::create_feature(&conn, &f).unwrap();
    let got = features::get_feature_by_id(&conn, id).unwrap().unwrap();

    // Timestamps should be parseable and recent.
    let now = chrono::Utc::now();
    let diff = (now - got.created_at).num_seconds().abs();
    assert!(diff < 60, "created_at should be recent, diff={diff}s");
}

#[test]
fn feature_empty_labels_roundtrip() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    let f = sample_feature("no-labels");
    let id = features::create_feature(&conn, &f).unwrap();
    let got = features::get_feature_by_id(&conn, id).unwrap().unwrap();
    assert!(got.labels.is_empty());
}

#[test]
fn feature_multiple_labels_roundtrip() {
    let adapter = make_adapter();
    let conn = adapter.conn_for_bench().unwrap();
    let mut f = sample_feature("multi-labels");
    f.labels = vec!["a".into(), "b".into(), "c".into()];
    let id = features::create_feature(&conn, &f).unwrap();
    let got = features::get_feature_by_id(&conn, id).unwrap().unwrap();
    assert_eq!(got.labels, vec!["a", "b", "c"]);
}
