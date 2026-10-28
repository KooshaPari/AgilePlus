//! Compile a downstream-style exhaustive literal and the established zero-arg API.
use agileplus_dashboard::app_state::DashboardStore;
use axum::response::IntoResponse;
use std::collections::HashMap;

#[test]
fn dashboard_store_remains_exhaustively_constructible() {
    let store = DashboardStore {
        features: Vec::new(),
        work_packages: HashMap::new(),
        modules: Vec::new(),
        cycles: Vec::new(),
        cycle_features: HashMap::new(),
        health: Vec::new(),
        projects: Vec::new(),
        active_project_id: None,
        governance_client: None,
        plane_client: None,
        plane_daemon: None,
    };
    assert!(store.features.is_empty());
}

#[tokio::test]
async fn legacy_epics_handler_has_zero_arguments_and_fails_without_source_context() {
    let response = agileplus_dashboard::routes::epics_stories_json()
        .await
        .into_response();
    assert_eq!(
        response.status(),
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
}
