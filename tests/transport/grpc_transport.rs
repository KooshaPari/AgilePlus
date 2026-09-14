//! Transport-layer tests for AgilePlus.
//!
//! Tests gRPC helper functions, CLI argument parsing, and SQLite storage
//! round-trip without requiring full mock port implementations.

use tempfile::TempDir;

use agileplus_domain::domain::governance::EvidenceType;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::StoragePort;
use agileplus_sqlite::SqliteStorageAdapter;

use agileplus_grpc::event_bus::EventBus;
use agileplus_grpc::proxy::ProxyRouter;
use agileplus_grpc::server::{domain_error_to_status, parse_evidence_requirement};


// ===========================================================================
// gRPC helper function tests — domain_error_to_status
// ===========================================================================

#[test]
fn domain_error_not_found_maps_to_not_found() {
    let status = domain_error_to_status(DomainError::NotFound("widget not found".into()));
    assert_eq!(status.code(), tonic::Code::NotFound);
    assert!(status.message().contains("widget not found"));
}

#[test]
fn domain_error_conflict_maps_to_already_exists() {
    let status = domain_error_to_status(DomainError::Conflict("dup slug".into()));
    assert_eq!(status.code(), tonic::Code::AlreadyExists);
}

#[test]
fn domain_error_not_implemented_maps_to_unimplemented() {
    let status = domain_error_to_status(DomainError::NotImplemented);
    assert_eq!(status.code(), tonic::Code::Unimplemented);
}

#[test]
fn domain_error_invalid_transition_maps_to_failed_precondition() {
    let status = domain_error_to_status(DomainError::InvalidTransition {
        from: "created".into(),
        to: "shipped".into(),
        reason: "skipped steps".into(),
    });
    assert_eq!(status.code(), tonic::Code::FailedPrecondition);
    assert!(status.message().contains("created"));
    assert!(status.message().contains("shipped"));
}

#[test]
fn domain_error_storage_maps_to_internal() {
    let status = domain_error_to_status(DomainError::Storage("disk full".into()));
    assert_eq!(status.code(), tonic::Code::Internal);
}

#[test]
fn domain_error_validation_maps_to_internal() {
    let status = domain_error_to_status(DomainError::Validation("bad input".into()));
    assert_eq!(status.code(), tonic::Code::Internal);
}

#[test]
fn domain_error_other_maps_to_internal() {
    let status = domain_error_to_status(DomainError::Other("something broke".into()));
    assert_eq!(status.code(), tonic::Code::Internal);
}

#[test]
fn domain_error_feature_not_found_maps_to_internal() {
    let status = domain_error_to_status(DomainError::FeatureNotFound("f-1".into()));
    assert_eq!(status.code(), tonic::Code::Internal);
}

#[test]
fn domain_error_lock_poisoned_maps_to_internal() {
    let status = domain_error_to_status(DomainError::LockPoisoned);
    assert_eq!(status.code(), tonic::Code::Internal);
}

// ===========================================================================
// gRPC helper function tests — parse_evidence_requirement
// ===========================================================================

#[test]
fn parse_evidence_requirement_plain_fr_id() {
    let (fr_id, etype, recognized) = parse_evidence_requirement("FR-001");
    assert_eq!(fr_id, "FR-001");
    assert!(etype.is_none());
    assert!(recognized);
}

#[test]
fn parse_evidence_requirement_with_test_result_type() {
    let (fr_id, etype, recognized) = parse_evidence_requirement("FR-042:test_result");
    assert_eq!(fr_id, "FR-042");
    assert_eq!(etype, Some(EvidenceType::TestResult));
    assert!(recognized);
}

#[test]
fn parse_evidence_requirement_with_ci_output_type() {
    let (fr_id, etype, recognized) = parse_evidence_requirement("FR-010:ci_output");
    assert_eq!(fr_id, "FR-010");
    assert_eq!(etype, Some(EvidenceType::CiOutput));
    assert!(recognized);
}

