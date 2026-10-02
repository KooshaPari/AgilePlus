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
use axum::{Router, middleware, routing::get};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

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
        .layer(CorsLayer::permissive())
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
