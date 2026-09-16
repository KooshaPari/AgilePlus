//! Governance route handlers.
//!
//! - GET /api/v1/features/:slug/governance
//! - POST /api/v1/features/:slug/validate  (trigger governance evaluation)
//!
//! Traceability: WP15-T086

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};

use agileplus_domain::ports::vcs::VcsPort;
use agileplus_domain::ports::{ObservabilityPort, StoragePort};

use crate::error::ApiError;
use crate::responses::GovernanceResponse;
use crate::state::AppState;

pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    Router::new()
        .route("/{slug}/governance", get(get_governance::<S, V, O>))
        .route("/{slug}/validate", post(trigger_validate::<S, V, O>))
}

/// `GET /api/v1/features/:slug/governance`
#[utoipa::path(
    get,
    path = "/api/v1/features/{slug}/governance",
    tag = "governance",
    params(
        ("slug" = String, Path, description = "Feature slug"),
    ),
    responses(
        (status = 200, description = "Latest governance contract for feature", body = GovernanceResponse),
        (status = 404, description = "Feature or contract not found"),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn get_governance<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
) -> Result<Json<GovernanceResponse>, ApiError>
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

    let contract = state
        .storage
        .get_latest_governance_contract(feature.id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::NotFound(format!("No governance contract for feature '{slug}'"))
        })?;

    Ok(Json(GovernanceResponse::from(contract)))
}

/// `POST /api/v1/features/:slug/validate`
///
/// Triggers governance validation and returns a summary report.
/// Full evaluator integration is handled by the GovernanceEvaluator from WP11.
pub async fn trigger_validate<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, ApiError>
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

    let contract = state
        .storage
        .get_latest_governance_contract(feature.id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::NotFound(format!("No governance contract for feature '{slug}'"))
        })?;

    // Get evidence for all WPs in this feature
    let wps = state
        .storage
        .list_wps_by_feature(feature.id)
        .await
        .map_err(ApiError::from)?;

    let mut total_rules = 0usize;
    let mut satisfied_rules = 0usize;

    let wp_ids: std::collections::HashSet<i64> = wps.iter().map(|w| w.id).collect();

    for rule in &contract.rules {
        total_rules += 1;
        // `required_evidence` entries are FR ids and/or evidence-type labels.
        let mut rule_satisfied = true;
        for req in &rule.required_evidence {
            let by_fr = state
                .storage
                .get_evidence_by_fr(req)
                .await
                .map_err(ApiError::from)?;
            let mut found = by_fr.iter().any(|e| wp_ids.contains(&e.wp_id));
            if !found {
                for wp in &wps {
                    let ev = state
                        .storage
                        .get_evidence_by_wp(wp.id)
                        .await
                        .map_err(ApiError::from)?;
                    if ev
                        .iter()
                        .any(|e| e.fr_id == *req || e.evidence_type.as_str() == req.as_str())
                    {
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                rule_satisfied = false;
                break;
            }
        }
        if rule_satisfied {
            satisfied_rules += 1;
        }
    }

    let compliant = satisfied_rules == total_rules;
    Ok(Json(json!({
        "feature_slug": slug,
        "governance_version": contract.version,
        "total_rules": total_rules,
        "satisfied_rules": satisfied_rules,
        "compliant": compliant,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_summary_structure() {
        let summary = serde_json::json!({
            "feature_slug": "my-feat",
            "governance_version": 3,
            "total_rules": 5,
            "satisfied_rules": 4,
            "compliant": false,
        });
        assert_eq!(summary["feature_slug"], "my-feat");
        assert_eq!(summary["governance_version"], 3);
        assert_eq!(summary["total_rules"], 5);
        assert_eq!(summary["satisfied_rules"], 4);
        assert_eq!(summary["compliant"], false);
    }

    #[test]
    fn validation_compliant_summary() {
        let summary = serde_json::json!({
            "feature_slug": "feat",
            "governance_version": 1,
            "total_rules": 2,
            "satisfied_rules": 2,
            "compliant": true,
        });
        assert!(summary["compliant"].as_bool().unwrap());
        assert_eq!(summary["total_rules"], summary["satisfied_rules"]);
    }

    #[test]
    fn validation_no_rules_compliant() {
        let summary = serde_json::json!({
            "feature_slug": "empty",
            "governance_version": 1,
            "total_rules": 0,
            "satisfied_rules": 0,
            "compliant": true,
        });
        assert!(summary["compliant"].as_bool().unwrap());
    }
}
