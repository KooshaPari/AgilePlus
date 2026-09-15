//! Integration tests for the impact analysis / blast radius engine.
//!
//! These tests exercise compute_impact, top_affected, conflicts_only, and
//! the BFS traversal with realistic graph structures.

use chrono::Utc;
use uuid::Uuid;

use traceability_core::{
    ArtifactRef, ImpactConfig, compute_impact, conflicts_only, top_affected,
    ids::RequirementId,
    matrix::{CoverageMatrix, CoverageState, MatrixCell},
    tracelink::{TraceLink, TraceLinkType},
};

fn req(id: &str) -> ArtifactRef {
    ArtifactRef::Requirement {
        id: RequirementId::from_string(id),
    }
}

fn test_ref(id: &str) -> ArtifactRef {
    ArtifactRef::Test {
        id: id.to_string(),
    }
}

fn code_ref(id: &str) -> ArtifactRef {
    ArtifactRef::CodeEntity {
        id: id.to_string(),
        lang: "rust".to_string(),
    }
}

fn make_link(
    from: ArtifactRef,
    to: ArtifactRef,
    ty: TraceLinkType,
    conf: f32,
) -> TraceLink {
    let project = Uuid::new_v4();
    let source = Uuid::new_v4();
    let target = Uuid::new_v4();
    let mut link = TraceLink::new(project, source, target, ty).unwrap();
    link.from = from;
    link.to = to;
    link.confidence = conf;
    link.created_at = Some(Utc::now());
    link.updated_at = Some(Utc::now());
    link
}

fn artifact_key(a: &ArtifactRef) -> String {
    match a {
        ArtifactRef::Requirement { id } => id.as_str().to_string(),
        ArtifactRef::NonFunctionalRequirement { id } => id.as_str().to_string(),
        ArtifactRef::Test { id } => format!("test:{id}"),
        ArtifactRef::CodeEntity { id, .. } => format!("code:{id}"),
        ArtifactRef::Journey { id } => format!("journey:{id}"),
        ArtifactRef::AgentRun { id } => format!("agent:{id}"),
        ArtifactRef::Evidence { id, .. } => format!("evidence:{id}"),
        ArtifactRef::Document { id, .. } => format!("document:{id}"),
    }
}

fn make_matrix(links: Vec<TraceLink>) -> CoverageMatrix {
    let mut cells: indexmap::IndexMap<(String, String), MatrixCell> =
        indexmap::IndexMap::with_hasher(Default::default());
    for link in links {
        let key = (artifact_key(&link.from), artifact_key(&link.to));
        let cell = cells.entry(key).or_insert_with(|| MatrixCell {
            from: artifact_key(&link.from),
            to: artifact_key(&link.to),
            trace_links: Vec::new(),
            coverage: CoverageState::Covered,
        });
        cell.trace_links.push(link);
    }
    CoverageMatrix {
        cells,
        generated_at: Utc::now(),
    }
}

// ---------------------------------------------------------------------------
// Diamond graph: A -> B, A -> C, B -> D, C -> D
// ---------------------------------------------------------------------------

#[test]
fn diamond_graph_all_nodes_reached() {
    let l1 = make_link(req("FR-001"), test_ref("T-001"), TraceLinkType::Satisfies, 0.9);
    let l2 = make_link(req("FR-001"), test_ref("T-002"), TraceLinkType::Satisfies, 0.8);
    let l3 = make_link(test_ref("T-001"), req("FR-002"), TraceLinkType::Satisfies, 0.9);
    let l4 = make_link(test_ref("T-002"), req("FR-002"), TraceLinkType::Satisfies, 0.8);

    let matrix = make_matrix(vec![l1, l2, l3, l4]);
    let report = compute_impact(&matrix, &[req("FR-001")], &ImpactConfig::default());

    assert!(report.blast.len() >= 4, "all 4 nodes should be reached");
    assert!(!report.conflicts.is_empty() || report.total_score > 0.0);
}

// ---------------------------------------------------------------------------
// Chain graph: A -> B -> C -> D -> E
// ---------------------------------------------------------------------------

#[test]
fn chain_graph_depth_propagation() {
    let l1 = make_link(req("FR-001"), test_ref("T-001"), TraceLinkType::Verifies, 0.9);
    let l2 = make_link(test_ref("T-001"), code_ref("mod::foo"), TraceLinkType::Implements, 0.85);
    let l3 = make_link(code_ref("mod::foo"), test_ref("T-002"), TraceLinkType::Verifies, 0.8);
    let l4 = make_link(test_ref("T-002"), req("FR-002"), TraceLinkType::Satisfies, 0.75);

    let matrix = make_matrix(vec![l1, l2, l3, l4]);
    let cfg = ImpactConfig {
        max_depth: 2,
        ..Default::default()
    };
    let report = compute_impact(&matrix, &[req("FR-001")], &cfg);

    // With max_depth=2, we should get seed + depth 1 + depth 2 = 3 nodes
    assert!(report.truncated, "should be truncated at depth 2");
    assert!(report.blast.len() <= 4);
}

// ---------------------------------------------------------------------------
// Conflicts only
// ---------------------------------------------------------------------------

#[test]
fn conflicts_only_filters_non_conflicts() {
    let l1 = make_link(
        req("FR-001"),
        test_ref("T-001"),
        TraceLinkType::ConflictsWith,
        0.9,
    );
    let l2 = make_link(req("FR-001"), test_ref("T-002"), TraceLinkType::Verifies, 0.9);
    let l3 = make_link(
        req("FR-002"),
        test_ref("T-003"),
        TraceLinkType::ConflictsWith,
        0.8,
    );

    let matrix = make_matrix(vec![l1, l2, l3]);
    let report = compute_impact(&matrix, &[req("FR-001"), req("FR-002")], &ImpactConfig::default());

    let conflicts = conflicts_only(&report);
    assert_eq!(conflicts.len(), 2);
    for c in conflicts {
        assert_eq!(c.link_type, TraceLinkType::ConflictsWith);
    }
}

