//! Integration tests for governance types and their serialization.

use chrono::Utc;

use traceability_core::{
    BuiltinPolicy, Evidence, EvidenceRequirement, EvidenceType, GovernanceContract,
    GovernanceRule, PolicyCheck, PolicyDefinition, PolicyDomain, PolicyRule,
};

#[test]
fn policy_domain_all_variants_as_str() {
    assert_eq!(PolicyDomain::Security.as_str(), "security");
    assert_eq!(PolicyDomain::Quality.as_str(), "quality");
    assert_eq!(PolicyDomain::Compliance.as_str(), "compliance");
    assert_eq!(PolicyDomain::Performance.as_str(), "performance");
    assert_eq!(PolicyDomain::Custom.as_str(), "custom");
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
fn evidence_type_all_variants_as_str() {
    assert_eq!(EvidenceType::TestResult.as_str(), "test_result");
    assert_eq!(EvidenceType::CiOutput.as_str(), "ci_output");
    assert_eq!(EvidenceType::ReviewApproval.as_str(), "review_approval");
    assert_eq!(EvidenceType::SecurityScan.as_str(), "security_scan");
    assert_eq!(EvidenceType::LintResult.as_str(), "lint_result");
    assert_eq!(EvidenceType::ManualAttestation.as_str(), "manual_attestation");
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
fn policy_check_serde_roundtrip() {
    for pc in [PolicyCheck::ManualApproval, PolicyCheck::Automated] {
        let json = serde_json::to_string(&pc).unwrap();
        let back: PolicyCheck = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pc);
    }
}

#[test]
fn policy_definition_serde_roundtrip() {
    let pd = PolicyDefinition {
        description: "All tests must pass before merge".into(),
        check: PolicyCheck::Automated,
    };
    let json = serde_json::to_string(&pd).unwrap();
    let back: PolicyDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(back.description, "All tests must pass before merge");
    assert_eq!(back.check, PolicyCheck::Automated);
}

#[test]
fn policy_rule_serde_roundtrip() {
    let now = Utc::now();
    let pr = PolicyRule {
        id: 42,
        domain: PolicyDomain::Security,
        rule: PolicyDefinition {
            description: "Security scan required".into(),
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
}

#[test]
fn evidence_requirement_serde_roundtrip() {
    let er = EvidenceRequirement {
        fr_id: "FR-001".into(),
        evidence_type: EvidenceType::TestResult,
    };
    let json = serde_json::to_string(&er).unwrap();
    let back: EvidenceRequirement = serde_json::from_str(&json).unwrap();
    assert_eq!(back.fr_id, "FR-001");
    assert_eq!(back.evidence_type, EvidenceType::TestResult);
}

#[test]
fn governance_rule_serde_roundtrip() {
    let gr = GovernanceRule {
        transition: "Implementing->Validated".into(),
        required_evidence: vec![
            EvidenceRequirement {
                fr_id: "FR-001".into(),
                evidence_type: EvidenceType::TestResult,
            },
            EvidenceRequirement {
                fr_id: "FR-002".into(),
                evidence_type: EvidenceType::CiOutput,
            },
        ],
        policy_refs: vec![1, 2, 3],
    };
    let json = serde_json::to_string(&gr).unwrap();
    let back: GovernanceRule = serde_json::from_str(&json).unwrap();
    assert_eq!(back.transition, "Implementing->Validated");
    assert_eq!(back.required_evidence.len(), 2);
    assert_eq!(back.policy_refs, vec![1, 2, 3]);
}

#[test]
fn governance_contract_full_serde_roundtrip() {
    let now = Utc::now();
    let contract = GovernanceContract {
        id: 100,
        feature_id: 200,
        version: 5,
        rules: vec![
            GovernanceRule {
                transition: "Created->Specified".into(),
                required_evidence: vec![EvidenceRequirement {
                    fr_id: "FR-010".into(),
                    evidence_type: EvidenceType::ReviewApproval,
                }],
                policy_refs: vec![10],
            },
            GovernanceRule {
                transition: "Implementing->Validated".into(),
                required_evidence: vec![
                    EvidenceRequirement {
                        fr_id: "FR-020".into(),
                        evidence_type: EvidenceType::TestResult,
                    },
                    EvidenceRequirement {
                        fr_id: "FR-020".into(),
                        evidence_type: EvidenceType::CiOutput,
                    },
                ],
                policy_refs: vec![20, 30],
            },
        ],
        bound_at: now,
    };
    let json = serde_json::to_string(&contract).unwrap();
    let back: GovernanceContract = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, 100);
    assert_eq!(back.feature_id, 200);
    assert_eq!(back.version, 5);
    assert_eq!(back.rules.len(), 2);
    assert_eq!(back.rules[0].transition, "Created->Specified");
    assert_eq!(back.rules[1].required_evidence.len(), 2);
}

#[test]
fn evidence_serde_roundtrip() {
    let ev = Evidence {
        id: 1,
        wp_id: 100,
        fr_id: "FR-001".into(),
        evidence_type: EvidenceType::TestResult,
        artifact_path: "/ci/results.xml".into(),
        metadata: Some(serde_json::json!({"suite": "unit", "passed": 42})),
        created_at: Utc::now(),
    };
    let json = serde_json::to_string(&ev).unwrap();
    let back: Evidence = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, 1);
    assert_eq!(back.fr_id, "FR-001");
    assert!(back.metadata.is_some());
}

#[test]
fn builtin_policy_all_known_refs() {
    let refs = [
        "tests-pass",
        "ci-green",
        "review-approved",
        "security-scan",
        "lint-pass",
    ];
    for r in refs {
        let p = BuiltinPolicy::from_ref(r);
        assert!(p.is_some(), "builtin policy {r:?} should exist");
    }
    assert!(BuiltinPolicy::from_ref("nonexistent").is_none());
}

#[test]
fn builtin_policy_labels_and_domains() {
    let tp = BuiltinPolicy::from_ref("tests-pass").unwrap();
    assert_eq!(tp.label, "Unit tests passing");
    assert_eq!(tp.domain, PolicyDomain::Quality);
    assert_eq!(tp.evidence_type, EvidenceType::TestResult);

    let ci = BuiltinPolicy::from_ref("ci-green").unwrap();
    assert_eq!(ci.label, "CI pipeline green");
    assert_eq!(ci.evidence_type, EvidenceType::CiOutput);

    let rev = BuiltinPolicy::from_ref("review-approved").unwrap();
    assert_eq!(rev.label, "Peer review approved");
    assert_eq!(rev.evidence_type, EvidenceType::ReviewApproval);

    let sec = BuiltinPolicy::from_ref("security-scan").unwrap();
    assert_eq!(sec.label, "Security scan clean");
    assert_eq!(sec.domain, PolicyDomain::Security);
    assert_eq!(sec.evidence_type, EvidenceType::SecurityScan);

    let lint = BuiltinPolicy::from_ref("lint-pass").unwrap();
    assert_eq!(lint.label, "Lint checks pass");
    assert_eq!(lint.evidence_type, EvidenceType::LintResult);
}
