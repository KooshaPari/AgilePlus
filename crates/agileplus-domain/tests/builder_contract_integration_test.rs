//! Integration contract tests for the node/edge builders in
//! `agileplus_domain::builder`.
//!
//! The builders are the only safe way to mint graph nodes and edges: they
//! enforce the required-field contract, the node-id grammar, and the
//! type→DAG-stage default mapping. A regression here is invisible until a graph
//! fails to validate in production, so these tests drive every setter and every
//! rejection path through the public API.
//!
//! Traceability: FR-021 (traceability graph), WP07

use agileplus_domain::builder::{EdgeBuilder, NodeBuilder};
use agileplus_domain::intent_graph::{
    CanonicalLinkType, CanonicalMap, DagStage, Node, NodeType, RelationshipType, Status,
    ValidationError,
};
use chrono::Utc;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn sample_meta() -> agileplus_domain::intent_graph::Meta {
    agileplus_domain::intent_graph::Meta {
        confidence: Some(0.75),
        source: "integration-test".to_string(),
        timestamp: Utc::now(),
        agent_id: Some("agent-7".to_string()),
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

// ---------------------------------------------------------------------------
// NodeBuilder happy paths
// ---------------------------------------------------------------------------

#[test]
fn every_node_type_defaults_to_its_matching_dag_stage() {
    let expected = [
        (NodeType::Intent, DagStage::Intent, "Intent#x"),
        (NodeType::Plan, DagStage::Plan, "Plan#x"),
        (NodeType::Feature, DagStage::Feature, "Feature#x"),
        (NodeType::Story, DagStage::Story, "Story#x"),
        (NodeType::Task, DagStage::Task, "Task#x"),
        (NodeType::Spec, DagStage::Spec, "Spec#x"),
        (NodeType::Commit, DagStage::Commit, "Commit#x"),
        (NodeType::Test, DagStage::Test, "Test#x"),
        // `PR` is the one acronym type; see the dedicated test below for why it
        // needs an explicit `Pr#…` id.
        (NodeType::PR, DagStage::PR, "Pr#x"),
        (NodeType::Bug, DagStage::Bug, "Bug#x"),
        (NodeType::Artifact, DagStage::Artifact, "Artifact#x"),
    ];
    assert_eq!(expected.len(), all_node_types().len());

    for (node_type, stage, id) in expected {
        let built = NodeBuilder::new(node_type)
            .id(id)
            .title("Some Title")
            .meta(sample_meta())
            .build()
            .unwrap_or_else(|e| panic!("{node_type:?} should build: {e:?}"));
        assert_eq!(built.node_type, node_type);
        assert_eq!(built.dag_stage, stage);
        // Defaults.
        assert_eq!(built.status, Status::Draft);
        assert!(built.tags.is_empty());
        assert!(built.description.is_none());
    }
}

#[test]
fn the_pr_node_type_requires_an_explicit_id_because_its_acronym_prefix_is_rejected() {
    // The node-id grammar is `^[A-Z][a-z]+#…`, so the auto-generated `PR#slug`
    // (node type Display is the PascalCase enum name) is not a legal id. Callers
    // that mint PR nodes must therefore pass an explicit id such as `Pr#123`.
    let generated = NodeBuilder::new(NodeType::PR)
        .title("Some Title")
        .meta(sample_meta())
        .build();
    assert_eq!(
        generated.unwrap_err(),
        ValidationError::InvalidNodeId("PR#some-title".to_string())
    );

    let explicit = NodeBuilder::new(NodeType::PR)
        .id("Pr#1234")
        .title("Some Title")
        .meta(sample_meta())
        .build()
        .expect("an explicit id makes PR nodes buildable");
    assert_eq!(explicit.id, "Pr#1234");
    assert_eq!(explicit.dag_stage, DagStage::PR);
}

#[test]
fn an_explicit_dag_stage_overrides_the_node_type_default() {
    let node = NodeBuilder::new(NodeType::Task)
        .title("Spec written before implementation")
        .dag_stage(DagStage::Spec)
        .meta(sample_meta())
        .build()
        .unwrap();
    assert_eq!(node.dag_stage, DagStage::Spec);
}

#[test]
fn the_generated_node_id_is_the_type_prefixed_slugified_title() {
    let cases = [
        ("OAuth2 Login", "Feature#oauth2-login"),
        ("  Trim   Me  ", "Feature#trim-me"),
        ("Auth & OAuth2!!!", "Feature#auth-oauth2"),
        ("PR #1234", "Feature#pr-1234"),
        ("Multi\nLine\tTitle", "Feature#multi-line-title"),
    ];
    for (title, expected_id) in cases {
        let node = NodeBuilder::new(NodeType::Feature)
            .title(title)
            .meta(sample_meta())
            .build()
            .unwrap_or_else(|e| panic!("title {title:?} should build: {e:?}"));
        assert_eq!(node.id, expected_id, "title {title:?}");
        assert_eq!(node.title, title);
    }
}

#[test]
fn an_explicit_id_wins_over_the_generated_slug() {
    let node = NodeBuilder::new(NodeType::Bug)
        .id("Bug#memory-leak")
        .title("Something completely different")
        .meta(sample_meta())
        .build()
        .unwrap();
    assert_eq!(node.id, "Bug#memory-leak");
    assert_eq!(node.title, "Something completely different");
}

#[test]
fn every_optional_field_is_carried_onto_the_node() {
    let node = NodeBuilder::new(NodeType::Story)
        .title("User dashboard")
        .description("A story about the dashboard")
        .status(Status::InProgress)
        .tag("frontend")
        .tag("ui")
        .properties(serde_json::json!({ "estimate": 5, "priority": "high" }))
        .table_ref("stories")
        .table_id("ST-42")
        .meta(sample_meta())
        .build()
        .unwrap();

    assert_eq!(
        node.description.as_deref(),
        Some("A story about the dashboard")
    );
    assert_eq!(node.status, Status::InProgress);
    assert_eq!(node.tags, vec!["frontend".to_string(), "ui".to_string()]);
    assert_eq!(
        node.properties,
        Some(serde_json::json!({ "estimate": 5, "priority": "high" }))
    );
    assert_eq!(node.table_ref.as_deref(), Some("stories"));
    assert_eq!(node.table_id.as_deref(), Some("ST-42"));
    assert_eq!(node.meta.source, "integration-test");
    assert_eq!(node.meta.confidence, Some(0.75));
    assert_eq!(node.meta.agent_id.as_deref(), Some("agent-7"));
}

#[test]
fn tags_replaces_the_accumulated_tag_list_while_tag_appends() {
    let appended = NodeBuilder::new(NodeType::Task)
        .title("Tag behavior")
        .tag("a")
        .tag("b")
        .meta(sample_meta())
        .build()
        .unwrap();
    assert_eq!(appended.tags, vec!["a".to_string(), "b".to_string()]);

    // `tags` overwrites whatever `tag` accumulated before it.
    let replaced = NodeBuilder::new(NodeType::Task)
        .title("Tag behavior")
        .tag("a")
        .tag("b")
        .tags(vec!["only".to_string()])
        .meta(sample_meta())
        .build()
        .unwrap();
    assert_eq!(replaced.tags, vec!["only".to_string()]);

    // ...and `tag` after `tags` appends to the replacement.
    let mixed = NodeBuilder::new(NodeType::Task)
        .title("Tag behavior")
        .tags(vec!["first".to_string()])
        .tag("second")
        .meta(sample_meta())
        .build()
        .unwrap();
    assert_eq!(mixed.tags, vec!["first".to_string(), "second".to_string()]);
}

#[test]
fn a_built_node_passes_the_graph_validator() {
    let root = NodeBuilder::new(NodeType::Intent)
        .title("Auth epic")
        .meta(sample_meta())
        .build()
        .unwrap();
    let child = NodeBuilder::new(NodeType::Feature)
        .title("OAuth2 login")
        .meta(sample_meta())
        .build()
        .unwrap();
    let edge = EdgeBuilder::new(
        root.id.clone(),
        child.id.clone(),
        RelationshipType::Implements,
    )
    .meta(sample_meta())
    .build()
    .unwrap();

    let graph = agileplus_domain::intent_graph::IntentGraph {
        nodes: vec![root, child],
        edges: vec![edge],
        metadata: agileplus_domain::intent_graph::GraphMetadata {
            version: "1.0.0".to_string(),
            schema_uri: "schema".to_string(),
            created_at: Utc::now(),
            updated_at: None,
            node_count: None,
            edge_count: None,
            dag_valid: None,
            source_system: None,
        },
    };
    assert!(graph.validate().is_ok(), "{:?}", graph.validate());
}

// ---------------------------------------------------------------------------
// NodeBuilder rejection paths
// ---------------------------------------------------------------------------

#[test]
fn a_node_without_a_title_or_meta_is_rejected() {
    let missing_title = NodeBuilder::new(NodeType::Task)
        .meta(sample_meta())
        .build()
        .unwrap_err();
    assert_eq!(
        missing_title,
        ValidationError::MissingRequiredField("title".to_string())
    );

    let missing_meta = NodeBuilder::new(NodeType::Task)
        .title("Fix auth")
        .build()
        .unwrap_err();
    assert_eq!(
        missing_meta,
        ValidationError::MissingRequiredField("meta".to_string())
    );
}

#[test]
fn a_blank_meta_source_is_rejected_for_nodes() {
    let meta = agileplus_domain::intent_graph::Meta {
        source: "  \t ".to_string(),
        ..sample_meta()
    };
    let error = NodeBuilder::new(NodeType::Bug)
        .title("Has a source")
        .meta(meta)
        .build()
        .unwrap_err();
    assert_eq!(
        error,
        ValidationError::MissingMeta("node: source is empty".to_string())
    );
}

#[test]
fn explicit_node_ids_must_satisfy_the_grammar() {
    for off_grammar in [
        "bad-id",
        "feature#slug",
        "Feature",
        "Feature#",
        "Feature#Slug",
        "F#slug",
        "FeAture#slug",
        "PR#1234",
        "Feature#slug_1",
        "Feature#slug one",
    ] {
        let error = NodeBuilder::new(NodeType::Task)
            .id(off_grammar)
            .title("Fix auth")
            .meta(sample_meta())
            .build()
            .unwrap_err();
        assert!(
            matches!(error, ValidationError::InvalidNodeId(ref id) if id == off_grammar),
            "`{off_grammar}` should be rejected, got {error:?}"
        );
    }
}

#[test]
fn a_title_that_slugs_to_nothing_cannot_produce_a_node() {
    for title in ["!!!", "   ", "---", "??? ..."] {
        let error = NodeBuilder::new(NodeType::Intent)
            .title(title)
            .meta(sample_meta())
            .build()
            .unwrap_err();
        assert_eq!(
            error,
            ValidationError::InvalidNodeId("Intent#".to_string()),
            "title {title:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// EdgeBuilder
// ---------------------------------------------------------------------------

#[test]
fn edge_builder_generates_unique_prefixed_ids() {
    let build = || {
        EdgeBuilder::new("Intent#a", "Feature#b", RelationshipType::Implements)
            .meta(sample_meta())
            .build()
            .unwrap()
    };
    let first = build();
    let second = build();

    assert!(first.id.starts_with("edge-"), "{}", first.id);
    assert_eq!(
        first.id.matches('-').count(),
        2,
        "id shape is `edge-<nanos>-<counter>`: {}",
        first.id
    );
    assert_ne!(first.id, second.id, "ids must not collide");
}

#[test]
fn edge_builder_carries_every_field() {
    let edge = EdgeBuilder::new("Feature#a", "Task#b", RelationshipType::Implements)
        .id("edge-explicit")
        .canonical_map(CanonicalMap {
            link_type: CanonicalLinkType::ParentOf,
            direction: Some("forward".to_string()),
        })
        .properties(serde_json::json!({ "weight": 1.5 }))
        .meta(sample_meta())
        .build()
        .unwrap();

    assert_eq!(edge.id, "edge-explicit");
    assert_eq!(edge.source, "Feature#a");
    assert_eq!(edge.target, "Task#b");
    assert_eq!(edge.relationship_type, RelationshipType::Implements);
    assert_eq!(edge.properties, Some(serde_json::json!({ "weight": 1.5 })));
    let canonical = edge.canonical_map.expect("canonical map");
    assert_eq!(canonical.link_type, CanonicalLinkType::ParentOf);
    assert_eq!(canonical.direction.as_deref(), Some("forward"));
}

#[test]
fn edge_ids_and_endpoints_are_not_validated_by_the_builder() {
    // The builder only enforces the meta contract; endpoint types are checked
    // later by `IntentGraph::check_edge_constraints`. Pin that division so a
    // future change does not silently start rejecting bare edges.
    let edge = EdgeBuilder::new("not-a-node-id", "also-not-one", RelationshipType::TracesTo)
        .meta(sample_meta())
        .build()
        .unwrap();
    assert_eq!(edge.source, "not-a-node-id");
    assert_eq!(edge.target, "also-not-one");
}

#[test]
fn an_edge_without_meta_or_with_blank_source_is_rejected() {
    let missing_meta = EdgeBuilder::new("A#1", "B#2", RelationshipType::TracesTo)
        .build()
        .unwrap_err();
    assert_eq!(
        missing_meta,
        ValidationError::MissingRequiredField("meta".to_string())
    );

    let blank = agileplus_domain::intent_graph::Meta {
        source: String::new(),
        ..sample_meta()
    };
    let error = EdgeBuilder::new("A#1", "B#2", RelationshipType::TracesTo)
        .meta(blank)
        .build()
        .unwrap_err();
    assert_eq!(
        error,
        ValidationError::MissingMeta("edge: source is empty".to_string())
    );
}

#[test]
fn node_display_for_built_values_matches_the_wire_label() {
    // `NodeType`/`RelationshipType` Display is what ends up in dashboards; a
    // built node must render its type consistently with the serde label.
    let node: Node = NodeBuilder::new(NodeType::Artifact)
        .title("Report")
        .meta(sample_meta())
        .build()
        .unwrap();
    assert_eq!(node.node_type.to_string(), "Artifact");
    assert_eq!(
        serde_json::to_string(&node.node_type).unwrap(),
        "\"Artifact\""
    );
    assert_eq!(node.dag_stage.to_string(), "artifact");
}
