use std::collections::BTreeSet;

use anyhow::{Context, Result};

use agileplus_domain::domain::governance::{
    BuiltinPolicy, Evidence, EvidenceRequirement, EvidenceType, GovernanceContract, PolicyCheck,
};
use agileplus_domain::ports::StoragePort;

use super::{EvidenceCheck, PolicyEvalResult};

/// Evaluate governance evidence requirements against stored evidence.
pub(crate) async fn evaluate_evidence<S: StoragePort>(
    storage: &S,
    contract: &GovernanceContract,
    feature_id: i64,
) -> Result<(Vec<EvidenceCheck>, Vec<(String, String)>)> {
    let mut results = Vec::new();
    let mut missing = Vec::new();
    let wp_ids = feature_wp_ids(storage, feature_id).await?;

    for rule in &contract.rules {
        for req in &rule.required_evidence {
            let evidence_list = storage
                .get_evidence_by_fr(&req.fr_id)
                .await
                .unwrap_or_default();

            let relevant: Vec<&Evidence> = evidence_list
                .iter()
                .filter(|evidence| {
                    evidence.evidence_type == req.evidence_type && wp_ids.contains(&evidence.wp_id)
                })
                .collect();
            let found = !relevant.is_empty();

            let threshold_met = if let (true, Some(threshold)) = (found, &req.threshold) {
                evaluate_threshold(relevant.as_slice(), threshold)
            } else {
                found
            };

            let message = if !found {
                format!("No evidence found for FR `{}`", req.fr_id)
            } else if !threshold_met {
                format!("Threshold not met for FR `{}`", req.fr_id)
            } else {
                "OK".to_string()
            };

            if !found {
                missing.push((req.fr_id.clone(), format!("{:?}", req.evidence_type)));
            }

            results.push(EvidenceCheck {
                fr_id: req.fr_id.clone(),
                evidence_type: format!("{:?}", req.evidence_type),
                found,
                threshold_met,
                message,
            });
        }
    }

    Ok((results, missing))
}

/// Check if evidence meets a threshold defined in the governance contract.
pub(crate) fn evaluate_threshold(evidence: &[&Evidence], threshold: &serde_json::Value) -> bool {
    if let Some(min_cov) = threshold.get("min_coverage").and_then(|v| v.as_f64()) {
        for ev in evidence {
            if let Some(meta) = &ev.metadata {
                if let Some(cov) = meta.get("coverage").and_then(|v| v.as_f64()) {
                    if cov >= min_cov {
                        return true;
                    }
                }
            }
        }
        return false;
    }
    if let Some(max_crit) = threshold.get("max_critical").and_then(|v| v.as_u64()) {
        let critical_count: u64 = evidence
            .iter()
            .filter_map(|ev| ev.metadata.as_ref())
            .filter_map(|meta| meta.get("critical_count"))
            .filter_map(|v| v.as_u64())
            .sum();
        return critical_count <= max_crit;
    }
    true
}

/// Evaluate active policy rules and built-in contract policy refs against stored evidence.
pub(crate) async fn evaluate_policies<S: StoragePort>(
    storage: &S,
    contract: &GovernanceContract,
    feature_id: i64,
) -> Result<Vec<PolicyEvalResult>> {
    let active_policies = storage
        .list_active_policies()
        .await
        .context("loading active policies")?;

    let referenced: BTreeSet<String> = contract
        .rules
        .iter()
        .flat_map(|r| r.policy_refs.iter().cloned())
        .collect();

    let mut results = Vec::new();
    let mut handled_refs = BTreeSet::new();

    for policy in &active_policies {
        let matched_refs: Vec<&String> = referenced
            .iter()
            .filter(|policy_ref| policy.matches_reference(policy_ref))
            .collect();
        let is_referenced = !matched_refs.is_empty() || referenced.is_empty();

        if !is_referenced {
            continue;
        }

        let (passed, message) = match &policy.rule.check {
            PolicyCheck::EvidencePresent { evidence_type } => {
                evaluate_evidence_policy(storage, contract, feature_id, *evidence_type).await?
            }
            PolicyCheck::ThresholdMet { metric, min } => {
                evaluate_metric_policy(storage, feature_id, metric, *min).await?
            }
            PolicyCheck::ManualApproval => {
                evaluate_evidence_policy(
                    storage,
                    contract,
                    feature_id,
                    EvidenceType::ManualAttestation,
                )
                .await?
            }
            PolicyCheck::Custom { script } => (
                false,
                format!(
                    "Custom policy requires an external evaluator: {}",
                    script.chars().take(60).collect::<String>()
                ),
            ),
        };

        for policy_ref in matched_refs {
            handled_refs.insert(policy_ref.clone());
        }

        results.push(PolicyEvalResult {
            policy_id: policy.id,
            domain: policy.domain.as_str().to_string(),
            passed,
            message,
        });
    }

    for policy_ref in referenced.difference(&handled_refs) {
        if let Some(builtin) = BuiltinPolicy::from_ref(policy_ref) {
            let (passed, message) =
                evaluate_evidence_policy(storage, contract, feature_id, builtin.evidence_type)
                    .await?;
            results.push(PolicyEvalResult {
                policy_id: 0,
                domain: builtin.domain.as_str().to_string(),
                passed,
                message: format!("{}: {message}", builtin.label),
            });
        } else {
            results.push(PolicyEvalResult {
                policy_id: 0,
                domain: "custom".to_string(),
                passed: false,
                message: format!(
                    "Policy ref '{policy_ref}' has no active policy rule or built-in evaluator"
                ),
            });
        }
    }

    Ok(results)
}

