// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tests for the top-level Error enum and module re-exports.

use agileplus_graph::{Error, GraphError, GraphStore, Node, NodeType, RelType, Relationship};

// ── Error enum ──────────────────────────────────────────────────────────────

#[test]
fn error_from_graph_error() {
    let graph_err = GraphError::NotFound("node-42".into());
    let error: Error = graph_err.into();
    let msg = format!("{error}");
    assert!(msg.contains("Graph error"));
    assert!(msg.contains("Not found: node-42"));
}

#[test]
fn error_config_variant_display() {
    let error = Error::Config("missing key 'neo4j.url'".into());
    let msg = format!("{error}");
    assert_eq!(msg, "Config error: missing key 'neo4j.url'");
}

#[test]
fn error_debug_format() {
    let error = Error::Config("test".into());
    let dbg = format!("{:?}", error);
    assert!(dbg.contains("Config"));
    assert!(dbg.contains("test"));
}

#[test]
fn error_graph_variant_debug() {
    let error: Error = GraphError::ConnectionError("refused".into()).into();
    let dbg = format!("{:?}", error);
    assert!(dbg.contains("Graph"));
}

// ── Re-exports are accessible ───────────────────────────────────────────────

#[test]
fn re_exported_types_are_useable() {
    let _node = Node::new(NodeType::Feature, serde_json::json!({}));
    let _rel = Relationship::new(
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        RelType::DependsOn,
    );
}

#[tokio::test]
async fn re_exported_graph_store_trait_is_useable() {
    let store = agileplus_graph::InMemoryGraphStore::new();
    assert!(store.health_check().await.is_ok());
}

#[test]
fn re_exported_graph_error_is_useable() {
    let err = GraphError::QueryError("bad".into());
    assert!(format!("{err}").contains("Query error"));
}

// ── NodeType and RelType are re-exported ────────────────────────────────────

#[test]
fn re_exported_node_type_variants() {
    assert_eq!(agileplus_graph::NodeType::Feature.as_str(), "Feature");
}

#[test]
fn re_exported_rel_type_variants() {
    assert_eq!(agileplus_graph::RelType::Blocks.as_str(), "BLOCKS");
}

// ── Deepened: error source chaining and trait bounds ────────────────────────

#[test]
fn error_from_graph_error_exposes_source() {
    use std::error::Error as _;
    let error: Error = GraphError::QueryError("cypher failed".into()).into();
    let source = error.source().expect("Graph variant should carry a source");
    assert!(source.to_string().contains("cypher failed"));
}

#[test]
fn config_error_has_no_source() {
    use std::error::Error as _;
    let error = Error::Config("bad config".into());
    assert!(error.source().is_none());
}

#[test]
fn error_is_send_sync_and_static() {
    fn assert_bounds<T: Send + Sync + 'static>() {}
    assert_bounds::<Error>();
}

#[test]
fn error_from_each_graph_variant_preserves_message() {
    for (err, needle) in [
        (
            GraphError::ConnectionError("c".into()),
            "Connection error: c",
        ),
        (GraphError::QueryError("q".into()), "Query error: q"),
        (
            GraphError::ConstraintViolation("v".into()),
            "Constraint violation: v",
        ),
        (GraphError::NotFound("n".into()), "Not found: n"),
        (GraphError::InvalidInput("i".into()), "Invalid input: i"),
    ] {
        let wrapped: Error = err.into();
        assert!(wrapped.to_string().contains(needle));
    }
}
