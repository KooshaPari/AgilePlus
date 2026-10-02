//! Authenticated terminal acceptance, distinct from governance-only `/validate`.

use crate::state::AppState;
use agileplus_application::{
    error::AppError,
    use_cases::accept_feature::{AcceptFeatureCommand, accept_feature},
};
use agileplus_domain::ports::vcs::VcsPort;
use agileplus_domain::{
    error::DomainError,
    ports::{ObservabilityPort, StoragePort},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptRequest {
    pub request_id: String,
    pub expected_governance_version: Option<i32>,
}

type HttpError = (StatusCode, Json<Value>);

pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    Router::new().route("/{slug}/accept", post(accept::<S, V, O>))
}

pub async fn accept<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
    Json(request): Json<AcceptRequest>,
) -> Result<Json<agileplus_domain::domain::acceptance::AcceptanceOutcome>, HttpError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    // Mounted under the existing API-key middleware. Actor is server-controlled.
    let port = state.atomic_acceptance.as_ref().map(Arc::clone).ok_or_else(|| (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error":"atomic_acceptance_not_configured"})),
    ))?;
    let feature = state.storage.get_feature_by_slug(&slug).await.map_err(domain_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error":"feature_not_found"}))))?;
    let command = AcceptFeatureCommand {
        request_id: request.request_id,
        feature_id: feature.id,
        actor: "http:api-key".into(),
        expected_governance_version: request.expected_governance_version,
    };
    command.validate().map_err(|e| (
        StatusCode::BAD_REQUEST, Json(json!({"error":e.to_string()})),
    ))?;
    let outcome = accept_feature(port.as_ref(), &command).await.map_err(|error| match error {
        AppError::Domain(error) => domain_error(error),
        AppError::NotFound(_) => (StatusCode::NOT_FOUND, Json(json!({"error":"not_found"}))),
        other => {
            tracing::error!(%other, "atomic acceptance failed");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"acceptance_persistence_failed"})))
        }
    })?;
    Ok(Json(outcome))
}

fn domain_error(error: DomainError) -> HttpError {
    match error {
        DomainError::Validation(message) | DomainError::Conflict(message) => (
            StatusCode::CONFLICT, Json(json!({"error":"acceptance_rejected", "message":message})),
        ),
        DomainError::NotFound(_) => (StatusCode::NOT_FOUND, Json(json!({"error":"not_found"}))),
        DomainError::NotImplemented => (
            StatusCode::NOT_IMPLEMENTED, Json(json!({"error":"atomic_acceptance_not_supported"})),
        ),
        other => {
            tracing::error!(%other, "atomic acceptance persistence failed");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"acceptance_persistence_failed"})))
        }
    }
}
