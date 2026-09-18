//! Integration tests for the IntentGraph validation engine.
//!
//! These tests exercise complex multi-node, multi-edge graph validation
//! scenarios including DAG checks, edge constraints, and error collection.

use chrono::Utc;

use traceability_core::{
    DagStage, Edge, GraphMetadata, IntentGraph, Meta, Node, NodeType,
    RelationshipType, ValidationError,
    intent_graph::Status as NodeStatus,
};

fn meta() -> Meta {
    Meta {
        confidence: Some(0.95),
        source: "test".into(),
        timestamp: Utc::now(),
        agent_id: None,
    }
}

fn node(id: &str, nt: NodeType, ds: DagStage) -> Node {
    Node {
        id: id.into(),
        node_type: nt,
        dag_stage: ds,
        title: format!("Node {id}"),
        description: None,
        status: NodeStatus::Active,
        tags: vec![],
        meta: meta(),
        properties: None,
        table_ref: None,
        table_id: None,
    }
}

fn edge(id: &str, src: &str, tgt: &str, rel: RelationshipType) -> Edge {
    Edge {
        id: id.into(),
        source: src.into(),
        target: tgt.into(),
        relationship_type: rel,
        canonical_map: None,
        meta: meta(),
        properties: None,
    }
}

fn graph_metadata() -> GraphMetadata {
    GraphMetadata {
        version: "1.0".into(),
        schema_uri: "https://phenotype.dev/schema/v1.json".into(),
        created_at: Utc::now(),
        updated_at: None,
        node_count: None,
        edge_count: None,
        dag_valid: None,
        source_system: None,
    }
}

fn make_graph(nodes: Vec<Node>, edges: Vec<Edge>) -> IntentGraph {
    IntentGraph {
        nodes,
        edges,
        metadata: graph_metadata(),
    }
}

// ---------------------------------------------------------------------------
// Valid complex graph: Intent -> Feature -> Task, Intent -> Story -> Task
// ---------------------------------------------------------------------------

#[test]
fn valid_diamond_hierarchy_passes() {
    let g = make_graph(
        vec![
            node("Intent#root", NodeType::Intent, DagStage::Intent),
            node("Feature#auth", NodeType::Feature, DagStage::Feature),
            node("Story#login", NodeType::Story, DagStage::Story),
            node("Task#oauth", NodeType::Task, DagStage::Task),
        ],
        vec![
            edge("e1", "Intent#root", "Feature#auth", RelationshipType::Implements),
            edge("e2", "Intent#root", "Story#login", RelationshipType::Implements),
            edge("e3", "Feature#auth", "Task#oauth", RelationshipType::Implements),
            edge("e4", "Story#login", "Task#oauth", RelationshipType::Implements),
        ],
    );
    assert!(g.validate().is_ok());
}

// ---------------------------------------------------------------------------
// Complex graph with cycles
// ---------------------------------------------------------------------------

#[test]
fn complex_cycle_detection() {
    let g = make_graph(
        vec![
            node("Intent#root", NodeType::Intent, DagStage::Intent),
            node("Task#a", NodeType::Task, DagStage::Task),
            node("Task#b", NodeType::Task, DagStage::Task),
            node("Task#c", NodeType::Task, DagStage::Task),
        ],
        vec![
            edge("e1", "Intent#root", "Task#a", RelationshipType::Implements),
            edge("e2", "Task#a", "Task#b", RelationshipType::DependsOn),
            edge("e3", "Task#b", "Task#c", RelationshipType::DependsOn),
            edge("e4", "Task#c", "Task#a", RelationshipType::DependsOn),
        ],
    );
    let err = g.validate().unwrap_err();
    assert!(err.iter().any(|e| matches!(e, ValidationError::CycleDetected)));
}

// ---------------------------------------------------------------------------
// Multiple error collection
// ---------------------------------------------------------------------------

#[test]
fn validate_collects_multiple_independent_errors() {
    let g = make_graph(
        vec![
            node("Intent#a", NodeType::Intent, DagStage::Intent),
            node("Intent#a", NodeType::Intent, DagStage::Intent), // duplicate
            node("intent#bad", NodeType::Intent, DagStage::Intent), // invalid id
        ],
        vec![],
    );
    let err = g.validate().unwrap_err();
    assert!(err.len() >= 2);
}

