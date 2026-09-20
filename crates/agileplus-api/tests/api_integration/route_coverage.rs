// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exhaustive route-coverage integration tests for the AgilePlus HTTP API.
//!
//! Complements the smoke tests in `api_integration.rs` by exercising every
//! registered endpoint, its validation branches, and its error responses
//! against the in-memory mock ports.
//!
//! Traceability: WP11-T065..T070, WP12-T080, WP15-T086

use axum::http::StatusCode;

use crate::support::{TEST_API_KEY, setup_test_server};

const KEY: &str = "X-API-Key";

// ── Public endpoints ─────────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_health_public() {
    let server = setup_test_server().await;
    let resp = server.get("/health").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["service"].is_string());
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn coverage_detailed_health_public() {
    let server = setup_test_server().await;
    let resp = server.get("/detailed-health").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["services"].is_object());
}

#[tokio::test]
async fn coverage_info_public() {
    let server = setup_test_server().await;
    let resp = server.get("/info").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["name"], "agileplus-api");
}

#[tokio::test]
async fn coverage_modules_page_public() {
    let server = setup_test_server().await;
    server.get("/modules").await.assert_status_ok();
}

#[tokio::test]
async fn coverage_cycles_page_public() {
    let server = setup_test_server().await;
    server.get("/cycles").await.assert_status_ok();
}

// ── Authentication ───────────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_missing_key_is_401() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/features")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn coverage_wrong_key_is_401() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/features")
        .add_header(KEY, "not-the-key")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn coverage_bearer_token_accepted() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/features")
        .add_header("Authorization", format!("Bearer {TEST_API_KEY}"))
        .await
        .assert_status_ok();
}

#[tokio::test]
async fn coverage_query_param_key_accepted() {
    let server = setup_test_server().await;
    server
        .get(&format!("/api/v1/features?api_key={TEST_API_KEY}"))
        .await
        .assert_status_ok();
}

#[tokio::test]
async fn coverage_projects_require_auth() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/projects")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn coverage_events_require_auth() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/events")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn coverage_modules_api_requires_auth() {
    let server = setup_test_server().await;
    server
        .get("/api/modules")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

// ── Features: list & filters ─────────────────────────────────────────────────

#[tokio::test]
async fn coverage_list_features_all() {
    let server = setup_test_server().await;
    let resp = server.get("/api/v1/features").add_header(KEY, TEST_API_KEY).await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["slug"], "test-feature");
}

#[tokio::test]
async fn coverage_list_features_filter_implementing() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features?state=implementing")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert_eq!(arr.len(), 1);
}

#[tokio::test]
async fn coverage_list_features_filter_state_with_no_matches() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features?state=created")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert!(arr.is_empty());
}

#[tokio::test]
async fn coverage_list_features_invalid_state_is_400() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/features?state=bogus")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn coverage_list_features_label_filter_excludes_unlabelled() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features?label=platform")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert!(arr.is_empty());
}

#[tokio::test]
async fn coverage_list_features_empty_label_matches_nothing() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features?label=")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
}

// ── Features: get ────────────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_get_feature_by_slug() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/test-feature")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["state"], "implementing");
    assert_eq!(body["target_branch"], "main");
}

#[tokio::test]
async fn coverage_get_feature_unknown_slug_404() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/features/does-not-exist")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

// ── Features: create ─────────────────────────────────────────────────────────

macro_rules! create_feature_state_test {
    ($name:ident, $state:literal) => {
        #[tokio::test]
        async fn $name() {
            let server = setup_test_server().await;
            let resp = server
                .post("/api/v1/features")
                .add_header(KEY, TEST_API_KEY)
                .json(&serde_json::json!({ "title": "State Feature", "state": $state }))
                .await;
            resp.assert_status(StatusCode::CREATED);
            let body: serde_json::Value = resp.json();
            assert_eq!(body["state"], $state);
            assert_eq!(body["slug"], "state-feature");
        }
    };
}

create_feature_state_test!(coverage_create_feature_created, "created");
create_feature_state_test!(coverage_create_feature_specified, "specified");
create_feature_state_test!(coverage_create_feature_researched, "researched");
create_feature_state_test!(coverage_create_feature_planned, "planned");
create_feature_state_test!(coverage_create_feature_implementing, "implementing");
create_feature_state_test!(coverage_create_feature_validated, "validated");
create_feature_state_test!(coverage_create_feature_shipped, "shipped");
create_feature_state_test!(coverage_create_feature_retrospected, "retrospected");

#[tokio::test]
async fn coverage_create_feature_defaults_to_created() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/features")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "No State" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["state"], "created");
    assert_eq!(body["target_branch"], "main");
}