#[test]
fn parse_evidence_requirement_with_review_approval_type() {
    let (fr_id, etype, recognized) = parse_evidence_requirement("FR-005:review_approval");
    assert_eq!(fr_id, "FR-005");
    assert_eq!(etype, Some(EvidenceType::ReviewApproval));
    assert!(recognized);
}

#[test]
fn parse_evidence_requirement_with_security_scan_type() {
    let (fr_id, etype, recognized) = parse_evidence_requirement("FR-099:security_scan");
    assert_eq!(fr_id, "FR-099");
    assert_eq!(etype, Some(EvidenceType::SecurityScan));
    assert!(recognized);
}

#[test]
fn parse_evidence_requirement_with_lint_result_type() {
    let (fr_id, etype, recognized) = parse_evidence_requirement("FR-003:lint_result");
    assert_eq!(fr_id, "FR-003");
    assert_eq!(etype, Some(EvidenceType::LintResult));
    assert!(recognized);
}

#[test]
fn parse_evidence_requirement_with_manual_attestation_type() {
    let (fr_id, etype, recognized) =
        parse_evidence_requirement("FR-007:manual_attestation");
    assert_eq!(fr_id, "FR-007");
    assert_eq!(etype, Some(EvidenceType::ManualAttestation));
    assert!(recognized);
}

#[test]
fn parse_evidence_requirement_unknown_type_is_unrecognized() {
    let (fr_id, etype, recognized) = parse_evidence_requirement("FR-001:bogus_type");
    assert_eq!(fr_id, "FR-001");
    assert!(etype.is_none());
    assert!(!recognized);
}

#[test]
fn parse_evidence_requirement_empty_string() {
    let (fr_id, etype, recognized) = parse_evidence_requirement("");
    assert_eq!(fr_id, "");
    assert!(etype.is_none());
    assert!(recognized);
}

#[test]
fn parse_evidence_requirement_colon_only() {
    let (fr_id, etype, recognized) = parse_evidence_requirement(":test_result");
    assert_eq!(fr_id, "");
    assert_eq!(etype, Some(EvidenceType::TestResult));
    assert!(recognized);
}

// ===========================================================================
// SQLite storage round-trip tests
// ===========================================================================

#[tokio::test]
async fn sqlite_open_creates_database_file() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("test.db");

    let _adapter = SqliteStorageAdapter::new(&db_path).expect("open should succeed");
    assert!(db_path.exists(), "database file should be created");
}

#[tokio::test]
async fn sqlite_list_features_empty_database() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("test.db");
    let adapter = SqliteStorageAdapter::new(&db_path).unwrap();

    let features = adapter.list_all_features().await.unwrap();
    assert!(features.is_empty(), "fresh DB should have no features");
}

#[tokio::test]
async fn sqlite_get_nonexistent_feature_returns_none() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("test.db");
    let adapter = SqliteStorageAdapter::new(&db_path).unwrap();

    let result = adapter.get_feature_by_slug("nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn sqlite_feature_state_round_trip() {
    use agileplus_domain::domain::state_machine::FeatureState;

    let state = FeatureState::Specified;
    let serialized = state.to_string();
    let deserialized: FeatureState = serialized.parse().unwrap();
    assert_eq!(state, deserialized);
}

#[tokio::test]
async fn sqlite_feature_state_all_variants() {
    use agileplus_domain::domain::state_machine::FeatureState;

    let variants = [
        "created", "specified", "researched", "planned",
        "implementing", "validated", "shipped", "retrospected",
    ];
    for variant in variants {
        let state: FeatureState = variant.parse().unwrap();
        assert_eq!(state.to_string(), variant);
    }
}

// ===========================================================================
// EventBus tests
// ===========================================================================

#[test]
fn event_bus_new_creates_empty_bus() {
    let bus = EventBus::new(64);
    let _arc = std::sync::Arc::new(bus);
}

// ===========================================================================
// ProxyRouter tests
// ===========================================================================

#[tokio::test]
async fn proxy_router_new_creates_instance() {
    let router = ProxyRouter::new(None, None).await;
    let _arc = std::sync::Arc::new(router);
}
