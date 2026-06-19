//! Axum route handlers for the dashboard. (T077)
//!
//! This module provides HTTP request handlers for the AgilePlus dashboard,
//! organized into per-domain submodules:
//! - **pages**: Root, home, features, events, settings, hub pages
//! - **dashboard**: Kanban board, work packages, SSE streams
//! - **features**: Feature detail, transitions, events, media
//! - **evidence**: Evidence bundles, artifacts, generation
//! - **agents**: Agent activity detection
//! - **health**: Service health monitoring and management
//! - **settings**: Configuration pages and persistence
//! - **helpers**: Shared utility functions and types
//!
//! Pattern: if the request carries `HX-Request: true`, return only the
//! relevant partial; otherwise return the full page layout.

use axum::{
    routing::{get, post},
    Router,
};

use crate::app_state::SharedState;

pub mod agents;
pub mod dashboard;
pub mod evidence;
pub mod features;
pub mod health;
pub mod helpers;
pub mod pages;
pub mod settings;

// ── Route Handler Re-exports ──────────────────────────────────────────────
// Exported for backward compatibility with call sites like routes::feature_detail

// From pages
pub use pages::{
    root, home, events_page, features_page, settings_page,
};

// From dashboard
pub use dashboard::{
    kanban_board, wp_list, all_work_packages_json, epics_stories_json,
    project_switcher, switch_project, time_footer, sse_stream,
    WorkPackageJson,
};

// From features
pub use features::{
    feature_detail, feature_page, feature_transition, feature_events, feature_media,
    FeatureTransitionForm,
};

// From evidence
pub use evidence::{
    evidence_content, evidence_preview, feature_evidence_list,
    feature_evidence_generate, feature_evidence_json,
    EvidenceGalleryJson, EvidenceArtifactJson,
};

// From agents
pub use agents::{
    agent_activity, agents_json,
};

// From health
pub use health::{
    health_panel, health_json, health_page, restart_service,
    toggle_service, patch_service_config,
    HealthStatus, ServiceHealthJson,
};

// From settings
pub use settings::{
    plane_settings_page, agent_settings_page, services_settings_page,
    save_plane_settings, save_agent_settings, save_dashboard_settings,
    save_services_settings, test_service_connection, test_plane_connection,
    test_agent_connection,
    // Types
    Config, PlaneConfig, AgentConfig, ServiceConfig, DashboardConfig,
    PlaneSettingsForm, AgentSettingsForm, ServiceSettingsForm, DashboardSettingsForm,
    SingleServiceTestForm,
};

// ── Event Timeline Handler ─────────────────────────────────────────────

pub async fn event_timeline(axum::extract::State(_state): axum::extract::State<SharedState>) -> axum::response::Response {
    use crate::templates::EventTimelinePartial;
    helpers::render(EventTimelinePartial {
        feature_id: 0,
        events: vec![],
    })
}

// Dashboard page from settings module (re-export to avoid duplication)
pub use settings::dashboard_page;

// Hub page from pages module (extracted handlers don't include it so use the one from pages)
pub use pages::hub_page;

