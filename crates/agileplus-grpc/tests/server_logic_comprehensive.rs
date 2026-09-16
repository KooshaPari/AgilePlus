//! Comprehensive tests for server-level logic: evidence parsing, error
//! mapping, and project scope validation.
//!
//! Complements the inline unit tests in `server/tests.rs` and the existing
//! `grpc_integration.rs` error mapping tests.
//!
//! Traceability: WP14-T079

use agileplus_domain::error::DomainError;
use agileplus_grpc::server::{domain_error_to_status, parse_evidence_requirement};

// ---------------------------------------------------------------------------
// domain_error_to_status - all variants
// ---------------------------------------------------------------------------

#[test]
fn not_found_preserves_message() {
    let status = domain_error_to_status(DomainError::NotFound("widget not found".into()));
    assert_eq!(status.code(), tonic::Code::NotFound);
    assert_eq!(status.message(), "widget not found");
}

#[test]
fn conflict_preserves_message() {
    let status = domain_error_to_status(DomainError::Conflict("slug collision".into()));
    assert_eq!(status.code(), tonic::Code::AlreadyExists);
    assert_eq!(status.message(), "slug collision");
}

#[test]
fn invalid_transition_preserves_from_to_reason() {
    let status = domain_error_to_status(DomainError::InvalidTransition {
        from: "planned".into(),
        to: "shipped".into(),
        reason: "must validate first".into(),
    });
    assert_eq!(status.code(), tonic::Code::FailedPrecondition);
    let msg = status.message();
    assert!(msg.contains("planned"));
    assert!(msg.contains("shipped"));
    assert!(msg.contains("must validate first"));
}

#[test]
fn not_implemented_maps_to_unimplemented() {
    let status = domain_error_to_status(DomainError::NotImplemented);
    assert_eq!(status.code(), tonic::Code::Unimplemented);
    assert!(status.message().contains("not implemented"));
}

#[test]
fn generic_error_maps_to_internal() {
    let status = domain_error_to_status(DomainError::Storage("disk full".into()));
    assert_eq!(status.code(), tonic::Code::Internal);
}

#[test]
fn generic_error_preserves_message() {
    let status = domain_error_to_status(DomainError::Storage("timeout connecting".into()));
    assert!(status.message().contains("timeout connecting"));
}

// ---------------------------------------------------------------------------
// parse_evidence_requirement - all evidence types
// ---------------------------------------------------------------------------

#[test]
fn parse_fr_id_only() {
    let (fr_id, evidence_type, recognized) = parse_evidence_requirement("FR-001");
    assert_eq!(fr_id, "FR-001");
    assert!(evidence_type.is_none());
    assert!(recognized);
}

#[test]
fn parse_test_result_type() {
    let (fr_id, evidence_type, recognized) = parse_evidence_requirement("FR-042:test_result");
    assert_eq!(fr_id, "FR-042");
    assert!(evidence_type.is_some());
    assert!(recognized);
}

#[test]
fn parse_ci_output_type() {
    let (fr_id, evidence_type, recognized) = parse_evidence_requirement("FR-100:ci_output");
    assert_eq!(fr_id, "FR-100");
    assert!(evidence_type.is_some());
    assert!(recognized);
}

#[test]
fn parse_review_approval_type() {
    let (fr_id, evidence_type, recognized) =
        parse_evidence_requirement("FR-7:review_approval");
    assert_eq!(fr_id, "FR-7");
    assert!(evidence_type.is_some());
    assert!(recognized);
}

#[test]
fn parse_security_scan_type() {
    let (fr_id, evidence_type, recognized) =
        parse_evidence_requirement("FR-99:security_scan");
    assert_eq!(fr_id, "FR-99");
    assert!(evidence_type.is_some());
    assert!(recognized);
}

#[test]
fn parse_lint_result_type() {
    let (fr_id, evidence_type, recognized) =
        parse_evidence_requirement("FR-12:lint_result");
    assert_eq!(fr_id, "FR-12");
    assert!(evidence_type.is_some());
    assert!(recognized);
}

#[test]
fn parse_manual_attestation_type() {
    let (fr_id, evidence_type, recognized) =
        parse_evidence_requirement("FR-50:manual_attestation");
    assert_eq!(fr_id, "FR-50");
    assert!(evidence_type.is_some());
    assert!(recognized);
}

#[test]
fn parse_unrecognized_type() {
    let (fr_id, evidence_type, recognized) =
        parse_evidence_requirement("FR-1:unknown_type");
    assert_eq!(fr_id, "FR-1");
    assert!(evidence_type.is_none());
    // When type is present but unrecognized, recognized should be false
    assert!(!recognized);
}

#[test]
fn parse_empty_string() {
    let (fr_id, evidence_type, recognized) = parse_evidence_requirement("");
    assert_eq!(fr_id, "");
    assert!(evidence_type.is_none());
    assert!(recognized);
}

#[test]
fn parse_colon_with_empty_type() {
    // "FR-1:" splits into ("FR-1", ""), which is an unrecognized type
    let (fr_id, evidence_type, recognized) = parse_evidence_requirement("FR-1:");
    assert_eq!(fr_id, "FR-1");
    assert!(evidence_type.is_none());
    assert!(!recognized);
}

#[test]
fn parse_multiple_colons_uses_first_split() {
    // "FR-1:ci_output:extra" splits at first colon -> ("FR-1", "ci_output:extra")
    // "ci_output:extra" won't match any known type
    let (fr_id, evidence_type, recognized) =
        parse_evidence_requirement("FR-1:ci_output:extra");
    assert_eq!(fr_id, "FR-1");
    assert!(evidence_type.is_none());
    assert!(!recognized);
}

#[test]
fn parse_fr_id_with_hyphens_and_numbers() {
    let (fr_id, _, _) = parse_evidence_requirement("FR-12345");
    assert_eq!(fr_id, "FR-12345");
}

#[test]
fn parse_fr_id_with_underscores() {
    let (fr_id, _, _) = parse_evidence_requirement("FR_FEATURE_01");
    assert_eq!(fr_id, "FR_FEATURE_01");
}
