// SPDX-License-Identifier: MIT OR Apache-2.0
//! Unit tests for the gRPC server's pure helpers.
//!
//! These run inside the crate so the private scope/evidence helpers can be
//! exercised directly, without standing up a tonic server.

use super::{
    domain_error_to_status, evidence_satisfies_requirement, missing_evidence_violation,
    parse_evidence_requirement, validate_project_scope,
};
use agileplus_domain::domain::governance::{
    Evidence, EvidenceType, GovernanceRule,
};
use chrono::Utc;
use std::collections::HashSet;
use tonic::Code;

fn evidence(id: i64, wp_id: i64, fr_id: &str, kind: EvidenceType) -> Evidence {
    Evidence {
        id,
        wp_id,
        fr_id: fr_id.to_string(),
        evidence_type: kind,
        artifact_path: format!("artifacts/{id}.txt"),
        metadata: None,
        created_at: Utc::now(),
    }
}

fn rule(transition: &str, required: &[&str]) -> GovernanceRule {
    GovernanceRule {
        transition: transition.to_string(),
        required_evidence: required.iter().map(|r| r.to_string()).collect(),
        policy_refs: Vec::new(),
    }
}

// ── domain_error_to_status ───────────────────────────────────────────────────

#[test]
fn domain_error_mapping() {
    use agileplus_domain::error::DomainError;

    let status = domain_error_to_status(DomainError::NotFound("feat".into()));
    assert_eq!(status.code(), Code::NotFound);
    assert_eq!(status.message(), "feat");

    let status = domain_error_to_status(DomainError::InvalidTransition {
        from: "a".into(),
        to: "b".into(),
        reason: "test".into(),
    });
    assert_eq!(status.code(), Code::FailedPrecondition);
    assert_eq!(status.message(), "invalid transition a->b: test");

    let status = domain_error_to_status(DomainError::Conflict("x".into()));
    assert_eq!(status.code(), Code::AlreadyExists);
    assert_eq!(status.message(), "x");
}

#[test]
fn not_implemented_maps_to_unimplemented() {
    use agileplus_domain::error::DomainError;

    let status = domain_error_to_status(DomainError::NotImplemented);
    assert_eq!(status.code(), Code::Unimplemented);
    assert_eq!(status.message(), "not implemented");
}

#[test]
fn catch_all_variant_maps_to_internal_with_original_text() {
    use agileplus_domain::error::DomainError;

    // A validation error is not one of the explicitly mapped variants, so it
    // must fall through to `internal` while preserving the domain message.
    let domain = DomainError::Validation("slug must not be empty".into());
    let expected = domain.to_string();
    let status = domain_error_to_status(domain);

    assert_eq!(status.code(), Code::Internal);
    assert_eq!(status.message(), expected);
}

// ── parse_evidence_requirement ───────────────────────────────────────────────

#[test]
fn requirement_without_type_is_recognized_and_untyped() {
    let (fr_id, kind, recognized) = parse_evidence_requirement("FR-12");
    assert_eq!(fr_id, "FR-12");
    assert_eq!(kind, None);
    assert!(recognized, "a bare FR id is a valid requirement");
}

#[test]
fn every_documented_evidence_type_round_trips() {
    let expected = [
        ("test_result", EvidenceType::TestResult),
        ("ci_output", EvidenceType::CiOutput),
        ("review_approval", EvidenceType::ReviewApproval),
        ("security_scan", EvidenceType::SecurityScan),
        ("lint_result", EvidenceType::LintResult),
        ("manual_attestation", EvidenceType::ManualAttestation),
    ];

    for (raw_type, expected_kind) in expected {
        let raw = format!("FR-7:{raw_type}");
        let (fr_id, kind, recognized) = parse_evidence_requirement(&raw);
        assert_eq!(fr_id, "FR-7", "raw: {raw}");
        assert_eq!(kind, Some(expected_kind), "raw: {raw}");
        assert!(recognized, "raw: {raw} should be recognized");
    }
}

#[test]
fn unknown_evidence_type_is_reported_as_unrecognized() {
    let (fr_id, kind, recognized) = parse_evidence_requirement("FR-7:screenshot");
    assert_eq!(fr_id, "FR-7");
    assert_eq!(kind, None);
    assert!(
        !recognized,
        "an unknown type must be flagged so callers can reject it"
    );
}

#[test]
fn evidence_type_matching_is_case_sensitive() {
    let (_, kind, recognized) = parse_evidence_requirement("FR-7:Test_Result");
    assert_eq!(kind, None);
    assert!(!recognized, "type names are the wire contract, not free text");
}

