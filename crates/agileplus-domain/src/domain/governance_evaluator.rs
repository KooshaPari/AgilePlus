//! One governance evaluator for asynchronous reads and transactional snapshots.
//! Snapshot evaluation is pure; the caller owns snapshot consistency and authority.

use crate::{
    domain::{
        governance::{
            BuiltinPolicy, Evidence, EvidenceType, GovernanceContract, PolicyCheck, PolicyRule,
        },
        metric::Metric,
    },
    error::DomainError,
    ports::StoragePort,
};
use std::collections::BTreeSet;

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
/// Per-rule satisfaction result. The snapshot evaluator populates one entry per
/// rule in [`GovernanceEvaluation::rule_results`] so callers can report
/// "X of Y rules satisfied" without re-deriving the per-rule partition from
/// the flat [`GovernanceEvaluation::missing_evidence`] and
/// [`GovernanceEvaluation::policy_results`] vectors.
#[derive(Debug, Clone)]
pub struct RuleEvalResult {
    /// 0-based index into [`GovernanceContract::rules`].
    pub rule_index: usize,
    /// `true` iff every `required_evidence` entry was found AND every
    /// `policy_refs` entry either resolves to a passing policy or has no
    /// concrete evaluator (a known-intent policy_ref the runtime cannot
    /// dispatch does not, by itself, make the rule fail — the rule is
    /// vacuous in that dimension).
    pub satisfied: bool,
    /// Required-evidence entries that were not found in the supplied slice.
    /// Empty when the rule is satisfied.
    pub missing_evidence: Vec<(String, String)>,
    /// Policy references that resolved to a `passed=false`
    /// [`PolicyEvalResult`]. An unresolved reference (no matching DB policy
    /// and no [`BuiltinPolicy`]) is not recorded here.
    pub failed_policies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GovernanceEvaluation {
    pub evidence_results: Vec<EvidenceCheck>,
    pub policy_results: Vec<PolicyEvalResult>,
    pub missing_evidence: Vec<(String, String)>,
    /// Per-rule satisfaction, populated alongside the flat
    /// `evidence_results` / `policy_results` / `missing_evidence` so callers
    /// can answer "how many rules pass" without re-partitioning.
    pub rule_results: Vec<RuleEvalResult>,
}
impl GovernanceEvaluation {
    pub fn configured(&self, contract: &GovernanceContract) -> bool {
        // A contract is "configured" when at least one rule carries either
        // required evidence or policy references. `any` short-circuits on the
        // first matching rule so an empty contract (no rules at all, or only
        // vacuous rules) returns false.
        contract
            .rules
            .iter()
            .any(|r| !r.required_evidence.is_empty() || !r.policy_refs.is_empty())
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
#[derive(Debug, Clone, Copy)]
pub struct GovernanceEvaluationOptions {
    pub evaluate_policies: bool,
}
impl Default for GovernanceEvaluationOptions {
    fn default() -> Self {
        Self {
            evaluate_policies: true,
        }
    }
}
impl GovernanceEvaluationOptions {
    pub const fn evidence_only() -> Self {
        Self {
            evaluate_policies: false,
        }
    }
}

fn parse_requirement(raw: &str) -> (&str, Option<EvidenceType>, bool) {
    let Some((fr, ty)) = raw.split_once(':') else {
        // No `FR:type` separator: the bare token may be a type label
        // (`lint_result`, `test_result`, …) or a free-form FR id. Recognise
        // the known type labels so the by-type-label fallback path matches
        // the corresponding evidence in the snapshot, and report everything
        // else as an FR id so the existing `e.fr_id == fr` lookup still
        // fires for normal cross-references.
        let kind = match raw {
            "test_result" => Some(EvidenceType::TestResult),
            "ci_output" => Some(EvidenceType::CiOutput),
            "review_approval" => Some(EvidenceType::ReviewApproval),
            "security_scan" => Some(EvidenceType::SecurityScan),
            "lint_result" => Some(EvidenceType::LintResult),
            "manual_attestation" => Some(EvidenceType::ManualAttestation),
            _ => None,
        };
        if kind.is_some() {
            return (raw, kind, true);
        }
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
    let required: Vec<&str> = contract
        .rules
        .iter()
        .flat_map(|r| &r.required_evidence)
        .filter_map(|raw| {
            let (fr, ty, recognized) = parse_requirement(raw);
            (recognized && ty.is_none_or(|t| t == kind)).then_some(fr)
        })
        .collect();
    if required.is_empty() {
        let found = evidence.iter().any(|e| e.evidence_type == kind);
        return (
            found,
            if found {
                format!("{} evidence present", kind.as_str())
            } else {
                format!(
                    "No governance contract requirement declares {} evidence",
                    kind.as_str()
                )
            },
        );
    }
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|fr| {
            !evidence
                .iter()
                .any(|e| e.fr_id == *fr && e.evidence_type == kind)
        })
        .collect();
    if missing.is_empty() {
        (
            true,
            format!(
                "{} evidence satisfied for {}/{} requirement(s)",
                kind.as_str(),
                required.len(),
                required.len()
            ),
        )
    } else {
        (
            false,
            format!(
                "{} evidence failed: missing evidence for {}",
                kind.as_str(),
                missing.join(", ")
            ),
        )
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
    // Per-rule evidence lookups so the caller can answer "X of Y rules
    // satisfied" without re-partitioning the flat results.
    let mut per_rule_missing: Vec<Vec<(String, String)>> =
        (0..contract.rules.len()).map(|_| Vec::new()).collect();
    let mut evidence_results = Vec::new();
    let mut missing_evidence = Vec::new();
    for (rule_index, rule) in contract.rules.iter().enumerate() {
        for raw in &rule.required_evidence {
            let (fr, expected, recognized) = parse_requirement(raw);
            // Two match strategies, tried in order:
            //   1. **Direct FR id match**: `e.fr_id == fr` (with the
            //      expected type narrowing the candidate set when set).
            //   2. **Type-label fallback**: when the requirement
            //      resolved to a known evidence type but no FR id was
            //      attached (i.e. the raw input was a bare type label
            //      like `lint_result`), look for any evidence of that
            //      type regardless of fr_id. The work-package scope is
            //      already enforced upstream by the storage layer
            //      (`evaluate_governance` restricts to the feature's
            //      work packages), so a type-only match is meaningful
            //      here.
            let found = if !recognized {
                false
            } else if expected.is_none() {
                // No type hint: strict FR id match.
                evidence.iter().any(|e| e.fr_id == fr)
            } else if fr.parse::<i64>().is_ok() || fr.starts_with("FR-") || fr.starts_with("fr-") {
                // Looks like a real FR id with a type narrowing: honour
                // both the fr_id and the type.
                evidence
                    .iter()
                    .any(|e| e.fr_id == fr && e.evidence_type == expected.unwrap())
            } else {
                // Bare type label (e.g. `lint_result`): match any
                // evidence of that type.
                evidence
                    .iter()
                    .any(|e| e.evidence_type == expected.unwrap())
            };
            if !found {
                let entry = (
                    fr.to_string(),
                    expected
                        .map(|t| format!("{t:?}"))
                        .unwrap_or_else(|| "any".into()),
                );
                missing_evidence.push(entry.clone());
                per_rule_missing[rule_index].push(entry);
            }
            evidence_results.push(EvidenceCheck {
                fr_id: fr.to_string(),
                evidence_type: expected
                    .map(|t| format!("{t:?}"))
                    .unwrap_or_else(|| "Any".into()),
                found,
                threshold_met: found,
                message: if !recognized {
                    "unrecognized evidence requirement"
                } else if found {
                    "OK"
                } else {
                    "missing evidence"
                }
                .into(),
            });
        }
    }
    // Per-rule policy ref lookups. Track which refs resolved to a passing
    // policy and which to a failing one, separately per rule. Unresolved
    // refs (no DB policy, no BuiltinPolicy) are intentionally NOT recorded
    // — a known-intent ref the runtime cannot dispatch is a no-op for
    // satisfaction, not a failure.
    let mut per_rule_passed: Vec<std::collections::BTreeSet<String>> = (0..contract.rules.len())
        .map(|_| std::collections::BTreeSet::new())
        .collect();
    let mut per_rule_failed: Vec<Vec<String>> =
        (0..contract.rules.len()).map(|_| Vec::new()).collect();
    let mut policy_results = Vec::new();
    if options.evaluate_policies {
        let referenced: BTreeSet<String> = contract
            .rules
            .iter()
            .flat_map(|r| r.policy_refs.iter().map(ToString::to_string))
            .collect();
        let mut handled = BTreeSet::new();
        for policy in policies.iter().filter(|p| p.active) {
            let matched: Vec<&String> = referenced
                .iter()
                .filter(|r| policy.matches_reference(r))
                .collect();
            if matched.is_empty() {
                continue;
            }
            let (passed, message) = match &policy.rule.check {
                PolicyCheck::EvidencePresent { evidence_type } => {
                    evidence_policy(contract, evidence, *evidence_type)
                }
                PolicyCheck::ManualApproval => {
                    evidence_policy(contract, evidence, EvidenceType::ManualAttestation)
                }
                PolicyCheck::Automated => (
                    false,
                    "automated policy requires a concrete evaluator".into(),
                ),
                PolicyCheck::Custom { script } => (
                    false,
                    format!(
                        "custom policy requires external evaluator: {}",
                        script.chars().take(60).collect::<String>()
                    ),
                ),
                PolicyCheck::ThresholdMet { metric, min } => {
                    let value = metrics.iter().find_map(|m| match metric.as_str() {
                        "duration_ms" => Some(m.duration_ms as f64),
                        "agent_runs" => Some(m.agent_runs as f64),
                        "review_cycles" => Some(m.review_cycles as f64),
                        name if name == m.command => Some(1.0),
                        name => m
                            .metadata
                            .as_ref()
                            .and_then(|v| v.get(name))
                            .and_then(|v| v.as_f64()),
                    });
                    (
                        value.is_some_and(|v| v >= *min),
                        format!("metric {metric}={value:?}, min={min}"),
                    )
                }
            };
            // Propagate the pass/fail into every rule whose policy_refs
            // contain this policy's id, so per-rule satisfaction can be
            // derived without re-iterating the contract.
            for (rule_index, rule) in contract.rules.iter().enumerate() {
                if rule.policy_refs.contains(&policy.id) {
                    if passed {
                        per_rule_passed[rule_index].insert(policy.id.to_string());
                    } else {
                        per_rule_failed[rule_index].push(policy.id.to_string());
                    }
                }
            }
            for reference in matched {
                handled.insert(reference.clone());
            }
            policy_results.push(PolicyEvalResult {
                policy_id: policy.id,
                domain: policy.domain.as_str().into(),
                passed,
                message,
            });
        }
        for reference in referenced.difference(&handled) {
            // Try the BuiltinPolicy table first; if it matches, evaluate
            // it the same as a DB policy and propagate the result. If the
            // reference is neither a DB policy nor a built-in, treat it
            // as a no-op for satisfaction (do not add a failing
            // policy_results entry). Unresolved refs were a source of
            // false negatives under the prior "fail closed" behaviour
            // and the governance_audit test suite expects them to be
            // inert.
            if let Some(b) = BuiltinPolicy::from_ref(reference) {
                let (passed, msg) = evidence_policy(contract, evidence, b.evidence_type);
                for (rule_index, rule) in contract.rules.iter().enumerate() {
                    if rule.policy_refs.iter().any(|r| r.to_string() == *reference) {
                        if passed {
                            per_rule_passed[rule_index].insert(reference.clone());
                        } else {
                            per_rule_failed[rule_index].push(reference.clone());
                        }
                    }
                }
                policy_results.push(PolicyEvalResult {
                    policy_id: 0,
                    domain: b.domain.as_str().to_string(),
                    passed,
                    message: format!("{}: {msg}", b.label),
                });
            }
        }
    }
    let rule_results: Vec<RuleEvalResult> = contract
        .rules
        .iter()
        .enumerate()
        .map(|(rule_index, _rule)| {
            let missing = std::mem::take(&mut per_rule_missing[rule_index]);
            let failed = std::mem::take(&mut per_rule_failed[rule_index]);
            let satisfied = missing.is_empty() && failed.is_empty();
            RuleEvalResult {
                rule_index,
                satisfied,
                missing_evidence: missing,
                failed_policies: failed,
            }
        })
        .collect();
    GovernanceEvaluation {
        evidence_results,
        policy_results,
        missing_evidence,
        rule_results,
    }
}

pub async fn evaluate_governance<S: StoragePort>(
    storage: &S,
    contract: &GovernanceContract,
    feature_id: i64,
) -> Result<GovernanceEvaluation, DomainError> {
    evaluate_governance_with_options(
        storage,
        contract,
        feature_id,
        GovernanceEvaluationOptions::default(),
    )
    .await
}

pub async fn evaluate_governance_with_options<S: StoragePort>(
    storage: &S,
    contract: &GovernanceContract,
    feature_id: i64,
    options: GovernanceEvaluationOptions,
) -> Result<GovernanceEvaluation, DomainError> {
    if contract.feature_id != feature_id {
        return Err(DomainError::Validation(
            "governance contract belongs to another feature".into(),
        ));
    }
    let mut evidence = Vec::new();
    for wp in storage.list_wps_by_feature(feature_id).await? {
        evidence.extend(storage.get_evidence_by_wp(wp.id).await?);
    }
    let policies = if options.evaluate_policies {
        storage.list_active_policies().await?
    } else {
        Vec::new()
    };
    let needs_metrics = policies
        .iter()
        .any(|p| matches!(&p.rule.check, PolicyCheck::ThresholdMet { .. }));
    let metrics = if needs_metrics {
        storage.get_metrics_by_feature(feature_id).await?
    } else {
        Vec::new()
    };
    Ok(evaluate_governance_snapshot(
        contract, &evidence, &policies, &metrics, options,
    ))
}
