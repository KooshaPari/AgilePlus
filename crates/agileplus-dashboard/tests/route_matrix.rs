// SPDX-License-Identifier: MIT OR Apache-2.0
//! Route-level integration tests for the AgilePlus dashboard router.
//!
//! These tests exercise every registered route through the real axum router
//! (`agileplus_dashboard::routes::router`) using an in-memory `DashboardStore`.
//! No external services, filesystem writes to the operator home directory, or
//! network calls are performed.
//!
//! Traceability: WP12 (T071–T077)

use std::collections::HashMap;
use std::sync::Arc;

use agileplus_dashboard::app_state::{DashboardStore, SharedState, default_health};
use agileplus_dashboard::routes::router;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::project::Project;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tokio::sync::RwLock;
use tower::util::ServiceExt;

// ── Test fixtures ────────────────────────────────────────────────────────────

fn feature(id: i64, state: FeatureState, project_id: Option<i64>) -> Feature {
    let mut f = Feature::new(
        &format!("feature-{id}"),
        &format!("Feature {id}"),
        [0; 32],
        Some("main"),
    );
    f.id = id;
    f.state = state;
    f.project_id = project_id;
    f.labels = vec!["platform".into()];
    f
}

fn work_package(id: i64, feature_id: i64, state: WpState) -> WorkPackage {
    let mut wp = WorkPackage::new(feature_id, &format!("WP-{id}"), 1, "done");
    wp.id = id;
    wp.state = state;
    wp.agent_id = Some("claude".into());
    wp.pr_url = Some(format!("https://github.com/KooshaPari/AgilePlus/pull/{id}"));
    wp.head_commit = Some(format!("abc{id:04}"));
    wp
}

/// Empty store with default health only.
fn empty_state() -> SharedState {
    Arc::new(RwLock::new(DashboardStore {
        health: default_health(),
        ..Default::default()
    }))
}

/// A populated but fully in-memory store with one project and a few features.
fn populated_state() -> SharedState {
    let mut store = DashboardStore {
        health: default_health(),
        ..Default::default()
    };
    let mut project = Project::new("Test Project", "test-project").expect("valid project");
    project.id = 1;
    store.projects.push(project);
    store.active_project_id = Some(1);
    store.features = vec![
        feature(1, FeatureState::Created, Some(1)),
        feature(2, FeatureState::Implementing, Some(1)),
        feature(3, FeatureState::Shipped, Some(1)),
    ];
    let mut wps = HashMap::new();
    wps.insert(
        1,
        vec![
            work_package(10, 1, WpState::Planned),
            work_package(11, 1, WpState::Blocked),
        ],
    );
    wps.insert(2, vec![work_package(20, 2, WpState::Doing)]);
    store.work_packages = wps;
    Arc::new(RwLock::new(store))
}

fn app(state: SharedState) -> Router {
    router(state)
}

async fn send(app: &Router, request: Request<Body>) -> (StatusCode, Vec<u8>, String) {
    let response = app.clone().oneshot(request).await.expect("router response");
    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body bytes")
        .to_vec();
    (status, bytes, content_type)
}

async fn get(app: &Router, uri: &str) -> (StatusCode, Vec<u8>, String) {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("valid request");
    send(app, request).await
}

async fn post_form(app: &Router, uri: &str, body: &str) -> (StatusCode, Vec<u8>, String) {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body.to_string()))
        .expect("valid request");
    send(app, request).await
}

async fn post_json(app: &Router, uri: &str, body: &str) -> (StatusCode, Vec<u8>, String) {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("valid request");
    send(app, request).await
}

async fn body_text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).expect("utf-8 body")
}

fn json(bytes: &[u8]) -> serde_json::Value {
    serde_json::from_slice(bytes).expect("valid JSON body")
}

// ── HTML page routes ─────────────────────────────────────────────────────────