#[test]
fn empty_type_after_colon_is_unrecognized() {
    let (fr_id, kind, recognized) = parse_evidence_requirement("FR-7:");
    assert_eq!(fr_id, "FR-7");
    assert_eq!(kind, None);
    assert!(!recognized, "an empty type is not a valid requirement");
}

#[test]
fn only_the_first_colon_splits_the_requirement() {
    let (fr_id, kind, recognized) = parse_evidence_requirement("FR-7:ci_output:extra");
    assert_eq!(fr_id, "FR-7");
    assert_eq!(
        kind, None,
        "the remaining text is part of the type and does not match any known type"
    );
    assert!(!recognized);
}

// ── validate_project_scope ───────────────────────────────────────────────────

#[test]
fn missing_project_scope_is_rejected() {
    let status = validate_project_scope(None, "/repo").expect_err("no scope must be rejected");
    assert_eq!(status.code(), Code::InvalidArgument);
}

#[test]
fn blank_project_scope_is_rejected() {
    for blank in ["", "   ", "\t\n"] {
        let status = validate_project_scope(Some(blank), "/repo")
            .expect_err("a blank scope must be rejected");
        assert_eq!(status.code(), Code::InvalidArgument, "blank={blank:?}");
    }
}

#[test]
fn mismatched_project_scope_is_denied() {
    let status = validate_project_scope(Some("/other/repo"), "/repo")
        .expect_err("a mismatched scope must be denied");
    assert_eq!(status.code(), Code::PermissionDenied);
}

#[test]
fn matching_project_scope_is_accepted() {
    assert!(validate_project_scope(Some("/repo"), "/repo").is_ok());
    assert!(
        validate_project_scope(Some("  /repo  "), "/repo").is_ok(),
        "surrounding whitespace is trimmed before comparison"
    );
}

// ── evidence_satisfies_requirement ───────────────────────────────────────────

#[test]
fn evidence_matches_only_the_requested_requirement() {
    let wps: HashSet<i64> = [1, 2].into_iter().collect();
    let evidence = vec![
        evidence(1, 1, "FR-1", EvidenceType::TestResult),
        evidence(2, 2, "FR-9", EvidenceType::CiOutput),
    ];

    assert!(evidence_satisfies_requirement(
        &evidence,
        &wps,
        "FR-1",
        None
    ));
    assert!(evidence_satisfies_requirement(
        &evidence,
        &wps,
        "FR-9",
        Some(EvidenceType::CiOutput)
    ));
    assert!(!evidence_satisfies_requirement(
        &evidence,
        &wps,
        "FR-404",
        None
    ));
}

#[test]
fn evidence_from_an_unrelated_work_package_does_not_count() {
    let wps: HashSet<i64> = [1].into_iter().collect();
    let evidence = vec![evidence(3, 99, "FR-1", EvidenceType::TestResult)];

    assert!(
        !evidence_satisfies_requirement(&evidence, &wps, "FR-1", None),
        "evidence must be linked to a work package of this feature"
    );
}

#[test]
fn evidence_type_must_match_when_the_requirement_demands_one() {
    let wps: HashSet<i64> = [1].into_iter().collect();
    let evidence = vec![evidence(4, 1, "FR-1", EvidenceType::LintResult)];

    assert!(evidence_satisfies_requirement(&evidence, &wps, "FR-1", None));
    assert!(!evidence_satisfies_requirement(
        &evidence,
        &wps,
        "FR-1",
        Some(EvidenceType::SecurityScan)
    ));
    assert!(evidence_satisfies_requirement(
        &evidence,
        &wps,
        "FR-1",
        Some(EvidenceType::LintResult)
    ));
}

#[test]
fn no_evidence_never_satisfies_a_requirement() {
    let wps: HashSet<i64> = [1].into_iter().collect();
    assert!(!evidence_satisfies_requirement(&[], &wps, "FR-1", None));
}

// ── missing_evidence_violation ───────────────────────────────────────────────

#[test]
fn violation_names_the_rule_and_the_requirement() {
    let rule = rule("specified->implementing", &["FR-1:test_result"]);
    let violation = missing_evidence_violation(&rule, "FR-1", "FR-1:test_result");

    assert_eq!(violation.fr_id, "FR-1");
    assert_eq!(violation.rule_id, "specified->implementing");
    assert!(violation.message.contains("FR-1:test_result"));
    assert!(violation.remediation.contains("FR-1:test_result"));
}
