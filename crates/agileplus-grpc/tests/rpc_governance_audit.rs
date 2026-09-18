//! RPC handler tests for the governance gate and the audit trail.
//!
//! Covers `CheckGovernanceGate`, the `GetAuditTrail` server stream, and
//! `VerifyAuditChain` against a real hash-chained store, including a tampered
//! row that verification must reject.
//!
//! Traceability: WP14-T079, T081

mod support;

use agileplus_domain::domain::governance::EvidenceType;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WpState;
use agileplus_proto::agileplus::v1::agile_plus_core_service_server::AgilePlusCoreService;
use agileplus_proto::agileplus::v1::{
    CheckGovernanceGateRequest, GetAuditTrailRequest, VerifyAuditChainRequest,
};
use futures::StreamExt;
use support::{Harness, rule, scope};
use tonic::{Code, Request};

// ---------------------------------------------------------------------------
// CheckGovernanceGate
// ---------------------------------------------------------------------------

#[tokio::test]
async fn check_governance_gate_without_contract_passes() {
    let harness = Harness::new().await;
    harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;

    let response = harness
        .server
        .check_governance_gate(Request::new(CheckGovernanceGateRequest {
            feature_slug: "alpha".into(),
            transition: "validate".into(),
            project_scope: scope(),
        }))
        .await
        .expect("a contract-less feature should still answer")
        .into_inner();

    assert!(response.passed);
    assert!(response.violations.is_empty());
}

#[tokio::test]
async fn check_governance_gate_passes_when_evidence_is_present() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    let wp = harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness
        .seed_evidence(wp.id, "FR-9", EvidenceType::CiOutput)
        .await;
    harness
        .seed_contract(feature.id, 1, vec![rule("validate", &["FR-9:ci_output"])])
        .await;

    let response = harness
        .server
        .check_governance_gate(Request::new(CheckGovernanceGateRequest {
            feature_slug: "alpha".into(),
            transition: "validate".into(),
            project_scope: scope(),
        }))
        .await
        .expect("gate check should succeed")
        .into_inner();

    assert!(response.passed, "violations: {:?}", response.violations);
    assert!(response.violations.is_empty());
}

#[tokio::test]
async fn check_governance_gate_reports_missing_evidence() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness
        .seed_contract(feature.id, 1, vec![rule("validate", &["FR-9:ci_output"])])
        .await;

    let response = harness
        .server
        .check_governance_gate(Request::new(CheckGovernanceGateRequest {
            feature_slug: "alpha".into(),
            transition: "validate".into(),
            project_scope: scope(),
        }))
        .await
        .expect("gate check should succeed")
        .into_inner();

    assert!(!response.passed);
    assert_eq!(response.violations.len(), 1);
    let violation = &response.violations[0];
    assert_eq!(violation.fr_id, "FR-9");
    assert_eq!(violation.rule_id, "validate");
    assert!(violation.message.contains("FR-9:ci_output"));
    assert!(violation.remediation.contains("FR-9:ci_output"));
}

#[tokio::test]
async fn check_governance_gate_requires_the_declared_evidence_type() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    let wp = harness.seed_wp(feature.id, 1, WpState::Done).await;
    // Evidence exists for the requirement, but of the wrong type.
    harness
        .seed_evidence(wp.id, "FR-9", EvidenceType::LintResult)
        .await;
    harness
        .seed_contract(feature.id, 1, vec![rule("validate", &["FR-9:ci_output"])])
        .await;

    let response = harness
        .server
        .check_governance_gate(Request::new(CheckGovernanceGateRequest {
            feature_slug: "alpha".into(),
            transition: "validate".into(),
            project_scope: scope(),
        }))
        .await
        .expect("gate check should succeed")
        .into_inner();

    assert!(
        !response.passed,
        "a type mismatch must not satisfy the rule"
    );
    assert_eq!(response.violations.len(), 1);
    assert_eq!(response.violations[0].fr_id, "FR-9");
}

#[tokio::test]
async fn check_governance_gate_accepts_any_type_when_none_is_declared() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    let wp = harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness
        .seed_evidence(wp.id, "FR-1", EvidenceType::ManualAttestation)
        .await;
    harness
        .seed_contract(feature.id, 1, vec![rule("validate", &["FR-1"])])
        .await;

    let response = harness
        .server
        .check_governance_gate(Request::new(CheckGovernanceGateRequest {
            feature_slug: "alpha".into(),
            transition: "validate".into(),
            project_scope: scope(),
        }))
        .await
        .expect("gate check should succeed")
        .into_inner();

    assert!(response.passed);
}

#[tokio::test]
async fn check_governance_gate_flags_unrecognized_evidence_types() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    let wp = harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness
        .seed_evidence(wp.id, "FR-9", EvidenceType::CiOutput)
        .await;
    // The contract asks for a type the wire contract does not define.
    harness
        .seed_contract(feature.id, 1, vec![rule("validate", &["FR-9:screenshot"])])
        .await;

    let response = harness
        .server
        .check_governance_gate(Request::new(CheckGovernanceGateRequest {
            feature_slug: "alpha".into(),
            transition: "validate".into(),
            project_scope: scope(),
        }))
        .await
        .expect("gate check should succeed")
        .into_inner();

    assert!(!response.passed);
    assert_eq!(response.violations.len(), 1);
    assert_eq!(response.violations[0].fr_id, "FR-9");
    assert!(response.violations[0].message.contains("FR-9:screenshot"));
}

