//! Integration contract tests for the intent graph ontology and validation.
//!
//! The ontology in `agileplus_domain::intent_graph` is the traceability spine:
//! every PM artifact (intent, plan, feature, task, commit, PR, bug, …) becomes a
//! `Node`, and every traceability link becomes an `Edge` whose endpoints must
//! satisfy the encoded ontology rules. Losing a rule here silently corrupts
//! traceability reports, so these tests pin the wire labels, the serde
//! representation, the node-id grammar, DAG acyclicity, and every allowed
//! relationship/endpoint pair from the public API.
//!
//! These tests only touch pure data structures — no filesystem, no env vars —
//! so they never need the shared environment lock.
//!
//! Traceability: FR-021 (traceability graph), WP07

use agileplus_domain::intent_graph::{
    CanonicalLinkType, CanonicalMap, DagStage, Edge, GraphMetadata, IntentGraph, Meta, Node,
    NodeType, RelationshipType, Status, ValidationError,
};
use chrono::Utc;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn meta() -> Meta {
    Meta {
        confidence: Some(0.9),
        source: "test".to_string(),
        timestamp: Utc::now(),
        agent_id: None,
    }
}

fn node(id: &str, node_type: NodeType, dag_stage: DagStage) -> Node {
    Node {
        id: id.to_string(),
        node_type,
        dag_stage,
        title: format!("title for {id}"),
        description: None,
        status: Status::Draft,
        tags: Vec::new(),
        meta: meta(),
        properties: None,
        table_ref: None,
        table_id: None,
    }
}

fn build_edge(id: &str, source: &str, target: &str, rel: RelationshipType) -> Edge {
    Edge {
        id: id.to_string(),
        source: source.to_string(),
        target: target.to_string(),
        relationship_type: rel,
        canonical_map: None,
        meta: meta(),
        properties: None,
    }
}

fn build_graph(nodes: Vec<Node>, edges: Vec<Edge>) -> IntentGraph {
    IntentGraph {
        nodes,
        edges,
        metadata: GraphMetadata {
            version: "1.0.0".to_string(),
            schema_uri: "https://agileplus.dev/schema/intent-graph-v1.json".to_string(),
            created_at: Utc::now(),
            updated_at: None,
            node_count: None,
            edge_count: None,
            dag_valid: None,
            source_system: None,
        },
    }
}

fn all_node_types() -> Vec<NodeType> {
    vec![
        NodeType::Intent,
        NodeType::Plan,
        NodeType::Feature,
        NodeType::Story,
        NodeType::Task,
        NodeType::Spec,
        NodeType::Commit,
        NodeType::Test,
        NodeType::PR,
        NodeType::Bug,
        NodeType::Artifact,
    ]
}

fn all_dag_stages() -> Vec<DagStage> {
    vec![
        DagStage::Intent,
        DagStage::Plan,
        DagStage::Feature,
        DagStage::Story,
        DagStage::Task,
        DagStage::Spec,
        DagStage::Commit,
        DagStage::Test,
        DagStage::PR,
        DagStage::Bug,
        DagStage::Artifact,
    ]
}

fn all_statuses() -> Vec<Status> {
    vec![
        Status::Draft,
        Status::Active,
        Status::Completed,
        Status::Deprecated,
        Status::Rejected,
        Status::Open,
        Status::InProgress,
        Status::Blocked,
        Status::Deferred,
        Status::Cancelled,
    ]
}

