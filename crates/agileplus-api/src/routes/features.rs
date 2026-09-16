// SPDX-License-Identifier: MIT OR Apache-2.0
//! Feature route handlers.
//!
//! - GET  /api/v1/features                  → list (with ?state=, ?label= filters)
//! - GET  /api/v1/features/:slug            → detail
//! - POST /api/v1/features                  → create
//! - PATCH /api/v1/features/:slug           → update
//! - POST /api/v1/features/:slug/transition → state transition
//!
//! Traceability: WP11-T066

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::vcs::VcsPort;
use agileplus_domain::ports::{ObservabilityPort, StoragePort};

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
            get(get_feature::<S, V, O>).patch(update_feature::<S, V, O>),
        )
        .route("/{slug}/transition", post(transition_feature::<S, V, O>))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct FeatureListParams {
    /// Filter by feature state (e.g. `planned`, `in_progress`, `done`).
    pub state: Option<String>,
    /// Filter by label.
    pub label: Option<String>,
}

/// `GET /api/v1/features[?state=<state>&label=<label>]`
#[utoipa::path(
    get,
    path = "/api/v1/features",
    tag = "features",
    params(FeatureListParams),
    responses(
        (status = 200, description = "List features", body = [FeatureResponse]),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_features<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Query(params): Query<FeatureListParams>,
) -> Result<Json<Vec<FeatureResponse>>, ApiError>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    let features = if let Some(state_filter) = &params.state {
        let fs = parse_feature_state(state_filter)?;
        state
            .storage
            .list_features_by_state(fs)
            .await
            .map_err(ApiError::from)?
    } else if let Some(label) = &params.label {
        state
            .storage
            .list_features_by_label(label)
            .await
            .map_err(ApiError::from)?
    } else {
        state
            .storage
            .list_all_features()
            .await
            .map_err(ApiError::from)?
    };

    let features: Vec<Feature> = if let Some(label_filter) = &params.label {
        features
            .into_iter()
            .filter(|f| f.labels.contains(label_filter))
            .collect()
    } else {
        features
    };

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

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFeatureRequest {
    /// Human-friendly title; used to derive the slug.
    pub title: String,
    /// Optional long-form description.
    pub description: Option<String>,
    /// Initial state (defaults to `planned`).
    pub state: Option<String>,
    /// Target git branch for the feature; defaults to `main`.
    pub target_branch: Option<String>,
}

/// `POST /api/v1/features`
#[utoipa::path(
    post,
    path = "/api/v1/features",
    tag = "features",
    request_body = CreateFeatureRequest,
    responses(
        (status = 201, description = "Feature created", body = FeatureResponse),
        (status = 400, description = "Invalid payload"),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
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

    app.storage
        .update_feature(&updated)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(FeatureResponse::from(updated)))
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
    let result = agileplus_domain::domain::state_machine::transition(feature.state, target)
        .map_err(ApiError::from)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_feature_state_created() {
        let state = parse_feature_state("created").unwrap();
        assert_eq!(state, FeatureState::Created);
    }

    #[test]
    fn parse_feature_state_specified() {
        let state = parse_feature_state("specified").unwrap();
        assert_eq!(state, FeatureState::Specified);
    }

    #[test]
    fn parse_feature_state_researched() {
        let state = parse_feature_state("researched").unwrap();
        assert_eq!(state, FeatureState::Researched);
    }

    #[test]
    fn parse_feature_state_planned() {
        let state = parse_feature_state("planned").unwrap();
        assert_eq!(state, FeatureState::Planned);
    }

    #[test]
    fn parse_feature_state_implementing() {
        let state = parse_feature_state("implementing").unwrap();
        assert_eq!(state, FeatureState::Implementing);
    }

    #[test]
    fn parse_feature_state_validated() {
        let state = parse_feature_state("validated").unwrap();
        assert_eq!(state, FeatureState::Validated);
    }

    #[test]
    fn parse_feature_state_shipped() {
        let state = parse_feature_state("shipped").unwrap();
        assert_eq!(state, FeatureState::Shipped);
    }

    #[test]
    fn parse_feature_state_retrospected() {
        let state = parse_feature_state("retrospected").unwrap();
        assert_eq!(state, FeatureState::Retrospected);
    }

    #[test]
    fn parse_feature_state_invalid() {
        let err = parse_feature_state("bogus");
        assert!(err.is_err());
        match err.unwrap_err() {
            ApiError::BadRequest(msg) => assert!(msg.contains("bogus") || !msg.is_empty()),
            other => panic!("expected BadRequest, got {other:?}"),
        }
    }

    #[test]
    fn parse_feature_state_empty_string() {
        assert!(parse_feature_state("").is_err());
    }

    #[test]
    fn parse_feature_state_case_sensitive() {
        // FromStr is case-sensitive
        assert!(parse_feature_state("Created").is_err());
    }

    #[test]
    fn feature_list_params_defaults() {
        let params = FeatureListParams {
            state: None,
            label: None,
        };
        assert!(params.state.is_none());
        assert!(params.label.is_none());
    }

    #[test]
    fn transition_response_fields() {
        let resp = TransitionResponse {
            feature_slug: "my-feat".into(),
            from_state: "created".into(),
            to_state: "specified".into(),
            timestamp: "2025-01-01T00:00:00Z".into(),
        };
        assert_eq!(resp.feature_slug, "my-feat");
        assert_eq!(resp.from_state, "created");
        assert_eq!(resp.to_state, "specified");
    }

    #[test]
    fn transition_response_serializes() {
        let resp = TransitionResponse {
            feature_slug: "slug".into(),
            from_state: "a".into(),
            to_state: "b".into(),
            timestamp: "t".into(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["feature_slug"], "slug");
        assert_eq!(json["from_state"], "a");
        assert_eq!(json["to_state"], "b");
    }
}
