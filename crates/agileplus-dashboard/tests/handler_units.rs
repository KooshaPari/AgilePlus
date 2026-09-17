// SPDX-License-Identifier: MIT OR Apache-2.0
//! Handler- and template-level unit tests for `agileplus-dashboard`.
//!
//! These tests call individual axum handlers directly (no router) and render
//! every Askama template with representative data. They complement
//! `route_matrix.rs`, which exercises the wiring through the router.
//!
//! Traceability: WP12 (T071–T077)

use std::collections::HashMap;
use std::sync::Arc;

use agileplus_dashboard::app_state::{DashboardStore, ServiceHealth as StoreServiceHealth, SharedState, default_health};
use agileplus_dashboard::routes::{
    agents, dashboard as dash_routes, evidence, features, health as health_routes, pages, settings,
};
use agileplus_dashboard::templates::*;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::project::Project;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use chrono::Utc;
use tokio::sync::RwLock;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn state_with(store: DashboardStore) -> SharedState {
    Arc::new(RwLock::new(store))
}

fn store() -> DashboardStore {
    DashboardStore {
        health: default_health(),
        ..Default::default()
    }
}

fn feature(id: i64, st: FeatureState, module_id: Option<i64>) -> Feature {
    let mut f = Feature::new(&format!("feat-{id}"), &format!("Feat {id}"), [0; 32], None);
    f.id = id;
    f.state = st;
    f.module_id = module_id;
    f.labels = vec!["platform".into()];
    f
}

fn wp(id: i64, feature_id: i64, st: WpState) -> WorkPackage {
    let mut w = WorkPackage::new(feature_id, &format!("WP-{id}"), 1, "criteria");
    w.id = id;
    w.state = st;
    w.agent_id = Some("claude".into());
    w
}

async fn text(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    String::from_utf8(bytes.to_vec()).expect("utf8")
}

fn sample_service() -> StoreServiceHealth {
    StoreServiceHealth {
        name: "NATS".into(),
        healthy: true,
        degraded: false,
        latency_ms: Some(2),
        last_check: Utc::now(),
    }
}

// ── pages handlers ───────────────────────────────────────────────────────────

#[tokio::test]
async fn pages_root_counts_active_and_shipped() {
    let mut store = store();
    store.features = vec![
        feature(1, FeatureState::Implementing, None),
        feature(2, FeatureState::Shipped, None),
        feature(3, FeatureState::Retrospected, None),
    ];
    let html = text(pages::root(State(state_with(store))).await).await;
    assert!(html.contains("AgilePlus"));
    assert!(html.contains('1')); // one active feature
}

#[tokio::test]
async fn pages_home_delegates_to_root() {
    let html = text(pages::home(State(state_with(store()))).await).await;
    assert!(html.contains("AgilePlus"));
}

#[tokio::test]
async fn pages_events_renders_sample_events() {
    let html = text(pages::events_page().await).await;
    assert!(html.contains("Dashboard booted"));
}

#[tokio::test]
async fn pages_settings_renders() {
    let html = text(pages::settings_page().await).await;
    assert!(!html.is_empty());
}

#[tokio::test]
async fn pages_hub_includes_all_nine_projects() {
    let html = text(pages::hub_page().await).await;
    for name in [
        "phenodocs",
        "AgilePlus",
        "heliosApp",
        "thegent",
        "bifrost-extensions",
        "civ",
        "TraceRTM",
        "agentapi-plusplus",
        "cliproxyapi-plusplus",
    ] {
        assert!(html.contains(name), "hub missing {name}");
    }
}

