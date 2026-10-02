//! One governance evaluator for asynchronous reads and transactional snapshots.
//! Snapshot evaluation is pure; the caller owns snapshot consistency and authority.

use std::collections::BTreeSet;
use crate::{
    domain::{
        governance::{BuiltinPolicy, Evidence, EvidenceType, GovernanceContract, PolicyCheck, PolicyRule},
        metric::Metric,
    },
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
        contract.rules.iter().any(|r| !r.required_evidence.is_empty() || !r.policy_refs.is_empty())
    }
    pub fn passed(&self, contract: &GovernanceContract) -> bool {
        self.configured(contract)
            && self.missing_evidence.is_empty()
            && self.evidence_results.iter().all(|e| e.found && e.threshold_met)
            && self.policy_results.iter().all(|p| p.passed)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct GovernanceEvaluationOptions { pub evaluate_policies: bool }
impl Default for GovernanceEvaluationOptions {
    fn default() -> Self { Self { evaluate_policies: true } }
}
impl GovernanceEvaluationOptions {
    pub const fn evidence_only() -> Self { Self { evaluate_policies: false } }
}

fn parse_requirement(raw: &str) -> (&str, Option<EvidenceType>, bool) {
    let Some((fr, ty)) = raw.split_once(':') else {
        return (raw, None, !raw.trim().is_empty());
    };
    let kind = match ty {
        "test_result" => Some(EvidenceType::TestResult),
        "ci_output" => Some(EvidenceType::CiOutput),
        "review_approval" => Some(EvidenceType::ReviewApproval),
        "security_scan" => Some(EvidenceType::SecurityScan),
        "lint_result" => Some(EvidenceType::LintResult),
        "manual_attestation" => Some(EvidenceType::ManualAttestation),
        _ => None,
    };
    (fr, kind, !fr.trim().is_empty() && kind.is_some())
}

fn evidence_policy(
    contract: &GovernanceContract,
    evidence: &[Evidence],
    kind: EvidenceType,
) -> (bool, String) {
    let required: Vec<&str> = contract.rules.iter()
        .flat_map(|r| &r.required_evidence)
        .filter_map(|raw| {
            let (fr, ty, recognized) = parse_requirement(raw);
            (recognized && ty.is_none_or(|t| t == kind)).then_some(fr)
        }).collect();
    if required.is_empty() {
        let found = evidence.iter().any(|e| e.evidence_type == kind);
        return (found, if found {
            format!("{} evidence present", kind.as_str())
        } else {
            format!("No governance contract requirement declares {} evidence", kind.as_str())
        });
    }
    let missing: Vec<&str> = required.iter().copied()
        .filter(|fr| !evidence.iter().any(|e| e.fr_id == *fr && e.evidence_type == kind))
        .collect();
    if missing.is_empty() {
        (true, format!("{} evidence satisfied for {}/{} requirement(s)", kind.as_str(), required.len(), required.len()))
    } else {
        (false, format!("{} evidence failed: missing evidence for {}", kind.as_str(), missing.join(", ")))
    }
}

/// Evidence must already be restricted to the feature's work packages.
/// Atomic acceptance supplies rows read within its write transaction.
pub fn evaluate_governance_snapshot(
    contract: &GovernanceContract,
    evidence: &[Evidence],
    policies: &[PolicyRule],
    metrics: &[Metric],
    options: GovernanceEvaluationOptions,
) -> GovernanceEvaluation {
    let mut evidence_results = Vec::new();
    let mut missing_evidence = Vec::new();
    for rule in &contract.rules {
        for raw in &rule.required_evidence {
            let (fr, expected, recognized) = parse_requirement(raw);
            let found = recognized && evidence.iter().any(|e|
                e.fr_id == fr && expected.is_none_or(|ty| e.evidence_type == ty));
            if !found {
                missing_evidence.push((fr.to_string(), expected.map(|t| format!("{t:?}")).unwrap_or_else(|| "any".into())));
            }
            evidence_results.push(EvidenceCheck {
                fr_id: fr.to_string(),
                evidence_type: expected.map(|t| format!("{t:?}")).unwrap_or_else(|| "Any".into()),
                found,
                threshold_met: found,
                message: if !recognized { "unrecognized evidence requirement" }
                    else if found { "OK" } else { "missing evidence" }.into(),
            });
        }
    }
    let mut policy_results = Vec::new();
    if options.evaluate_policies {
        let referenced: BTreeSet<String> = contract.rules.iter()
            .flat_map(|r| r.policy_refs.iter().map(ToString::to_string)).collect();
        let mut handled = BTreeSet::new();
        for policy in policies.iter().filter(|p| p.active) {
            let matched: Vec<&String> = referenced.iter().filter(|r| policy.matches_reference(r)).collect();
            if matched.is_empty() { continue; }
            let (passed, message) = match &policy.rule.check {
                PolicyCheck::EvidencePresent { evidence_type } => evidence_policy(contract, evidence, *evidence_type),
                PolicyCheck::ManualApproval => evidence_policy(contract, evidence, EvidenceType::ManualAttestation),
                PolicyCheck::Automated => (false, "automated policy requires a concrete evaluator".into()),
                PolicyCheck::Custom { script } => (false, format!("custom policy requires external evaluator: {}", script.chars().take(60).collect::<String>())),
                PolicyCheck::ThresholdMet { metric, min } => {
                    let value = metrics.iter().find_map(|m| match metric.as_str() {
                        "duration_ms" => Some(m.duration_ms as f64),
                        "agent_runs" => Some(m.agent_runs as f64),
                        "review_cycles" => Some(m.review_cycles as f64),
                        name if name == m.command => Some(1.0),
                        name => m.metadata.as_ref().and_then(|v| v.get(name)).and_then(|v| v.as_f64()),
                    });
                    (value.is_some_and(|v| v >= *min), format!("metric {metric}={value:?}, min={min}"))
                }
            };
            for reference in matched { handled.insert(reference.clone()); }
            policy_results.push(PolicyEvalResult {
                policy_id: policy.id, domain: policy.domain.as_str().into(), passed, message,
            });
        }
        for reference in referenced.difference(&handled) {
            let (domain, passed, message) = if let Some(b) = BuiltinPolicy::from_ref(reference) {
                let (passed, msg) = evidence_policy(contract, evidence, b.evidence_type);
                (b.domain.as_str().to_string(), passed, format!("{}: {msg}", b.label))
            } else {
                ("custom".into(), false, format!("policy ref {reference} has no evaluator"))
            };
            policy_results.push(PolicyEvalResult { policy_id: 0, domain, passed, message });
        }
    }
    GovernanceEvaluation { evidence_results, policy_results, missing_evidence }
}

pub async fn evaluate_governance<S: StoragePort>(
    storage: &S, contract: &GovernanceContract, feature_id: i64,
) -> Result<GovernanceEvaluation, DomainError> {
    evaluate_governance_with_options(storage, contract, feature_id, GovernanceEvaluationOptions::default()).await
}

pub async fn evaluate_governance_with_options<S: StoragePort>(
    storage: &S, contract: &GovernanceContract, feature_id: i64,
    options: GovernanceEvaluationOptions,
) -> Result<GovernanceEvaluation, DomainError> {
    if contract.feature_id != feature_id {
        return Err(DomainError::Validation("governance contract belongs to another feature".into()));
    }
    let mut evidence = Vec::new();
    for wp in storage.list_wps_by_feature(feature_id).await? {
        evidence.extend(storage.get_evidence_by_wp(wp.id).await?);
    }
    let policies = if options.evaluate_policies { storage.list_active_policies().await? } else { Vec::new() };
    let needs_metrics = policies.iter().any(|p| matches!(&p.rule.check, PolicyCheck::ThresholdMet { .. }));
    let metrics = if needs_metrics { storage.get_metrics_by_feature(feature_id).await? } else { Vec::new() };
    Ok(evaluate_governance_snapshot(contract, &evidence, &policies, &metrics, options))
}
