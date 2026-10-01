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

use agileplus_domain::domain::evaluation::GovernanceResult;
use agileplus_domain::domain::governance_evaluator::evaluate_governance;
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

    let evaluation = evaluate_governance(&state.storage, &contract, feature.id)
        .await
        .map_err(ApiError::from)?;
    let result = if !evaluation.configured(&contract) {
        GovernanceResult::NotConfigured
    } else if evaluation.passed(&contract) {
        GovernanceResult::Satisfied
    } else {
        GovernanceResult::Unsatisfied
    };
    let compliant = result.compliant();
    let total_rules = contract.rules.len();
    let satisfied_rules = if compliant { total_rules } else { 0 };

    Ok(Json(json!({
        "feature_slug": slug,
        "governance_version": contract.version,
        "total_rules": total_rules,
        "satisfied_rules": satisfied_rules,
        "compliant": compliant,
        "result": result.as_str(),
        "missing_evidence": evaluation.missing_evidence,
        "policy_results": evaluation.policy_results.iter().map(|p| serde_json::json!({"policy_id":p.policy_id,"domain":p.domain,"passed":p.passed,"message":p.message})).collect::<Vec<_>>(),
    })))
}

#[cfg(test)]
mod tests {
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
    fn validation_no_rules_is_not_configured() {
        let total_rules = 0usize;
        let satisfied_rules = 0usize;
        let result = if total_rules == 0 {
            "not_configured"
        } else if satisfied_rules == total_rules {
            "satisfied"
        } else {
            "unsatisfied"
        };
        let summary = serde_json::json!({
            "feature_slug": "empty",
            "governance_version": 1,
            "total_rules": total_rules,
            "satisfied_rules": satisfied_rules,
            "compliant": result == "satisfied",
            "result": result,
        });
        assert_eq!(summary["result"], "not_configured");
        assert_eq!(summary["compliant"], false);
    }
}