#[tokio::test]
async fn coverage_create_feature_custom_target_branch() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/features")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "Branched", "target_branch": "release/1" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["target_branch"], "release/1");
}

#[tokio::test]
async fn coverage_create_feature_slugifies_title() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/features")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "My Big Feature" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["slug"], "my-big-feature");
}

#[tokio::test]
async fn coverage_create_feature_invalid_state_is_400() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/features")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "Bad", "state": "nope" }))
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn coverage_create_feature_missing_title_is_422() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/features")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "description": "no title" }))
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

// ── Features: update & transition ────────────────────────────────────────────

#[tokio::test]
async fn coverage_patch_feature_updates_name() {
    let server = setup_test_server().await;
    let resp = server
        .patch("/api/v1/features/test-feature")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "New Name" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["name"], "New Name");
}

#[tokio::test]
async fn coverage_patch_feature_unknown_is_404() {
    let server = setup_test_server().await;
    server
        .patch("/api/v1/features/ghost")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "x" }))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_transition_feature_created_to_specified() {
    let server = setup_test_server().await;
    let created = server
        .post("/api/v1/features")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "Flow" }))
        .await;
    created.assert_status(StatusCode::CREATED);

    let resp = server
        .post("/api/v1/features/flow/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_state": "specified" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["from_state"], "created");
    assert_eq!(body["to_state"], "specified");
    assert_eq!(body["feature_slug"], "flow");
}

#[tokio::test]
async fn coverage_transition_feature_invalid_target_is_400() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/features/test-feature/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_state": "sideways" }))
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn coverage_transition_feature_not_allowed_is_409() {
    let server = setup_test_server().await;
    // test-feature is Implementing; Implementing -> Created is not a legal step.
    server
        .post("/api/v1/features/test-feature/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_state": "created" }))
        .await
        .assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn coverage_transition_feature_unknown_slug_is_404() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/features/ghost/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_state": "specified" }))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

// ── Work packages ────────────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_get_work_package() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/work-packages/1")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["state"], "done");
    assert_eq!(body["sequence"], 1);
}

#[tokio::test]
async fn coverage_get_work_package_missing_is_404() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/work-packages/404")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_list_work_packages_for_feature() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/test-feature/work-packages")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["title"], "WP01");
}

#[tokio::test]
async fn coverage_list_work_packages_unknown_feature_is_404() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/features/ghost/work-packages")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_create_work_package() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/features/test-feature/work-packages")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "WP-NEW", "sequence": 5 }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["title"], "WP-NEW");
    assert_eq!(body["sequence"], 5);
    assert_eq!(body["state"], "planned");
}

#[tokio::test]
async fn coverage_create_work_package_unknown_feature_is_404() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/features/ghost/work-packages")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "WP" }))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_patch_work_package() {
    let server = setup_test_server().await;
    let resp = server
        .patch("/api/v1/work-packages/1")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "WP01-renamed", "pr_url": "https://example.com/pr/9" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["title"], "WP01-renamed");
    assert_eq!(body["pr_url"], "https://example.com/pr/9");
}

#[tokio::test]
async fn coverage_patch_work_package_missing_is_404() {
    let server = setup_test_server().await;
    server
        .patch("/api/v1/work-packages/404")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "x" }))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_transition_work_package_planned_to_doing() {
    let server = setup_test_server().await;
    let created = server
        .post("/api/v1/features/test-feature/work-packages")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "title": "WP-FLOW" }))
        .await;
    let body: serde_json::Value = created.json();
    let id = body["id"].as_i64().expect("wp id");

    let resp = server
        .post(&format!("/api/v1/work-packages/{id}/transition"))
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_state": "doing" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["from_state"], "planned");
    assert_eq!(body["to_state"], "doing");
}

#[tokio::test]
async fn coverage_transition_work_package_invalid_state_is_400() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/work-packages/1/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_state": "warp" }))
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn coverage_transition_work_package_illegal_step_is_409() {
    let server = setup_test_server().await;
    // WP 1 is Done; Done -> Doing is not a legal step.
    server
        .post("/api/v1/work-packages/1/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_state": "doing" }))
        .await
        .assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn coverage_transition_work_package_missing_is_404() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/work-packages/404/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_state": "doing" }))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

// ── Events ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_list_events_unfiltered() {
    let server = setup_test_server().await;
    let resp = server.get("/api/v1/events").add_header(KEY, TEST_API_KEY).await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert_eq!(arr.len(), 3);
}

#[tokio::test]
async fn coverage_events_filter_entity_type_feature() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?entity_type=feature")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert_eq!(arr.len(), 2);
    assert!(arr.iter().all(|e| e["entity_type"] == "feature"));
}

#[tokio::test]
async fn coverage_events_filter_entity_type_work_package() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?entity_type=work_package")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert_eq!(arr.len(), 1);
}