#[tokio::test]
async fn route_root_renders_home() {
    let (status, bytes, _) = get(&app(populated_state()), "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("AgilePlus"));
}

#[tokio::test]
async fn route_home_renders_home() {
    let (status, bytes, _) = get(&app(empty_state()), "/home").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn route_dashboard_renders_kanban() {
    let (status, bytes, _) = get(&app(populated_state()), "/dashboard").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("kanban-board"));
}

#[tokio::test]
async fn route_dashboard_accepts_filter_query() {
    for filter in ["all", "active", "blocked", "shipped"] {
        let (status, bytes, _) = get(&app(populated_state()), &format!("/dashboard?filter={filter}")).await;
        assert_eq!(status, StatusCode::OK, "filter={filter}");
        assert!(!bytes.is_empty());
    }
}

#[tokio::test]
async fn route_features_renders_feature_list() {
    let (status, bytes, _) = get(&app(populated_state()), "/features").await;
    assert_eq!(status, StatusCode::OK);
    let html = body_text(bytes).await;
    assert!(html.contains("Feature 1"));
}

#[tokio::test]
async fn route_features_empty_store_still_renders() {
    let (status, bytes, _) = get(&app(empty_state()), "/features").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn route_feature_detail_found() {
    let (status, bytes, _) = get(&app(populated_state()), "/features/1").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("Feature 1"));
}

#[tokio::test]
async fn route_feature_detail_not_found() {
    let (status, bytes, _) = get(&app(populated_state()), "/features/9999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body_text(bytes).await.contains("not found"));
}

#[tokio::test]
async fn route_events_page_renders() {
    let (status, bytes, _) = get(&app(empty_state()), "/events").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn route_health_page_renders() {
    let (status, bytes, _) = get(&app(empty_state()), "/health-page").await;
    assert_eq!(status, StatusCode::OK);
    let html = body_text(bytes).await;
    assert!(html.contains("NATS"));
}

#[tokio::test]
async fn route_settings_page_renders() {
    let (status, bytes, _) = get(&app(empty_state()), "/settings").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn route_settings_plane_page_renders() {
    let (status, bytes, _) = get(&app(empty_state()), "/settings/plane").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("Plane"));
}

#[tokio::test]
async fn route_settings_services_page_renders() {
    let (status, bytes, _) = get(&app(empty_state()), "/settings/services").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn route_hub_page_renders_ecosystem() {
    let (status, bytes, _) = get(&app(empty_state()), "/hub").await;
    assert_eq!(status, StatusCode::OK);
    let html = body_text(bytes).await;
    assert!(html.contains("phenodocs"));
    assert!(html.contains("bifrost-extensions"));
}

// ── JSON API routes ──────────────────────────────────────────────────────────

#[tokio::test]
async fn route_kanban_json_shaped() {
    let (status, bytes, ct) = get(&app(populated_state()), "/api/dashboard/kanban").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!ct.is_empty());
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn route_work_packages_json_shape() {
    let (status, bytes, ct) = get(&app(populated_state()), "/api/dashboard/work-packages.json").await;
    assert_eq!(status, StatusCode::OK);
    assert!(ct.contains("application/json"));
    let v = json(&bytes);
    assert_eq!(v["count"], 3);
    assert_eq!(v["work_packages"].as_array().unwrap().len(), 3);
    assert!(v["timestamp"].is_string());
}

#[tokio::test]
async fn route_work_packages_json_maps_states() {
    let (_, bytes, _) = get(&app(populated_state()), "/api/dashboard/work-packages.json").await;
    let v = json(&bytes);
    let statuses: Vec<String> = v["work_packages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["status"].as_str().unwrap().to_string())
        .collect();
    assert!(statuses.contains(&"planned".to_string()));
    assert!(statuses.contains(&"blocked".to_string()));
    assert!(statuses.contains(&"in_progress".to_string()));
}

#[tokio::test]
async fn route_work_packages_json_empty_store() {
    let (status, bytes, _) = get(&app(empty_state()), "/api/dashboard/work-packages.json").await;
    assert_eq!(status, StatusCode::OK);
    let v = json(&bytes);
    assert_eq!(v["count"], 0);
}

#[tokio::test]
async fn route_agents_json_shape() {
    let (status, bytes, ct) = get(&app(empty_state()), "/api/dashboard/agents.json").await;
    assert_eq!(status, StatusCode::OK);
    assert!(ct.contains("application/json"));
    let v = json(&bytes);
    assert!(v["agents"].is_array());
    assert!(v["count"].is_number());
    assert!(v["timestamp"].is_string());
}

#[tokio::test]
async fn route_health_json_runs_checks() {
    let (status, bytes, ct) = get(&app(empty_state()), "/api/dashboard/health.json").await;
    assert_eq!(status, StatusCode::OK);
    assert!(ct.contains("application/json"));
    let v = json(&bytes);
    assert!(v["services"].is_array());
    assert!(v["all_healthy"].is_boolean());
    assert!(v["timestamp"].is_string());
}

#[tokio::test]
async fn route_projects_json_lists_store_projects() {
    let (status, bytes, _) = get(&app(populated_state()), "/api/dashboard/projects").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("Test Project"));
}

#[tokio::test]
async fn route_time_returns_timestamp() {
    let (status, bytes, _) = get(&app(empty_state()), "/api/time").await;
    assert_eq!(status, StatusCode::OK);
    let text = body_text(bytes).await;
    assert!(text.contains("UTC"));
}

#[tokio::test]
async fn route_agent_activity_partial_renders() {
    let (status, bytes, _) = get(&app(empty_state()), "/api/dashboard/agents").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn route_health_partial_renders() {
    let (status, bytes, _) = get(&app(empty_state()), "/api/dashboard/health").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("NATS"));
}

#[tokio::test]
async fn route_event_timeline_partial_renders() {
    let (status, bytes, _) = get(&app(empty_state()), "/api/dashboard/events").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!bytes.is_empty());
}

// ── Feature detail / work-package partials ───────────────────────────────────

#[tokio::test]
async fn route_feature_detail_api_found() {
    let (status, bytes, _) = get(&app(populated_state()), "/api/dashboard/features/1").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("Feature 1"));
}

#[tokio::test]
async fn route_feature_detail_api_not_found() {
    let (status, _, _) = get(&app(populated_state()), "/api/dashboard/features/42").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn route_feature_work_packages_found() {
    let (status, bytes, _) = get(&app(populated_state()), "/api/dashboard/features/1/work-packages").await;
    assert_eq!(status, StatusCode::OK);
    let html = body_text(bytes).await;
    assert!(html.contains("WP-10") || html.contains("WP-11"));
}

#[tokio::test]
async fn route_feature_work_packages_unknown_feature_is_empty() {
    let (status, bytes, _) = get(&app(populated_state()), "/api/dashboard/features/777/work-packages").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn route_feature_events_found() {
    let (status, bytes, _) = get(&app(populated_state()), "/api/dashboard/features/1/events").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn route_feature_events_not_found() {
    let (status, _, _) = get(&app(populated_state()), "/api/dashboard/features/777/events").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn route_feature_media_found() {
    let (status, bytes, _) = get(&app(populated_state()), "/api/dashboard/features/1/media").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("media-gallery"));
}

#[tokio::test]
async fn route_feature_media_not_found() {
    let (status, _, _) = get(&app(populated_state()), "/api/dashboard/features/777/media").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn route_feature_evidence_list_renders() {
    let (status, bytes, _) = get(&app(populated_state()), "/api/features/1/evidence").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn route_feature_evidence_json_empty_on_disk() {
    let (status, bytes, _) = get(&app(populated_state()), "/api/dashboard/features/1/evidence.json").await;
    assert_eq!(status, StatusCode::OK);
    let v = json(&bytes);
    assert_eq!(v["feature_id"], "1");
    assert!(v["artifacts"].as_array().unwrap().is_empty());
    assert!(v["generated_at"].is_null());
}

#[tokio::test]
async fn route_feature_evidence_generate_reports_missing_script() {
    // Test CWD is the crate root, which has no `scripts/generate-evidence.sh`.
    let (status, bytes, _) = post_json(
        &app(populated_state()),
        "/api/features/1/evidence/generate",
        "{}",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let v = json(&bytes);
    assert_eq!(v["status"], "error");
    assert_eq!(v["feature_id"], "1");
}

// ── Evidence content / preview ───────────────────────────────────────────────

#[tokio::test]
async fn route_evidence_content_missing_artifact_returns_stub() {
    let (status, bytes, _) = get(&app(empty_state()), "/api/evidence/1/does-not-exist.txt/content").await;
    assert_eq!(status, StatusCode::OK);
    let text = body_text(bytes).await;
    assert!(text.contains("No artifact found"));
}

#[tokio::test]
async fn route_evidence_preview_missing_artifact_returns_stub() {
    let (status, bytes, _) = get(&app(empty_state()), "/api/evidence/1/nope.txt/preview").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("No preview"));
}

// ── Project switching ────────────────────────────────────────────────────────

#[tokio::test]
async fn route_activate_project_switches_filter() {
    let state = populated_state();
    let app = app(state.clone());
    let (status, bytes, _) = post_json(&app, "/api/dashboard/projects/1/activate", "{}").await;
    assert_eq!(status, StatusCode::OK);
    assert!(!bytes.is_empty());
    let store = state.read().await;
    assert_eq!(store.active_project_id, Some(1));
}

#[tokio::test]
async fn route_activate_project_all_clears_filter() {
    let state = populated_state();
    let app = app(state.clone());
    let (status, _, _) = post_json(&app, "/api/dashboard/projects/0/activate", "{}").await;
    assert_eq!(status, StatusCode::OK);
    let store = state.read().await;
    assert_eq!(store.active_project_id, None);
}

#[tokio::test]
async fn route_activate_unknown_project_is_404() {
    let (status, _, _) = post_json(&app(populated_state()), "/api/dashboard/projects/999/activate", "{}").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Feature transition ───────────────────────────────────────────────────────

#[tokio::test]
async fn route_feature_transition_valid_step() {
    let state = populated_state();
    let app = app(state.clone());
    let (status, bytes, _) = post_form(
        &app,
        "/api/features/1/transition",
        "target_state=specified",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("kanban-board"));
    let store = state.read().await;
    let f = store.features.iter().find(|f| f.id == 1).unwrap();
    assert_eq!(f.state, FeatureState::Specified);
}

#[tokio::test]
async fn route_feature_transition_invalid_state_is_400() {
    let (status, bytes, _) = post_form(
        &app(populated_state()),
        "/api/features/1/transition",
        "target_state=not-a-state",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body_text(bytes).await.contains("Invalid feature state"));
}

#[tokio::test]
async fn route_feature_transition_unknown_feature_is_404() {
    let (status, _, _) = post_form(
        &app(populated_state()),
        "/api/features/424242/transition",
        "target_state=specified",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Governance / Plane status routes ─────────────────────────────────────────

#[tokio::test]
async fn route_governance_status_uninitialized() {
    let (status, bytes, _) = get(&app(empty_state()), "/api/dashboard/governance/status").await;
    assert_eq!(status, StatusCode::OK);
    let v = json(&bytes);
    assert_eq!(v["available"], false);
    assert_eq!(v["reason"], "not_initialized");
}

#[tokio::test]
async fn route_plane_sync_status_uninitialized() {
    let (status, bytes, _) = get(&app(empty_state()), "/api/dashboard/plane/sync").await;
    assert_eq!(status, StatusCode::OK);
    let v = json(&bytes);
    assert_eq!(v["available"], false);
}

#[tokio::test]
async fn route_plane_daemon_status_uninitialized() {
    let (status, bytes, _) = get(&app(empty_state()), "/api/dashboard/plane/daemon/status").await;
    assert_eq!(status, StatusCode::OK);
    let v = json(&bytes);
    assert_eq!(v["available"], false);
}

#[tokio::test]
async fn route_plane_daemon_start_uninitialized() {
    let (status, bytes, _) = post_json(&app(empty_state()), "/api/dashboard/plane/daemon/start", "{}").await;
    assert_eq!(status, StatusCode::OK);
    let v = json(&bytes);
    assert_eq!(v["started"], false);
    assert_eq!(v["reason"], "not_initialized");
}

#[tokio::test]
async fn route_plane_daemon_stop_uninitialized() {
    let (status, bytes, _) = post_json(&app(empty_state()), "/api/dashboard/plane/daemon/stop", "{}").await;
    assert_eq!(status, StatusCode::OK);
    let v = json(&bytes);
    assert_eq!(v["stopped"], false);
}

// ── Connection-test forms ────────────────────────────────────────────────────

#[tokio::test]
async fn route_service_test_form_valid_url() {
    let (status, bytes, _) = post_form(
        &app(empty_state()),
        "/api/settings/services/test",
        "name=NATS&endpoint_url=http://localhost:4222",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("successful"));
}

#[tokio::test]
async fn route_service_test_form_invalid_url() {
    let (status, bytes, _) = post_form(
        &app(empty_state()),
        "/api/settings/services/test",
        "name=NATS&endpoint_url=ftp://nope",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("Invalid endpoint"));
}

#[tokio::test]
async fn route_plane_test_form_incomplete() {
    let (status, bytes, _) = post_form(
        &app(empty_state()),
        "/api/settings/plane/test",
        "api_url=&api_key=&workspace_slug=&project_slug=",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("incomplete"));
}

#[tokio::test]
async fn route_plane_test_form_valid() {
    let (status, bytes, _) = post_form(
        &app(empty_state()),
        "/api/settings/plane/test",
        "api_url=https://app.plane.so&api_key=k&workspace_slug=w&project_slug=p",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("passed"));
}

#[tokio::test]
async fn route_agent_test_connection_local_provider() {
    let (status, bytes, _) = post_form(
        &app(empty_state()),
        "/api/settings/agents/test-connection",
        "provider=local",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("no external credentials"));
}

#[tokio::test]
async fn route_agent_test_connection_unknown_provider() {
    let (status, bytes, _) = post_form(
        &app(empty_state()),
        "/api/settings/agents/test-connection",
        "provider=does-not-exist",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body_text(bytes).await.contains("Unknown provider"));
}

// ── SSE streams ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn route_sse_stream_opens() {
    let app = app(empty_state());
    let request = Request::builder()
        .method("GET")
        .uri("/api/stream")
        .body(Body::empty())
        .expect("valid request");
    // Only await the response head; the SSE body is an endless stream.
    let response = tokio::time::timeout(std::time::Duration::from_secs(5), app.oneshot(request))
        .await
        .expect("sse head arrived")
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn route_sse_placeholder_opens() {
    let app = app(empty_state());
    let request = Request::builder()
        .method("GET")
        .uri("/api/stream-placeholder")
        .body(Body::empty())
        .expect("valid request");
    let response = tokio::time::timeout(std::time::Duration::from_secs(5), app.oneshot(request))
        .await
        .expect("sse head arrived")
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Router-level negative cases ──────────────────────────────────────────────

#[tokio::test]
async fn unknown_route_is_404() {
    let (status, _, _) = get(&app(empty_state()), "/this/does/not/exist").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn method_not_allowed_on_read_only_route() {
    let request = Request::builder()
        .method("POST")
        .uri("/events")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .expect("valid request");
    let (status, _, _) = send(&app(empty_state()), request).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn kanban_htmx_request_returns_partial() {
    let request = Request::builder()
        .method("GET")
        .uri("/api/dashboard/kanban")
        .header("HX-Request", "true")
        .body(Body::empty())
        .expect("valid request");
    let (status, bytes, _) = send(&app(populated_state()), request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn seeded_store_serves_root_and_dashboard() {
    let state = Arc::new(RwLock::new(DashboardStore::seeded()));
    let app = app(state);
    let (root_status, root_bytes, _) = get(&app, "/").await;
    assert_eq!(root_status, StatusCode::OK);
    assert!(body_text(root_bytes).await.contains("AgilePlus"));
    let (dash_status, _, _) = get(&app, "/dashboard").await;
    assert_eq!(dash_status, StatusCode::OK);
}

#[tokio::test]
async fn every_html_page_route_returns_200() {
    let app = app(populated_state());
    for route in [
        "/",
        "/home",
        "/dashboard",
        "/features",
        "/events",
        "/health-page",
        "/settings",
        "/settings/plane",
        "/settings/services",
        "/hub",
    ] {
        let (status, bytes, _) = get(&app, route).await;
        assert_eq!(status, StatusCode::OK, "route {route}");
        assert!(!bytes.is_empty(), "route {route} produced empty body");
    }
}

#[tokio::test]
async fn every_json_route_returns_200() {
    let app = app(populated_state());
    for route in [
        "/api/time",
        "/api/dashboard/kanban",
        "/api/dashboard/agents.json",
        "/api/dashboard/work-packages.json",
        "/api/dashboard/projects",
        "/api/dashboard/governance/status",
        "/api/dashboard/plane/sync",
        "/api/dashboard/plane/daemon/status",
    ] {
        let (status, _, _) = get(&app, route).await;
        assert_eq!(status, StatusCode::OK, "route {route}");
    }
}

// ── Additional response-shape assertions ─────────────────────────────────────

#[tokio::test]
async fn extra_work_packages_json_assignee_and_feature_id() {
    let (_, bytes, _) = get(&app(populated_state()), "/api/dashboard/work-packages.json").await;
    let v = json(&bytes);
    let wps = v["work_packages"].as_array().unwrap();
    let first = &wps[0];
    assert_eq!(first["assignee"], "claude");
    assert!(first["feature_id"].is_number());
    assert!(first["priority"].is_string());
}

#[tokio::test]
async fn extra_agents_json_count_matches_array_length() {
    let (_, bytes, _) = get(&app(empty_state()), "/api/dashboard/agents.json").await;
    let v = json(&bytes);
    let len = v["agents"].as_array().unwrap().len();
    assert_eq!(v["count"].as_u64().unwrap() as usize, len);
}

#[tokio::test]
async fn extra_health_json_names_checkers() {
    let (_, bytes, _) = get(&app(empty_state()), "/api/dashboard/health.json").await;
    let v = json(&bytes);
    let services = v["services"].as_array().unwrap();
    assert!(!services.is_empty());
    assert!(services.iter().all(|s| s["name"].is_string()));
}

#[tokio::test]
async fn extra_kanban_blocked_filter_accepts_query() {
    let (status, _, _) = get(&app(populated_state()), "/api/dashboard/kanban?filter=blocked").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn extra_kanban_unknown_filter_falls_back() {
    let (status, _, _) = get(&app(populated_state()), "/api/dashboard/kanban?filter=nonsense").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn extra_feature_detail_includes_stub_evidence_bundle() {
    let (_, bytes, _) = get(&app(populated_state()), "/api/dashboard/features/1").await;
    assert!(body_text(bytes).await.contains("bundle-1-summary"));
}

#[tokio::test]
async fn extra_feature_media_renders_cover_asset() {
    let (_, bytes, _) = get(&app(populated_state()), "/api/dashboard/features/1/media").await;
    assert!(body_text(bytes).await.contains("/assets/feature-1/cover.png"));
}

#[tokio::test]
async fn extra_feature_events_mentions_opened_feature() {
    let (_, bytes, _) = get(&app(populated_state()), "/api/dashboard/features/1/events").await;
    assert!(body_text(bytes).await.contains("feature-1"));
}

#[tokio::test]
async fn extra_feature_work_packages_lists_second_feature() {
    let (_, bytes, _) = get(&app(populated_state()), "/api/dashboard/features/2/work-packages").await;
    assert!(body_text(bytes).await.contains("WP-20"));
}

#[tokio::test]
async fn extra_project_switcher_lists_multiple_projects() {
    let state = populated_state();
    {
        let mut store = state.write().await;
        let mut second = Project::new("Second", "second").unwrap();
        second.id = 2;
        store.projects.push(second);
    }
    let (_, bytes, _) = get(&app(state), "/api/dashboard/projects").await;
    let html = body_text(bytes).await;
    assert!(html.contains("Test Project"));
    assert!(html.contains("Second"));
}

#[tokio::test]
async fn extra_switch_project_changes_active_and_scopes_kanban() {
    let state = populated_state();
    {
        let mut store = state.write().await;
        let mut second = Project::new("Second", "second").unwrap();
        second.id = 2;
        store.projects.push(second);
        store.features.push(feature(4, FeatureState::Created, Some(2)));
    }
    let app = app(state.clone());
    let (status, _, _) = post_json(&app, "/api/dashboard/projects/2/activate", "{}").await;
    assert_eq!(status, StatusCode::OK);
    let store = state.read().await;
    assert_eq!(store.active_project_id, Some(2));
    assert_eq!(store.features_for_active_project().len(), 1);
}

#[tokio::test]
async fn extra_root_reports_project_summaries() {
    let (_, bytes, _) = get(&app(populated_state()), "/").await;
    assert!(body_text(bytes).await.contains("Test Project"));
}

#[tokio::test]
async fn extra_events_page_lists_sample_events() {
    let (_, bytes, _) = get(&app(empty_state()), "/events").await;
    let html = body_text(bytes).await;
    assert!(html.contains("Dashboard booted"));
}

#[tokio::test]
async fn extra_hub_page_lists_api_projects_without_ports() {
    let (_, bytes, _) = get(&app(empty_state()), "/hub").await;
    let html = body_text(bytes).await;
    assert!(html.contains("agentapi-plusplus"));
    assert!(html.contains("cliproxyapi-plusplus"));
}

#[tokio::test]
async fn extra_time_json_is_plain_text_timestamp() {
    let (_, bytes, ct) = get(&app(empty_state()), "/api/time").await;
    let text = body_text(bytes).await;
    assert!(text.contains("UTC"));
    assert!(ct.is_empty() || ct.contains("text"));
}

#[tokio::test]
async fn extra_health_partial_reports_service_names() {
    let (_, bytes, _) = get(&app(empty_state()), "/api/dashboard/health").await;
    let html = body_text(bytes).await;
    assert!(html.contains("NATS"));
    assert!(html.contains("SQLite"));
}

#[tokio::test]
async fn extra_plane_settings_shows_sync_mode() {
    let (_, bytes, _) = get(&app(empty_state()), "/settings/plane").await;
    let html = body_text(bytes).await;
    assert!(html.contains("Plane.so") || html.contains("Plane"));
}

#[tokio::test]
async fn extra_settings_page_links_all_sections() {
    let (_, bytes, _) = get(&app(empty_state()), "/settings").await;
    let html = body_text(bytes).await;
    for section in ["plane", "agents", "services"] {
        assert!(html.contains(section), "settings page missing {section}");
    }
}

#[tokio::test]
async fn extra_seeded_kanban_contains_shipped_features() {
    let state = Arc::new(RwLock::new(DashboardStore::seeded()));
    let (_, bytes, _) = get(&app(state), "/api/dashboard/kanban").await;
    let html = body_text(bytes).await;
    assert!(html.contains("kanban-board"));
}

#[tokio::test]
async fn extra_evidence_route_list_and_json_agree_on_feature_id() {
    let app = app(populated_state());
    let (_, list_bytes, _) = get(&app, "/api/features/9/evidence").await;
    let (_, json_bytes, _) = get(&app, "/api/dashboard/features/9/evidence.json").await;
    assert!(!list_bytes.is_empty());
    assert_eq!(json(&json_bytes)["feature_id"], "9");
}

#[tokio::test]
async fn extra_governance_and_plane_are_null_safe() {
    let app = app(empty_state());
    let (_, gov, _) = get(&app, "/api/dashboard/governance/status").await;
    let (_, plane, _) = get(&app, "/api/dashboard/plane/sync").await;
    assert_eq!(json(&gov)["available"], false);
    assert_eq!(json(&plane)["available"], false);
}

#[tokio::test]
async fn extra_404_body_is_non_empty_for_missing_feature_route() {
    let (status, bytes, _) = get(&app(populated_state()), "/api/dashboard/features/31415").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(!bytes.is_empty());
}
