//! Integration tests for matrix building and contract evaluation.
//!
//! These tests exercise the full pipeline: build a matrix from trace links,
//! evaluate acceptance contracts against it, and run progression gates.

use chrono::Utc;
use uuid::Uuid;

use traceability_core::{
    AcceptanceContract, ArtifactRef, CoverageState, Criterion, Evidence,
    EvidenceType, Layer, ProgressionGate,
    VerificationMethod,
    build_matrix, classify_cell,
    ids::RequirementId,
    tracelink::{TraceLink, TraceLinkType},
    contract::{GateContext, GateReason, GatePredicate},
};

fn make_link(
    source: Uuid,
    target: Uuid,
    link_type: TraceLinkType,
    confidence: f32,
) -> TraceLink {
    let project = Uuid::new_v4();
    let mut link = TraceLink::new(project, source, target, link_type).unwrap();
    link.confidence = confidence;
    link.created_at = Some(Utc::now());
    link.updated_at = Some(Utc::now());
    link
}

// ---------------------------------------------------------------------------
// Build matrix from mixed link types and verify cell classification
// ---------------------------------------------------------------------------

#[test]
fn matrix_with_all_link_types_classifies_correctly() {
    let src1 = Uuid::new_v4();
    let tgt1 = Uuid::new_v4();
    let src2 = Uuid::new_v4();
    let tgt2 = Uuid::new_v4();
    let src3 = Uuid::new_v4();
    let tgt3 = Uuid::new_v4();

    let links = vec![
        make_link(src1, tgt1, TraceLinkType::Verifies, 0.95),
        make_link(src2, tgt2, TraceLinkType::Satisfies, 0.5),
        make_link(src3, tgt3, TraceLinkType::DerivesFrom, 0.3),
    ];

    let result = build_matrix(&links);
    assert_eq!(result.link_count, 3);
    assert_eq!(result.cell_count, 3);

    // Find cells and check coverage
    let cells: Vec<_> = result.matrix.cells.values().collect();
    let covered = cells.iter().filter(|c| c.coverage == CoverageState::Covered).count();
    let partial = cells.iter().filter(|c| c.coverage == CoverageState::Partial).count();
    let stale_or_missing = cells
        .iter()
        .filter(|c| c.coverage == CoverageState::Stale || c.coverage == CoverageState::Missing)
        .count();

    assert_eq!(covered, 1, "high-confidence Verifies should be Covered");
    assert_eq!(partial, 1, "low-confidence Satisfies should be Partial");
    assert_eq!(
        stale_or_missing, 1,
        "DerivesFrom with low confidence should be Stale or Missing"
    );
}

// ---------------------------------------------------------------------------
// Acceptance contract with multiple criteria
// ---------------------------------------------------------------------------

#[test]
fn acceptance_contract_three_criteria_two_covered() {
    let src_a = Uuid::new_v4();
    let tgt_a = Uuid::new_v4();
    let src_b = Uuid::new_v4();
    let tgt_b = Uuid::new_v4();

    // Build links: A is high-confidence covered, B is partial
    let links = vec![
        make_link(src_a, tgt_a, TraceLinkType::Verifies, 0.95),
        make_link(src_b, tgt_b, TraceLinkType::Satisfies, 0.5),
    ];
    let result = build_matrix(&links);
    let matrix = result.matrix;

    let contract = AcceptanceContract {
        artifact_ref: ArtifactRef::Requirement {
            id: RequirementId::new(),
        },
        criteria: vec![
            Criterion {
                id: "AC-1".into(),
                test_ref: src_a.to_string(),
                evidence_ref: tgt_a.to_string(),
            },
            Criterion {
                id: "AC-2".into(),
                test_ref: src_b.to_string(),
                evidence_ref: tgt_b.to_string(),
            },
            Criterion {
                id: "AC-3".into(),
                test_ref: "nonexistent-src".into(),
                evidence_ref: "nonexistent-tgt".into(),
            },
        ],
        verification: VerificationMethod::Test,
        bdd: vec![],
    };

    assert!(!contract.is_satisfied(&matrix));
    let unsatisfied = contract.unsatisfied_criteria(&matrix);
    assert_eq!(unsatisfied.len(), 2);
    assert!(unsatisfied.contains(&"AC-2".to_string()));
    assert!(unsatisfied.contains(&"AC-3".to_string()));
}

// ---------------------------------------------------------------------------
// ProgressionGate: execution_to_evidence with full context
// ---------------------------------------------------------------------------

