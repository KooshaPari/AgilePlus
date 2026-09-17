//! Page route handlers for AgilePlus dashboard.
//!
//! Handlers for main page views (root, home, features, events, settings, hub, health, feature details).

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{Html, Response},
};
use std::collections::HashMap;

use crate::app_state::SharedState;
use crate::templates::{
    DashboardPage, EcosystemProject, EventsPage, FeatureView, FeaturesPage, HomePage, HubPage,
    SettingsPage,
};

use super::features;
use super::helpers::{self};

/// GET /
/// Home page with project summary statistics
pub async fn root(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let total_features = store.features.len();
    let active_features = store
        .features
        .iter()
        .filter(|feature| {
            !matches!(
                feature.state,
                agileplus_domain::domain::state_machine::FeatureState::Shipped
                    | agileplus_domain::domain::state_machine::FeatureState::Retrospected
            )
        })
        .count();
    let shipped_features = store
        .features
        .iter()
        .filter(|feature| {
            matches!(
                feature.state,
                agileplus_domain::domain::state_machine::FeatureState::Shipped
                    | agileplus_domain::domain::state_machine::FeatureState::Retrospected
            )
        })
        .count();
    let projects = helpers::build_project_summaries(&store);

    helpers::render(HomePage {
        total_features,
        active_features,
        shipped_features,
        projects,
    })
}

/// GET /home
/// Alias for root page
pub async fn home(State(state): State<SharedState>) -> Response {
    root(State(state)).await
}

/// GET /dashboard
/// Dashboard page with kanban board
pub async fn dashboard_page(
    State(state): State<SharedState>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let store = state.read().await;
    let filter = helpers::dashboard_filter_from_query(&query);
    let cards = helpers::build_kanban_cards(&store, filter);
    let (projects, active_project) = helpers::load_projects(&store);
    let active_filter = query.get("filter").cloned().unwrap_or_else(|| "all".into());
    helpers::render(DashboardPage {
        kanban_cards: cards,
        health: store.health.clone(),
        projects,
        active_project,
        active_filter,
    })
}

/// GET /features
/// Features list page
pub async fn features_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let features = store
        .features
        .iter()
        .map(FeatureView::from_feature)
        .collect::<Vec<_>>();
    helpers::render(FeaturesPage { features })
}

/// GET /events
/// Events timeline page
pub async fn events_page() -> Response {
    helpers::render(EventsPage {
        events: helpers::sample_events(),
    })
}

/// GET /settings
/// Settings page
pub async fn settings_page() -> Response {
    helpers::render(SettingsPage)
}

/// GET /hub
/// Ecosystem projects hub page
pub async fn hub_page() -> Response {
    let projects = vec![
        EcosystemProject {
            name: "phenodocs",
            tagline: "Ecosystem docs hub",
            stack: "TypeScript · Vue",
            port: Some(4100),
            github: "https://github.com/KooshaPari/phenodocs",
            category: "docs",
        },
        EcosystemProject {
            name: "AgilePlus",
            tagline: "Spec-driven PM platform",
            stack: "Rust · Tauri",
            port: Some(4101),
            github: "https://github.com/KooshaPari/AgilePlus",
            category: "app",
        },
        EcosystemProject {
            name: "heliosApp",
            tagline: "TypeScript runtime app",
            stack: "TypeScript · Bun",
            port: Some(4102),
            github: "https://github.com/KooshaPari/heliosApp",
            category: "app",
        },
        EcosystemProject {
            name: "thegent",
            tagline: "Agent framework",
            stack: "TypeScript · Python",
            port: Some(4103),
            github: "https://github.com/KooshaPari/thegent",
            category: "lib",
        },
        EcosystemProject {
            name: "bifrost-extensions",
            tagline: "LLM gateway extensions",
            stack: "Go",
            port: Some(4104),
            github: "https://github.com/KooshaPari/bifrost-extensions",
            category: "lib",
        },
        EcosystemProject {
            name: "civ",
            tagline: "CI validation",
            stack: "TypeScript",
            port: Some(4105),
            github: "https://github.com/KooshaPari/civ",
            category: "docs",
        },
        EcosystemProject {
            name: "TraceRTM",
            tagline: "Requirements traceability",
            stack: "Python · Go · TS",
            port: Some(4110),
            github: "https://github.com/KooshaPari/trace",
            category: "app",
        },
        EcosystemProject {
            name: "agentapi-plusplus",
            tagline: "Agent HTTP API",
            stack: "Go",
            port: None,
            github: "https://github.com/KooshaPari/agentapi-plusplus",
            category: "api",
        },
        EcosystemProject {
            name: "cliproxyapi-plusplus",
            tagline: "Multi-provider CLI proxy",
            stack: "Go",
            port: None,
            github: "https://github.com/KooshaPari/cliproxyapi-plusplus",
            category: "api",
        },
    ];
    helpers::render(HubPage { projects })
}