/// `(relationship, allowed source type, allowed target type)` triples from the
/// ontology schema. Every entry must pass, and a representative disallowed pair
/// per relationship must fail.
fn allowed_triples() -> Vec<(RelationshipType, NodeType, NodeType)> {
    use NodeType::*;
    use RelationshipType::*;
    vec![
        (Implements, Intent, Feature),
        (Implements, Intent, Story),
        (Implements, Feature, Task),
        (Implements, Story, Task),
        (Implements, Task, Commit),
        (Implements, Spec, Feature),
        (Implements, Spec, Task),
        (Tests, Feature, Test),
        (Tests, Task, Test),
        (Tests, Commit, Test),
        (Tests, PR, Test),
        (Tests, Bug, Test),
        (Covers, Feature, Test),
        (Covers, Task, Test),
        (Covers, Feature, Artifact),
        (Covers, Task, Artifact),
        (Covers, Spec, Feature),
        (DerivesFrom, Feature, Intent),
        (DerivesFrom, Story, Intent),
        (DerivesFrom, Story, Feature),
        (DerivesFrom, Task, Feature),
        (DerivesFrom, Task, Story),
        (DerivesFrom, Task, Bug),
        (Resolves, Bug, Commit),
        (Resolves, Bug, PR),
        (Resolves, Bug, Task),
        (Blocks, Task, Task),
        (Blocks, Bug, Task),
        (Blocks, Bug, PR),
        (Blocks, Task, PR),
        (Blocks, PR, Feature),
        (DependsOn, Task, Task),
        (DependsOn, Feature, Feature),
        (DependsOn, Story, Story),
        (DependsOn, PR, PR),
        (DependsOn, Task, Artifact),
        // `traces_to` is the wildcard relationship: any endpoint pair is legal.
        (TracesTo, Intent, Test),
        (TracesTo, Artifact, Bug),
    ]
}

// ---------------------------------------------------------------------------
// Wire labels and conversions
// ---------------------------------------------------------------------------

#[test]
fn node_type_display_from_and_tryfrom_agree_for_every_variant() {
    for node_type in all_node_types() {
        let label = node_type.to_string();
        // `Display` is PascalCase and matches the enum variant name.
        assert_eq!(label, format!("{node_type:?}"));
        assert_eq!(NodeType::from(label.as_str()), node_type);
        assert_eq!(NodeType::try_from(label).unwrap(), node_type);
    }
    // Unknown labels fail the fallible path.
    for bad in ["", "Intent ", "intent", "Story#1", "Unknown"] {
        assert!(
            NodeType::try_from(bad.to_string()).is_err(),
            "expected `{bad}` to be rejected"
        );
    }
}

#[test]
fn node_type_serde_uses_pascal_case() {
    for node_type in all_node_types() {
        let encoded = serde_json::to_string(&node_type).unwrap();
        let decoded: NodeType = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, node_type);
        assert_eq!(encoded, format!("\"{node_type:?}\""));
    }
}

#[test]
fn dag_stage_display_from_and_tryfrom_agree_for_every_variant() {
    for stage in all_dag_stages() {
        let label = stage.to_string();
        assert_eq!(DagStage::from(label.as_str()), stage);
        // Stage labels are lowercase.
        assert_eq!(label, label.to_ascii_lowercase());
        assert_eq!(DagStage::try_from(label.clone()).unwrap(), stage);
    }
    for bad in ["", "Intent", "task ", "Commit"] {
        assert!(DagStage::try_from(bad.to_string()).is_err());
    }
}