#[test]
fn execution_to_evidence_gate_full_pass() {
    let gate = ProgressionGate::execution_to_evidence();

    // Build a covered matrix
    let src = Uuid::new_v4();
    let tgt = Uuid::new_v4();
    let links = vec![make_link(src, tgt, TraceLinkType::Verifies, 0.95)];
    let matrix = build_matrix(&links).matrix;

    let contract = AcceptanceContract {
        artifact_ref: ArtifactRef::Requirement {
            id: RequirementId::new(),
        },
        criteria: vec![Criterion {
            id: "AC-1".into(),
            test_ref: src.to_string(),
            evidence_ref: tgt.to_string(),
        }],
        verification: VerificationMethod::Test,
        bdd: vec![],
    };

    let evidence = Evidence {
        id: 1,
        wp_id: 100,
        fr_id: "FR-1".into(),
        evidence_type: EvidenceType::TestResult,
        artifact_path: "/ci/results.xml".into(),
        metadata: None,
        created_at: Utc::now(),
    };

    let ctx = GateContext {
        acceptance: Some(&contract),
        matrix: Some(&matrix),
        evidence: &[evidence],
        has_implementation: true,
        has_test_links: true,
        requirement: None,
    };

    assert!(gate.evaluate(&ctx).is_ok());
}

#[test]
fn execution_to_evidence_gate_fails_on_missing_evidence() {
    let gate = ProgressionGate::execution_to_evidence();
    let ctx = GateContext {
        has_test_links: true,
        has_implementation: true,
        ..Default::default()
    };
    // Missing acceptance (no contract/matrix) and missing evidence
    let result = gate.evaluate(&ctx);
    // First predicate is MissingAcceptance
    assert_eq!(result, Err(GateReason::MissingAcceptance));
}

// ---------------------------------------------------------------------------
// Matrix diff: added, removed, changed
// ---------------------------------------------------------------------------

#[test]
fn matrix_diff_detects_new_and_removed_links() {
    let src = Uuid::new_v4();
    let tgt_a = Uuid::new_v4();
    let tgt_b = Uuid::new_v4();

    let link_a = make_link(src, tgt_a, TraceLinkType::Verifies, 0.95);
    let link_b = make_link(src, tgt_b, TraceLinkType::Satisfies, 0.8);

    let old = build_matrix(&[link_a.clone()]).matrix;
    let new = build_matrix(&[link_a, link_b]).matrix;

    let added = traceability_core::matrix::added(&old, &new);
    assert_eq!(added.len(), 1, "one new cell should be added");

    let removed = traceability_core::matrix::removed(&old, &new);
    assert!(removed.is_empty(), "nothing should be removed");
}

#[test]
fn matrix_diff_detects_coverage_change() {
    let src = Uuid::new_v4();
    let tgt = Uuid::new_v4();

    let link_high = make_link(src, tgt, TraceLinkType::Verifies, 0.95);
    let old = build_matrix(&[link_high]).matrix;

    // Build a new matrix with the same key but low confidence
    // We need to create a link with the same source/target UUIDs
    let link_low = {
        let project = Uuid::new_v4();
        let mut l = TraceLink::new(project, src, tgt, TraceLinkType::Satisfies).unwrap();
        l.confidence = 0.3;
        l.created_at = Some(Utc::now());
        l.updated_at = Some(Utc::now());
        l
    };
    let new = build_matrix(&[link_low]).matrix;

    let changed = traceability_core::matrix::changed(&old, &new);
    assert_eq!(changed.len(), 1, "coverage should have changed");
}

// ---------------------------------------------------------------------------
// Classify cell edge cases
// ---------------------------------------------------------------------------

#[test]
fn classify_empty_links_returns_missing() {
    assert_eq!(classify_cell(&[]), CoverageState::Missing);
}

#[test]
fn classify_only_conflict_returns_conflict() {
    let link = make_link(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TraceLinkType::ConflictsWith,
        0.9,
    );
    assert_eq!(classify_cell(&[link]), CoverageState::Conflict);
}

#[test]
fn classify_high_confidence_satisfies_returns_covered() {
    let link = make_link(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TraceLinkType::Satisfies,
        0.95,
    );
    assert_eq!(classify_cell(&[link]), CoverageState::Covered);
}

#[test]
fn classify_low_confidence_verifies_returns_partial() {
    let link = make_link(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TraceLinkType::Verifies,
        0.5,
    );
    assert_eq!(classify_cell(&[link]), CoverageState::Partial);
}

// ---------------------------------------------------------------------------
// Layer stack chain
// ---------------------------------------------------------------------------

#[test]
fn layer_chain_complete() {
    let all = Layer::all();
    assert_eq!(all.len(), 6);
    for window in all.windows(2) {
        assert_eq!(window[0].next(), Some(window[1]));
    }
    assert_eq!(all.last().unwrap().next(), None);
}

// ---------------------------------------------------------------------------
// GatePredicate -> GateReason conversions
// ---------------------------------------------------------------------------

#[test]
fn all_gate_predicates_convert_to_reasons() {
    let cases = [
        (GatePredicate::NotApproved, GateReason::NotApproved),
        (GatePredicate::MissingAcceptance, GateReason::MissingAcceptance),
        (GatePredicate::MissingEvidence, GateReason::MissingEvidence),
        (
            GatePredicate::MissingImplementation,
            GateReason::MissingImplementation,
        ),
        (GatePredicate::MissingTest, GateReason::MissingTest),
    ];
    for (pred, expected) in cases {
        let reason: GateReason = pred.into();
        assert_eq!(reason, expected);
    }
}
