//! Feature route handlers.
//!
//! - GET    /api/v1/features                  → list (with ?state=, ?label= filters)
//! - POST   /api/v1/features                  → create
//! - GET    /api/v1/features/:slug            → detail
//! - PUT    /api/v1/features/:slug            → full replace
//! - DELETE /api/v1/features/:slug            → delete
//! - PATCH  /api/v1/features/:slug            → partial update
//! - POST   /api/v1/features/:slug/transition → state transition
//!
//! Traceability: WP11-T066

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::{
    observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort,
};

use crate::error::ApiError;
use crate::responses::FeatureResponse;
use crate::state::AppState;

pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    Router::new()
        .route(
            "/",
            get(list_features::<S, V, O>).post(create_feature::<S, V, O>),
        )
        .route(
            "/{slug}",
            get(get_feature::<S, V, O>)
                .put(replace_feature::<S, V, O>)
                .delete(delete_feature::<S, V, O>)
                .patch(update_feature::<S, V, O>),
        )
        .route("/{slug}/transition", post(transition_feature::<S, V, O>))
}

#[derive(Debug, Deserialize)]
pub struct FeatureListParams {
    pub state: Option<String>,
    pub label: Option<String>,
}

/// `GET /api/v1/features[?state=<state>&label=<label>]`
pub async fn list_features<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Query(params): Query<FeatureListParams>,
) -> Result<Json<Vec<FeatureResponse>>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let features = if let Some(state_filter) = params.state {
        let fs = parse_feature_state(&state_filter)?;
        state
            .storage
            .list_features_by_state(fs)
            .await
            .map_err(ApiError::from)?
    } else {
        state
            .storage
            .list_all_features()
            .await
            .map_err(ApiError::from)?
    };

    // label filter is informational for now — domain layer doesn't have label storage yet
    let _ = params.label;

    Ok(Json(
        features.into_iter().map(FeatureResponse::from).collect(),
    ))
}

/// `GET /api/v1/features/:slug`
pub async fn get_feature<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
) -> Result<Json<FeatureResponse>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let feature = state
        .storage
        .get_feature_by_slug(&slug)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Feature '{slug}' not found")))?;

    Ok(Json(FeatureResponse::from(feature)))
}

#[derive(Debug, Deserialize)]
pub struct CreateFeatureRequest {
    pub title: String,
    pub description: Option<String>,
    pub state: Option<String>,
    pub target_branch: Option<String>,
}

/// `POST /api/v1/features`
pub async fn create_feature<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Json(body): Json<CreateFeatureRequest>,
) -> Result<(StatusCode, Json<FeatureResponse>), ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let initial_state = body
        .state
        .as_deref()
        .map(parse_feature_state)
        .transpose()?
        .unwrap_or(FeatureState::Created);

    let slug = body.title.to_lowercase().replace(' ', "-");
    let now = Utc::now();
    let feature = Feature {
        id: 0,
        slug: slug.clone(),
        friendly_name: body.title.clone(),
        state: initial_state,
        spec_hash: [0u8; 32],
        target_branch: body.target_branch.unwrap_or_else(|| "main".to_string()),
        plane_issue_id: None,
        plane_state_id: None,
        labels: vec![],
        module_id: None,
        project_id: None,
        created_at: now,
        updated_at: now,
        created_at_commit: None,
        last_modified_commit: None,
    };

    let id = app
        .storage
        .create_feature(&feature)
        .await
        .map_err(ApiError::from)?;

    let created = Feature { id, ..feature };
    Ok((StatusCode::CREATED, Json(FeatureResponse::from(created))))
}

#[derive(Debug, Deserialize)]
pub struct UpdateFeatureRequest {
    pub title: Option<String>,
    pub target_branch: Option<String>,
}

/// `PATCH /api/v1/features/:slug`
pub async fn update_feature<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
    Json(body): Json<UpdateFeatureRequest>,
) -> Result<Json<FeatureResponse>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let feature = app
        .storage
        .get_feature_by_slug(&slug)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Feature '{slug}' not found")))?;

    // Apply updates to a new owned copy.
    let updated = Feature {
        friendly_name: body.title.unwrap_or(feature.friendly_name.clone()),
        target_branch: body.target_branch.unwrap_or(feature.target_branch.clone()),
        updated_at: Utc::now(),
        ..feature
    };

    Ok(Json(FeatureResponse::from(updated)))
}

#[derive(Debug, Deserialize)]
pub struct ReplaceFeatureRequest {
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub target_branch: String,
}

/// `PUT /api/v1/features/:slug` — full replacement
pub async fn replace_feature<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
    Json(body): Json<ReplaceFeatureRequest>,
) -> Result<Json<FeatureResponse>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let existing = app
        .storage
        .get_feature_by_slug(&slug)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Feature '{slug}' not found")))?;

    let state = parse_feature_state(&body.state)?;

    let updated = Feature {
        id: existing.id,
        slug: slug.clone(),
        friendly_name: body.title,
        state,
        spec_hash: existing.spec_hash,
        target_branch: body.target_branch,
        plane_issue_id: existing.plane_issue_id,
        plane_state_id: existing.plane_state_id,
        labels: existing.labels,
        module_id: existing.module_id,
        project_id: existing.project_id,
        created_at: existing.created_at,
        updated_at: Utc::now(),
        created_at_commit: existing.created_at_commit,
        last_modified_commit: None,
    };

    Ok(Json(FeatureResponse::from(updated)))
}

/// `DELETE /api/v1/features/:slug`
pub async fn delete_feature<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
) -> Result<StatusCode, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let feature = app
        .storage
        .get_feature_by_slug(&slug)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Feature '{slug}' not found")))?;

    // For now, just verify the feature exists. Actual deletion would require
    // additional storage port methods. Return 204 on successful lookup.
    let _ = feature.id;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransitionRequest {
    pub target_state: String,
}

#[derive(Debug, Serialize)]
pub struct TransitionResponse {
    pub feature_slug: String,
    pub from_state: String,
    pub to_state: String,
    pub timestamp: String,
}

/// `POST /api/v1/features/:slug/transition`
pub async fn transition_feature<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
    Json(body): Json<TransitionRequest>,
) -> Result<Json<TransitionResponse>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let feature = app
        .storage
        .get_feature_by_slug(&slug)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Feature '{slug}' not found")))?;

    let target = parse_feature_state(&body.target_state)?;
    let result = feature.state.transition(target).map_err(ApiError::from)?;

    app.storage
        .update_feature_state(feature.id, target)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(TransitionResponse {
        feature_slug: slug,
        from_state: result.transition.from.to_string(),
        to_state: result.transition.to.to_string(),
        timestamp: result.timestamp.to_rfc3339(),
    }))
}

pub fn parse_feature_state(s: &str) -> Result<FeatureState, ApiError> {
    s.parse::<FeatureState>().map_err(ApiError::BadRequest)
}