#[tokio::test]
async fn check_governance_gate_ignores_rules_bound_to_other_transitions() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness
        .seed_contract(feature.id, 1, vec![rule("ship", &["FR-9:ci_output"])])
        .await;

    let response = harness
        .server
        .check_governance_gate(Request::new(CheckGovernanceGateRequest {
            feature_slug: "alpha".into(),
            transition: "validate".into(),
            project_scope: scope(),
        }))
        .await
        .expect("gate check should succeed")
        .into_inner();

    assert!(response.passed, "the ship rule must not gate validate");
}

#[tokio::test]
async fn check_governance_gate_applies_rules_without_a_transition_to_all_requests() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness
        .seed_contract(feature.id, 1, vec![rule("", &["FR-2:test_result"])])
        .await;

    let response = harness
        .server
        .check_governance_gate(Request::new(CheckGovernanceGateRequest {
            feature_slug: "alpha".into(),
            transition: "ship".into(),
            project_scope: scope(),
        }))
        .await
        .expect("gate check should succeed")
        .into_inner();

    assert!(!response.passed);
    assert_eq!(response.violations.len(), 1);
    assert_eq!(response.violations[0].rule_id, "");
    assert_eq!(response.violations[0].fr_id, "FR-2");
}

#[tokio::test]
async fn check_governance_gate_ignores_evidence_from_another_feature() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    let other = harness
        .seed_feature("beta", FeatureState::Implementing)
        .await;
    let other_wp = harness.seed_wp(other.id, 1, WpState::Done).await;
    harness
        .seed_evidence(other_wp.id, "FR-9", EvidenceType::CiOutput)
        .await;
    harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness
        .seed_contract(feature.id, 1, vec![rule("validate", &["FR-9:ci_output"])])
        .await;

    let response = harness
        .server
        .check_governance_gate(Request::new(CheckGovernanceGateRequest {
            feature_slug: "alpha".into(),
            transition: "validate".into(),
            project_scope: scope(),
        }))
        .await
        .expect("gate check should succeed")
        .into_inner();

    assert!(
        !response.passed,
        "evidence attached to another feature's work package must not count"
    );
}

#[tokio::test]
async fn check_governance_gate_unknown_feature_is_not_found() {
    let harness = Harness::new().await;

    let status = harness
        .server
        .check_governance_gate(Request::new(CheckGovernanceGateRequest {
            feature_slug: "ghost".into(),
            transition: "validate".into(),
            project_scope: scope(),
        }))
        .await
        .expect_err("missing feature must fail");

    assert_eq!(status.code(), Code::NotFound);
    assert!(status.message().contains("ghost"));
}

#[tokio::test]
async fn check_governance_gate_requires_project_scope() {
    let harness = Harness::new().await;
    harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;

    let status = harness
        .server
        .check_governance_gate(Request::new(CheckGovernanceGateRequest {
            feature_slug: "alpha".into(),
            transition: "validate".into(),
            project_scope: None,
        }))
        .await
        .expect_err("a scope-less request must be rejected");

    assert_eq!(status.code(), Code::InvalidArgument);
}

// ---------------------------------------------------------------------------
// GetAuditTrail (server stream)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_audit_trail_streams_entries_in_order_with_feature_slug() {
    let harness = Harness::new().await;
    let feature = harness.seed_feature("alpha", FeatureState::Shipped).await;
    let first = harness.seed_audit_entry(feature.id, "alice").await;
    let second = harness.seed_audit_entry(feature.id, "bob").await;

    let stream = harness
        .server
        .get_audit_trail(Request::new(GetAuditTrailRequest {
            feature_slug: "alpha".into(),
            after_id: 0,
            project_scope: scope(),
        }))
        .await
        .expect("audit trail should be streamed")
        .into_inner();
    let entries: Vec<_> = stream
        .map(|item| item.expect("audit entry should not fail").audit_entry)
        .collect()
        .await;

    assert_eq!(entries.len(), 2);
    let first_proto = entries[0].as_ref().expect("entry payload");
    let second_proto = entries[1].as_ref().expect("entry payload");

    assert_eq!(first_proto.id, first.id);
    assert_eq!(first_proto.actor, "alice");
    assert_eq!(first_proto.transition, "alice->recorded");
    // The handler back-fills the slug from the request context.
    assert_eq!(first_proto.feature_slug, "alpha");
    assert_eq!(first_proto.hash, first.hash.to_vec());
    assert_eq!(first_proto.prev_hash, vec![0u8; 32]);

    assert_eq!(second_proto.id, second.id);
    assert_eq!(second_proto.actor, "bob");
    assert_eq!(second_proto.prev_hash, first.hash.to_vec());
    assert_eq!(second_proto.hash, second.hash.to_vec());
    assert!(second_proto.id > first_proto.id);
}