// ── Router builder ───────────────────────────────────────────────────────

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/home", get(home))
        .route("/dashboard", get(dashboard_page))
        .route("/features", get(features_page))
        .route("/features/{id}", get(feature_page))
        .route("/events", get(events_page))
        // NOTE: /health is owned by agileplus-api (router.rs, T070, detailed
        // JSON health). The HTML health page lives at /health-page to avoid
        // a route conflict panic when build_router merges the two routers.
        .route("/health-page", get(health_page))
        .route("/settings", get(settings_page))
        .route("/settings/plane", get(plane_settings_page))
        .route("/settings/agents", get(agent_settings_page))
        .route("/settings/services", get(services_settings_page))
        .route("/api/settings/services", post(save_services_settings))
        .route("/api/settings/services/test", post(test_service_connection))
        .route("/hub", get(hub_page))
        .route("/api/settings/plane", post(save_plane_settings))
        .route("/api/settings/plane/test", post(test_plane_connection))
        .route("/api/settings/agents", post(save_agent_settings))
        .route(
            "/api/settings/agents/test-connection",
            post(test_agent_connection),
        )
        .route("/api/settings/dashboard", post(save_dashboard_settings))
        .route(
            "/api/dashboard/services/{name}/restart",
            post(restart_service),
        )
        .route(
            "/api/dashboard/services/{name}/config",
            axum::routing::patch(patch_service_config),
        )
        .route(
            "/api/dashboard/services/{name}/toggle",
            post(toggle_service),
        )
        .route("/api/dashboard/kanban", get(kanban_board))
        .route("/api/dashboard/features/{id}", get(feature_detail))
        .route("/api/dashboard/features/{id}/work-packages", get(wp_list))
        .route("/api/dashboard/features/{id}/events", get(feature_events))
        .route("/api/dashboard/features/{id}/media", get(feature_media))
        // HTML partial endpoints (HTMX-compatible)
        .route("/api/dashboard/health", get(health_panel))
        .route("/api/dashboard/events", get(event_timeline))
        .route("/api/dashboard/agents", get(agent_activity))
        // JSON API endpoints (for polling from JavaScript templates)
        .route("/api/dashboard/agents.json", get(agents_json))
        .route("/api/dashboard/health.json", get(health_json))
        .route(
            "/api/dashboard/work-packages.json",
            get(all_work_packages_json),
        )
        .route("/api/dashboard/epics-stories.json", get(epics_stories_json))
        .route("/api/dashboard/projects", get(project_switcher))
        .route(
            "/api/dashboard/projects/{id}/activate",
            post(switch_project),
        )
        .route("/api/time", get(time_footer))
        .route("/api/stream", get(sse_stream))
        // NOTE: /api/v1/stream is owned by agileplus-api (router.rs, T069).
        // Previously this crate registered an alias (#334), but that caused a
        // route conflict panic when build_router merged the two routers.
        .route("/api/stream-placeholder", get(sse_stream))
        .route(
            "/api/evidence/{feature_id}/{artifact_id}/content",
            get(evidence_content),
        )
        .route(
            "/api/evidence/{feature_id}/{artifact_id}/preview",
            get(evidence_preview),
        )
        .route("/api/features/{id}/evidence", get(feature_evidence_list))
        .route(
            "/api/features/{id}/evidence/generate",
            post(feature_evidence_generate),
        )
        .route(
            "/api/dashboard/features/{id}/evidence.json",
            get(feature_evidence_json),
        )
        .route("/api/features/{id}/transition", post(feature_transition))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::{default_health, DashboardStore};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tower::util::ServiceExt;

    fn make_state() -> SharedState {
        let store = DashboardStore {
            health: default_health(),
            ..Default::default()
        };
        Arc::new(RwLock::new(store))
    }

    #[tokio::test]
    async fn toggle_service_updates_store_and_responds() {
        let state = make_state();
        let app = router(state.clone());

        let request_body = serde_json::json!({ "enabled": false }).to_string();
        let request = axum::http::Request::builder()
            .method("POST")
            .uri("/api/dashboard/services/NATS/toggle")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(request_body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_text.contains("\"status\":\"ok\""));
        assert!(body_text.contains("\"enabled\":false"));

        let store = state.read().await;
        let health = store.health.iter().find(|s| s.name == "NATS").unwrap();
        assert!(!health.healthy);
        assert!(health.degraded);
    }

    #[tokio::test]
    async fn restart_service_executes_command() {
        let state = make_state();
        let app = router(state.clone());

        unsafe {
            std::env::set_var("AGILEPLUS_SERVICE_RESTART_CMD", "echo restarted {}");
        }

        let request = axum::http::Request::builder()
            .method("POST")
            .uri("/api/dashboard/services/NATS/restart")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_text.contains("\"status\":\"ok\""));
        assert!(body_text.contains("\"service\":\"NATS\""));
        assert!(body_text.contains("restarted NATS"));
    }

    #[test]
    fn test_html_escape_ampersand() {
        assert_eq!(helpers::html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_html_escape_angle_brackets() {
        assert_eq!(helpers::html_escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_html_escape_quotes() {
        assert_eq!(helpers::html_escape("say \"hello\""), "say &quot;hello&quot;");
        assert_eq!(helpers::html_escape("it's"), "it&#39;s");
    }

    #[test]
    fn test_is_htmx_true() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("HX-Request", "true".parse().unwrap());
        assert!(helpers::is_htmx(&headers));
    }

    #[test]
    fn test_is_htmx_false_absent() {
        let headers = axum::http::HeaderMap::new();
        assert!(!helpers::is_htmx(&headers));
    }

    #[test]
    fn test_is_htmx_false_wrong_value() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("HX-Request", "1".parse().unwrap());
        assert!(!helpers::is_htmx(&headers));
    }

    #[tokio::test]
    async fn json_endpoints_integrated_agents_in_router() {
        let state = make_state();
        let app = router(state);

        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/api/dashboard/agents.json")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();

        let json: serde_json::Value = serde_json::from_str(&body_text).expect("valid JSON");
        assert!(json.get("agents").is_some());
        assert!(json.get("count").is_some());
        assert!(json.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn json_endpoints_integrated_health_in_router() {
        let state = make_state();
        let app = router(state);

        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/api/dashboard/health.json")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();

        let json: serde_json::Value = serde_json::from_str(&body_text).expect("valid JSON");
        assert!(json.get("services").is_some());
        assert!(json.get("timestamp").is_some());
        assert!(json.get("all_healthy").is_some());
    }
}