/// GET /features/:id
/// Feature detail page (full HTML page)
pub async fn feature_page(State(state): State<SharedState>, Path(id): Path<i64>) -> Response {
    features::feature_detail(State(state), Path(id), HeaderMap::new()).await
}

pub async fn time_footer() -> Html<String> {
    Html(
        chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::{DashboardStore, SharedState, default_health};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tower::util::ServiceExt;

    fn make_state() -> SharedState {
        let store = DashboardStore::seeded();
        Arc::new(RwLock::new(store))
    }

    fn make_empty_state() -> SharedState {
        let store = DashboardStore {
            health: default_health(),
            ..Default::default()
        };
        Arc::new(RwLock::new(store))
    }

    #[tokio::test]
    async fn root_renders_home_page_with_seed_data() {
        let state = make_state();
        let response = root(State(state)).await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains("AgilePlus"));
    }

    #[tokio::test]
    async fn root_counts_features_correctly() {
        let state = make_state();
        let store = state.read().await;
        let total = store.features.len();
        let active = store
            .features
            .iter()
            .filter(|f| {
                !matches!(
                    f.state,
                    agileplus_domain::domain::state_machine::FeatureState::Shipped
                        | agileplus_domain::domain::state_machine::FeatureState::Retrospected
                )
            })
            .count();
        let shipped = store.features.len() - active;
        drop(store);

        let response = root(State(state)).await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains(&total.to_string()));
        assert!(html.contains(&active.to_string()));
        assert!(html.contains(&shipped.to_string()));
    }

    #[tokio::test]
    async fn home_delegates_to_root() {
        let state = make_state();
        let response = home(State(state)).await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains("AgilePlus"));
    }

    #[tokio::test]
    async fn dashboard_page_renders_kanban() {
        let state = make_state();
        let query = HashMap::new();
        let response = dashboard_page(State(state), Query(query)).await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains("kanban-board"));
    }

    #[tokio::test]
    async fn dashboard_page_with_filter_query() {
        let state = make_state();
        let mut query = HashMap::new();
        query.insert("filter".to_string(), "active".to_string());
        let response = dashboard_page(State(state), Query(query)).await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains("kanban-board"));
    }

    #[tokio::test]
    async fn features_page_renders_feature_list() {
        let state = make_state();
        let response = features_page(State(state)).await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains("features") || html.contains("feature"));
    }

    #[tokio::test]
    async fn features_page_with_empty_store() {
        let state = make_empty_state();
        let response = features_page(State(state)).await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        assert_eq!(bytes.len() > 0, true);
    }

    #[tokio::test]
    async fn events_page_renders() {
        let response = events_page().await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains("event"));
    }

    #[tokio::test]
    async fn settings_page_renders() {
        let response = settings_page().await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains("settings") || html.contains("Settings"));
    }

    #[tokio::test]
    async fn hub_page_renders_ecosystem_projects() {
        let response = hub_page().await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains("AgilePlus"));
        assert!(html.contains("phenodocs"));
    }

    #[tokio::test]
    async fn feature_page_delegates_to_feature_detail() {
        let state = make_state();
        let response = feature_page(State(state), Path(1)).await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains("feature") || html.contains("Feature"));
    }

    #[tokio::test]
    async fn feature_page_not_found() {
        let state = make_state();
        let response = feature_page(State(state), Path(99999)).await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let text = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(text.contains("not found") || text.contains("Not found"));
    }

    #[tokio::test]
    async fn time_footer_returns_utc_timestamp() {
        let response = time_footer().await;
        let text = response.0;
        assert!(text.contains("UTC"));
        assert!(text.contains("-"));
    }

    #[tokio::test]
    async fn dashboard_page_via_oneshot_router() {
        let state = make_state();
        let app = crate::routes::router(state);
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/dashboard")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn home_page_via_oneshot_router() {
        let state = make_state();
        let app = crate::routes::router(state);
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/home")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn features_page_via_oneshot_router() {
        let state = make_state();
        let app = crate::routes::router(state);
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/features")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn events_page_via_oneshot_router() {
        let state = make_state();
        let app = crate::routes::router(state);
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/events")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn settings_page_via_oneshot_router() {
        let state = make_state();
        let app = crate::routes::router(state);
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/settings")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn hub_page_via_oneshot_router() {
        let state = make_state();
        let app = crate::routes::router(state);
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/hub")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn feature_detail_api_via_oneshot_router() {
        let state = make_state();
        let app = crate::routes::router(state);
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/api/dashboard/features/1")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn feature_detail_not_found_via_oneshot_router() {
        let state = make_state();
        let app = crate::routes::router(state);
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/api/dashboard/features/99999")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn health_page_via_oneshot_router() {
        let state = make_state();
        let app = crate::routes::router(state);
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/health-page")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn time_endpoint_via_oneshot_router() {
        let state = make_state();
        let app = crate::routes::router(state);
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/api/time")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}
