//! axum router factory and HTTP server startup.
//!
//! Route layout:
//!
//! Public (no auth):
//!   GET  /health    — simple health check
//!   GET  /detailed-health — detailed health check (T070)
//!   GET  /info      — API metadata
//!
//! Protected (Bearer token or X-API-Key):
//!   GET  /api/v1/features                           — list features (T066)
//!   POST /api/v1/features                           — create feature (T066)
//!   GET  /api/v1/features/:slug                     — get feature (T066)
//!   PATCH /api/v1/features/:slug                    — update feature (T066)
//!   POST /api/v1/features/:slug/transition          — transition feature state (T066)
//!   GET  /api/v1/features/:slug/work-packages       — list WPs (T067)
//!   POST /api/v1/features/:slug/work-packages       — create WP (T067)
//!   GET  /api/v1/work-packages/:id                  — get WP (T067)
//!   PATCH /api/v1/work-packages/:id                 — update WP (T067)
//!   POST /api/v1/work-packages/:id/transition       — transition WP state (T067)
//!   GET  /api/v1/features/:slug/audit               — audit trail
//!   POST /api/v1/features/:slug/audit/verify        — verify audit chain
//!   GET  /api/v1/features/:slug/governance          — governance contract
//!   POST /api/v1/features/:slug/validate            — run governance validation
//!   GET  /api/v1/events                             — query events (T068)
//!   GET  /api/v1/events/:id                         — single event (T068)
//!   GET  /api/v1/stream                             — SSE real-time events (T069)
//!
//! Traceability: WP11-T065..T070

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::routing::get;
use axum::Json;
use axum::{middleware, Router};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use agileplus_domain::ports::{
    observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort,
};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

use crate::responses::{DetailedHealthResponse, SimpleHealthResponse};
use crate::routes::{audit, cycle, events, features, governance, module, stream, work_packages};
use crate::state::AppState;

/// Build the axum [`Router`] with all routes, middleware, and shared state.
pub fn create_router<S, V, O>(state: AppState<S, V, O>) -> Router
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let token_verifier = Arc::clone(&state.token_verifier);

    // Public routes -- no auth middleware.
    let public = Router::new()
        .route("/health", get(simple_health_handler))
        .route("/detailed-health", get(health_handler::<S, V, O>))
        .route("/info", get(info_handler))
        // HTML dashboard pages (no auth for browser access)
        .route("/modules", get(module::module_tree_page::<S, V, O>))
        .route("/cycles", get(cycle::cycle_kanban_page::<S, V, O>))
        .route("/cycles/{id}", get(cycle::cycle_detail_page::<S, V, O>))
        .with_state(state.clone());

    // Protected routes — all require a valid API key.
    let protected = Router::new()
        // Feature CRUD + transitions
        .nest("/api/v1/features", features::routes::<S, V, O>())
        // Work-package CRUD + transitions
        .nest("/api/v1/work-packages", work_packages::routes::<S, V, O>())
        // Work-package routes nested under features
        .nest(
            "/api/v1/features",
            work_packages::feature_wp_routes::<S, V, O>(),
        )
        // Governance and audit nested under features
        .nest("/api/v1/features", governance::routes::<S, V, O>())
        .nest("/api/v1/features", audit::routes::<S, V, O>())
        // Module and Cycle API routes
        .nest("/api/modules", module::routes::<S, V, O>())
        .nest("/api/cycles", cycle::routes::<S, V, O>())
        // Event query endpoints
        .nest("/api/v1/events", events::routes::<S, V, O>())
        // SSE streaming
        .route("/api/v1/stream", get(stream::stream_events::<S, V, O>))
        .layer(middleware::from_fn_with_state(
            token_verifier,
            crate::middleware::auth::authorize,
        ))
        .with_state(state);

    // Dashboard UI routes (no auth, seeded with dogfood data).
    let dashboard_state = std::sync::Arc::new(tokio::sync::RwLock::new(
        agileplus_dashboard::app_state::DashboardStore::seeded(),
    ));
    let dashboard = agileplus_dashboard::routes::router(dashboard_state);

    Router::new()
        .merge(public)
        .merge(protected)
        .merge(dashboard)
        // NOTE: "templates/static" is relative to the process CWD, which must
        // be the workspace root (where the `templates/` directory lives).
        // A future improvement could use a compile-time or env-based path.
        .nest_service("/static", ServeDir::new("templates/static"))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}

/// `GET /health` — simple health check, no auth required.
async fn simple_health_handler() -> Json<SimpleHealthResponse> {
    Json(SimpleHealthResponse::ok())
}