#[tokio::test]
async fn pages_feature_page_not_found() {
    let response = pages::feature_page(State(state_with(store())), Path(5)).await;
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn pages_dashboard_accepts_filter() {
    let mut query = HashMap::new();
    query.insert("filter".to_string(), "active".to_string());
    let html = text(pages::dashboard_page(State(state_with(store())), Query(query)).await).await;
    assert!(html.contains("kanban-board"));
}

#[tokio::test]
async fn pages_time_footer_formats_utc() {
    let html = pages::time_footer().await;
    assert!(html.0.contains("UTC"));
}

// ── dashboard handlers ───────────────────────────────────────────────────────

#[tokio::test]
async fn dash_wp_list_returns_partial_with_wp() {
    let mut store = store();
    store.work_packages.insert(1, vec![wp(7, 1, WpState::Doing)]);
    let html = text(dash_routes::wp_list(State(state_with(store)), Path(1)).await).await;
    assert!(html.contains("WP-7"));
    assert!(html.contains("wp-list-1"));
}

#[tokio::test]
async fn dash_wp_list_unknown_feature_empty() {
    let html = text(dash_routes::wp_list(State(state_with(store())), Path(9)).await).await;
    assert!(html.contains("No work packages"));
}

#[tokio::test]
async fn dash_health_panel_lists_services() {
    let html = text(dash_routes::health_panel(State(state_with(store()))).await).await;
    assert!(html.contains("NATS"));
}

#[tokio::test]
async fn dash_event_timeline_renders_empty() {
    let html = text(dash_routes::event_timeline(State(state_with(store()))).await).await;
    assert!(html.contains("event-timeline"));
}

#[tokio::test]
async fn dash_agent_activity_lists_placeholder_agents() {
    let html = text(dash_routes::agent_activity(State(state_with(store()))).await).await;
    assert!(html.contains("spec-agent"));
    assert!(html.contains("impl-agent"));
}

#[tokio::test]
async fn dash_project_switcher_lists_projects() {
    let mut store = store();
    let mut project = Project::new("Alpha", "alpha").unwrap();
    project.id = 1;
    store.projects.push(project);
    store.active_project_id = Some(1);
    let html = text(dash_routes::project_switcher(State(state_with(store))).await).await;
    assert!(html.contains("Alpha"));
}

#[tokio::test]
async fn dash_all_work_packages_json_flat_map() {
    let mut store = store();
    store.work_packages.insert(1, vec![wp(1, 1, WpState::Done)]);
    store.work_packages.insert(2, vec![wp(2, 2, WpState::Review)]);
    let value = dash_routes::all_work_packages_json(State(state_with(store)))
        .await
        .into_response();
    let bytes = axum::body::to_bytes(value.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["count"], 2);
}

#[tokio::test]
async fn dash_time_footer_has_utc() {
    let html = dash_routes::time_footer().await;
    assert!(html.0.contains("UTC"));
}

// ── features handlers ────────────────────────────────────────────────────────

#[tokio::test]
async fn features_detail_renders_with_work_packages() {
    let mut store = store();
    store.features = vec![feature(1, FeatureState::Implementing, None)];
    store.work_packages.insert(1, vec![wp(3, 1, WpState::Done)]);
    let html = text(features::feature_detail(State(state_with(store)), Path(1), axum::http::HeaderMap::new()).await).await;
    assert!(html.contains("Feat 1"));
    assert!(html.contains("WP-3"));
}

#[tokio::test]
async fn features_detail_not_found() {
    let response = features::feature_detail(State(state_with(store())), Path(1), axum::http::HeaderMap::new()).await;
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn features_events_not_found() {
    let response = features::feature_events(State(state_with(store())), Path(1)).await;
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn features_media_not_found() {
    let response = features::feature_media(State(state_with(store())), Path(1)).await;
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn features_media_contains_gallery_wrapper() {
    let mut store = store();
    store.features = vec![feature(1, FeatureState::Implementing, None)];
    let html = text(features::feature_media(State(state_with(store)), Path(1)).await).await;
    assert!(html.contains("media-gallery"));
}

// ── evidence handlers ────────────────────────────────────────────────────────

#[tokio::test]
async fn evidence_content_rejects_path_traversal() {
    let response = evidence::evidence_content(
        State(state_with(store())),
        Path((1_i64, "..".to_string())),
    )
    .await;
    let html = text(response).await;
    assert!(html.contains("Forbidden"));
}

#[tokio::test]
async fn evidence_content_missing_artifact_stub() {
    let html = text(
        evidence::evidence_content(
            State(state_with(store())),
            Path((1_i64, "missing.md".to_string())),
        )
        .await,
    )
    .await;
    assert!(html.contains("No artifact found"));
}

#[tokio::test]
async fn evidence_preview_missing_artifact() {
    let html = text(
        evidence::evidence_preview(
            State(state_with(store())),
            Path((1_i64, "missing.md".to_string())),
        )
        .await,
    )
    .await;
    assert!(html.contains("No preview"));
}

#[tokio::test]
async fn evidence_list_empty_renders() {
    let html = text(
        evidence::feature_evidence_list(State(state_with(store())), Path("1".to_string())).await,
    )
    .await;
    assert!(!html.is_empty());
}

#[tokio::test]
async fn evidence_json_empty_artifacts() {
    let response = evidence::feature_evidence_json(State(state_with(store())), Path("1".to_string()))
        .await
        .into_response();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["feature_id"], "1");
    assert!(v["artifacts"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn evidence_generate_reports_missing_script() {
    let response = evidence::feature_evidence_generate(
        State(state_with(store())),
        Path("1".to_string()),
    )
    .await
    .into_response();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["status"], "error");
}

#[test]
fn evidence_gallery_json_serializes() {
    let gallery = evidence::EvidenceGalleryJson {
        feature_id: "7".into(),
        artifacts: vec![evidence::EvidenceArtifactJson {
            id: "bundle-7".into(),
            type_: "generated_bundle".into(),
            title: "Bundle".into(),
            path: "/x/bundle.json".into(),
            url: "/api/evidence/7/bundle-7/preview".into(),
            created_at: "2026-01-01".into(),
        }],
        generated_at: Some("2026-01-01".into()),
    };
    let v = serde_json::to_value(&gallery).unwrap();
    assert_eq!(v["feature_id"], "7");
    assert_eq!(v["artifacts"][0]["type_"], "generated_bundle");
}

#[test]
fn evidence_artifact_json_roundtrips() {
    let artifact = evidence::EvidenceArtifactJson {
        id: "a".into(),
        type_: "t".into(),
        title: "T".into(),
        path: "p".into(),
        url: "u".into(),
        created_at: "c".into(),
    };
    let s = serde_json::to_string(&artifact).unwrap();
    let back: evidence::EvidenceArtifactJson = serde_json::from_str(&s).unwrap();
    assert_eq!(back.id, "a");
    assert_eq!(back.type_, "t");
}

// ── health handlers ──────────────────────────────────────────────────────────

#[tokio::test]
async fn health_page_counts_services() {
    let mut store = store();
    store.health = vec![
        StoreServiceHealth { healthy: true, degraded: false, ..sample_service() },
        StoreServiceHealth { healthy: false, degraded: true, ..sample_service() },
        StoreServiceHealth { healthy: false, degraded: false, ..sample_service() },
    ];
    let html = text(health_routes::health_page(State(state_with(store))).await).await;
    assert!(!html.is_empty());
}

#[tokio::test]
async fn health_json_returns_all_healthy_flag() {
    let response = health_routes::health_json(State(state_with(store())))
        .await
        .into_response();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(v["services"].is_array());
    assert!(v["all_healthy"].is_boolean());
}

#[test]
fn health_status_serializes() {
    let status = health_routes::HealthStatus {
        services: vec![health_routes::ServiceHealthJson {
            name: "NATS".into(),
            healthy: true,
            degraded: false,
            latency_ms: Some(1),
            last_check: "2026-01-01 00:00:00 UTC".into(),
        }],
        timestamp: "2026-01-01T00:00:00Z".into(),
        all_healthy: true,
    };
    let v = serde_json::to_value(&status).unwrap();
    assert_eq!(v["services"][0]["name"], "NATS");
    assert_eq!(v["all_healthy"], true);
}

#[test]
fn service_toggle_body_defaults_none() {
    let body: health_routes::ServiceToggleBody = serde_json::from_str("{}").unwrap();
    assert!(body.enabled.is_none());
}

#[test]
fn service_config_form_deserializes() {
    let form: health_routes::ServiceConfigForm =
        serde_json::from_str(r#"{"endpoint_url":"http://x","timeout_ms":10,"max_retries":2}"#).unwrap();
    assert_eq!(form.endpoint_url.as_deref(), Some("http://x"));
    assert_eq!(form.timeout_ms, Some(10));
    assert_eq!(form.max_retries, Some(2));
}

// ── agents handlers ──────────────────────────────────────────────────────────

#[tokio::test]
async fn agents_activity_partial_from_processes() {
    let html = text(agents::agent_activity(State(state_with(store()))).await).await;
    assert!(html.contains("agent-activity"));
}

#[tokio::test]
async fn agents_json_has_shape() {
    let response = agents::agents_json(State(state_with(store()))).await.into_response();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(v["agents"].is_array());
    assert!(v["count"].is_number());
}

#[tokio::test]
async fn agents_test_connection_local_true() {
    let response = agents::test_agent_connection(axum::Form(agents::AgentTestConnectionForm {
        provider: "local".into(),
    }))
    .await
    .into_response();
    let html = text(response).await;
    assert!(html.contains("no external credentials"));
}

#[tokio::test]
async fn agents_test_connection_unknown_false() {
    let response = agents::test_agent_connection(axum::Form(agents::AgentTestConnectionForm {
        provider: "bogus".into(),
    }))
    .await
    .into_response();
    let html = text(response).await;
    assert!(html.contains("Unknown provider"));
    assert!(html.contains("text-red-400"));
}

#[test]
fn agent_info_serializes() {
    let info = agents::AgentInfo {
        name: "claude".into(),
        status: "running".into(),
        current_task: "WP1".into(),
        pid: Some(42),
        started_at: Some("5m".into()),
        worktree: "/x".into(),
        uptime: "running for 5m".into(),
    };
    let v = serde_json::to_value(&info).unwrap();
    assert_eq!(v["name"], "claude");
    assert_eq!(v["pid"], 42);
}

#[test]
fn agent_settings_form_deserializes() {
    let form: agents::AgentSettingsForm = serde_json::from_str(
        r#"{"pool_size":4,"retry_budget":2,"dispatch_mode":"balanced","default_provider":"claude"}"#,
    )
    .unwrap();
    assert_eq!(form.pool_size, 4);
    assert_eq!(form.dispatch_mode, "balanced");
}

// ── settings handlers ────────────────────────────────────────────────────────

#[tokio::test]
async fn settings_test_service_connection_valid() {
    let response = settings::test_service_connection(axum::Form(settings::SingleServiceTestForm {
        name: "NATS".into(),
        endpoint_url: "http://localhost:4222".into(),
    }))
    .await;
    let html = text(response).await;
    assert!(html.contains("successful"));
}

#[tokio::test]
async fn settings_test_service_connection_invalid() {
    let response = settings::test_service_connection(axum::Form(settings::SingleServiceTestForm {
        name: "NATS".into(),
        endpoint_url: "nope".into(),
    }))
    .await;
    let html = text(response).await;
    assert!(html.contains("Invalid endpoint"));
}

#[tokio::test]
async fn settings_test_plane_connection_invalid() {
    let response = settings::test_plane_connection(axum::Form(settings::PlaneSettingsForm {
        api_url: "ftp://x".into(),
        api_key: "k".into(),
        workspace_slug: "w".into(),
        project_slug: "p".into(),
    }))
    .await;
    let html = text(response).await;
    assert!(html.contains("incomplete") || html.contains("invalid"));
}

#[tokio::test]
async fn settings_services_page_renders() {
    let html = text(settings::services_settings_page(State(state_with(store()))).await).await;
    assert!(!html.is_empty());
}

#[tokio::test]
async fn settings_plane_page_renders() {
    let html = text(settings::plane_settings_page(State(state_with(store()))).await).await;
    assert!(html.contains("Plane"));
}

#[test]
fn plane_settings_form_deserializes() {
    let form: settings::PlaneSettingsForm = serde_json::from_str(
        r#"{"api_url":"https://p","api_key":"k","workspace_slug":"w","project_slug":"p"}"#,
    )
    .unwrap();
    assert_eq!(form.api_url, "https://p");
    assert_eq!(form.workspace_slug, "w");
}

#[test]
fn dashboard_settings_form_deserializes() {
    let form: settings::DashboardSettingsForm = serde_json::from_str(
        r#"{"theme":"dark","log_level":"info","data_directory":"/data"}"#,
    )
    .unwrap();
    assert_eq!(form.theme, "dark");
    assert_eq!(form.data_directory, "/data");
}

#[test]
fn service_config_default_enabled_is_true() {
    assert!(settings::default_service_enabled());
}

#[test]
fn plane_config_serialization_hides_api_key() {
    let config = settings::PlaneConfig {
        api_url: "https://plane".into(),
        api_key_ref: "plane_api_key".into(),
        api_key: Some("super-secret".into()),
        workspace_slug: "w".into(),
        project_slug: "p".into(),
    };
    let serialized = toml::to_string(&config).unwrap();
    assert!(!serialized.contains("super-secret"));
    assert!(serialized.contains("api_key_ref"));
}

// ── Template rendering ───────────────────────────────────────────────────────

fn project_view() -> ProjectView {
    ProjectView {
        id: 1,
        slug: "p".into(),
        name: "P".into(),
        description: "d".into(),
    }
}

#[test]
fn template_home_renders() {
    let tpl = HomePage {
        total_features: 3,
        active_features: 1,
        shipped_features: 2,
        projects: vec![ProjectSummaryView {
            project: project_view(),
            feature_count: 3,
            active_count: 1,
            shipped_count: 2,
        }],
    };
    assert!(tpl.render().unwrap().contains("AgilePlus"));
}

#[test]
fn template_features_page_renders() {
    let tpl = FeaturesPage {
        features: vec![FeatureView {
            id: 1,
            slug: "f".into(),
            title: "Feature X".into(),
            state: "created".into(),
            labels: vec![],
        }],
    };
    assert!(tpl.render().unwrap().contains("Feature X"));
}

#[test]
fn template_events_page_renders() {
    let tpl = EventsPage {
        events: vec![EventView {
            id: "e1".into(),
            kind: "system".into(),
            description: "hello".into(),
            timestamp: "now".into(),
            agent_name: None,
            agent_link: None,
            wp_id: None,
            wp_link: None,
            commit_sha: None,
            commit_link: None,
            ci_run_id: None,
            ci_run_link: None,
        }],
    };
    assert!(tpl.render().unwrap().contains("hello"));
}

#[test]
fn template_settings_page_renders() {
    assert!(!SettingsPage.render().unwrap().is_empty());
}

#[test]
fn template_hub_page_renders() {
    let tpl = HubPage {
        projects: vec![EcosystemProject {
            name: "x",
            tagline: "t",
            stack: "s",
            port: Some(1),
            github: "https://github.com/x",
            category: "app",
        }],
    };
    assert!(tpl.render().unwrap().contains('x'));
}

#[test]
fn template_health_page_renders() {
    let tpl = HealthPage {
        services: vec![ServiceHealthView {
            name: "NATS".into(),
            healthy: true,
            degraded: false,
            latency_ms: Some(1),
            last_check_str: "now".into(),
        }],
        healthy_count: 1,
        degraded_count: 0,
        unhealthy_count: 0,
    };
    assert!(tpl.render().unwrap().contains("NATS"));
}

#[test]
fn template_services_settings_page_renders() {
    let tpl = ServicesSettingsPage {
        services: vec![sample_service()],
        configs: vec![ServiceConfigView {
            name: "n".into(),
            endpoint_url: "http://x".into(),
        }],
    };
    assert!(tpl.render().unwrap().contains("NATS"));
}

#[test]
fn template_agent_settings_page_renders() {
    let tpl = AgentSettingsPage {
        agent_pool_size: 6,
        retry_budget: 3,
        dispatch_mode: "balanced".into(),
        default_provider: "claude".into(),
    };
    let html = tpl.render().unwrap();
    assert!(html.contains("balanced"));
    assert!(html.contains("claude"));
}

#[test]
fn template_plane_settings_page_renders() {
    let tpl = PlaneSettingsPage {
        workspace_name: "ws".into(),
        workspace_slug: "ws".into(),
        project_slug: "p".into(),
        plane_api_url: "https://plane".into(),
        plane_web_url: "https://plane".into(),
        plane_api_url_set: true,
        plane_web_url_set: true,
        plane_api_key_hint: "a••••••z".into(),
        plane_api_key_set: true,
        sync_enabled: true,
        sync_mode: "One-way".into(),
        connected: true,
        connection_status: "Connected".into(),
        connection_status_configured: true,
        plane_service_healthy: true,
        plane_api_latency_ms: Some(12),
        plane_health_endpoints: vec![PlaneHealthEndpointView {
            name: "Plane API".into(),
            healthy: true,
            degraded: false,
            latency_ms: Some(12),
            last_check_utc: "now".into(),
        }],
        mapped_features_coverage: "1/1 (100%)".into(),
        mapped_work_packages_coverage: "0/0 (0%)".into(),
        mapped_features: 1,
        mapped_work_packages: 0,
        config_warnings: vec!["warn".into()],
    };
    let html = tpl.render().unwrap();
    assert!(html.contains("Plane"));
}

#[test]
fn template_kanban_partial_renders() {
    let mut cards: HashMap<String, Vec<FeatureView>> = HashMap::new();
    cards.insert(
        "created".into(),
        vec![FeatureView {
            id: 1,
            slug: "f".into(),
            title: "Card".into(),
            state: "created".into(),
            labels: vec![],
        }],
    );
    assert!(KanbanPartial { cards }.render().unwrap().contains("Card"));
}

#[test]
fn template_evidence_partial_renders() {
    let tpl = FeatureEvidencePartial {
        evidence_bundles: vec![sample_bundle()],
    };
    assert!(tpl.render().unwrap().contains("bundle-1"));
}

#[test]
fn template_media_partial_renders() {
    let tpl = FeatureMediaPartial {
        media_assets: vec![MediaAssetView {
            id: "m1".into(),
            source: "s".into(),
            name: "shot.png".into(),
            kind: "image".into(),
            mime: "image/png".into(),
            url_or_path: "/a/shot.png".into(),
            size_bytes: 1,
            uploaded_at: "now".into(),
        }],
    };
    assert!(tpl.render().unwrap().contains("shot.png"));
}

#[test]
fn template_reports_partial_renders() {
    let tpl = FeatureReportsPartial {
        reports: vec![ReportArtifactView {
            id: "r1".into(),
            name: "Coverage".into(),
            source: "coverage-engine".into(),
            status: "completed".into(),
            generated_at: "now".into(),
            rule_count: 5,
            satisfied_count: 2,
            compliant: true,
        }],
    };
    assert!(tpl.render().unwrap().contains("Coverage"));
}

#[test]
fn template_health_panel_renders() {
    let tpl = HealthPanelPartial {
        services: vec![sample_service()],
    };
    assert!(tpl.render().unwrap().contains("NATS"));
}

#[test]
fn template_event_timeline_renders() {
    let tpl = EventTimelinePartial {
        feature_id: 1,
        events: vec![],
    };
    assert!(tpl.render().unwrap().contains("event-timeline"));
}

#[test]
fn template_agent_activity_renders() {
    let tpl = AgentActivityPartial {
        agents: vec![AgentView {
            name: "claude".into(),
            status: "running".into(),
            current_task: "t".into(),
            last_action: "now".into(),
            pid: Some(1),
            started_at: Some("1m".into()),
            worktree: "/x".into(),
            worktree_label: "x".into(),
            is_live: true,
        }],
    };
    assert!(tpl.render().unwrap().contains("claude"));
}

#[test]
fn template_toast_renders_message() {
    let tpl = ToastPartial {
        message: "Saved".into(),
        success: true,
    };
    assert!(tpl.render().unwrap().contains("Saved"));
}

#[test]
fn template_project_switcher_renders() {
    let tpl = ProjectSwitcherPartial {
        projects: vec![project_view()],
        active_id: Some(1),
    };
    assert!(tpl.render().unwrap().contains('P'));
}

#[test]
fn template_wp_list_renders_progress() {
    let tpl = WpListPartial {
        feature_id: 1,
        workpackages: vec![WpView {
            id: 1,
            title: "WP-1".into(),
            state: "doing".into(),
            agent: "claude".into(),
            progress: 30,
            task_count: 2,
            agent_id: Some("claude".into()),
            pr_url: None,
            head_commit: None,
        }],
    };
    let html = tpl.render().unwrap();
    assert!(html.contains("WP-1"));
    assert!(html.contains("30%"));
}

#[test]
fn template_dashboard_page_renders() {
    let tpl = DashboardPage {
        kanban_cards: HashMap::new(),
        health: vec![sample_service()],
        projects: vec![project_view()],
        active_project: Some(project_view()),
        active_filter: "all".into(),
    };
    let html = tpl.render().unwrap();
    assert!(html.contains("kanban-board"));
    assert!(html.contains("NATS"));
}

fn sample_bundle() -> EvidenceBundleView {
    EvidenceBundleView {
        id: "bundle-1".into(),
        fr_id: "FR-1".into(),
        evidence_type: "feature_summary".into(),
        wp_id: "dashboard".into(),
        wp_title: "Summary".into(),
        artifact_path: "/a.md".into(),
        created_at: "now".into(),
        artifact_ext: "md".into(),
        status: "available".into(),
        content_preview: Some("preview".into()),
        is_text_artifact: true,
        is_image_artifact: false,
        download_url: "/x".into(),
        test_passed: Some(true),
        tests_passed_count: 1,
        tests_failed_count: 0,
        test_summary: Some("ok".into()),
        commit_count: 0,
        pr_count: 0,
        ci_links: vec![],
        git_commits: vec![],
        pr_links: vec![],
    }
}