// ---------------------------------------------------------------------------
// Edge constraint: Resolves only from Bug
// ---------------------------------------------------------------------------

#[test]
fn resolves_only_from_bug() {
    // Valid: Bug -> Commit via Resolves
    let g = make_graph(
        vec![
            node("Bug#crash", NodeType::Bug, DagStage::Bug),
            node("Commit#c1", NodeType::Commit, DagStage::Commit),
        ],
        vec![edge(
            "e1",
            "Bug#crash",
            "Commit#c1",
            RelationshipType::Resolves,
        )],
    );
    assert!(g.check_edge_constraints().is_ok());

    // Invalid: Task -> Commit via Resolves
    let g2 = make_graph(
        vec![
            node("Task#a", NodeType::Task, DagStage::Task),
            node("Commit#c1", NodeType::Commit, DagStage::Commit),
        ],
        vec![edge(
            "e1",
            "Task#a",
            "Commit#c1",
            RelationshipType::Resolves,
        )],
    );
    assert!(matches!(
        g2.check_edge_constraints().unwrap_err(),
        ValidationError::InvalidEdgeConstraint { .. }
    ));
}

// ---------------------------------------------------------------------------
// TracesTo is a wildcard: allows any pair
// ---------------------------------------------------------------------------

#[test]
fn traces_to_wildcard_allows_any_pair() {
    let g = make_graph(
        vec![
            node("Bug#a", NodeType::Bug, DagStage::Bug),
            node("Artifact#bin", NodeType::Artifact, DagStage::Artifact),
            node("PR#mr-123", NodeType::PR, DagStage::PR),
        ],
        vec![
            edge("e1", "Bug#a", "Artifact#bin", RelationshipType::TracesTo),
            edge("e2", "PR#mr-123", "Bug#a", RelationshipType::TracesTo),
        ],
    );
    assert!(g.check_edge_constraints().is_ok());
}

// ---------------------------------------------------------------------------
// DependsOn constraints
// ---------------------------------------------------------------------------

#[test]
fn depends_on_valid_and_invalid_pairs() {
    // Valid: Task -> Artifact
    let g = make_graph(
        vec![
            node("Task#a", NodeType::Task, DagStage::Task),
            node("Artifact#lib", NodeType::Artifact, DagStage::Artifact),
        ],
        vec![edge(
            "e1",
            "Task#a",
            "Artifact#lib",
            RelationshipType::DependsOn,
        )],
    );
    assert!(g.check_edge_constraints().is_ok());

    // Invalid: Feature -> Task via DependsOn (not in allowed list)
    let g2 = make_graph(
        vec![
            node("Feature#a", NodeType::Feature, DagStage::Feature),
            node("Task#b", NodeType::Task, DagStage::Task),
        ],
        vec![edge(
            "e1",
            "Feature#a",
            "Task#b",
            RelationshipType::DependsOn,
        )],
    );
    assert!(matches!(
        g2.check_edge_constraints().unwrap_err(),
        ValidationError::InvalidEdgeConstraint { .. }
    ));
}

// ---------------------------------------------------------------------------
// Blocks constraints
// ---------------------------------------------------------------------------

#[test]
fn blocks_valid_pairs() {
    let pairs: Vec<(&str, NodeType, DagStage, &str, NodeType, DagStage)> = vec![
        ("Task#a", NodeType::Task, DagStage::Task, "Task#b", NodeType::Task, DagStage::Task),
        ("Bug#x", NodeType::Bug, DagStage::Bug, "Task#y", NodeType::Task, DagStage::Task),
        ("Bug#x", NodeType::Bug, DagStage::Bug, "PR#z", NodeType::PR, DagStage::PR),
        ("Task#a", NodeType::Task, DagStage::Task, "PR#z", NodeType::PR, DagStage::PR),
        ("PR#1", NodeType::PR, DagStage::PR, "Feature#f", NodeType::Feature, DagStage::Feature),
    ];
    for (sid, st, ds_s, tid, tt, ds_t) in pairs {
        let g = make_graph(
            vec![node(sid, st, ds_s), node(tid, tt, ds_t)],
            vec![edge("e1", sid, tid, RelationshipType::Blocks)],
        );
        assert!(g.check_edge_constraints().is_ok());
    }
}

