//! Wire-format vocabulary tests for the intent-graph ontology enums.
//!
//! Every ontology enum publishes the *same* token set through three separate
//! code paths:
//!
//! * `Display` — log lines, Neo4j labels, `InvalidEdgeConstraint` messages,
//! * `From<&str>` / `TryFrom<String>` — SQL rows, config files, RPC payloads,
//! * `serde` — JSON payloads exchanged with Tracera / AgilePlus.
//!
//! The token tables below are written out by hand from the ontology schema, so
//! these tests fail if any single encoding drifts away from the schema (for
//! example a renamed `serde(rename_all)` or one stale match arm), even when the
//! other two encodings keep working.

use traceability_core::{
    CanonicalLinkType, DagStage, NodeStatus, NodeType, RelationshipType, ValidationError,
};

fn node_types() -> Vec<(NodeType, &'static str)> {
    vec![
        (NodeType::Intent, "Intent"),
        (NodeType::Plan, "Plan"),
        (NodeType::Feature, "Feature"),
        (NodeType::Story, "Story"),
        (NodeType::Task, "Task"),
        (NodeType::Spec, "Spec"),
        (NodeType::Commit, "Commit"),
        (NodeType::Test, "Test"),
        (NodeType::PR, "PR"),
        (NodeType::Bug, "Bug"),
        (NodeType::Artifact, "Artifact"),
    ]
}

fn dag_stages() -> Vec<(DagStage, &'static str)> {
    vec![
        (DagStage::Intent, "intent"),
        (DagStage::Plan, "plan"),
        (DagStage::Feature, "feature"),
        (DagStage::Story, "story"),
        (DagStage::Task, "task"),
        (DagStage::Spec, "spec"),
        (DagStage::Commit, "commit"),
        (DagStage::Test, "test"),
        (DagStage::PR, "pr"),
        (DagStage::Bug, "bug"),
        (DagStage::Artifact, "artifact"),
    ]
}

fn relationship_types() -> Vec<(RelationshipType, &'static str)> {
    vec![
        (RelationshipType::Implements, "implements"),
        (RelationshipType::Tests, "tests"),
        (RelationshipType::Covers, "covers"),
        (RelationshipType::TracesTo, "traces-to"),
        (RelationshipType::DerivesFrom, "derives-from"),
        (RelationshipType::Resolves, "resolves"),
        (RelationshipType::Blocks, "blocks"),
        (RelationshipType::DependsOn, "depends-on"),
    ]
}

fn canonical_link_types() -> Vec<(CanonicalLinkType, &'static str)> {
    vec![
        (CanonicalLinkType::ParentOf, "parent_of"),
        (CanonicalLinkType::ChildOf, "child_of"),
        (CanonicalLinkType::DependsOn, "depends_on"),
        (CanonicalLinkType::Blocks, "blocks"),
        (CanonicalLinkType::Implements, "implements"),
        (CanonicalLinkType::Verifies, "verifies"),
        (CanonicalLinkType::References, "references"),
        (CanonicalLinkType::Duplicates, "duplicates"),
    ]
}

fn node_statuses() -> Vec<(NodeStatus, &'static str)> {
    vec![
        (NodeStatus::Draft, "draft"),
        (NodeStatus::Active, "active"),
        (NodeStatus::Completed, "completed"),
        (NodeStatus::Deprecated, "deprecated"),
        (NodeStatus::Rejected, "rejected"),
        (NodeStatus::Open, "open"),
        (NodeStatus::InProgress, "in_progress"),
        (NodeStatus::Blocked, "blocked"),
        (NodeStatus::Deferred, "deferred"),
        (NodeStatus::Cancelled, "cancelled"),
    ]
}

/// Assert `Display`, `serde` and `TryFrom` all reproduce the schema token.
fn assert_three_encodings_agree<T>(
    enum_name: &str,
    pairs: &[(T, &'static str)],
    parse: impl Fn(&str) -> Result<T, ValidationError>,
) where
    T: std::fmt::Display + std::fmt::Debug + PartialEq + serde::Serialize,
{
    for (variant, token) in pairs {
        assert_eq!(
            variant.to_string(),
            *token,
            "{enum_name}::{variant:?} Display token"
        );

        let json = serde_json::to_string(variant).expect("serializable");
        assert_eq!(
            json,
            format!("\"{token}\""),
            "{enum_name}::{variant:?} JSON token"
        );

        let parsed = parse(token).unwrap_or_else(|e| {
            panic!("{enum_name}: TryFrom<String> rejected schema token {token:?}: {e}")
        });
        assert_eq!(
            parsed, *variant,
            "{enum_name}: parse round-trip for {token:?}"
        );
    }
}

#[test]
fn node_type_encodings_agree_for_every_variant() {
    assert_three_encodings_agree("NodeType", &node_types(), |s| {
        NodeType::try_from(s.to_string())
    });
}

#[test]
fn dag_stage_encodings_agree_for_every_variant() {
    assert_three_encodings_agree("DagStage", &dag_stages(), |s| {
        DagStage::try_from(s.to_string())
    });
}

#[test]
fn relationship_type_encodings_agree_for_every_variant() {
    assert_three_encodings_agree("RelationshipType", &relationship_types(), |s| {
        RelationshipType::try_from(s.to_string())
    });
}

#[test]
fn canonical_link_type_encodings_agree_for_every_variant() {
    assert_three_encodings_agree("CanonicalLinkType", &canonical_link_types(), |s| {
        CanonicalLinkType::try_from(s.to_string())
    });
}

#[test]
fn node_status_encodings_agree_for_every_variant() {
    assert_three_encodings_agree("Status", &node_statuses(), |s| {
        NodeStatus::try_from(s.to_string())
    });
}

/// `From<&str>` is the infallible parser used on already-validated config /
/// SQL rows. It must accept exactly the same token set as `TryFrom`, otherwise
/// callers that pick `From` silently map a valid token onto the wrong variant.
#[test]
fn infallible_from_str_accepts_every_schema_token() {
    for (variant, token) in node_types() {
        assert_eq!(NodeType::from(token), variant, "NodeType::from({token:?})");
    }
    for (variant, token) in dag_stages() {
        assert_eq!(DagStage::from(token), variant, "DagStage::from({token:?})");
    }
    for (variant, token) in relationship_types() {
        assert_eq!(
            RelationshipType::from(token),
            variant,
            "RelationshipType::from({token:?})"
        );
    }
    for (variant, token) in canonical_link_types() {
        assert_eq!(
            CanonicalLinkType::from(token),
            variant,
            "CanonicalLinkType::from({token:?})"
        );
    }
    for (variant, token) in node_statuses() {
        assert_eq!(NodeStatus::from(token), variant, "Status::from({token:?})");
    }
}