async fn evaluate_evidence_policy<S: StoragePort>(
    storage: &S,
    contract: &GovernanceContract,
    feature_id: i64,
    evidence_type: EvidenceType,
) -> Result<(bool, String)> {
    let requirements = requirements_for_evidence_type(contract, evidence_type);
    let wp_ids = feature_wp_ids(storage, feature_id).await?;
    if requirements.is_empty() {
        return Ok((
            false,
            format!(
                "No governance contract requirement declares {} evidence",
                evidence_type.as_str()
            ),
        ));
    }

    let mut satisfied = 0usize;
    let mut missing = Vec::new();
    let mut below_threshold = Vec::new();

    for req in &requirements {
        let evidence = storage
            .get_evidence_by_fr(&req.fr_id)
            .await
            .with_context(|| format!("loading evidence for {}", req.fr_id))?;
        let relevant: Vec<_> = evidence
            .iter()
            .filter(|ev| ev.evidence_type == evidence_type && wp_ids.contains(&ev.wp_id))
            .collect();

        if relevant.is_empty() {
            missing.push(req.fr_id.clone());
            continue;
        }

        if let Some(threshold) = &req.threshold {
            if !evaluate_threshold(&relevant, threshold) {
                below_threshold.push(req.fr_id.clone());
                continue;
            }
        }

        satisfied += 1;
    }

    if missing.is_empty() && below_threshold.is_empty() {
        return Ok((
            true,
            format!(
                "{} evidence satisfied for {satisfied}/{} requirement(s)",
                evidence_type.as_str(),
                requirements.len()
            ),
        ));
    }

    let mut failures = Vec::new();
    if !missing.is_empty() {
        failures.push(format!("missing evidence for {}", missing.join(", ")));
    }
    if !below_threshold.is_empty() {
        failures.push(format!(
            "threshold not met for {}",
            below_threshold.join(", ")
        ));
    }

    Ok((
        false,
        format!(
            "{} evidence failed: {}",
            evidence_type.as_str(),
            failures.join("; ")
        ),
    ))
}

async fn feature_wp_ids<S: StoragePort>(storage: &S, feature_id: i64) -> Result<BTreeSet<i64>> {
    storage
        .list_wps_by_feature(feature_id)
        .await
        .map(|wps| wps.into_iter().map(|wp| wp.id).collect())
        .with_context(|| format!("loading work packages for feature {feature_id}"))
}

async fn evaluate_metric_policy<S: StoragePort>(
    storage: &S,
    feature_id: i64,
    metric: &str,
    min: f64,
) -> Result<(bool, String)> {
    let metrics = storage
        .get_metrics_by_feature(feature_id)
        .await
        .context("loading metrics for policy evaluation")?;
    let best = metrics
        .iter()
        .filter_map(|m| metric_value(m, metric))
        .fold(None, |best: Option<f64>, value| {
            Some(best.map_or(value, |current| current.max(value)))
        });

    Ok(match best {
        Some(value) if value >= min => (
            true,
            format!("Metric '{metric}' value {value} satisfies minimum {min}"),
        ),
        Some(value) => (
            false,
            format!("Metric '{metric}' value {value} is below minimum {min}"),
        ),
        None => (
            false,
            format!("Metric '{metric}' not found for feature {feature_id}"),
        ),
    })
}

fn requirements_for_evidence_type(
    contract: &GovernanceContract,
    evidence_type: EvidenceType,
) -> Vec<&EvidenceRequirement> {
    contract
        .rules
        .iter()
        .flat_map(|rule| rule.required_evidence.iter())
        .filter(|req| req.evidence_type == evidence_type)
        .collect()
}

fn metric_value(metric: &agileplus_domain::domain::metric::Metric, name: &str) -> Option<f64> {
    let direct = match name {
        "duration_ms" => Some(metric.duration_ms as f64),
        "agent_runs" => Some(metric.agent_runs as f64),
        "review_cycles" => Some(metric.review_cycles as f64),
        command if command == metric.command => Some(1.0),
        _ => None,
    };

    if direct.is_some() {
        return direct;
    }

    metric
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get(name))
        .and_then(|value| value.as_f64().or_else(|| value.as_i64().map(|v| v as f64)))
}
