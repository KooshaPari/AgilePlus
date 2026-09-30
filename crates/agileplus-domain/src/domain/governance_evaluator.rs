//! Shared governance evaluation service.
//!
//! This is migrated from the CLI validator so transports do not invent
//! independent compliance semantics.

use std::collections::BTreeSet;

use crate::{
    domain::governance::{BuiltinPolicy, Evidence, EvidenceType, GovernanceContract, PolicyCheck},
    error::DomainError,
    ports::StoragePort,
};

#[derive(Debug, Clone)]
pub struct EvidenceCheck {
    pub fr_id: String,
    pub evidence_type: String,
    pub found: bool,
    pub threshold_met: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct PolicyEvalResult {
    pub policy_id: i64,
    pub domain: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct GovernanceEvaluation {
    pub evidence_results: Vec<EvidenceCheck>,
    pub policy_results: Vec<PolicyEvalResult>,
    pub missing_evidence: Vec<(String, String)>,
}

impl GovernanceEvaluation {
    pub fn configured(&self, contract: &GovernanceContract) -> bool {
        !contract.rules.is_empty()
    }

    pub fn passed(&self, contract: &GovernanceContract) -> bool {
        self.configured(contract)
            && self.missing_evidence.is_empty()
            && self
                .evidence_results
                .iter()
                .all(|e| e.found && e.threshold_met)
            && self.policy_results.iter().all(|p| p.passed)
    }
}

fn parse_requirement(raw: &str) -> (String, Option<EvidenceType>) {
    if let Some((fr, ty)) = raw.split_once(':') {
        let evidence_type = match ty {
            "test_result" => Some(EvidenceType::TestResult),
            "ci_output" => Some(EvidenceType::CiOutput),
            "review_approval" => Some(EvidenceType::ReviewApproval),
            "security_scan" => Some(EvidenceType::SecurityScan),
            "lint_result" => Some(EvidenceType::LintResult),
            "manual_attestation" => Some(EvidenceType::ManualAttestation),
            _ => None,
        };
        (fr.to_string(), evidence_type)
    } else {
        (raw.to_string(), None)
    }
}

async fn feature_evidence<S: StoragePort>(
    storage: &S,
    feature_id: i64,
) -> Result<Vec<Evidence>, DomainError> {
    let mut out = Vec::new();
    for wp in storage.list_wps_by_feature(feature_id).await? {
        out.extend(storage.get_evidence_by_wp(wp.id).await?);
    }
    Ok(out)
}

fn requirements_for_evidence_type(
    contract: &GovernanceContract,
    evidence_type: EvidenceType,
) -> Vec<String> {
    contract
        .rules
        .iter()
        .flat_map(|rule| rule.required_evidence.iter())
        .filter_map(|raw| {
            let (fr, ty) = parse_requirement(raw);
            match ty {
                Some(t) if t == evidence_type => Some(fr),
                None => Some(fr),
                _ => None,
            }
        })
        .collect()
}

fn evaluate_evidence_policy(
    contract: &GovernanceContract,
    evidence: &[Evidence],
    evidence_type: EvidenceType,
) -> (bool, String) {
    let requirements = requirements_for_evidence_type(contract, evidence_type);

    if requirements.is_empty() {
        let any = evidence.iter().any(|e| e.evidence_type == evidence_type);
        return if any {
            (
                true,
                format!("{} evidence present", evidence_type.as_str()),
            )
        } else {
            (
                false,
                format!(
                    "No governance contract requirement declares {} evidence",
                    evidence_type.as_str()
                ),
            )
        };
    }

    let missing: Vec<String> = requirements
        .iter()
        .filter(|fr_id| {
            !evidence
                .iter()
                .any(|e| e.fr_id == fr_id.as_str() && e.evidence_type == evidence_type)
        })
        .cloned()
        .collect();

    if missing.is_empty() {
        (
            true,
            format!(
                "{} evidence satisfied for {}/{} requirement(s)",
                evidence_type.as_str(),
                requirements.len(),
                requirements.len()
            ),
        )
    } else {
        (
            false,
            format!(
                "{} evidence failed: missing evidence for {}",
                evidence_type.as_str(),
                missing.join(", ")
            ),
        )
    }
}

pub async fn evaluate_governance<S: StoragePort>(
    storage: &S,
    contract: &GovernanceContract,
    feature_id: i64,
) -> Result<GovernanceEvaluation, DomainError> {
    let evidence = feature_evidence(storage, feature_id).await?;
    let mut evidence_results = Vec::new();
    let mut missing = Vec::new();

    for rule in &contract.rules {
        for raw in &rule.required_evidence {
            let (fr, expected) = parse_requirement(raw);
            let relevant: Vec<&Evidence> = evidence
                .iter()
                .filter(|e| {
                    e.fr_id == fr
                        && expected
                            .map(|evidence_type| evidence_type == e.evidence_type)
                            .unwrap_or(true)
                })
                .collect();
            let found = !relevant.is_empty();
            let evidence_type = expected
                .map(|t| t.as_str().to_string())
                .unwrap_or_else(|| "any".to_string());

            if !found {
                missing.push((fr.clone(), evidence_type.clone()));
            }

            evidence_results.push(EvidenceCheck {
                fr_id: fr,
                evidence_type,
                found,
                threshold_met: found,
                message: if found {
                    "OK".to_string()
                } else {
                    "missing evidence".to_string()
                },
            });
        }
    }

    let active = storage.list_active_policies().await?;
    let referenced: BTreeSet<String> = contract
        .rules
        .iter()
        .flat_map(|r| r.policy_refs.iter().map(ToString::to_string))
        .collect();
    let mut handled = BTreeSet::new();
    let mut policy_results = Vec::new();

    for policy in &active {
        let matched: Vec<&String> = referenced
            .iter()
            .filter(|r| policy.matches_reference(r))
            .collect();
        if matched.is_empty() {
            continue;
        }

        let (passed, message) = match &policy.rule.check {
            PolicyCheck::EvidencePresent { evidence_type } => {
                evaluate_evidence_policy(contract, &evidence, *evidence_type)
            }
            PolicyCheck::ManualApproval | PolicyCheck::Automated => {
                evaluate_evidence_policy(contract, &evidence, EvidenceType::ManualAttestation)
            }
            PolicyCheck::ThresholdMet { metric, min } => {
                let metrics = storage.get_metrics_by_feature(feature_id).await?;
                let value = metrics.iter().find_map(|m| match metric.as_str() {
                    "duration_ms" => Some(m.duration_ms as f64),
                    "agent_runs" => Some(m.agent_runs as f64),
                    "review_cycles" => Some(m.review_cycles as f64),
                    name => m
                        .metadata
                        .as_ref()
                        .and_then(|v| v.get(name))
                        .and_then(|v| v.as_f64()),
                });
                (
                    value.map(|v| v >= *min).unwrap_or(false),
                    format!("metric {metric}={value:?}, min={min}"),
                )
            }
            PolicyCheck::Custom { script } => (
                false,
                format!(
                    "custom policy requires external evaluator: {}",
                    script.chars().take(60).collect::<String>()
                ),
            ),
        };

        for r in matched {
            handled.insert(r.clone());
        }
        policy_results.push(PolicyEvalResult {
            policy_id: policy.id,
            domain: policy.domain.as_str().to_string(),
            passed,
            message,
        });
    }

    for r in referenced.difference(&handled) {
        if let Some(b) = BuiltinPolicy::from_ref(r) {
            let (passed, message) =
                evaluate_evidence_policy(contract, &evidence, b.evidence_type);
            policy_results.push(PolicyEvalResult {
                policy_id: 0,
                domain: b.domain.as_str().to_string(),
                passed,
                message: format!("{}: {message}", b.label),
            });
        } else {
            policy_results.push(PolicyEvalResult {
                policy_id: 0,
                domain: "custom".to_string(),
                passed: false,
                message: format!("policy ref {r} has no evaluator"),
            });
        }
    }

    Ok(GovernanceEvaluation {
        evidence_results,
        policy_results,
        missing_evidence: missing,
    })
}