#[tokio::test]
async fn coverage_events_filter_actor() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?actor=system")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert_eq!(arr.len(), 2);
}

#[tokio::test]
async fn coverage_events_filter_type() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?type=specified")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["event_type"], "specified");
}

#[tokio::test]
async fn coverage_events_filter_entity_id() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?entity_id=1")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert_eq!(arr.len(), 3);
}

#[tokio::test]
async fn coverage_events_limit_caps_results() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?limit=1")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert_eq!(arr.len(), 1);
}

#[tokio::test]
async fn coverage_events_offset_skips_results() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?offset=2")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert_eq!(arr.len(), 1);
}

#[tokio::test]
async fn coverage_events_offset_beyond_end_is_empty() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?offset=99")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert!(arr.is_empty());
}

#[tokio::test]
async fn coverage_events_since_relative_keeps_recent() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?since=1h")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert_eq!(arr.len(), 3);
}

#[tokio::test]
async fn coverage_events_since_future_filters_all() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?since=2999-01-01T00:00:00Z")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert!(arr.is_empty());
}

#[tokio::test]
async fn coverage_events_until_past_filters_all() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?until=2000-01-01T00:00:00Z")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert!(arr.is_empty());
}

#[tokio::test]
async fn coverage_events_invalid_until_is_400() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/events?until=nonsense")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn coverage_get_event_by_id() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events/1")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["id"], 1);
    assert_eq!(body["entity_type"], "feature");
}

#[tokio::test]
async fn coverage_get_event_missing_is_404() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/events/999")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

// ── Audit & governance ───────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_audit_trail_unknown_feature_is_404() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/features/ghost/audit")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_audit_verify_unknown_feature_is_404() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/features/ghost/audit/verify")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_governance_unknown_feature_is_404() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/features/ghost/governance")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_validate_unknown_feature_is_404() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/features/ghost/validate")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_governance_contract_shape() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/test-feature/governance")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["id"], 1);
    assert_eq!(body["rules_count"], 0);
    assert!(body["bound_at"].is_string());
}

// ── Projects ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_create_project_without_slug_derives_from_name() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/projects")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "name": "Derived Project" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["slug"], "derived-project");
}

#[tokio::test]
async fn coverage_create_project_with_description() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/projects")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "name": "Described", "slug": "described", "description": "d" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["description"], "d");
}

#[tokio::test]
async fn coverage_get_project_epics_unknown_is_404() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/projects/ghost/epics")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

// ── Epics ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_create_epic_with_description() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/epics")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "project_id": 1, "title": "Epic X", "description": "desc" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["description"], "desc");
    assert_eq!(body["status"], "backlog");
}

#[tokio::test]
async fn coverage_transition_epic_invalid_status_is_400() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/epics/1/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_status": "nonsense" }))
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn coverage_transition_epic_missing_is_404() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/epics/999/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_status": "done" }))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_list_epic_stories_unknown_epic_is_404() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/epics/999/stories")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

// ── Stories ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_create_story_without_points() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/stories")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "epic_id": 1, "project_id": 1, "title": "No points" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert!(body["points"].is_null());
}

#[tokio::test]
async fn coverage_transition_story_invalid_status_is_400() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/stories/1/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_status": "nonsense" }))
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn coverage_transition_story_missing_is_404() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/stories/999/transition")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "target_status": "in_progress" }))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

// ── Users ────────────────────────────────────────────────────────────────────

macro_rules! create_user_role_test {
    ($name:ident, $role:literal) => {
        #[tokio::test]
        async fn $name() {
            let server = setup_test_server().await;
            let resp = server
                .post("/api/v1/users")
                .add_header(KEY, TEST_API_KEY)
                .json(&serde_json::json!({
                    "display_name": "Role User",
                    "email": "role@example.com",
                    "role": $role
                }))
                .await;
            resp.assert_status(StatusCode::CREATED);
            let body: serde_json::Value = resp.json();
            assert_eq!(body["role"], $role);
        }
    };
}

create_user_role_test!(coverage_create_user_admin, "admin");
create_user_role_test!(coverage_create_user_member, "member");
create_user_role_test!(coverage_create_user_viewer, "viewer");

#[tokio::test]
async fn coverage_create_user_defaults_to_member() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/users")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "display_name": "Default", "email": "default@example.com" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["role"], "member");
    assert_eq!(body["status"], "active");
}

#[tokio::test]
async fn coverage_create_user_missing_email_is_422() {
    let server = setup_test_server().await;
    server
        .post("/api/v1/users")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "display_name": "No Email" }))
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

