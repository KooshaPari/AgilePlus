//! Governance types — contracts, rules, and evidence.
//!
//! Source: [`AgilePlus/crates/agileplus-domain/src/domain/governance.rs`](https://example.invalid/AgilePlus/crates/agileplus-domain/src/domain/governance.rs)
//! (1:1 port of vocabulary; `fr_id` stays `String` at the boundary per ADR-0001 §2).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Policy domain category.
///
/// Source: AgilePlus `governance.rs:7-15`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyDomain {
    Security,
    Quality,
    Compliance,
    Performance,
    Custom,
}

impl PolicyDomain {
    /// Stable lowercase string for SQL / JSON round-trips.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Security => "security",
            Self::Quality => "quality",
            Self::Compliance => "compliance",
            Self::Performance => "performance",
            Self::Custom => "custom",
        }
    }
}

/// The definition of a policy rule (stored as JSON blob).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefinition {
    pub description: String,
    pub check: PolicyCheck,
}

/// An active policy rule in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: i64,
    pub domain: PolicyDomain,
    pub rule: PolicyDefinition,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A required-evidence entry inside a governance rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequirement {
    /// Functional-requirement ID the evidence must satisfy.
    pub fr_id: String,
    /// Type of evidence required.
    pub evidence_type: EvidenceType,
}

/// A governance rule captured inside a contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRule {
    pub transition: String,
    pub required_evidence: Vec<EvidenceRequirement>,
    pub policy_refs: Vec<i64>,
}

/// A versioned governance contract bound to a feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceContract {
    pub id: i64,
    pub feature_id: i64,
    pub version: i32,
    pub rules: Vec<GovernanceRule>,
    pub bound_at: DateTime<Utc>,
}

/// Type of evidence artifact.
///
/// Source: AgilePlus `governance.rs:74-84`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    TestResult,
    CiOutput,
    ReviewApproval,
    SecurityScan,
    LintResult,
    ManualAttestation,
}

impl EvidenceType {
    /// Stable snake_case string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TestResult => "test_result",
            Self::CiOutput => "ci_output",
            Self::ReviewApproval => "review_approval",
            Self::SecurityScan => "security_scan",
            Self::LintResult => "lint_result",
            Self::ManualAttestation => "manual_attestation",
        }
    }
}

/// An evidence artifact attached to a work package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: i64,
    pub wp_id: i64,
    pub fr_id: String,
    pub evidence_type: EvidenceType,
    pub artifact_path: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// The result of a policy check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyCheck {
    ManualApproval,
    Automated,
}

/// A well-known built-in policy that maps a short reference key to a
/// `PolicyDomain` + `EvidenceType` pair.
///
/// Source: AgilePlus `governance.rs:118-184`.
#[derive(Debug, Clone, Copy)]
pub struct BuiltinPolicy {
    /// Short human-readable label (e.g. "Unit tests passing").
    pub label: &'static str,
    /// Governance domain for grouping.
    pub domain: PolicyDomain,
    /// Required evidence kind.
    pub evidence_type: EvidenceType,
}

