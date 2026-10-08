//! Router composition. Terminal acceptance is distinct from governance preflight.

use super::handlers::info_handler;
use super::health::{health_handler, simple_health_handler};
use crate::routes::{
    acceptance, audit, branch, cycle, epics, events, features, governance, module, projects,
    stories, stream, users, work_packages, worktree,
};
use crate::state::AppState;
use agileplus_domain::ports::vcs::VcsPort;
use agileplus_domain::ports::{ContentStoragePort, ObservabilityPort, StoragePort};
use axum::{
    http::{header, HeaderValue, Method},
    middleware, routing::get, Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    cors::{AllowOrigin, CorsLayer}, services::ServeDir, trace::TraceLayer,
};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub fn create_router<S, V, O>(state: AppState<S, V, O>) -> Router
where
    S: StoragePort + ContentStoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let credentials = Arc::clone(&state.credentials);
    let public = Router::new()
        .route("/health", get(simple_health_handler))
        .route("/detailed-health", get(health_handler::<S, V, O>))
        .route("/info", get(info_handler))
        .route("/modules", get(module::module_tree_page::<S, V, O>))
        .route("/cycles", get(cycle::cycle_kanban_page::<S, V, O>))
        .route("/cycles/{id}", get(cycle::cycle_detail_page::<S, V, O>))
        .with_state(state.clone());
    let protected = Router::new()
        .nest("/api/v1/features", features::routes::<S, V, O>())
        .nest("/api/v1/work-packages", work_packages::routes::<S, V, O>())
        .nest(
            "/api/v1/features",
            work_packages::feature_wp_routes::<S, V, O>(),
        )
        .nest("/api/v1/features", governance::routes::<S, V, O>())
        .nest("/api/v1/features", acceptance::routes::<S, V, O>())
        .nest("/api/v1/features", audit::routes::<S, V, O>())
        .nest("/api/modules", module::routes::<S, V, O>())
        .nest("/api/cycles", cycle::routes::<S, V, O>())
        .nest("/api/v1/branches", branch::routes::<S, V, O>())
        .nest("/api/v1/worktrees", worktree::routes::<S, V, O>())
        .nest("/api/v1/events", events::routes::<S, V, O>())
        .route("/api/v1/stream", get(stream::stream_events::<S, V, O>))
        .nest("/api/v1/projects", projects::routes::<S, V, O>())
        .nest("/api/v1/epics", epics::routes::<S, V, O>())
        .nest("/api/v1/stories", stories::routes::<S, V, O>())
        .nest("/api/v1/users", users::routes::<S, V, O>())
        .layer(middleware::from_fn_with_state(
            credentials,
            crate::middleware::auth::validate_api_key,
        ))
        .with_state(state);
    Router::new()
        .merge(public)
        .merge(protected)
        .nest_service("/static", ServeDir::new("templates/static"))
        .layer(TraceLayer::new_for_http())
        .layer(configured_cors())
}

pub async fn start_api<S, V, O>(addr: SocketAddr, state: AppState<S, V, O>) -> Result<(), BoxError>
where
    S: StoragePort + ContentStoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "HTTP API listening");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Default deny for cross-origin browser access. Tailnet routing is not a
/// substitute for browser authorization, and operator API credentials must
/// never be exposed to arbitrary web origins.
fn configured_cors() -> CorsLayer {
    let configured = std::env::var("AGILEPLUS_ALLOWED_ORIGINS").ok();
    let origins = parse_allowed_origins(configured.as_deref());
    if origins.is_empty() {
        return CorsLayer::new();
    }
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::HeaderName::from_static("x-api-key"),
            header::HeaderName::from_static("x-request-id"),
        ])
}

fn parse_allowed_origins(configured: Option<&str>) -> Vec<HeaderValue> {
    configured
        .unwrap_or_default()
        .split(',')
        .filter_map(|candidate| {
            let candidate = candidate.trim();
            // Origin is scheme + authority, not a full URL. Only explicit
            // HTTPS origins or local HTTP development origins are supported.
            let scheme_ok = candidate.starts_with("https://")
                || candidate.starts_with("http://localhost:")
                || candidate.starts_with("http://127.0.0.1:");
            let authority = candidate
                .split_once("://")
                .map(|(_, authority)| authority)
                .unwrap_or_default();
            if !scheme_ok
                || authority.is_empty()
                || authority.contains(['/', '?', '#', '@', ' '])
                || candidate == "*"
                || candidate == "null"
            {
                if !candidate.is_empty() {
                    tracing::warn!(origin = %candidate, "ignoring invalid AgilePlus CORS origin");
                }
                return None;
            }
            HeaderValue::from_str(candidate).ok()
        })
        .collect()
}

#[cfg(test)]
mod cors_policy_tests {
    use super::*;

    #[test]
    fn cross_origin_is_disabled_without_explicit_configuration() {
        assert!(parse_allowed_origins(None).is_empty());
        assert!(parse_allowed_origins(Some("")).is_empty());
    }

    #[test]
    fn wildcard_http_remote_and_path_bearing_origins_are_rejected() {
        let origins = parse_allowed_origins(Some(
            "*,null,http://public.example,https://dashboard.example/path,https://bad.example?x=1",
        ));
        assert!(origins.is_empty());
    }

    #[test]
    fn explicit_https_and_local_development_origins_are_accepted() {
        let origins = parse_allowed_origins(Some(
            "https://agileplus.pheno.studio, http://localhost:5173",
        ));
        assert_eq!(origins.len(), 2);
        assert_eq!(origins[0], "https://agileplus.pheno.studio");
        assert_eq!(origins[1], "http://localhost:5173");
    }
}
