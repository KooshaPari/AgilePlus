//! Governance types — re-exported from `traceability-core`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Policy domain category.
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
    pub fn as_str(self) -> &'static str {
        match self {
            PolicyDomain::Security => "security",
            PolicyDomain::Quality => "quality",
            PolicyDomain::Compliance => "compliance",
            PolicyDomain::Performance => "performance",
            PolicyDomain::Custom => "custom",
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

impl PolicyRule {
    /// Whether this policy matches a contract policy-reference string.
    pub fn matches_reference(&self, policy_ref: &str) -> bool {
        self.id.to_string() == policy_ref || format!("policy:{}", self.id) == policy_ref
    }
}

/// A governance rule captured inside a contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRule {
    pub transition: String,
    pub required_evidence: Vec<String>,
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
    pub fn as_str(self) -> &'static str {
        match self {
            EvidenceType::TestResult => "test_result",
            EvidenceType::CiOutput => "ci_output",
            EvidenceType::ReviewApproval => "review_approval",
            EvidenceType::SecurityScan => "security_scan",
            EvidenceType::LintResult => "lint_result",
            EvidenceType::ManualAttestation => "manual_attestation",
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCheck {
    ManualApproval,
    Automated,
    EvidencePresent { evidence_type: EvidenceType },
    ThresholdMet { metric: String, min: f64 },
    Custom { script: String },
}

/// A well-known built-in policy that maps a short reference key to a
/// `PolicyDomain` + `EvidenceType` pair.  Used by the `validate` command to
/// resolve policy references without requiring a database lookup.
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
    /// Well-known built-in policies keyed by their reference string.
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
    /// Returns `None` for unknown (custom) policy references.
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
    use traceability_core::governance::{
        EvidenceRequirement, EvidenceType as TcEvidenceType, GovernanceRule as TcGovernanceRule,
    };

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
        assert_eq!(EvidenceType::ReviewApproval.as_str(), "review_approval");
        assert_eq!(EvidenceType::SecurityScan.as_str(), "security_scan");
        assert_eq!(EvidenceType::LintResult.as_str(), "lint_result");
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
    fn policy_domain_serde_roundtrip() {
        let json = serde_json::to_string(&PolicyDomain::Security).unwrap();
        let back: PolicyDomain = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PolicyDomain::Security);
    }

    #[test]
    fn evidence_type_serde_snake_case() {
        let json = serde_json::to_string(&EvidenceType::TestResult).unwrap();
        assert_eq!(json, "\"test_result\"");
    }

    #[test]
    fn policy_definition_construction_and_serde() {
        let def = PolicyDefinition {
            description: "All tests must pass".to_string(),
            check: PolicyCheck::Automated,
        };
        assert_eq!(def.description, "All tests must pass");
        assert_eq!(def.check, PolicyCheck::Automated);

        let json = serde_json::to_string(&def).unwrap();
        let back: PolicyDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(back.description, def.description);
        assert_eq!(back.check, def.check);
    }

    #[test]
    fn policy_rule_construction_and_serde() {
        let now = Utc::now();
        let rule = PolicyRule {
            id: 1,
            domain: PolicyDomain::Quality,
            rule: PolicyDefinition {
                description: "Test rule".to_string(),
                check: PolicyCheck::ManualApproval,
            },
            active: true,
            created_at: now,
            updated_at: now,
        };
        assert_eq!(rule.id, 1);
        assert_eq!(rule.domain, PolicyDomain::Quality);
        assert!(rule.active);

        let json = serde_json::to_string(&rule).unwrap();
        let back: PolicyRule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, rule.id);
        assert_eq!(back.domain, rule.domain);
        assert_eq!(back.active, rule.active);
        assert_eq!(back.rule.description, rule.rule.description);
        assert_eq!(back.rule.check, rule.rule.check);
    }

    #[test]
    fn evidence_requirement_construction_and_serde() {
        let req = EvidenceRequirement {
            fr_id: "FR-001".to_string(),
            evidence_type: TcEvidenceType::TestResult,
        };
        assert_eq!(req.fr_id, "FR-001");
        assert_eq!(req.evidence_type, TcEvidenceType::TestResult);

        let json = serde_json::to_string(&req).unwrap();
        let back: EvidenceRequirement = serde_json::from_str(&json).unwrap();
        assert_eq!(back.fr_id, req.fr_id);
        assert_eq!(back.evidence_type, req.evidence_type);
    }

    #[test]
    fn governance_rule_construction_and_serde() {
        let rule = TcGovernanceRule {
            transition: "Draft->Active".to_string(),
            required_evidence: vec![
                EvidenceRequirement {
                    fr_id: "FR-001".to_string(),
                    evidence_type: TcEvidenceType::TestResult,
                },
                EvidenceRequirement {
                    fr_id: "FR-002".to_string(),
                    evidence_type: TcEvidenceType::ReviewApproval,
                },
            ],
            policy_refs: vec![1, 2, 3],
        };
        assert_eq!(rule.transition, "Draft->Active");
        assert_eq!(rule.required_evidence.len(), 2);
        assert_eq!(rule.policy_refs, vec![1, 2, 3]);

        let json = serde_json::to_string(&rule).unwrap();
        let back: TcGovernanceRule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.transition, rule.transition);
        assert_eq!(back.required_evidence.len(), rule.required_evidence.len());
        assert_eq!(back.policy_refs, rule.policy_refs);
    }

    #[test]
    fn governance_contract_construction_and_serde() {
        let now = Utc::now();
        let contract = GovernanceContract {
            id: 42,
            feature_id: 100,
            version: 3,
            rules: vec![GovernanceRule {
                transition: "Active->Done".to_string(),
                required_evidence: vec![],
                policy_refs: vec![1],
            }],
            bound_at: now,
        };
        assert_eq!(contract.id, 42);
        assert_eq!(contract.feature_id, 100);
        assert_eq!(contract.version, 3);
        assert_eq!(contract.rules.len(), 1);

        let json = serde_json::to_string(&contract).unwrap();
        let back: GovernanceContract = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, contract.id);
        assert_eq!(back.feature_id, contract.feature_id);
        assert_eq!(back.version, contract.version);
        assert_eq!(back.rules.len(), contract.rules.len());
    }

    #[test]
    fn evidence_construction_and_serde() {
        let now = Utc::now();
        let evidence = Evidence {
            id: 1,
            wp_id: 5,
            fr_id: "FR-001".to_string(),
            evidence_type: EvidenceType::SecurityScan,
            artifact_path: "/path/to/scan.json".to_string(),
            metadata: Some(serde_json::json!({"scanner": "trivy"})),
            created_at: now,
        };
        assert_eq!(evidence.id, 1);
        assert_eq!(evidence.wp_id, 5);
        assert_eq!(evidence.fr_id, "FR-001");
        assert_eq!(evidence.evidence_type, EvidenceType::SecurityScan);
        assert_eq!(evidence.artifact_path, "/path/to/scan.json");
        assert!(evidence.metadata.is_some());

        let json = serde_json::to_string(&evidence).unwrap();
        let back: Evidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, evidence.id);
        assert_eq!(back.wp_id, evidence.wp_id);
        assert_eq!(back.fr_id, evidence.fr_id);
        assert_eq!(back.evidence_type, evidence.evidence_type);
        assert_eq!(back.artifact_path, evidence.artifact_path);
        assert_eq!(back.metadata, evidence.metadata);
    }

    #[test]
    fn evidence_without_metadata_serde() {
        let now = Utc::now();
        let evidence = Evidence {
            id: 2,
            wp_id: 6,
            fr_id: "FR-002".to_string(),
            evidence_type: EvidenceType::LintResult,
            artifact_path: "/path/to/lint.json".to_string(),
            metadata: None,
            created_at: now,
        };
        let json = serde_json::to_string(&evidence).unwrap();
        let back: Evidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back.metadata, None);
    }

    #[test]
    fn policy_check_serde() {
        let manual = PolicyCheck::ManualApproval;
        let automated = PolicyCheck::Automated;

        let json_manual = serde_json::to_string(&manual).unwrap();
        let json_auto = serde_json::to_string(&automated).unwrap();

        let back_manual: PolicyCheck = serde_json::from_str(&json_manual).unwrap();
        let back_auto: PolicyCheck = serde_json::from_str(&json_auto).unwrap();

        assert_eq!(back_manual, PolicyCheck::ManualApproval);
        assert_eq!(back_auto, PolicyCheck::Automated);
    }

    // --- Additional tests for uncovered paths ---

    #[test]
    fn evidence_type_serde_roundtrip_all_variants() {
        let variants = [
            EvidenceType::TestResult,
            EvidenceType::CiOutput,
            EvidenceType::ReviewApproval,
            EvidenceType::SecurityScan,
            EvidenceType::LintResult,
            EvidenceType::ManualAttestation,
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let back: EvidenceType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn evidence_type_serde_snake_case_all() {
        assert_eq!(
            serde_json::to_string(&EvidenceType::CiOutput).unwrap(),
            "\"ci_output\""
        );
        assert_eq!(
            serde_json::to_string(&EvidenceType::ReviewApproval).unwrap(),
            "\"review_approval\""
        );
        assert_eq!(
            serde_json::to_string(&EvidenceType::SecurityScan).unwrap(),
            "\"security_scan\""
        );
        assert_eq!(
            serde_json::to_string(&EvidenceType::LintResult).unwrap(),
            "\"lint_result\""
        );
        assert_eq!(
            serde_json::to_string(&EvidenceType::ManualAttestation).unwrap(),
            "\"manual_attestation\""
        );
    }

    #[test]
    fn policy_domain_serde_roundtrip_all_variants() {
        let variants = [
            PolicyDomain::Security,
            PolicyDomain::Quality,
            PolicyDomain::Compliance,
            PolicyDomain::Performance,
            PolicyDomain::Custom,
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let back: PolicyDomain = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn policy_domain_serde_snake_case_all() {
        assert_eq!(
            serde_json::to_string(&PolicyDomain::Compliance).unwrap(),
            "\"compliance\""
        );
        assert_eq!(
            serde_json::to_string(&PolicyDomain::Performance).unwrap(),
            "\"performance\""
        );
        assert_eq!(
            serde_json::to_string(&PolicyDomain::Custom).unwrap(),
            "\"custom\""
        );
    }

    #[test]
    fn policy_check_complex_variants_serde() {
        // EvidencePresent
        let ep = PolicyCheck::EvidencePresent {
            evidence_type: EvidenceType::TestResult,
        };
        let json = serde_json::to_string(&ep).unwrap();
        let back: PolicyCheck = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ep);

        // ThresholdMet
        let tm = PolicyCheck::ThresholdMet {
            metric: "coverage".to_string(),
            min: 80.0,
        };
        let json = serde_json::to_string(&tm).unwrap();
        let back: PolicyCheck = serde_json::from_str(&json).unwrap();
        assert_eq!(back, tm);

        // Custom
        let c = PolicyCheck::Custom {
            script: "check.sh".to_string(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: PolicyCheck = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn policy_rule_matches_reference() {
        let rule = PolicyRule {
            id: 42,
            domain: PolicyDomain::Quality,
            rule: PolicyDefinition {
                description: "Test".to_string(),
                check: PolicyCheck::Automated,
            },
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(rule.matches_reference("42"));
        assert!(rule.matches_reference("policy:42"));
        assert!(!rule.matches_reference("43"));
        assert!(!rule.matches_reference("policy:43"));
        assert!(!rule.matches_reference(""));
        assert!(!rule.matches_reference("policy:"));
    }

    #[test]
    fn builtin_policy_all_known_refs_resolve() {
        let known = ["tests-pass", "ci-green", "review-approved", "security-scan", "lint-pass"];
        for key in known {
            assert!(
                BuiltinPolicy::from_ref(key).is_some(),
                "expected known ref: {key}"
            );
        }
    }

    #[test]
    fn builtin_policy_labels_are_nonempty() {
        let known = ["tests-pass", "ci-green", "review-approved", "security-scan", "lint-pass"];
        for key in known {
            let bp = BuiltinPolicy::from_ref(key).unwrap();
            assert!(!bp.label.is_empty(), "label should not be empty for {key}");
        }
    }

    #[test]
    fn governance_rule_serde_empty_evidence() {
        let rule = GovernanceRule {
            transition: "A->B".to_string(),
            required_evidence: vec![],
            policy_refs: vec![],
        };
        let json = serde_json::to_string(&rule).unwrap();
        let back: GovernanceRule = serde_json::from_str(&json).unwrap();
        assert_eq!(back.required_evidence.len(), 0);
        assert_eq!(back.policy_refs.len(), 0);
    }

    #[test]
    fn evidence_serde_null_metadata() {
        let evidence = Evidence {
            id: 10,
            wp_id: 1,
            fr_id: "FR-10".to_string(),
            evidence_type: EvidenceType::CiOutput,
            artifact_path: "/ci/output.json".to_string(),
            metadata: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&evidence).unwrap();
        let back: Evidence = serde_json::from_str(&json).unwrap();
        assert!(back.metadata.is_none());
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn builtin_policy_full_table() {
        let expected: &[(&str, PolicyDomain, EvidenceType, &str)] = &[
            ("tests-pass", PolicyDomain::Quality, EvidenceType::TestResult, "Unit tests passing"),
            ("ci-green", PolicyDomain::Quality, EvidenceType::CiOutput, "CI pipeline green"),
            ("review-approved", PolicyDomain::Quality, EvidenceType::ReviewApproval, "Peer review approved"),
            ("security-scan", PolicyDomain::Security, EvidenceType::SecurityScan, "Security scan clean"),
            ("lint-pass", PolicyDomain::Quality, EvidenceType::LintResult, "Lint checks pass"),
        ];
        for (key, domain, evidence, label) in expected {
            let bp = BuiltinPolicy::from_ref(key).unwrap_or_else(|| panic!("missing {key}"));
            assert_eq!(bp.domain, *domain, "{key}");
            assert_eq!(bp.evidence_type, *evidence, "{key}");
            assert_eq!(bp.label, *label, "{key}");
        }
    }

    #[test]
    fn builtin_policy_lookup_is_exact_case_sensitive() {
        assert!(BuiltinPolicy::from_ref("Tests-Pass").is_none());
        assert!(BuiltinPolicy::from_ref("TESTS-PASS").is_none());
        assert!(BuiltinPolicy::from_ref(" tests-pass").is_none());
        assert!(BuiltinPolicy::from_ref("tests-pass ").is_none());
    }

    #[test]
    fn builtin_policy_is_copy_and_debug() {
        let bp = *BuiltinPolicy::from_ref("ci-green").unwrap();
        assert_eq!(bp.domain, PolicyDomain::Quality);
        assert!(!format!("{bp:?}").is_empty());
    }

    #[test]
    fn policy_domain_equality() {
        assert_eq!(PolicyDomain::Security, PolicyDomain::Security);
        assert_ne!(PolicyDomain::Quality, PolicyDomain::Compliance);
        assert_ne!(PolicyDomain::Custom, PolicyDomain::Performance);
    }

    #[test]
    fn evidence_type_equality() {
        assert_eq!(EvidenceType::TestResult, EvidenceType::TestResult);
        assert_ne!(EvidenceType::TestResult, EvidenceType::CiOutput);
        assert_ne!(EvidenceType::LintResult, EvidenceType::ManualAttestation);
    }

    #[test]
    fn policy_domain_debug_and_copy() {
        let d = PolicyDomain::Performance;
        let copy = d;
        assert_eq!(format!("{d:?}"), "Performance");
        assert_eq!(copy, d);
    }

    #[test]
    fn evidence_type_debug_and_copy() {
        let e = EvidenceType::ManualAttestation;
        let copy = e;
        assert_eq!(format!("{e:?}"), "ManualAttestation");
        assert_eq!(copy, e);
    }

    #[test]
    fn policy_rule_matches_reference_matrix() {
        let rule = PolicyRule {
            id: 0,
            domain: PolicyDomain::Custom,
            rule: PolicyDefinition {
                description: "d".into(),
                check: PolicyCheck::Automated,
            },
            active: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(rule.matches_reference("0"));
        assert!(rule.matches_reference("policy:0"));
        assert!(!rule.matches_reference(" 0"));
        assert!(!rule.matches_reference("policy:0 "));
        assert!(!rule.matches_reference("policy:00"));
        assert!(!rule.matches_reference("policy:"));
    }

    #[test]
    fn policy_check_equality_all_variants() {
        assert_eq!(PolicyCheck::ManualApproval, PolicyCheck::ManualApproval);
        assert_ne!(PolicyCheck::ManualApproval, PolicyCheck::Automated);
        assert_eq!(
            PolicyCheck::EvidencePresent { evidence_type: EvidenceType::CiOutput },
            PolicyCheck::EvidencePresent { evidence_type: EvidenceType::CiOutput }
        );
        assert_ne!(
            PolicyCheck::EvidencePresent { evidence_type: EvidenceType::CiOutput },
            PolicyCheck::EvidencePresent { evidence_type: EvidenceType::LintResult }
        );
        assert_ne!(
            PolicyCheck::ThresholdMet { metric: "c".into(), min: 1.0 },
            PolicyCheck::ThresholdMet { metric: "c".into(), min: 2.0 }
        );
    }

    #[test]
    fn policy_check_wire_strings() {
        assert_eq!(
            serde_json::to_string(&PolicyCheck::ManualApproval).unwrap(),
            "\"manual_approval\""
        );
        assert_eq!(
            serde_json::to_string(&PolicyCheck::Automated).unwrap(),
            "\"automated\""
        );
        let ep = serde_json::to_string(&PolicyCheck::EvidencePresent {
            evidence_type: EvidenceType::TestResult,
        })
        .unwrap();
        assert!(ep.contains("evidence_present"), "got {ep}");
    }

    #[test]
    fn policy_definition_clone_and_debug() {
        let d = PolicyDefinition {
            description: "desc".into(),
            check: PolicyCheck::Custom { script: "s.sh".into() },
        };
        let c = d.clone();
        assert_eq!(c.description, d.description);
        assert!(!format!("{d:?}").is_empty());
    }

    #[test]
    fn governance_rule_and_contract_clone() {
        let rule = GovernanceRule {
            transition: "A->B".into(),
            required_evidence: vec!["FR-1".into()],
            policy_refs: vec![7],
        };
        let c = rule.clone();
        assert_eq!(c.policy_refs, vec![7]);
        let contract = GovernanceContract {
            id: 1,
            feature_id: 2,
            version: 3,
            rules: vec![rule],
            bound_at: Utc::now(),
        };
        let cc = contract.clone();
        assert_eq!(cc.rules.len(), 1);
        assert!(!format!("{contract:?}").is_empty());
    }

    #[test]
    fn evidence_metadata_serde_with_nested_json() {
        let e = Evidence {
            id: 3,
            wp_id: 4,
            fr_id: "FR-3".into(),
            evidence_type: EvidenceType::SecurityScan,
            artifact_path: "/a".into(),
            metadata: Some(serde_json::json!({"nested": {"k": [1, 2, 3]}})),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&e).unwrap();
        let back: Evidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back.metadata, e.metadata);
    }

    #[test]
    fn policy_rule_inactive_flag_roundtrip() {
        let rule = PolicyRule {
            id: 9,
            domain: PolicyDomain::Compliance,
            rule: PolicyDefinition {
                description: "d".into(),
                check: PolicyCheck::ManualApproval,
            },
            active: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let back: PolicyRule =
            serde_json::from_str(&serde_json::to_string(&rule).unwrap()).unwrap();
        assert!(!back.active);
        assert_eq!(back.id, 9);
    }

    #[test]
    fn policy_domain_wire_strings_all() {
        for (d, s) in [
            (PolicyDomain::Security, "\"security\""),
            (PolicyDomain::Quality, "\"quality\""),
            (PolicyDomain::Compliance, "\"compliance\""),
            (PolicyDomain::Performance, "\"performance\""),
            (PolicyDomain::Custom, "\"custom\""),
        ] {
            assert_eq!(serde_json::to_string(&d).unwrap(), s);
        }
    }
}