// ---------------------------------------------------------------------------
// Top affected ranking
// ---------------------------------------------------------------------------

#[test]
fn top_affected_returns_sorted_by_abs_score() {
    let l1 = make_link(req("FR-001"), test_ref("T-001"), TraceLinkType::Verifies, 0.95);
    let l2 = make_link(req("FR-001"), test_ref("T-002"), TraceLinkType::Satisfies, 0.5);
    let l3 = make_link(req("FR-001"), code_ref("src/main"), TraceLinkType::Implements, 0.9);

    let matrix = make_matrix(vec![l1, l2, l3]);
    let report = compute_impact(&matrix, &[req("FR-001")], &ImpactConfig::default());

    let top = top_affected(&report, 3);
    assert_eq!(top.len(), 3);
    // Check sorted by |score| descending
    for i in 0..top.len() - 1 {
        assert!(top[i].score.abs() >= top[i + 1].score.abs());
    }
}

// ---------------------------------------------------------------------------
// Multi-seed with overlapping blast radius
// ---------------------------------------------------------------------------

#[test]
fn overlapping_seeds_deduplicate() {
    let l1 = make_link(req("FR-001"), test_ref("T-001"), TraceLinkType::Verifies, 0.9);
    let l2 = make_link(req("FR-002"), test_ref("T-001"), TraceLinkType::Verifies, 0.8);

    let matrix = make_matrix(vec![l1, l2]);
    let report = compute_impact(
        &matrix,
        &[req("FR-001"), req("FR-002")],
        &ImpactConfig::default(),
    );

    // T-001 appears in both seeds' blast but should only appear once
    let t1_count = report.blast.iter().filter(|n| {
        matches!(&n.artifact, ArtifactRef::Test { id } if id == "T-001")
    }).count();
    assert_eq!(t1_count, 1, "T-001 should appear exactly once");
}

// ---------------------------------------------------------------------------
// Empty seeds
// ---------------------------------------------------------------------------

#[test]
fn empty_seeds_returns_empty_report() {
    let matrix = make_matrix(vec![]);
    let report = compute_impact(&matrix, &[], &ImpactConfig::default());
    assert!(report.blast.is_empty());
    assert_eq!(report.total_score, 0.0);
}

// ---------------------------------------------------------------------------
// Impact report by_kind bucketing
// ---------------------------------------------------------------------------

#[test]
fn by_kind_bucketing_accurate() {
    let l1 = make_link(req("FR-001"), test_ref("T-001"), TraceLinkType::Verifies, 0.9);
    let l2 = make_link(req("FR-001"), code_ref("mod::x"), TraceLinkType::Implements, 0.85);

    let matrix = make_matrix(vec![l1, l2]);
    let report = compute_impact(&matrix, &[req("FR-001")], &ImpactConfig::default());

    let kinds = report.affected_kinds();
    assert!(kinds.contains("requirement"));
    assert!(kinds.contains("test"));
    assert!(kinds.contains("code"));
}

// ---------------------------------------------------------------------------
// Serialization roundtrip for full impact report
// ---------------------------------------------------------------------------

#[test]
fn impact_report_full_serde_roundtrip() {
    let l1 = make_link(req("FR-001"), test_ref("T-001"), TraceLinkType::Verifies, 0.9);
    let l2 = make_link(
        req("FR-001"),
        test_ref("T-002"),
        TraceLinkType::ConflictsWith,
        0.8,
    );
    let matrix = make_matrix(vec![l1, l2]);
    let report = compute_impact(&matrix, &[req("FR-001")], &ImpactConfig::default());

    let json = serde_json::to_string(&report).unwrap();
    let back: traceability_core::ImpactReport = serde_json::from_str(&json).unwrap();

    assert_eq!(back.seeds, report.seeds);
    assert_eq!(back.blast.len(), report.blast.len());
    assert!((back.total_score - report.total_score).abs() < 0.001);
    assert_eq!(back.conflicts.len(), report.conflicts.len());
}

// ---------------------------------------------------------------------------
// Max depth 0 = unbounded
// ---------------------------------------------------------------------------

#[test]
fn max_depth_zero_unbounded_traverses_all() {
    // Chain of 5
    let l1 = make_link(req("FR-001"), test_ref("T-001"), TraceLinkType::Satisfies, 0.9);
    let l2 = make_link(test_ref("T-001"), test_ref("T-002"), TraceLinkType::DerivesFrom, 0.8);
    let l3 = make_link(test_ref("T-002"), test_ref("T-003"), TraceLinkType::DerivesFrom, 0.7);
    let l4 = make_link(test_ref("T-003"), test_ref("T-004"), TraceLinkType::DerivesFrom, 0.6);
    let l5 = make_link(test_ref("T-004"), req("FR-002"), TraceLinkType::Satisfies, 0.5);

    let matrix = make_matrix(vec![l1, l2, l3, l4, l5]);
    let cfg = ImpactConfig {
        max_depth: 0,
        ..Default::default()
    };
    let report = compute_impact(&matrix, &[req("FR-001")], &cfg);
    assert!(!report.truncated);
    assert!(report.blast.len() >= 6, "should reach all 6 nodes");
}