#[test]
fn relationship_type_wire_labels_are_kebab_case() {
    let expected = [
        (RelationshipType::Implements, "implements"),
        (RelationshipType::Tests, "tests"),
        (RelationshipType::Covers, "covers"),
        (RelationshipType::TracesTo, "traces-to"),
        (RelationshipType::DerivesFrom, "derives-from"),
        (RelationshipType::Resolves, "resolves"),
        (RelationshipType::Blocks, "blocks"),
        (RelationshipType::DependsOn, "depends-on"),
    ];
    for (rel, label) in expected {
        assert_eq!(rel.to_string(), label);
        assert_eq!(RelationshipType::from(label), rel);
        assert_eq!(RelationshipType::try_from(label.to_string()).unwrap(), rel);
        assert_eq!(serde_json::to_string(&rel).unwrap(), format!("\"{label}\""));
    }
    // Snake case and PascalCase spellings are not accepted on the wire.
    for bad in ["", "TracesTo", "traces_to", "depends_on"] {
        assert!(
            RelationshipType::try_from(bad.to_string()).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn canonical_link_type_wire_labels_are_snake_case() {
    let expected = [
        (CanonicalLinkType::ParentOf, "parent_of"),
        (CanonicalLinkType::ChildOf, "child_of"),
        (CanonicalLinkType::DependsOn, "depends_on"),
        (CanonicalLinkType::Blocks, "blocks"),
        (CanonicalLinkType::Implements, "implements"),
        (CanonicalLinkType::Verifies, "verifies"),
        (CanonicalLinkType::References, "references"),
        (CanonicalLinkType::Duplicates, "duplicates"),
    ];
    for (link, label) in expected {
        assert_eq!(link.to_string(), label);
        assert_eq!(CanonicalLinkType::from(label), link);
        assert_eq!(
            CanonicalLinkType::try_from(label.to_string()).unwrap(),
            link
        );
    }
    for bad in ["", "ParentOf", "parent-of", "PARENT_OF"] {
        assert!(
            CanonicalLinkType::try_from(bad.to_string()).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn status_wire_labels_are_snake_case() {
    let expected = [
        (Status::Draft, "draft"),
        (Status::Active, "active"),
        (Status::Completed, "completed"),
        (Status::Deprecated, "deprecated"),
        (Status::Rejected, "rejected"),
        (Status::Open, "open"),
        (Status::InProgress, "in_progress"),
        (Status::Blocked, "blocked"),
        (Status::Deferred, "deferred"),
        (Status::Cancelled, "cancelled"),
    ];
    assert_eq!(expected.len(), all_statuses().len());
    for (status, label) in expected {
        assert_eq!(status.to_string(), label);
        assert_eq!(Status::from(label), status);
        assert_eq!(Status::try_from(label.to_string()).unwrap(), status);
    }
    for bad in ["", "Draft", "in-progress", "Unknown"] {
        assert!(Status::try_from(bad.to_string()).is_err(), "{bad}");
    }
}

#[test]
fn node_id_grammar_accepts_typed_slugs_and_rejects_off_grammar_ids() {
    // A well-formed graph: `Intent#auth` is the only root.
    let singleton = build_graph(
        vec![node("Intent#auth", NodeType::Intent, DagStage::Intent)],
        vec![],
    );
    assert!(singleton.validate().is_ok());

    for bad_id in [
        "intent#auth",   // lowercase type
        "Intent",        // missing separator
        "Intent#",       // empty slug
        "#auth",         // empty type
        "Intent#Auth",   // uppercase slug
        "Intent#auth_1", // underscore is not in the slug alphabet
        "Intent#a#b",    // extra separator
        "PR#1234",       // acronym type is not `[A-Z][a-z]+`
        "I#auth",        // single-letter type
    ] {
        let bad = build_graph(
            vec![node(bad_id, NodeType::Intent, DagStage::Intent)],
            vec![],
        );
        let errors = bad.validate().unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, ValidationError::InvalidNodeId(id) if id == bad_id)),
            "`{bad_id}` should be an invalid node id, got {errors:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Node-level validation
// ---------------------------------------------------------------------------

#[test]
fn duplicate_node_ids_are_reported_once_per_extra_occurrence() {
    let duplicated = build_graph(
        vec![
            node("Intent#a", NodeType::Intent, DagStage::Intent),
            node("Intent#a", NodeType::Intent, DagStage::Intent),
            node("Intent#a", NodeType::Intent, DagStage::Intent),
        ],
        vec![],
    );
    let errors = duplicated.validate().unwrap_err();
    let duplicates: Vec<_> = errors
        .iter()
        .filter(|e| matches!(e, ValidationError::DuplicateNodeId(id) if id == "Intent#a"))
        .collect();
    assert_eq!(
        duplicates.len(),
        2,
        "each repeat after the first is an error"
    );
}

#[test]
fn node_meta_source_must_not_be_blank_and_confidence_must_be_a_probability() {
    let mut blank_source = node("Intent#a", NodeType::Intent, DagStage::Intent);
    blank_source.meta.source = "   ".to_string();
    let graph = build_graph(vec![blank_source], vec![]);
    assert!(
        graph
            .validate()
            .unwrap_err()
            .iter()
            .any(|e| matches!(e, ValidationError::MissingMeta(_)))
    );

    for out_of_range in [-0.001, 1.001, 42.0] {
        let mut bad_confidence = node("Intent#a", NodeType::Intent, DagStage::Intent);
        bad_confidence.meta.confidence = Some(out_of_range);
        let errors = build_graph(vec![bad_confidence], vec![])
            .validate()
            .unwrap_err();
        assert!(
            errors.iter().any(
                |e| matches!(e, ValidationError::ConfidenceOutOfRange(c) if *c == out_of_range)
            ),
            "confidence {out_of_range} should be rejected, got {errors:?}"
        );
    }

    // NaN is not within `0.0..=1.0`, so it is rejected as well.
    let mut nan_confidence = node("Intent#a", NodeType::Intent, DagStage::Intent);
    nan_confidence.meta.confidence = Some(f64::NAN);
    let errors = build_graph(vec![nan_confidence], vec![])
        .validate()
        .unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::ConfidenceOutOfRange(c) if c.is_nan())),
        "NaN confidence should be rejected, got {errors:?}"
    );

    // Both endpoints of the valid range and `None` are accepted.
    for in_range in [0.0, 0.5, 1.0] {
        let mut ok = node("Intent#a", NodeType::Intent, DagStage::Intent);
        ok.meta.confidence = Some(in_range);
        assert!(build_graph(vec![ok], vec![]).validate().is_ok());
    }
    let mut no_confidence = node("Intent#a", NodeType::Intent, DagStage::Intent);
    no_confidence.meta.confidence = None;
    assert!(build_graph(vec![no_confidence], vec![]).validate().is_ok());
}

#[test]
fn validate_collects_node_edge_and_dag_errors_together() {
    let mut bad_node = node("bad-id", NodeType::Feature, DagStage::Feature);
    bad_node.meta.source = String::new();
    let bad_edge = build_edge(
        "edge-1",
        "Feature#missing",
        "Task#also-missing",
        RelationshipType::Implements,
    );

    let errors = build_graph(vec![bad_node], vec![bad_edge])
        .validate()
        .unwrap_err();

    // Invalid id, blank meta source, and the root node not being an Intent.
    let seen_invalid_id = errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidNodeId(_)));
    let seen_missing_meta = errors
        .iter()
        .any(|e| matches!(e, ValidationError::MissingMeta(_)));
    let seen_bad_root = errors
        .iter()
        .any(|e| matches!(e, ValidationError::InvalidRootNode(_)));
    assert!(seen_invalid_id, "{errors:?}");
    assert!(seen_missing_meta, "{errors:?}");
    assert!(seen_bad_root, "{errors:?}");
}

// ---------------------------------------------------------------------------
// Edge-level validation
// ---------------------------------------------------------------------------

#[test]
fn every_allowed_relationship_endpoint_pair_passes_constraint_checking() {
    for (rel, source_type, target_type) in allowed_triples() {
        let graph = build_graph(
            vec![
                node("Intent#root", NodeType::Intent, DagStage::Intent),
                node("Source#one", source_type, DagStage::Feature),
                node("Target#one", target_type, DagStage::Feature),
            ],
            vec![build_edge("edge-1", "Source#one", "Target#one", rel)],
        );
        assert!(
            graph.check_edge_constraints().is_ok(),
            "{rel} {source_type} -> {target_type} should be allowed"
        );
    }
}

#[test]
fn relationships_reject_endpoint_pairs_the_ontology_forbids() {
    use NodeType::*;
    use RelationshipType::*;
    // One clearly-illegal pair per relationship (skipping the wildcard).
    let cases = [
        (Implements, Intent, Test),
        (Tests, Intent, Feature),
        (Covers, Intent, Commit),
        (DerivesFrom, Intent, Feature),
        (Resolves, Feature, Commit),
        (Blocks, Intent, Task),
        (DependsOn, Intent, Task),
    ];
    for (rel, source_type, target_type) in cases {
        let graph = build_graph(
            vec![
                node("Intent#root", NodeType::Intent, DagStage::Intent),
                node("Source#one", source_type, DagStage::Feature),
                node("Target#one", target_type, DagStage::Feature),
            ],
            vec![build_edge("edge-1", "Source#one", "Target#one", rel)],
        );
        let error = graph.check_edge_constraints().expect_err(&format!(
            "{rel} {source_type} -> {target_type} must be rejected"
        ));
        match error {
            ValidationError::InvalidEdgeConstraint {
                relationship,
                from,
                to,
            } => {
                assert_eq!(relationship, rel.to_string());
                assert_eq!(from, source_type.to_string());
                assert_eq!(to, target_type.to_string());
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}

#[test]
fn traces_to_accepts_every_endpoint_pair() {
    for source_type in all_node_types() {
        for target_type in all_node_types() {
            let graph = build_graph(
                vec![
                    node("Intent#root", NodeType::Intent, DagStage::Intent),
                    node("Source#one", source_type, DagStage::Feature),
                    node("Target#one", target_type, DagStage::Feature),
                ],
                vec![build_edge(
                    "edge-1",
                    "Source#one",
                    "Target#one",
                    RelationshipType::TracesTo,
                )],
            );
            assert!(
                graph.check_edge_constraints().is_ok(),
                "traces-to {source_type} -> {target_type} must be allowed"
            );
        }
    }
}

#[test]
fn edges_reference_missing_endpoints_are_ignored_by_constraint_checks() {
    // `check_edge_against_node_map` returns `None` when either endpoint is
    // unknown, so an edge to nowhere is not a constraint violation. This is the
    // documented (and relied-upon) lenient behavior for partially-loaded graphs.
    let graph = build_graph(
        vec![node("Intent#a", NodeType::Intent, DagStage::Intent)],
        vec![build_edge(
            "edge-1",
            "Intent#a",
            "Feature#missing",
            RelationshipType::Implements,
        )],
    );
    assert!(graph.check_edge_constraints().is_ok());
}

#[test]
fn edge_meta_source_and_confidence_are_validated() {
    let mut blank = build_edge(
        "edge-1",
        "Intent#a",
        "Feature#b",
        RelationshipType::Implements,
    );
    blank.meta.source = "  ".to_string();
    let graph = build_graph(
        vec![
            node("Intent#a", NodeType::Intent, DagStage::Intent),
            node("Feature#b", NodeType::Feature, DagStage::Feature),
        ],
        vec![blank],
    );
    assert!(matches!(
        graph.check_edge_constraints().unwrap_err(),
        ValidationError::MissingMeta(_)
    ));

    let mut out_of_range = build_edge(
        "edge-1",
        "Intent#a",
        "Feature#b",
        RelationshipType::Implements,
    );
    out_of_range.meta.confidence = Some(1.5);
    let graph = build_graph(
        vec![
            node("Intent#a", NodeType::Intent, DagStage::Intent),
            node("Feature#b", NodeType::Feature, DagStage::Feature),
        ],
        vec![out_of_range],
    );
    assert!(matches!(
        graph.check_edge_constraints().unwrap_err(),
        ValidationError::ConfidenceOutOfRange(_)
    ));
}

// ---------------------------------------------------------------------------
// DAG validation
// ---------------------------------------------------------------------------

#[test]
fn a_cycle_of_any_length_is_rejected() {
    // Self loop.
    let self_loop = build_graph(
        vec![node("Task#a", NodeType::Task, DagStage::Task)],
        vec![build_edge(
            "edge-1",
            "Task#a",
            "Task#a",
            RelationshipType::DependsOn,
        )],
    );
    assert_eq!(
        self_loop.check_dag().unwrap_err(),
        ValidationError::CycleDetected
    );

    // Two-node cycle.
    let two_cycle = build_graph(
        vec![
            node("Task#a", NodeType::Task, DagStage::Task),
            node("Task#b", NodeType::Task, DagStage::Task),
        ],
        vec![
            build_edge("edge-1", "Task#a", "Task#b", RelationshipType::DependsOn),
            build_edge("edge-2", "Task#b", "Task#a", RelationshipType::DependsOn),
        ],
    );
    assert_eq!(
        two_cycle.check_dag().unwrap_err(),
        ValidationError::CycleDetected
    );

    // Three-node cycle.
    let three_cycle = build_graph(
        vec![
            node("Task#a", NodeType::Task, DagStage::Task),
            node("Task#b", NodeType::Task, DagStage::Task),
            node("Task#c", NodeType::Task, DagStage::Task),
        ],
        vec![
            build_edge("edge-1", "Task#a", "Task#b", RelationshipType::DependsOn),
            build_edge("edge-2", "Task#b", "Task#c", RelationshipType::DependsOn),
            build_edge("edge-3", "Task#c", "Task#a", RelationshipType::DependsOn),
        ],
    );
    assert_eq!(
        three_cycle.check_dag().unwrap_err(),
        ValidationError::CycleDetected
    );
}

#[test]
fn a_diamond_is_acyclic_and_the_shared_node_has_in_degree_two() {
    let diamond = build_graph(
        vec![
            node("Intent#root", NodeType::Intent, DagStage::Intent),
            node("Feature#left", NodeType::Feature, DagStage::Feature),
            node("Feature#right", NodeType::Feature, DagStage::Feature),
            node("Task#join", NodeType::Task, DagStage::Task),
        ],
        vec![
            build_edge(
                "edge-1",
                "Intent#root",
                "Feature#left",
                RelationshipType::Implements,
            ),
            build_edge(
                "edge-2",
                "Intent#root",
                "Feature#right",
                RelationshipType::Implements,
            ),
            build_edge(
                "edge-3",
                "Feature#left",
                "Task#join",
                RelationshipType::Implements,
            ),
            build_edge(
                "edge-4",
                "Feature#right",
                "Task#join",
                RelationshipType::Implements,
            ),
        ],
    );
    assert!(diamond.check_dag().is_ok());
    assert!(diamond.validate().is_ok());
}

#[test]
fn the_only_root_must_be_the_intent_node() {
    let feature_root = build_graph(
        vec![node("Feature#a", NodeType::Feature, DagStage::Feature)],
        vec![],
    );
    assert_eq!(
        feature_root.check_dag().unwrap_err(),
        ValidationError::InvalidRootNode("Feature".to_string())
    );

    // A second root that is not an Intent is also rejected.
    let two_roots = build_graph(
        vec![
            node("Intent#root", NodeType::Intent, DagStage::Intent),
            node("Task#orphan", NodeType::Task, DagStage::Task),
        ],
        vec![],
    );
    assert!(matches!(
        two_roots.check_dag().unwrap_err(),
        ValidationError::InvalidRootNode(_)
    ));
}

#[test]
fn an_empty_graph_is_trivially_valid() {
    let empty = build_graph(vec![], vec![]);
    assert!(empty.check_dag().is_ok());
    assert!(empty.check_edge_constraints().is_ok());
    assert!(empty.validate().is_ok());
}

// ---------------------------------------------------------------------------
// Error rendering
// ---------------------------------------------------------------------------

#[test]
fn validation_error_display_messages_name_the_offending_input() {
    let cases: Vec<(ValidationError, String)> = vec![
        (
            ValidationError::InvalidNodeId("bad".into()),
            "invalid node ID: bad".into(),
        ),
        (
            ValidationError::MissingRequiredField("title".into()),
            "missing required field: title".into(),
        ),
        (
            ValidationError::UnknownNodeType("Widget".into()),
            "unknown node type: Widget".into(),
        ),
        (
            ValidationError::InvalidDagStage("warp".into()),
            "unknown DAG stage: warp".into(),
        ),
        (
            ValidationError::UnknownRelationshipType("links".into()),
            "unknown relationship type: links".into(),
        ),
        (
            ValidationError::UnknownCanonicalLinkType("links".into()),
            "unknown canonical link type: links".into(),
        ),
        (
            ValidationError::UnknownStatus("half-done".into()),
            "unknown status: half-done".into(),
        ),
        (
            ValidationError::InvalidEdgeConstraint {
                relationship: "implements".into(),
                from: "Intent".into(),
                to: "Test".into(),
            },
            "invalid edge constraint: implements from Intent to Test".into(),
        ),
        (
            ValidationError::CycleDetected,
            "cycle detected in graph".into(),
        ),
        (
            ValidationError::InvalidRootNode("Feature".into()),
            "invalid root node: expected Intent, got Feature".into(),
        ),
        (
            ValidationError::MissingMeta("node Intent#a: source is empty".into()),
            "missing meta on node Intent#a: source is empty".into(),
        ),
        (
            ValidationError::DuplicateNodeId("Intent#a".into()),
            "duplicate node ID: Intent#a".into(),
        ),
        (
            ValidationError::OrphanedEdge {
                edge_id: "edge-1".into(),
                node_id: "Feature#gone".into(),
            },
            "orphaned edge: edge-1 references missing node Feature#gone".into(),
        ),
        (
            ValidationError::ConfidenceOutOfRange(1.5),
            "confidence out of range: 1.5".into(),
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}

// ---------------------------------------------------------------------------
// Serde representation
// ---------------------------------------------------------------------------

#[test]
fn node_and_edge_round_trip_through_json() {
    let original = Node {
        id: "Feature#oauth2-login".to_string(),
        node_type: NodeType::Feature,
        dag_stage: DagStage::Feature,
        title: "OAuth2 Login".to_string(),
        description: Some("Adds OAuth2".to_string()),
        status: Status::Active,
        tags: vec!["auth".to_string()],
        meta: meta(),
        properties: Some(serde_json::json!({ "estimate": 3 })),
        table_ref: Some("features".to_string()),
        table_id: Some("F-1".to_string()),
    };
    let restored: Node = serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap();
    assert_eq!(restored.id, original.id);
    assert_eq!(restored.node_type, NodeType::Feature);
    assert_eq!(restored.status, Status::Active);
    assert_eq!(restored.tags, original.tags);
    assert_eq!(restored.properties, original.properties);
    assert_eq!(restored.table_id, original.table_id);

    let edge = Edge {
        id: "edge-1".to_string(),
        source: "Intent#auth".to_string(),
        target: "Feature#oauth2-login".to_string(),
        relationship_type: RelationshipType::Implements,
        canonical_map: Some(CanonicalMap {
            link_type: CanonicalLinkType::ParentOf,
            direction: Some("forward".to_string()),
        }),
        meta: meta(),
        properties: None,
    };
    let restored: Edge = serde_json::from_str(&serde_json::to_string(&edge).unwrap()).unwrap();
    assert_eq!(restored.source, edge.source);
    assert_eq!(restored.target, edge.target);
    assert_eq!(restored.relationship_type, RelationshipType::Implements);
    assert_eq!(
        restored.canonical_map.unwrap().link_type,
        CanonicalLinkType::ParentOf
    );

    // A whole graph survives a JSON round trip and still validates.
    let full = build_graph(
        vec![
            node("Intent#auth", NodeType::Intent, DagStage::Intent),
            node("Feature#oauth2", NodeType::Feature, DagStage::Feature),
        ],
        vec![build_edge(
            "edge-1",
            "Intent#auth",
            "Feature#oauth2",
            RelationshipType::Implements,
        )],
    );
    let restored: IntentGraph =
        serde_json::from_str(&serde_json::to_string(&full).unwrap()).unwrap();
    assert_eq!(restored.nodes.len(), 2);
    assert_eq!(restored.edges.len(), 1);
    assert!(restored.validate().is_ok());
}

#[test]
fn optional_fields_are_omitted_from_serialized_json() {
    let bare = Node {
        id: "Intent#a".to_string(),
        node_type: NodeType::Intent,
        dag_stage: DagStage::Intent,
        title: "A".to_string(),
        description: None,
        status: Status::Draft,
        tags: Vec::new(),
        meta: Meta {
            confidence: None,
            source: "test".to_string(),
            timestamp: Utc::now(),
            agent_id: None,
        },
        properties: None,
        table_ref: None,
        table_id: None,
    };
    let json: serde_json::Value = serde_json::to_value(&bare).unwrap();
    for absent in ["description", "properties", "table_ref", "table_id", "tags"] {
        assert!(
            json.get(absent).is_none(),
            "`{absent}` should be omitted when empty"
        );
    }
    assert!(json["meta"].get("confidence").is_none());
    assert!(json["meta"].get("agent_id").is_none());

    let metadata = GraphMetadata {
        version: "1.0.0".to_string(),
        schema_uri: "s".to_string(),
        created_at: Utc::now(),
        updated_at: None,
        node_count: None,
        edge_count: None,
        dag_valid: None,
        source_system: None,
    };
    let json: serde_json::Value = serde_json::to_value(&metadata).unwrap();
    for absent in [
        "updated_at",
        "node_count",
        "edge_count",
        "dag_valid",
        "source_system",
    ] {
        assert!(json.get(absent).is_none(), "`{absent}` should be omitted");
    }
}
