use agileplus_dashboard::app_state::DashboardStore;
use agileplus_dashboard::live_store;
use agileplus_dashboard::resolve_dashboard_host;
use agileplus_dashboard::routes::router;
use axum::{Extension, middleware, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let runtime = live_store::initialize_from_environment();
    if let Some(diagnostic) = runtime.source.diagnostic.as_deref() {
        tracing::warn!(%diagnostic, "dashboard started without a live project data source");
    } else {
        tracing::info!(
            mode = ?runtime.source.mode,
            project = ?runtime.source.project_name,
            features = runtime.source.feature_count,
            work_packages = runtime.source.work_package_count,
            "dashboard data source initialized"
        );
    }
    let source = Arc::new(runtime.source);
    let mut store: DashboardStore = runtime.store;

    // Optional auxiliary clients do not replace the canonical project source.
    store = match agileplus_governance::GovernanceClient::with_defaults().await {
        Ok(client) => store.with_governance(client),
        Err(error) => {
            tracing::warn!(%error, "GovernanceClient::with_defaults failed; governance status will return not_initialized");
            store
        }
    };

    let plane_args = (
        std::env::var("PLANE_BASE_URL").unwrap_or_else(|_| "https://api.plane.so".to_string()),
        std::env::var("PLANE_API_KEY").ok(),
        std::env::var("PLANE_WORKSPACE").ok(),
        std::env::var("PLANE_PROJECT").ok(),
    );
    store = match plane_args {
        (base_url, Some(api_key), Some(workspace), Some(project)) => {
            store.with_plane(agileplus_plane::PlaneClient::new(
                base_url,
                api_key,
                workspace,
                project,
            ))
        }
        _ => {
            tracing::info!(
                "PLANE_API_KEY/PLANE_WORKSPACE/PLANE_PROJECT not set; Plane projection remains unavailable"
            );
            store
        }
    };

    let state = Arc::new(tokio::sync::RwLock::new(store));
    let app = router(state)
        .route(
            "/api/dashboard/source.json",
            get(live_store::source_status),
        )
        .layer(Extension(source.clone()))
        .layer(middleware::from_fn_with_state(
            source,
            live_store::mutation_guard,
        ));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(8770);
    let dashboard_host = std::env::var("DASHBOARD_HOST");
    let host = resolve_dashboard_host(dashboard_host.ok().as_deref());
    let address = SocketAddr::from((host, port));
    tracing::info!(%address, "agileplus-dashboard listening");

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