impl BuiltinPolicy {
    const KNOWN: &'static [(&'static str, BuiltinPolicy)] = &[
        (
            "tests-pass",
            BuiltinPolicy {
                label: "Unit tests passing",
                domain: PolicyDomain::Quality,
                evidence_type: EvidenceType::TestResult,
            },
        ),
        (
            "ci-green",
            BuiltinPolicy {
                label: "CI pipeline green",
                domain: PolicyDomain::Quality,
                evidence_type: EvidenceType::CiOutput,
            },
        ),
        (
            "review-approved",
            BuiltinPolicy {
                label: "Peer review approved",
                domain: PolicyDomain::Quality,
                evidence_type: EvidenceType::ReviewApproval,
            },
        ),
        (
            "security-scan",
            BuiltinPolicy {
                label: "Security scan clean",
                domain: PolicyDomain::Security,
                evidence_type: EvidenceType::SecurityScan,
            },
        ),
        (
            "lint-pass",
            BuiltinPolicy {
                label: "Lint checks pass",
                domain: PolicyDomain::Quality,
                evidence_type: EvidenceType::LintResult,
            },
        ),
    ];

    /// Look up a built-in policy by its reference key.
    pub fn from_ref(policy_ref: &str) -> Option<&'static BuiltinPolicy> {
        Self::KNOWN
            .iter()
            .find(|(key, _)| *key == policy_ref)
            .map(|(_, bp)| bp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_domain_as_str() {
        assert_eq!(PolicyDomain::Security.as_str(), "security");
        assert_eq!(PolicyDomain::Quality.as_str(), "quality");
        assert_eq!(PolicyDomain::Compliance.as_str(), "compliance");
        assert_eq!(PolicyDomain::Performance.as_str(), "performance");
        assert_eq!(PolicyDomain::Custom.as_str(), "custom");
    }

    #[test]
    fn evidence_type_as_str() {
        assert_eq!(EvidenceType::TestResult.as_str(), "test_result");
        assert_eq!(EvidenceType::CiOutput.as_str(), "ci_output");
        assert_eq!(
            EvidenceType::ManualAttestation.as_str(),
            "manual_attestation"
        );
    }

    #[test]
    fn builtin_policy_known_refs_resolve() {
        let tests_pass = BuiltinPolicy::from_ref("tests-pass").unwrap();
        assert_eq!(tests_pass.domain, PolicyDomain::Quality);
        assert_eq!(tests_pass.evidence_type, EvidenceType::TestResult);

        let security = BuiltinPolicy::from_ref("security-scan").unwrap();
        assert_eq!(security.domain, PolicyDomain::Security);
    }

    #[test]
    fn builtin_policy_unknown_ref_returns_none() {
        assert!(BuiltinPolicy::from_ref("nonexistent-policy").is_none());
        assert!(BuiltinPolicy::from_ref("").is_none());
    }

    #[test]
    fn governance_contract_serde_roundtrip() {
        let now = Utc::now();
        let contract = GovernanceContract {
            id: 42,
            feature_id: 100,
            version: 3,
            rules: vec![GovernanceRule {
                transition: "Active->Done".to_string(),
                required_evidence: vec![EvidenceRequirement {
                    fr_id: "FR-001".to_string(),
                    evidence_type: EvidenceType::TestResult,
                }],
                policy_refs: vec![1],
            }],
            bound_at: now,
        };
        let json = serde_json::to_string(&contract).unwrap();
        let back: GovernanceContract = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, contract.id);
        assert_eq!(back.rules.len(), 1);
    }

    #[test]
    fn evidence_type_as_str_remaining_variants() {
        assert_eq!(EvidenceType::ReviewApproval.as_str(), "review_approval");
        assert_eq!(EvidenceType::SecurityScan.as_str(), "security_scan");
        assert_eq!(EvidenceType::LintResult.as_str(), "lint_result");
    }

    #[test]
    fn builtin_policy_ci_green() {
        let p = BuiltinPolicy::from_ref("ci-green").unwrap();
        assert_eq!(p.domain, PolicyDomain::Quality);
        assert_eq!(p.evidence_type, EvidenceType::CiOutput);
    }

    #[test]
    fn builtin_policy_review_approved() {
        let p = BuiltinPolicy::from_ref("review-approved").unwrap();
        assert_eq!(p.domain, PolicyDomain::Quality);
        assert_eq!(p.evidence_type, EvidenceType::ReviewApproval);
    }

    #[test]
    fn builtin_policy_lint_pass() {
        let p = BuiltinPolicy::from_ref("lint-pass").unwrap();
        assert_eq!(p.domain, PolicyDomain::Quality);
        assert_eq!(p.evidence_type, EvidenceType::LintResult);
    }

    #[test]
    fn policy_domain_serde_roundtrip() {
        for domain in [
            PolicyDomain::Security,
            PolicyDomain::Quality,
            PolicyDomain::Compliance,
            PolicyDomain::Performance,
            PolicyDomain::Custom,
        ] {
            let json = serde_json::to_string(&domain).unwrap();
            let back: PolicyDomain = serde_json::from_str(&json).unwrap();
            assert_eq!(back, domain);
        }
    }

    #[test]
    fn evidence_type_serde_roundtrip() {
        for et in [
            EvidenceType::TestResult,
            EvidenceType::CiOutput,
            EvidenceType::ReviewApproval,
            EvidenceType::SecurityScan,
            EvidenceType::LintResult,
            EvidenceType::ManualAttestation,
        ] {
            let json = serde_json::to_string(&et).unwrap();
            let back: EvidenceType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, et);
        }
    }

    #[test]
    fn evidence_serde_roundtrip() {
        let e = Evidence {
            id: 10,
            wp_id: 20,
            fr_id: "FR-42".to_string(),
            evidence_type: EvidenceType::CiOutput,
            artifact_path: "/ci/build.log".to_string(),
            metadata: Some(serde_json::json!({"key": "value"})),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: Evidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, 10);
        assert_eq!(back.fr_id, "FR-42");
        assert_eq!(back.evidence_type, EvidenceType::CiOutput);
        assert!(back.metadata.is_some());
    }

    #[test]
    fn evidence_without_metadata_serde() {
        let e = Evidence {
            id: 1,
            wp_id: 1,
            fr_id: "FR-1".to_string(),
            evidence_type: EvidenceType::TestResult,
            artifact_path: "/test.xml".to_string(),
            metadata: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: Evidence = serde_json::from_str(&json).unwrap();
        assert!(back.metadata.is_none());
    }

    #[test]
    fn policy_check_serde_roundtrip() {
        for check in [PolicyCheck::ManualApproval, PolicyCheck::Automated] {
            let json = serde_json::to_string(&check).unwrap();
            let back: PolicyCheck = serde_json::from_str(&json).unwrap();
            assert_eq!(back, check);
        }
    }

    #[test]
    fn evidence_requirement_serde_roundtrip() {
        let er = EvidenceRequirement {
            fr_id: "FR-99".to_string(),
            evidence_type: EvidenceType::SecurityScan,
        };
        let json = serde_json::to_string(&er).unwrap();
        let back: EvidenceRequirement = serde_json::from_str(&json).unwrap();
        assert_eq!(back.fr_id, "FR-99");
        assert_eq!(back.evidence_type, EvidenceType::SecurityScan);
    }

    #[test]
    fn governance_rule_serde_roundtrip() {
        let gr = GovernanceRule {
            transition: "Draft->Approved".to_string(),
            required_evidence: vec![EvidenceRequirement {
                fr_id: "FR-5".to_string(),
                evidence_type: EvidenceType::ReviewApproval,
            }],
            policy_refs: vec![1, 2, 3],
        };
        let json = serde_json::to_string(&gr).unwrap();
        let back: GovernanceRule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.transition, "Draft->Approved");
        assert_eq!(back.required_evidence.len(), 1);
        assert_eq!(back.policy_refs, vec![1, 2, 3]);
    }

    #[test]
    fn governance_contract_empty_rules() {
        let contract = GovernanceContract {
            id: 1,
            feature_id: 1,
            version: 1,
            rules: vec![],
            bound_at: Utc::now(),
        };
        let json = serde_json::to_string(&contract).unwrap();
        let back: GovernanceContract = serde_json::from_str(&json).unwrap();
        assert!(back.rules.is_empty());
    }

    #[test]
    fn policy_rule_serde_roundtrip() {
        let now = Utc::now();
        let pr = PolicyRule {
            id: 42,
            domain: PolicyDomain::Security,
            rule: PolicyDefinition {
                description: "Must pass security scan".to_string(),
                check: PolicyCheck::Automated,
            },
            active: true,
            created_at: now,
            updated_at: now,
        };
        let json = serde_json::to_string(&pr).unwrap();
        let back: PolicyRule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, 42);
        assert_eq!(back.domain, PolicyDomain::Security);
        assert!(back.active);
        assert_eq!(back.rule.check, PolicyCheck::Automated);
    }

    #[test]
    fn builtin_policy_all_known_refs_exist() {
        let refs = ["tests-pass", "ci-green", "review-approved", "security-scan", "lint-pass"];
        for r in refs {
            assert!(BuiltinPolicy::from_ref(r).is_some(), "missing ref: {r}");
        }
    }
}