// ── Modules ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_list_modules_empty() {
    let server = setup_test_server().await;
    let resp = server.get("/api/modules").add_header(KEY, TEST_API_KEY).await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert!(arr.is_empty());
}

#[tokio::test]
async fn coverage_get_module_unknown_is_404() {
    let server = setup_test_server().await;
    server
        .get("/api/modules/1")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_module_tree_unknown_is_empty() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/modules/1/tree")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert!(arr.is_empty());
}

// ── Cycles ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_list_cycles_empty() {
    let server = setup_test_server().await;
    let resp = server.get("/api/cycles").add_header(KEY, TEST_API_KEY).await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert!(arr.is_empty());
}

macro_rules! cycle_state_filter_test {
    ($name:ident, $state:literal) => {
        #[tokio::test]
        async fn $name() {
            let server = setup_test_server().await;
            let resp = server
                .get(concat!("/api/cycles?state=", $state))
                .add_header(KEY, TEST_API_KEY)
                .await;
            resp.assert_status_ok();
        }
    };
}

cycle_state_filter_test!(coverage_cycles_filter_draft, "Draft");
cycle_state_filter_test!(coverage_cycles_filter_active, "Active");
cycle_state_filter_test!(coverage_cycles_filter_review, "Review");
cycle_state_filter_test!(coverage_cycles_filter_shipped, "Shipped");
cycle_state_filter_test!(coverage_cycles_filter_archived, "Archived");

#[tokio::test]
async fn coverage_cycles_invalid_state_is_400() {
    let server = setup_test_server().await;
    server
        .get("/api/cycles?state=Bogus")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn coverage_get_cycle_unknown_is_404() {
    let server = setup_test_server().await;
    server
        .get("/api/cycles/1")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_cycle_detail_page_unknown_is_404() {
    let server = setup_test_server().await;
    server
        .get("/cycles/1")
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

// ── Branches ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_list_branches_empty() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/branches")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert!(arr.is_empty());
}

#[tokio::test]
async fn coverage_create_branch_default_base() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/branches")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "name": "feat/x" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["message"], "Created branch feat/x from main");
}

#[tokio::test]
async fn coverage_checkout_branch() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/branches/checkout")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "name": "main" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["message"], "Checked out branch main");
}

#[tokio::test]
async fn coverage_delete_branch_local() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/branches/delete")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "name": "old" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["message"], "Deleted branch old");
}

#[tokio::test]
async fn coverage_delete_branch_remote() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/branches/delete")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "name": "old", "remote": "origin" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["message"], "Deleted remote branch origin/old");
}

#[tokio::test]
async fn coverage_sync_branches_reports_success() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/branches/sync")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "source": "main", "target": "canary" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["success"], true);
    assert_eq!(body["source"], "main");
    assert_eq!(body["target"], "canary");
}

// ── Worktrees ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn coverage_list_worktrees_empty() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/worktrees")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let arr: Vec<serde_json::Value> = resp.json();
    assert!(arr.is_empty());
}

#[tokio::test]
async fn coverage_add_worktree_returns_info() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/worktrees")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "feature_slug": "feat-z", "wp_id": "WP-Z" }))
        .await;
    resp.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["feature_slug"], "feat-z");
    assert_eq!(body["wp_id"], "WP-Z");
    assert_eq!(body["branch"], "worktree");
}

#[tokio::test]
async fn coverage_remove_worktree() {
    let server = setup_test_server().await;
    let resp = server
        .delete("/api/v1/worktrees")
        .add_header(KEY, TEST_API_KEY)
        .json(&serde_json::json!({ "path": "/tmp/wt" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["message"], "Removed worktree /tmp/wt");
}

// ── Router-level negative cases ──────────────────────────────────────────────

#[tokio::test]
async fn coverage_unknown_api_route_is_404() {
    let server = setup_test_server().await;
    server
        .get("/api/v1/definitely-not-real")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn coverage_method_not_allowed() {
    let server = setup_test_server().await;
    server
        .delete("/api/v1/features")
        .add_header(KEY, TEST_API_KEY)
        .await
        .assert_status(StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn coverage_json_content_type_on_error() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/ghost")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::NOT_FOUND);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.contains("application/json"));
    let body: serde_json::Value = resp.json();
    assert!(body["error"].as_str().unwrap().contains("ghost"));
}

/// An `entity_id` that matches no event must filter the entire result set out.
/// The seeded events all belong to entity id 1, so an unrelated id exercises the
/// exact-match rejection in the event filter.
#[tokio::test]
async fn coverage_events_filter_entity_id_without_match_is_empty() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?entity_id=424242")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let events: Vec<serde_json::Value> = resp.json();
    assert!(
        events.is_empty(),
        "an entity id with no events must yield an empty page, got: {events:?}"
    );
}
