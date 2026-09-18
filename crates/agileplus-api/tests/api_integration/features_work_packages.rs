use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;

use crate::support::{
    MockStorage, TEST_API_KEY, setup_test_server, setup_test_server_with_storage,
};

const KEY: &str = "X-API-Key";

#[tokio::test]
async fn list_features_with_valid_key() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    let arr = body
        .as_array()
        .expect("features response should be an array");
    assert!(!arr.is_empty());
    assert_eq!(arr[0]["slug"], "test-feature");
}

#[tokio::test]
async fn get_feature_found() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/test-feature")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["slug"], "test-feature");
    assert_eq!(body["name"], "Test Feature");
}

#[tokio::test]
async fn get_feature_not_found() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/nonexistent")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn patch_feature_persists_mutations() {
    let server = setup_test_server().await;
    let resp = server
        .patch("/api/v1/features/test-feature")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({
            "title": "Renamed Feature",
            "target_branch": "release/stable"
        }))
        .await;
    resp.assert_status_ok();

    let reread = server
        .get("/api/v1/features/test-feature")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    reread.assert_status_ok();
    let body: serde_json::Value = reread.json();
    assert_eq!(body["name"], "Renamed Feature");
    assert_eq!(body["target_branch"], "release/stable");
}

#[tokio::test]
async fn get_work_package_found() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/work-packages/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["id"], 1);
    assert_eq!(body["title"], "WP01");
}

#[tokio::test]
async fn get_work_package_not_found() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/work-packages/999")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn patch_work_package_persists_mutations() {
    let server = setup_test_server().await;
    let resp = server
        .patch("/api/v1/work-packages/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({
            "title": "Updated WP",
            "acceptance_criteria": "Updated criteria",
            "pr_url": "https://example.com/pr/42"
        }))
        .await;
    resp.assert_status_ok();

    let reread = server
        .get("/api/v1/work-packages/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    reread.assert_status_ok();
    let body: serde_json::Value = reread.json();
    assert_eq!(body["title"], "Updated WP");
    assert_eq!(body["acceptance_criteria"], "Updated criteria");
    assert_eq!(body["pr_url"], "https://example.com/pr/42");
}

// ── Feature list filtering: the label branch and its composition with state ──

/// The default fixture's only feature carries no labels, so every label-filter
/// assertion so far took the "matches nothing" path. This fixture gives one
/// feature a label and the other none.
fn seeded_labelled_storage() -> MockStorage {
    let storage = MockStorage::default();
    let now = chrono::Utc::now();

    let mut features = storage.features.lock().expect("features lock poisoned");
    for (id, slug, state, labels) in [
        (
            1,
            "labelled-feature",
            FeatureState::Implementing,
            vec!["platform".to_string(), "beta".to_string()],
        ),
        (2, "unlabelled-feature", FeatureState::Created, vec![]),
    ] {
        features.push(Feature {
            id,
            slug: slug.to_string(),
            friendly_name: slug.to_string(),
            state,
            spec_hash: [0u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels,
            module_id: None,
            project_id: None,
            created_at_commit: None,
            last_modified_commit: None,
            created_at: now,
            updated_at: now,
        });
    }
    drop(features);

    storage
}

#[tokio::test]
async fn list_features_label_filter_matches_labelled_feature_only() {
    let server = setup_test_server_with_storage(seeded_labelled_storage()).await;
    let resp = server
        .get("/api/v1/features?label=platform")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: Vec<serde_json::Value> = resp.json();
    assert_eq!(body.len(), 1, "got: {body:?}");
    assert_eq!(body[0]["slug"], "labelled-feature");

    // A second label on the same feature is also a match.
    let beta = server
        .get("/api/v1/features?label=beta")
        .add_header(KEY, TEST_API_KEY)
        .await;
    let beta_body: Vec<serde_json::Value> = beta.json();
    assert_eq!(beta_body.len(), 1, "got: {beta_body:?}");
}

#[tokio::test]
async fn list_features_combines_state_and_label_filters() {
    let server = setup_test_server_with_storage(seeded_labelled_storage()).await;

    // Both filters agree on the labelled feature.
    let both = server
        .get("/api/v1/features?state=implementing&label=platform")
        .add_header(KEY, TEST_API_KEY)
        .await;
    both.assert_status_ok();
    let both_body: Vec<serde_json::Value> = both.json();
    assert_eq!(both_body.len(), 1, "got: {both_body:?}");
    assert_eq!(both_body[0]["slug"], "labelled-feature");

    // The state filter selects a feature the label filter then rejects: the
    // label must still be applied after the state query.
    let conflict = server
        .get("/api/v1/features?state=created&label=platform")
        .add_header(KEY, TEST_API_KEY)
        .await;
    conflict.assert_status_ok();
    let conflict_body: Vec<serde_json::Value> = conflict.json();
    assert!(
        conflict_body.is_empty(),
        "label filtering must compose with state filtering, got: {conflict_body:?}"
    );
}

// ── Work packages: create defaults and partial updates ───────────────────────

#[tokio::test]
async fn create_work_package_defaults_sequence_and_acceptance_criteria() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/features/test-feature/work-packages")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "WP-BARE" }))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["title"], "WP-BARE");
    assert_eq!(body["feature_id"], 1);
    assert_eq!(body["state"], "planned");
    assert_eq!(body["sequence"], 1, "sequence defaults to 1");
    assert_eq!(
        body["acceptance_criteria"], "",
        "an omitted acceptance criteria defaults to the empty string"
    );
    assert!(body["pr_url"].is_null());
}

#[tokio::test]
async fn create_work_package_keeps_explicit_acceptance_criteria() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/features/test-feature/work-packages")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({
            "title": "WP-AC",
            "acceptance_criteria": "coverage >= 85%",
            "sequence": 7
        }))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["acceptance_criteria"], "coverage >= 85%");
    assert_eq!(body["sequence"], 7);
}

/// A PATCH with no fields at all is a no-op, not a wipe: every field must fall
/// back to the stored value.
#[tokio::test]
async fn patch_work_package_with_empty_body_retains_stored_fields() {
    let server = setup_test_server().await;
    let before = server
        .get("/api/v1/work-packages/1")
        .add_header(KEY, TEST_API_KEY)
        .await;
    before.assert_status_ok();
    let before: serde_json::Value = before.json();

    let resp = server
        .patch("/api/v1/work-packages/1")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({}))
        .await;
    resp.assert_status_ok();
    let after: serde_json::Value = resp.json();

    assert_eq!(after["title"], before["title"]);
    assert_eq!(after["acceptance_criteria"], before["acceptance_criteria"]);
    assert_eq!(after["pr_url"], before["pr_url"]);
    assert_eq!(after["state"], before["state"]);
    assert_eq!(after["sequence"], before["sequence"]);
}

#[tokio::test]
async fn patch_work_package_omitting_pr_url_retains_it() {
    let server = setup_test_server().await;
    let resp = server
        .patch("/api/v1/work-packages/1")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "Renamed Only" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["title"], "Renamed Only");
    assert_eq!(
        body["pr_url"], "https://github.com/org/repo/pull/1",
        "an omitted pr_url must not clear the stored value"
    );
}
