//! Schema-first PM routes; the operator credential identifies the actor.
use crate::{error::ApiError, state::AppState};
use agileplus_domain::{
    domain::work_item::{AuditEvent, NewWorkItem, WorkItem, WorkItemState},
    error::DomainError,
    ports::{ObservabilityPort, StoragePort, vcs::VcsPort, work_item::WorkItemPort},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;
pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    O: ObservabilityPort + 'static,
{
    Router::new()
        .route("/", post(create::<S, V, O>))
        .route("/{id}/transition", post(transition::<S, V, O>))
        .route("/{id}/audit-events", get(events::<S, V, O>))
}
fn port<S, V, O>(app: &AppState<S, V, O>) -> Result<Arc<dyn WorkItemPort>, ApiError>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    O: ObservabilityPort + 'static,
{
    app.work_items
        .clone()
        .ok_or_else(|| ApiError::Internal("work-item adapter not configured".into()))
}
fn error(e: DomainError) -> ApiError {
    match e {
        DomainError::Validation(m) => ApiError::UnprocessableEntity(m),
        other => other.into(),
    }
}
async fn create<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Json(body): Json<NewWorkItem>,
) -> Result<(StatusCode, Json<WorkItem>), ApiError>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    O: ObservabilityPort + 'static,
{
    let w = port(&app)?
        .create_work_item(body, "api-key")
        .await
        .map_err(error)?;
    Ok((StatusCode::CREATED, Json(w)))
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TransitionRequest {
    state: WorkItemState,
}
async fn transition<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
    Json(body): Json<TransitionRequest>,
) -> Result<Json<WorkItem>, ApiError>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    O: ObservabilityPort + 'static,
{
    Ok(Json(
        port(&app)?
            .transition_work_item(id, body.state, "api-key")
            .await
            .map_err(error)?,
    ))
}
async fn events<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<AuditEvent>>, ApiError>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    O: ObservabilityPort + 'static,
{
    Ok(Json(
        port(&app)?.list_work_item_audit(id).await.map_err(error)?,
    ))
}