// ---------------------------------------------------------------------------
// Edge meta validation
// ---------------------------------------------------------------------------

#[test]
fn edge_empty_meta_source_rejected() {
    let mut e = edge("e1", "Intent#a", "Feature#b", RelationshipType::Implements);
    e.meta.source = "  ".into();
    let g = make_graph(
        vec![
            node("Intent#a", NodeType::Intent, DagStage::Intent),
            node("Feature#b", NodeType::Feature, DagStage::Feature),
        ],
        vec![e],
    );
    assert!(matches!(
        g.check_edge_constraints().unwrap_err(),
        ValidationError::MissingMeta(_)
    ));
}

#[test]
fn edge_negative_confidence_rejected() {
    let mut e = edge("e1", "Intent#a", "Feature#b", RelationshipType::Implements);
    e.meta.confidence = Some(-0.1);
    let g = make_graph(
        vec![
            node("Intent#a", NodeType::Intent, DagStage::Intent),
            node("Feature#b", NodeType::Feature, DagStage::Feature),
        ],
        vec![e],
    );
    assert!(matches!(
        g.check_edge_constraints().unwrap_err(),
        ValidationError::ConfidenceOutOfRange(_)
    ));
}

// ---------------------------------------------------------------------------
// Node meta validation
// ---------------------------------------------------------------------------

#[test]
fn node_empty_meta_source_rejected() {
    let mut n = node("Intent#root", NodeType::Intent, DagStage::Intent);
    n.meta.source = "".into();
    let g = make_graph(vec![n], vec![]);
    let err = g.validate().unwrap_err();
    assert!(err.iter().any(|e| matches!(e, ValidationError::MissingMeta(_))));
}

// ---------------------------------------------------------------------------
// DAG: non-intent root
// ---------------------------------------------------------------------------

#[test]
fn non_intent_root_rejected() {
    let g = make_graph(
        vec![
            node("Feature#root", NodeType::Feature, DagStage::Feature),
            node("Task#child", NodeType::Task, DagStage::Task),
        ],
        vec![edge("e1", "Feature#root", "Task#child", RelationshipType::Implements)],
    );
    assert!(matches!(
        g.check_dag().unwrap_err(),
        ValidationError::InvalidRootNode(_)
    ));
}

// ---------------------------------------------------------------------------
// DAG: disconnected graph with multiple intent roots is valid
// ---------------------------------------------------------------------------

#[test]
fn multiple_intent_roots_no_edges_valid() {
    let g = make_graph(
        vec![
            node("Intent#a", NodeType::Intent, DagStage::Intent),
            node("Intent#b", NodeType::Intent, DagStage::Intent),
        ],
        vec![],
    );
    assert!(g.check_dag().is_ok());
}

// ---------------------------------------------------------------------------
// Full validation pipeline on complex graph
// ---------------------------------------------------------------------------

#[test]
fn full_validation_pipeline() {
    let g = make_graph(
        vec![
            node("Intent#root", NodeType::Intent, DagStage::Intent),
            node("Feature#oauth", NodeType::Feature, DagStage::Feature),
            node("Task#impl", NodeType::Task, DagStage::Task),
            node("Test#unit", NodeType::Test, DagStage::Test),
            node("Bug#issue-one", NodeType::Bug, DagStage::Bug),
        ],
        vec![
            edge("e1", "Intent#root", "Feature#oauth", RelationshipType::Implements),
            edge("e2", "Feature#oauth", "Task#impl", RelationshipType::Implements),
            edge("e3", "Feature#oauth", "Test#unit", RelationshipType::Tests),
            edge("e4", "Task#impl", "Bug#issue-one", RelationshipType::DerivesFrom),
        ],
    );
    assert!(g.validate().is_ok());
}

// ---------------------------------------------------------------------------
// Serialize/deserialize full graph
// ---------------------------------------------------------------------------