/// `GET /detailed-health` — aggregated health check, no auth required (T070).
async fn health_handler<S, V, O>(
    axum::extract::State(app): axum::extract::State<AppState<S, V, O>>,
) -> Json<DetailedHealthResponse>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    use std::collections::HashMap;

    // Probe storage with a lightweight call.
    let mut services: HashMap<String, crate::responses::ServiceHealth> = HashMap::new();

    let t0 = Instant::now();
    let sqlite_health = match app.storage.list_all_features().await {
        Ok(_) => crate::responses::ServiceHealth::healthy(t0.elapsed().as_millis() as u64),
        Err(e) => crate::responses::ServiceHealth::unavailable(e.to_string()),
    };
    services.insert("sqlite".to_string(), sqlite_health);

    // --- Env-gated service probes (2 s timeout each) ---
    let probe_timeout = std::time::Duration::from_secs(2);

    // NATS — check NATS_URL, attempt TCP connect
    services.insert(
        "nats".to_string(),
        probe_tcp_env("NATS_URL", probe_timeout).await,
    );

    // Dragonfly / Redis — check DRAGONFLY_URL then REDIS_URL
    services.insert(
        "dragonfly".to_string(),
        probe_tcp_env_multi(&["DRAGONFLY_URL", "REDIS_URL"], probe_timeout).await,
    );

    // Neo4j — check NEO4J_URI, attempt TCP connect to host:port
    services.insert(
        "neo4j".to_string(),
        probe_tcp_env("NEO4J_URI", probe_timeout).await,
    );

    // MinIO/S3 — check S3_ENDPOINT, attempt TCP connect
    services.insert(
        "minio".to_string(),
        probe_tcp_env("S3_ENDPOINT", probe_timeout).await,
    );

    let overall = DetailedHealthResponse::compute_status(&services).to_string();

    Json(DetailedHealthResponse {
        status: overall,
        timestamp: chrono::Utc::now().to_rfc3339(),
        services,
        api: crate::responses::ApiHealth {
            status: "healthy".to_string(),
            uptime_seconds: 0, // uptime tracking requires a startup timestamp in AppState
        },
    })
}

async fn info_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "agileplus-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Probe a single env var: if set, TCP-connect to host:port with timeout.
/// Returns `not_configured` if the env var is absent.
async fn probe_tcp_env(
    env_key: &str,
    timeout: std::time::Duration,
) -> crate::responses::ServiceHealth {
    let url = match std::env::var(env_key) {
        Ok(v) => v,
        Err(_) => return crate::responses::ServiceHealth::not_configured(),
    };
    probe_tcp_url(&url, timeout).await
}

/// Try multiple env var names in order; return the first that is set and probed.
/// If none are set, return `not_configured`.
async fn probe_tcp_env_multi(
    env_keys: &[&str],
    timeout: std::time::Duration,
) -> crate::responses::ServiceHealth {
    for key in env_keys {
        if let Ok(url) = std::env::var(key) {
            return probe_tcp_url(&url, timeout).await;
        }
    }
    crate::responses::ServiceHealth::not_configured()
}

/// Parse `host:port` from a URL string and TCP-connect with the given timeout.
/// Accepts schemes like `http://host:port`, `nats://host:port`, or bare `host:port`.
async fn probe_tcp_url(url: &str, timeout: std::time::Duration) -> crate::responses::ServiceHealth {
    let addr = extract_host_port(url);
    let t0 = Instant::now();
    match tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => crate::responses::ServiceHealth::healthy(t0.elapsed().as_millis() as u64),
        Ok(Err(e)) => crate::responses::ServiceHealth::unavailable(format!("{addr}: {e}")),
        Err(_) => {
            crate::responses::ServiceHealth::unavailable(format!("{addr}: connection timed out"))
        }
    }
}

/// Extract `host:port` from a URL or bare address string.
fn extract_host_port(url: &str) -> String {
    // Strip scheme prefix (e.g. "nats://", "http://")
    let stripped = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .or_else(|| url.strip_prefix("nats://"))
        .or_else(|| url.strip_prefix("bolt://"))
        .or_else(|| url.strip_prefix("bolt+routing://"))
        .unwrap_or(url);
    // Strip trailing path/query
    let host_port = stripped.split('/').next().unwrap_or(stripped);
    // If port is missing, default to 4222 for nats-like schemes, 80 otherwise
    if host_port.contains(':') {
        host_port.to_string()
    } else if url.starts_with("nats://") {
        format!("{host_port}:4222")
    } else {
        format!("{host_port}:80")
    }
}

/// Start the HTTP API server, binding to `addr`.
pub async fn start_api<S, V, O>(addr: SocketAddr, state: AppState<S, V, O>) -> Result<(), BoxError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("HTTP API listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