#[tokio::test]
async fn get_audit_trail_skips_entries_at_or_before_after_id() {
    let harness = Harness::new().await;
    let feature = harness.seed_feature("alpha", FeatureState::Shipped).await;
    let first = harness.seed_audit_entry(feature.id, "alice").await;
    let second = harness.seed_audit_entry(feature.id, "bob").await;

    let entries: Vec<_> = harness
        .server
        .get_audit_trail(Request::new(GetAuditTrailRequest {
            feature_slug: "alpha".into(),
            after_id: first.id,
            project_scope: scope(),
        }))
        .await
        .expect("audit trail should be streamed")
        .into_inner()
        .map(|item| item.expect("audit entry should not fail").audit_entry)
        .collect()
        .await;

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].as_ref().expect("entry payload").id, second.id);
}

#[tokio::test]
async fn get_audit_trail_is_empty_for_a_feature_without_entries() {
    let harness = Harness::new().await;
    harness.seed_feature("alpha", FeatureState::Created).await;

    let entries: Vec<_> = harness
        .server
        .get_audit_trail(Request::new(GetAuditTrailRequest {
            feature_slug: "alpha".into(),
            after_id: 0,
            project_scope: scope(),
        }))
        .await
        .expect("audit trail should be streamed")
        .into_inner()
        .collect()
        .await;

    assert!(entries.is_empty());
}

#[tokio::test]
async fn get_audit_trail_unknown_feature_is_not_found() {
    let harness = Harness::new().await;

    let status = match harness
        .server
        .get_audit_trail(Request::new(GetAuditTrailRequest {
            feature_slug: "ghost".into(),
            after_id: 0,
            project_scope: scope(),
        }))
        .await
    {
        Ok(_) => panic!("missing feature must fail"),
        Err(status) => status,
    };

    assert_eq!(status.code(), Code::NotFound);
    assert!(status.message().contains("ghost"));
}

// ---------------------------------------------------------------------------
// VerifyAuditChain
// ---------------------------------------------------------------------------

#[tokio::test]
async fn verify_audit_chain_reports_a_valid_chain() {
    let harness = Harness::new().await;
    let feature = harness.seed_feature("alpha", FeatureState::Shipped).await;
    harness.seed_audit_entry(feature.id, "alice").await;
    harness.seed_audit_entry(feature.id, "bob").await;

    let response = harness
        .server
        .verify_audit_chain(Request::new(VerifyAuditChainRequest {
            feature_slug: "alpha".into(),
            project_scope: scope(),
        }))
        .await
        .expect("verification should succeed")
        .into_inner();

    assert!(response.valid);
    assert_eq!(response.entries_verified, 2);
    assert!(response.first_invalid_id.is_empty());
    assert!(response.error_message.is_empty());
}

#[tokio::test]
async fn verify_audit_chain_detects_a_tampered_entry() {
    let harness = Harness::new().await;
    let feature = harness.seed_feature("alpha", FeatureState::Shipped).await;
    harness.seed_audit_entry(feature.id, "alice").await;
    harness.seed_tampered_audit_entry(feature.id, "bob").await;

    let response = harness
        .server
        .verify_audit_chain(Request::new(VerifyAuditChainRequest {
            feature_slug: "alpha".into(),
            project_scope: scope(),
        }))
        .await
        .expect("verification should succeed")
        .into_inner();

    assert!(!response.valid, "a rewritten row must break the chain");
    assert_eq!(response.entries_verified, 0);
    assert!(response.error_message.contains("hash mismatch"));
}

#[tokio::test]
async fn verify_audit_chain_rejects_an_empty_chain() {
    let harness = Harness::new().await;
    harness.seed_feature("alpha", FeatureState::Created).await;

    let response = harness
        .server
        .verify_audit_chain(Request::new(VerifyAuditChainRequest {
            feature_slug: "alpha".into(),
            project_scope: scope(),
        }))
        .await
        .expect("verification should succeed")
        .into_inner();

    assert!(!response.valid);
    assert_eq!(response.entries_verified, 0);
    assert_eq!(response.error_message, "empty audit chain");
}

#[tokio::test]
async fn verify_audit_chain_unknown_feature_is_not_found() {
    let harness = Harness::new().await;

    let status = harness
        .server
        .verify_audit_chain(Request::new(VerifyAuditChainRequest {
            feature_slug: "ghost".into(),
            project_scope: scope(),
        }))
        .await
        .expect_err("missing feature must fail");

    assert_eq!(status.code(), Code::NotFound);
    assert!(status.message().contains("ghost"));
}

#[tokio::test]
async fn verify_audit_chain_requires_project_scope() {
    let harness = Harness::new().await;
    harness.seed_feature("alpha", FeatureState::Shipped).await;

    let status = harness
        .server
        .verify_audit_chain(Request::new(VerifyAuditChainRequest {
            feature_slug: "alpha".into(),
            project_scope: None,
        }))
        .await
        .expect_err("a scope-less request must be rejected");

    assert_eq!(status.code(), Code::InvalidArgument);
}