#[test]
fn full_graph_serde_roundtrip() {
    let g = make_graph(
        vec![
            node("Intent#root", NodeType::Intent, DagStage::Intent),
            node("Feature#a", NodeType::Feature, DagStage::Feature),
        ],
        vec![edge("e1", "Intent#root", "Feature#a", RelationshipType::Implements)],
    );
    let json = serde_json::to_string_pretty(&g).unwrap();
    let back: IntentGraph = serde_json::from_str(&json).unwrap();
    assert_eq!(back.nodes.len(), 2);
    assert_eq!(back.edges.len(), 1);
    assert_eq!(back.metadata.version, "1.0");
}

// ---------------------------------------------------------------------------
// ValidationError display messages
// ---------------------------------------------------------------------------

#[test]
fn all_validation_error_variants_display() {
    let errors: Vec<ValidationError> = vec![
        ValidationError::InvalidNodeId("bad".into()),
        ValidationError::MissingRequiredField("field".into()),
        ValidationError::UnknownNodeType("X".into()),
        ValidationError::InvalidDagStage("y".into()),
        ValidationError::UnknownRelationshipType("z".into()),
        ValidationError::UnknownCanonicalLinkType("w".into()),
        ValidationError::UnknownStatus("s".into()),
        ValidationError::InvalidEdgeConstraint {
            relationship: "implements".into(),
            from: "Bug".into(),
            to: "Feature".into(),
        },
        ValidationError::CycleDetected,
        ValidationError::InvalidRootNode("Feature".into()),
        ValidationError::MissingMeta("node".into()),
        ValidationError::DuplicateNodeId("dup".into()),
        ValidationError::OrphanedEdge {
            edge_id: "e1".into(),
            node_id: "n1".into(),
        },
        ValidationError::ConfidenceOutOfRange(1.5),
    ];
    for e in errors {
        let msg = e.to_string();
        assert!(!msg.is_empty(), "display should produce non-empty string");
    }
}

// ---------------------------------------------------------------------------
// Dangling edges (endpoint not present in `nodes`)
// ---------------------------------------------------------------------------

/// Pins the current *permissive* contract for dangling edges.
///
/// `check_edge_against_node_map` bails out with `None` when either endpoint is
/// missing from the node map, so an edge whose endpoint is absent from
/// `graph.nodes` is never checked against the ontology constraints and never
/// reported as `OrphanedEdge`. The presence of the endpoint — not the edge
/// itself — decides whether the constraint is enforced. Pinned here so that
/// changing that contract becomes a deliberate decision instead of an
/// accidental regression.
#[test]
fn dangling_edge_endpoint_suppresses_constraint_check() {
    // `Tests` requires a (Feature|Task|Commit|PR|Bug) -> Test pair, so with the
    // target node present this exact edge is rejected.
    let present = make_graph(
        vec![
            node("Intent#root", NodeType::Intent, DagStage::Intent),
            node("Feature#a", NodeType::Feature, DagStage::Feature),
        ],
        vec![edge(
            "e1",
            "Intent#root",
            "Feature#a",
            RelationshipType::Tests,
        )],
    );
    let err = present
        .check_edge_constraints()
        .expect_err("Intent -> Feature is not a legal Tests pair");
    assert!(matches!(err, ValidationError::InvalidEdgeConstraint { .. }));

    // Same edge, but the target is absent from `nodes`: nothing is reported.
    let dangling_target = make_graph(
        vec![node("Intent#root", NodeType::Intent, DagStage::Intent)],
        vec![edge(
            "e1",
            "Intent#root",
            "Feature#ghost",
            RelationshipType::Tests,
        )],
    );
    assert!(dangling_target.check_edge_constraints().is_ok());
    assert!(dangling_target.validate().is_ok());

    // An absent *source* is skipped the same way.
    let dangling_source = make_graph(
        vec![node("Feature#a", NodeType::Feature, DagStage::Feature)],
        vec![edge(
            "e1",
            "Intent#ghost",
            "Feature#a",
            RelationshipType::Tests,
        )],
    );
    assert!(dangling_source.check_edge_constraints().is_ok());
    assert!(dangling_source.validate().is_ok());
}
