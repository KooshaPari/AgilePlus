// SPDX-License-Identifier: MIT OR Apache-2.0
//! Comprehensive tests for InMemoryGraphStore (the GraphStore trait implementation).

use agileplus_graph::{
    GraphError, GraphStore, InMemoryGraphStore, Node, NodeType, RelType, Relationship,
};
use serde_json::json;

// ── Construction ────────────────────────────────────────────────────────────

#[tokio::test]
async fn new_store_is_empty() {
    let store = InMemoryGraphStore::new();
    assert!(store.get_nodes_by_type(NodeType::Feature).is_empty());
    assert!(store.get_nodes_by_type(NodeType::WorkPackage).is_empty());
    assert!(store.get_nodes_by_type(NodeType::Agent).is_empty());
}

#[tokio::test]
async fn default_store_is_empty() {
    let store = InMemoryGraphStore::default();
    assert!(store.get_nodes_by_type(NodeType::Feature).is_empty());
}

// ── get_node / get_nodes_by_type ────────────────────────────────────────────

#[tokio::test]
async fn get_node_returns_none_for_missing() {
    let store = InMemoryGraphStore::new();
    assert!(store.get_node(uuid::Uuid::new_v4()).is_none());
}

#[tokio::test]
async fn get_node_returns_inserted_node() {
    let store = InMemoryGraphStore::new();
    let id = uuid::Uuid::new_v4();
    let node = Node::with_id(id, NodeType::Feature, json!({"slug": "auth"}));
    store.upsert_node(&node).await.unwrap();

    let fetched = store.get_node(id).unwrap();
    assert_eq!(fetched.id, id);
    assert_eq!(fetched.node_type, NodeType::Feature);
    assert_eq!(fetched.properties["slug"], "auth");
}

#[tokio::test]
async fn get_nodes_by_type_filters_correctly() {
    let store = InMemoryGraphStore::new();
    let f1 = Node::new(NodeType::Feature, json!({"name": "a"}));
    let f2 = Node::new(NodeType::Feature, json!({"name": "b"}));
    let wp = Node::new(NodeType::WorkPackage, json!({"name": "c"}));
    let ag = Node::new(NodeType::Agent, json!({"name": "d"}));

    store.upsert_node(&f1).await.unwrap();
    store.upsert_node(&f2).await.unwrap();
    store.upsert_node(&wp).await.unwrap();
    store.upsert_node(&ag).await.unwrap();

    let features = store.get_nodes_by_type(NodeType::Feature);
    assert_eq!(features.len(), 2);

    let work_packages = store.get_nodes_by_type(NodeType::WorkPackage);
    assert_eq!(work_packages.len(), 1);

    let agents = store.get_nodes_by_type(NodeType::Agent);
    assert_eq!(agents.len(), 1);

    let projects = store.get_nodes_by_type(NodeType::Project);
    assert!(projects.is_empty());
}

// ── upsert_node (update semantics) ─────────────────────────────────────────

#[tokio::test]
async fn upsert_node_updates_existing() {
    let store = InMemoryGraphStore::new();
    let id = uuid::Uuid::new_v4();
    let node_v1 = Node::with_id(id, NodeType::Feature, json!({"version": 1}));
    store.upsert_node(&node_v1).await.unwrap();

    let node_v2 = Node::with_id(id, NodeType::Feature, json!({"version": 2}));
    store.upsert_node(&node_v2).await.unwrap();

    let fetched = store.get_node(id).unwrap();
    assert_eq!(fetched.properties["version"], 2);
}

#[tokio::test]
async fn upsert_node_preserves_id() {
    let store = InMemoryGraphStore::new();
    let id = uuid::Uuid::new_v4();
    let node = Node::with_id(id, NodeType::WorkPackage, json!({"title": "task-1"}));
    store.upsert_node(&node).await.unwrap();

    let fetched = store.get_node(id).unwrap();
    assert_eq!(fetched.id, id);
    assert_eq!(fetched.properties["title"], "task-1");
}

// ── get_relationships_from / get_relationships_to ───────────────────────────

#[tokio::test]
async fn get_relationships_from_returns_outgoing() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    let c = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store.upsert_node(&c).await.unwrap();

    let r1 = Relationship::new(a.id, b.id, RelType::DependsOn);
    let r2 = Relationship::new(a.id, c.id, RelType::Blocks);
    store.create_relationship(&r1).await.unwrap();
    store.create_relationship(&r2).await.unwrap();

    let from_a = store.get_relationships_from(a.id);
    assert_eq!(from_a.len(), 2);

    let from_b = store.get_relationships_from(b.id);
    assert!(from_b.is_empty());
}

#[tokio::test]
async fn get_relationships_to_returns_incoming() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();

    let r1 = Relationship::new(a.id, b.id, RelType::DependsOn);
    store.create_relationship(&r1).await.unwrap();

    let to_b = store.get_relationships_to(b.id);
    assert_eq!(to_b.len(), 1);
    assert_eq!(to_b[0].from_node_id, a.id);

    let to_a = store.get_relationships_to(a.id);
    assert!(to_a.is_empty());
}

// ── create_relationship / delete_relationship ──────────────────────────────

#[tokio::test]
async fn delete_nonexistent_relationship_is_ok() {
    let store = InMemoryGraphStore::new();
    store
        .delete_relationship(uuid::Uuid::new_v4())
        .await
        .unwrap();
}

#[tokio::test]
async fn create_multiple_relationships_distinct_ids() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();

    let r1 = Relationship::new(a.id, b.id, RelType::DependsOn);
    let r2 = Relationship::new(a.id, b.id, RelType::Tagged);
    let r3 = Relationship::new(a.id, b.id, RelType::Blocks);
    store.create_relationship(&r1).await.unwrap();
    store.create_relationship(&r2).await.unwrap();
    store.create_relationship(&r3).await.unwrap();

    let from_a = store.get_relationships_from(a.id);
    assert_eq!(from_a.len(), 3);
}

#[tokio::test]
async fn delete_one_of_many_relationships() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    let c = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store.upsert_node(&c).await.unwrap();

    let r1 = Relationship::new(a.id, b.id, RelType::DependsOn);
    let r2 = Relationship::new(a.id, c.id, RelType::Blocks);
    store.create_relationship(&r1).await.unwrap();
    store.create_relationship(&r2).await.unwrap();

    store.delete_relationship(r1.id).await.unwrap();

    let from_a = store.get_relationships_from(a.id);
    assert_eq!(from_a.len(), 1);
    assert_eq!(from_a[0].to_node_id, c.id);
}

// ── get_dependencies ────────────────────────────────────────────────────────

#[tokio::test]
async fn get_dependencies_empty_when_none() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    let deps = store.get_dependencies(a.id).await.unwrap();
    assert!(deps.is_empty());
}

#[tokio::test]
async fn get_dependencies_filters_by_depends_on_only() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    let c = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store.upsert_node(&c).await.unwrap();

    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::DependsOn))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(a.id, c.id, RelType::Blocks))
        .await
        .unwrap();

    let deps = store.get_dependencies(a.id).await.unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0], b.id);
}

#[tokio::test]
async fn get_dependencies_multiple_depends_on() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::WorkPackage, json!({}));
    let c = Node::new(NodeType::WorkPackage, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store.upsert_node(&c).await.unwrap();

    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::DependsOn))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(a.id, c.id, RelType::DependsOn))
        .await
        .unwrap();

    let deps = store.get_dependencies(a.id).await.unwrap();
    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&b.id));
    assert!(deps.contains(&c.id));
}

#[tokio::test]
async fn get_dependencies_only_outgoing() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();

    store
        .create_relationship(&Relationship::new(b.id, a.id, RelType::DependsOn))
        .await
        .unwrap();

    let deps_a = store.get_dependencies(a.id).await.unwrap();
    assert!(deps_a.is_empty());

    let deps_b = store.get_dependencies(b.id).await.unwrap();
    assert_eq!(deps_b, vec![a.id]);
}

// ── get_blocking_path ──────────────────────────────────────────────────────

#[tokio::test]
async fn get_blocking_path_empty_when_none() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::WorkPackage, json!({}));
    store.upsert_node(&a).await.unwrap();
    let blockers = store.get_blocking_path(a.id).await.unwrap();
    assert!(blockers.is_empty());
}

#[tokio::test]
async fn get_blocking_path_filters_by_blocks_only() {
    let store = InMemoryGraphStore::new();
    let blocker = Node::new(NodeType::WorkPackage, json!({}));
    let target = Node::new(NodeType::WorkPackage, json!({}));
    let other = Node::new(NodeType::WorkPackage, json!({}));
    store.upsert_node(&blocker).await.unwrap();
    store.upsert_node(&target).await.unwrap();
    store.upsert_node(&other).await.unwrap();

    store
        .create_relationship(&Relationship::new(blocker.id, target.id, RelType::Blocks))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(other.id, target.id, RelType::DependsOn))
        .await
        .unwrap();

    let blockers = store.get_blocking_path(target.id).await.unwrap();
    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0], blocker.id);
}

#[tokio::test]
async fn get_blocking_path_multiple_blockers() {
    let store = InMemoryGraphStore::new();
    let b1 = Node::new(NodeType::WorkPackage, json!({}));
    let b2 = Node::new(NodeType::WorkPackage, json!({}));
    let target = Node::new(NodeType::WorkPackage, json!({}));
    store.upsert_node(&b1).await.unwrap();
    store.upsert_node(&b2).await.unwrap();
    store.upsert_node(&target).await.unwrap();

    store
        .create_relationship(&Relationship::new(b1.id, target.id, RelType::Blocks))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(b2.id, target.id, RelType::Blocks))
        .await
        .unwrap();

    let blockers = store.get_blocking_path(target.id).await.unwrap();
    assert_eq!(blockers.len(), 2);
    assert!(blockers.contains(&b1.id));
    assert!(blockers.contains(&b2.id));
}

#[tokio::test]
async fn get_blocking_path_not_outgoing() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::WorkPackage, json!({}));
    let b = Node::new(NodeType::WorkPackage, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();

    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::Blocks))
        .await
        .unwrap();

    let blockers_a = store.get_blocking_path(a.id).await.unwrap();
    assert!(blockers_a.is_empty());

    let blockers_b = store.get_blocking_path(b.id).await.unwrap();
    assert_eq!(blockers_b, vec![a.id]);
}

// ── health_check ───────────────────────────────────────────────────────────

#[tokio::test]
async fn health_check_passes_on_empty_store() {
    let store = InMemoryGraphStore::new();
    assert!(store.health_check().await.is_ok());
}

#[tokio::test]
async fn health_check_passes_after_inserts() {
    let store = InMemoryGraphStore::new();
    let node = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&node).await.unwrap();
    let rel = Relationship::new(node.id, node.id, RelType::Owns);
    store.create_relationship(&rel).await.unwrap();
    assert!(store.health_check().await.is_ok());
}

// ── GraphError display ─────────────────────────────────────────────────────

#[test]
fn graph_error_display_connection() {
    let e = GraphError::ConnectionError("refused".into());
    assert_eq!(format!("{e}"), "Connection error: refused");
}

#[test]
fn graph_error_display_query() {
    let e = GraphError::QueryError("bad syntax".into());
    assert_eq!(format!("{e}"), "Query error: bad syntax");
}

#[test]
fn graph_error_display_constraint() {
    let e = GraphError::ConstraintViolation("unique".into());
    assert_eq!(format!("{e}"), "Constraint violation: unique");
}

#[test]
fn graph_error_display_not_found() {
    let e = GraphError::NotFound("node-123".into());
    assert_eq!(format!("{e}"), "Not found: node-123");
}

#[test]
fn graph_error_display_invalid_input() {
    let e = GraphError::InvalidInput("empty name".into());
    assert_eq!(format!("{e}"), "Invalid input: empty name");
}

// ── Integration: complex graph operations ───────────────────────────────────

#[tokio::test]
async fn full_workflow_upsert_create_query_delete() {
    let store = InMemoryGraphStore::new();

    let feat = Node::new(NodeType::Feature, json!({"name": "Login"}));
    let wp = Node::new(NodeType::WorkPackage, json!({"title": "Auth backend"}));
    let agent = Node::new(NodeType::Agent, json!({"model": "gpt-4"}));
    let label = Node::new(NodeType::Label, json!({"color": "blue"}));

    store.upsert_node(&feat).await.unwrap();
    store.upsert_node(&wp).await.unwrap();
    store.upsert_node(&agent).await.unwrap();
    store.upsert_node(&label).await.unwrap();

    store
        .create_relationship(&Relationship::new(feat.id, wp.id, RelType::DependsOn))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(agent.id, feat.id, RelType::Owns))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(feat.id, label.id, RelType::Tagged))
        .await
        .unwrap();

    let deps = store.get_dependencies(feat.id).await.unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0], wp.id);

    let from_feat = store.get_relationships_from(feat.id);
    assert_eq!(from_feat.len(), 2);

    let to_feat = store.get_relationships_to(feat.id);
    assert_eq!(to_feat.len(), 1);

    let owns_rel = to_feat.into_iter().next().unwrap();
    store.delete_relationship(owns_rel.id).await.unwrap();

    let to_feat_after = store.get_relationships_to(feat.id);
    assert!(to_feat_after.is_empty());

    assert_eq!(store.get_nodes_by_type(NodeType::Feature).len(), 1);
    assert_eq!(store.get_nodes_by_type(NodeType::WorkPackage).len(), 1);
    assert_eq!(store.get_nodes_by_type(NodeType::Agent).len(), 1);
    assert_eq!(store.get_nodes_by_type(NodeType::Label).len(), 1);
}

#[tokio::test]
async fn transitive_blocking_chain() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::WorkPackage, json!({"n": "a"}));
    let b = Node::new(NodeType::WorkPackage, json!({"n": "b"}));
    let c = Node::new(NodeType::WorkPackage, json!({"n": "c"}));
    let d = Node::new(NodeType::WorkPackage, json!({"n": "d"}));

    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store.upsert_node(&c).await.unwrap();
    store.upsert_node(&d).await.unwrap();

    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::Blocks))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(b.id, c.id, RelType::Blocks))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(c.id, d.id, RelType::Blocks))
        .await
        .unwrap();

    let blockers_d = store.get_blocking_path(d.id).await.unwrap();
    assert_eq!(blockers_d, vec![c.id]);

    let blockers_c = store.get_blocking_path(c.id).await.unwrap();
    assert_eq!(blockers_c, vec![b.id]);

    let blockers_a = store.get_blocking_path(a.id).await.unwrap();
    assert!(blockers_a.is_empty());
}

#[tokio::test]
async fn re_upsert_node_updates_properties() {
    let store = InMemoryGraphStore::new();
    let id = uuid::Uuid::new_v4();
    let n1 = Node::with_id(id, NodeType::Feature, json!({"title": "old"}));
    store.upsert_node(&n1).await.unwrap();
    assert_eq!(store.get_node(id).unwrap().properties["title"], "old");

    let n2 = Node::with_id(id, NodeType::Feature, json!({"title": "new"}));
    store.upsert_node(&n2).await.unwrap();
    assert_eq!(store.get_node(id).unwrap().properties["title"], "new");
}

#[tokio::test]
async fn nodes_and_relationships_are_independent() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();

    let r = Relationship::new(a.id, b.id, RelType::DependsOn);
    store.create_relationship(&r).await.unwrap();

    let from_a = store.get_relationships_from(a.id);
    assert_eq!(from_a.len(), 1);
}

// ── Additional edge-case tests (expanded coverage) ────────────────────────────

/// Test: GraphError variants all display correctly.
#[test]
fn graph_error_all_variants_display() {
    use agileplus_graph::GraphError;
    let errors = vec![
        GraphError::ConnectionError("conn".into()),
        GraphError::QueryError("query".into()),
        GraphError::ConstraintViolation("constraint".into()),
        GraphError::NotFound("node".into()),
        GraphError::InvalidInput("input".into()),
    ];
    for err in errors {
        let msg = format!("{err}");
        assert!(!msg.is_empty());
    }
}

/// Test: InMemoryGraphStore::default creates empty store (all types).
#[tokio::test]
async fn default_store_is_empty_all_types() {
    let store = InMemoryGraphStore::default();
    assert!(store.get_nodes_by_type(NodeType::Feature).is_empty());
    assert!(store.get_nodes_by_type(NodeType::WorkPackage).is_empty());
    assert!(store.get_nodes_by_type(NodeType::Agent).is_empty());
    assert!(store.get_nodes_by_type(NodeType::Label).is_empty());
    assert!(store.get_nodes_by_type(NodeType::Project).is_empty());
}

/// Test: get_node returns None for non-existent ID.
#[tokio::test]
async fn get_node_returns_none_for_missing_id() {
    let store = InMemoryGraphStore::new();
    let id = uuid::Uuid::new_v4();
    assert!(store.get_node(id).is_none());
}

/// Test: create_relationship allows self-referencing relationship.
#[tokio::test]
async fn create_relationship_allows_self_reference() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    let rel = Relationship::new(a.id, a.id, RelType::Owns);
    store.create_relationship(&rel).await.unwrap();
    let outgoing = store.get_relationships_from(a.id);
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].to_node_id, a.id);
}

/// Test: delete_relationship removes existing relationship.
#[tokio::test]
async fn delete_relationship_removes_existing() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    let rel = Relationship::new(a.id, b.id, RelType::DependsOn);
    store.create_relationship(&rel).await.unwrap();
    assert_eq!(store.get_relationships_from(a.id).len(), 1);
    store.delete_relationship(rel.id).await.unwrap();
    assert!(store.get_relationships_from(a.id).is_empty());
    assert!(store.get_relationships_to(b.id).is_empty());
}

/// Test: delete_relationship on non-existent ID is no-op (no error).
#[tokio::test]
async fn delete_relationship_nonexistent_is_noop() {
    let store = InMemoryGraphStore::new();
    let result = store.delete_relationship(uuid::Uuid::new_v4()).await;
    assert!(result.is_ok());
}

/// Test: get_dependencies returns empty for node with no outgoing edges.
#[tokio::test]
async fn get_dependencies_empty_for_isolated_node() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::WorkPackage, json!({}));
    store.upsert_node(&a).await.unwrap();
    let deps = store.get_dependencies(a.id).await.unwrap();
    assert!(deps.is_empty());
}

/// Test: get_dependencies follows DependsOn relationships.
#[tokio::test]
async fn get_dependencies_follows_depends_on() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::WorkPackage, json!({}));
    let b = Node::new(NodeType::WorkPackage, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store
        .create_relationship(&Relationship::new(b.id, a.id, RelType::DependsOn))
        .await
        .unwrap();
    let deps = store.get_dependencies(b.id).await.unwrap();
    assert_eq!(deps, vec![a.id]);
}

/// Test: get_dependencies ignores non-DependsOn relationships.
#[tokio::test]
async fn get_dependencies_ignores_other_relations() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::WorkPackage, json!({}));
    let b = Node::new(NodeType::WorkPackage, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store
        .create_relationship(&Relationship::new(b.id, a.id, RelType::Owns))
        .await
        .unwrap();
    let deps = store.get_dependencies(b.id).await.unwrap();
    assert!(deps.is_empty());
}

/// Test: get_blocking_path returns empty when no blockers.
#[tokio::test]
async fn get_blocking_path_empty_when_no_blockers() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::WorkPackage, json!({}));
    store.upsert_node(&a).await.unwrap();
    let blockers = store.get_blocking_path(a.id).await.unwrap();
    assert!(blockers.is_empty());
}

/// Test: get_blocking_path finds direct Blocks relationship.
#[tokio::test]
async fn get_blocking_path_finds_direct_blocker() {
    let store = InMemoryGraphStore::new();
    let blocker = Node::new(NodeType::WorkPackage, json!({}));
    let target = Node::new(NodeType::WorkPackage, json!({}));
    store.upsert_node(&blocker).await.unwrap();
    store.upsert_node(&target).await.unwrap();
    store
        .create_relationship(&Relationship::new(blocker.id, target.id, RelType::Blocks))
        .await
        .unwrap();
    let blockers = store.get_blocking_path(target.id).await.unwrap();
    assert_eq!(blockers, vec![blocker.id]);
}

/// Test: get_blocking_path returns direct blockers only (no traversal).
#[tokio::test]
async fn get_blocking_path_returns_direct_blockers() {
    let store = InMemoryGraphStore::new();
    let b1 = Node::new(NodeType::WorkPackage, json!({}));
    let b2 = Node::new(NodeType::WorkPackage, json!({}));
    let target = Node::new(NodeType::WorkPackage, json!({}));
    store.upsert_node(&b1).await.unwrap();
    store.upsert_node(&b2).await.unwrap();
    store.upsert_node(&target).await.unwrap();
    store
        .create_relationship(&Relationship::new(b1.id, b2.id, RelType::Blocks))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(b2.id, target.id, RelType::Blocks))
        .await
        .unwrap();
    let blockers = store.get_blocking_path(target.id).await.unwrap();
    // Direct blocker of target is b2 only.
    assert_eq!(blockers, vec![b2.id]);
}

/// Test: get_blocking_path ignores non-Blocks relationships.
#[tokio::test]
async fn get_blocking_path_ignores_other_relations() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::WorkPackage, json!({}));
    let b = Node::new(NodeType::WorkPackage, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::DependsOn))
        .await
        .unwrap();
    let blockers = store.get_blocking_path(b.id).await.unwrap();
    assert!(blockers.is_empty());
}

/// Test: get_blocking_path on non-existent node returns empty.
#[tokio::test]
async fn get_blocking_path_nonexistent_returns_empty() {
    let store = InMemoryGraphStore::new();
    let blockers = store.get_blocking_path(uuid::Uuid::new_v4()).await.unwrap();
    assert!(blockers.is_empty());
}

/// Test: health_check always succeeds for InMemoryGraphStore.
#[tokio::test]
async fn health_check_succeeds_for_in_memory() {
    let store = InMemoryGraphStore::new();
    let result = store.health_check().await;
    assert!(result.is_ok());
}

/// Test: get_relationships_from returns outgoing edges (mixed types).
#[tokio::test]
async fn get_relationships_from_returns_outgoing_mixed() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    let c = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store.upsert_node(&c).await.unwrap();
    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::Owns))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(a.id, c.id, RelType::AssignedTo))
        .await
        .unwrap();
    let outgoing = store.get_relationships_from(a.id);
    assert_eq!(outgoing.len(), 2);
}

/// Test: get_relationships_to returns incoming edges (mixed sources).
#[tokio::test]
async fn get_relationships_to_returns_incoming_mixed() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    let c = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store.upsert_node(&c).await.unwrap();
    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::Owns))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(c.id, b.id, RelType::AssignedTo))
        .await
        .unwrap();
    let incoming = store.get_relationships_to(b.id);
    assert_eq!(incoming.len(), 2);
}

/// Test: upsert_node preserves node ID when updating with new properties.
#[tokio::test]
async fn upsert_node_preserves_id_with_new_props() {
    let store = InMemoryGraphStore::new();
    let id = uuid::Uuid::new_v4();
    let node = Node::with_id(id, NodeType::WorkPackage, json!({"title": "task"}));
    store.upsert_node(&node).await.unwrap();
    let fetched = store.get_node(id).unwrap();
    assert_eq!(fetched.id, id);
    assert_eq!(fetched.properties["title"], "task");
}

/// Test: NodeType serde roundtrip for all variants.
#[test]
fn node_type_serde_roundtrip_all_variants() {
    use agileplus_graph::NodeType;
    use serde_json;
    let variants = vec![
        NodeType::Feature,
        NodeType::WorkPackage,
        NodeType::Agent,
        NodeType::Label,
        NodeType::Project,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let restored: NodeType = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, v);
    }
}

/// Test: RelType serde roundtrip for all variants.
#[test]
fn rel_type_serde_roundtrip_all_variants() {
    use agileplus_graph::RelType;
    use serde_json;
    let variants = vec![
        RelType::Owns,
        RelType::AssignedTo,
        RelType::DependsOn,
        RelType::Blocks,
        RelType::Tagged,
        RelType::InProject,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let restored: RelType = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, v);
    }
}

/// Test: Node with_id preserves custom UUID.
#[test]
fn node_with_id_preserves_uuid() {
    use agileplus_graph::Node;
    use serde_json::json;
    let id = uuid::Uuid::new_v4();
    let node = Node::with_id(id, NodeType::Project, json!({"name": "Proj"}));
    assert_eq!(node.id, id);
}

/// Test: Relationship with_id preserves custom UUID.
#[test]
fn relationship_with_id_preserves_uuid() {
    use agileplus_graph::{RelType, Relationship};
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    let id = uuid::Uuid::new_v4();
    let rel = Relationship::with_id(id, from, to, RelType::Owns);
    assert_eq!(rel.id, id);
    assert_eq!(rel.from_node_id, from);
    assert_eq!(rel.to_node_id, to);
}

/// Test: Relationship new generates unique ID.
#[test]
fn relationship_new_generates_unique_id() {
    use agileplus_graph::{RelType, Relationship};
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    let r1 = Relationship::new(from, to, RelType::Owns);
    let r2 = Relationship::new(from, to, RelType::Owns);
    assert_ne!(r1.id, r2.id);
}

/// Test: Node clone produces independent copy.
#[test]
fn node_clone_independent() {
    use agileplus_graph::Node;
    use serde_json::json;
    let n1 = Node::new(NodeType::Feature, json!({"val": 1}));
    let n2 = n1.clone();
    assert_eq!(n1.id, n2.id);
    assert_eq!(n1.node_type, n2.node_type);
    assert_eq!(n1.properties, n2.properties);
}

/// Test: Relationship clone produces independent copy.
#[test]
fn relationship_clone_independent() {
    use agileplus_graph::{RelType, Relationship};
    let r1 = Relationship::new(uuid::Uuid::new_v4(), uuid::Uuid::new_v4(), RelType::Owns);
    let r2 = r1.clone();
    assert_eq!(r1.id, r2.id);
    assert_eq!(r1.from_node_id, r2.from_node_id);
    assert_eq!(r1.to_node_id, r2.to_node_id);
}

/// Test: Node can have empty properties object.
#[test]
fn node_empty_properties_allowed() {
    use agileplus_graph::Node;
    use serde_json::json;
    let node = Node::new(NodeType::Label, json!({}));
    assert!(node.properties.is_object());
}

/// Test: Relationship can have empty properties.
#[test]
fn relationship_empty_properties_allowed() {
    use agileplus_graph::{RelType, Relationship};
    let rel = Relationship::new(uuid::Uuid::new_v4(), uuid::Uuid::new_v4(), RelType::Tagged);
    assert!(rel.properties.is_object());
}

/// Test: InMemoryGraphStore handles many nodes efficiently.
#[tokio::test]
async fn store_handles_many_nodes() {
    let store = InMemoryGraphStore::new();
    let mut ids = Vec::new();
    for i in 0..100 {
        let node = Node::new(NodeType::Feature, json!({"idx": i}));
        ids.push(node.id);
        store.upsert_node(&node).await.unwrap();
    }
    assert_eq!(store.get_nodes_by_type(NodeType::Feature).len(), 100);
}

/// Test: InMemoryGraphStore handles many relationships efficiently.
#[tokio::test]
async fn store_handles_many_relationships() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    for i in 0..50 {
        let b = Node::new(NodeType::Feature, json!({"idx": i}));
        store.upsert_node(&b).await.unwrap();
        store
            .create_relationship(&Relationship::new(a.id, b.id, RelType::Owns))
            .await
            .unwrap();
    }
    assert_eq!(store.get_relationships_from(a.id).len(), 50);
}

/// Test: Multiple relationship types between same nodes allowed.
#[tokio::test]
async fn multiple_relation_types_between_same_nodes() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::Owns))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::DependsOn))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::Tagged))
        .await
        .unwrap();
    let rels = store.get_relationships_from(a.id);
    assert_eq!(rels.len(), 3);
}

/// Test: get_nodes_by_type filters correctly across types.
#[tokio::test]
async fn get_nodes_by_type_filters_by_type() {
    let store = InMemoryGraphStore::new();
    let f = Node::new(NodeType::Feature, json!({}));
    let w = Node::new(NodeType::WorkPackage, json!({}));
    let ag = Node::new(NodeType::Agent, json!({}));
    let l = Node::new(NodeType::Label, json!({}));
    let p = Node::new(NodeType::Project, json!({}));
    store.upsert_node(&f).await.unwrap();
    store.upsert_node(&w).await.unwrap();
    store.upsert_node(&ag).await.unwrap();
    store.upsert_node(&l).await.unwrap();
    store.upsert_node(&p).await.unwrap();
    assert_eq!(store.get_nodes_by_type(NodeType::Feature).len(), 1);
    assert_eq!(store.get_nodes_by_type(NodeType::WorkPackage).len(), 1);
    assert_eq!(store.get_nodes_by_type(NodeType::Agent).len(), 1);
    assert_eq!(store.get_nodes_by_type(NodeType::Label).len(), 1);
    assert_eq!(store.get_nodes_by_type(NodeType::Project).len(), 1);
}

/// Test: get_relationships_from/to together cover all relationships.
#[tokio::test]
async fn get_relationships_from_and_to_cover_all() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    let c = Node::new(NodeType::Feature, json!({}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store.upsert_node(&c).await.unwrap();
    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::Owns))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(b.id, c.id, RelType::Owns))
        .await
        .unwrap();
    let from_a = store.get_relationships_from(a.id);
    let from_b = store.get_relationships_from(b.id);
    let to_b = store.get_relationships_to(b.id);
    let to_c = store.get_relationships_to(c.id);
    assert_eq!(from_a.len(), 1);
    assert_eq!(from_b.len(), 1);
    assert_eq!(to_b.len(), 1);
    assert_eq!(to_c.len(), 1);
}

/// Test: NodeType Debug formatting matches variant names.
#[test]
fn node_type_debug_matches_names() {
    use agileplus_graph::NodeType;
    assert_eq!(format!("{:?}", NodeType::Feature), "Feature");
    assert_eq!(format!("{:?}", NodeType::WorkPackage), "WorkPackage");
    assert_eq!(format!("{:?}", NodeType::Agent), "Agent");
    assert_eq!(format!("{:?}", NodeType::Label), "Label");
    assert_eq!(format!("{:?}", NodeType::Project), "Project");
}

/// Test: RelType Debug formatting matches variant names.
#[test]
fn rel_type_debug_matches_names() {
    use agileplus_graph::RelType;
    assert_eq!(format!("{:?}", RelType::Owns), "Owns");
    assert_eq!(format!("{:?}", RelType::AssignedTo), "AssignedTo");
    assert_eq!(format!("{:?}", RelType::DependsOn), "DependsOn");
    assert_eq!(format!("{:?}", RelType::Blocks), "Blocks");
    assert_eq!(format!("{:?}", RelType::Tagged), "Tagged");
    assert_eq!(format!("{:?}", RelType::InProject), "InProject");
}

/// Test: InMemoryGraphStore is Send + Sync.
#[test]
fn store_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<InMemoryGraphStore>();
}

/// Test: Relationship properties can be set and retrieved.
#[test]
fn relationship_properties_roundtrip() {
    use agileplus_graph::{RelType, Relationship};
    use serde_json::json;
    let rel = Relationship {
        id: uuid::Uuid::new_v4(),
        from_node_id: uuid::Uuid::new_v4(),
        to_node_id: uuid::Uuid::new_v4(),
        rel_type: RelType::Owns,
        properties: json!({"weight": 10, "meta": "data"}),
    };
    assert_eq!(rel.properties["weight"], 10);
    assert_eq!(rel.properties["meta"], "data");
}

/// Test: Node properties can be set and retrieved.
#[test]
fn node_properties_roundtrip() {
    use agileplus_graph::Node;
    use serde_json::json;
    let node = Node::new(NodeType::Project, json!({"name": "Proj", "version": 2}));
    assert_eq!(node.properties["name"], "Proj");
    assert_eq!(node.properties["version"], 2);
}

/// Test: InMemoryGraphStore relationships_from/to return empty for missing node.
#[tokio::test]
async fn relationships_from_to_missing_node_empty() {
    let store = InMemoryGraphStore::new();
    let missing_id = uuid::Uuid::new_v4();
    assert!(store.get_relationships_from(missing_id).is_empty());
    assert!(store.get_relationships_to(missing_id).is_empty());
}

/// Test: create_relationship with non-existent nodes is allowed (no FK check).
#[tokio::test]
async fn create_relationship_missing_nodes_allowed() {
    let store = InMemoryGraphStore::new();
    let rel = Relationship::new(uuid::Uuid::new_v4(), uuid::Uuid::new_v4(), RelType::Owns);
    store.create_relationship(&rel).await.unwrap();
    let outgoing = store.get_relationships_from(rel.from_node_id);
    assert_eq!(outgoing.len(), 1);
}

/// Test: Error enum from GraphError conversion.
#[test]
fn error_from_graph_error_conversion() {
    use agileplus_graph::{Error, GraphError};
    let graph_err = GraphError::NotFound("test".into());
    let err: Error = graph_err.into();
    let msg = format!("{err}");
    assert!(msg.contains("Graph error"));
    assert!(msg.contains("Not found"));
}

/// Test: Error Config variant display.
#[test]
fn error_config_variant_display() {
    use agileplus_graph::Error;
    let err = Error::Config("missing config".into());
    assert_eq!(format!("{err}"), "Config error: missing config");
}

/// Test: Error Debug formatting includes variant.
#[test]
fn error_debug_includes_variant() {
    use agileplus_graph::Error;
    let err = Error::Config("test".into());
    let dbg = format!("{:?}", err);
    assert!(dbg.contains("Config"));
}

/// Test: Error Graph variant Debug formatting.
#[test]
fn error_graph_variant_debug() {
    use agileplus_graph::{Error, GraphError};
    let err: Error = GraphError::ConnectionError("refused".into()).into();
    let dbg = format!("{:?}", err);
    assert!(dbg.contains("Graph"));
}

// ════════════════════════════════════════════════════════════════════════════
// Deepened coverage: traversal semantics, concurrency, trait objects, errors
// ════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn get_nodes_by_type_returns_empty_for_absent_type() {
    let store = InMemoryGraphStore::new();
    store
        .upsert_node(&Node::new(NodeType::Feature, json!({"slug": "f"})))
        .await
        .unwrap();

    assert!(store.get_nodes_by_type(NodeType::Agent).is_empty());
}

#[tokio::test]
async fn get_nodes_by_type_returns_all_matches() {
    let store = InMemoryGraphStore::new();
    for n in 0..5 {
        store
            .upsert_node(&Node::new(NodeType::WorkPackage, json!({"n": n})))
            .await
            .unwrap();
    }
    store
        .upsert_node(&Node::new(NodeType::Feature, json!({"slug": "f"})))
        .await
        .unwrap();

    assert_eq!(store.get_nodes_by_type(NodeType::WorkPackage).len(), 5);
}

#[tokio::test]
async fn get_nodes_by_type_same_type_distinct_properties_all_retrieved() {
    let store = InMemoryGraphStore::new();
    store
        .upsert_node(&Node::new(NodeType::Label, json!({"name": "a"})))
        .await
        .unwrap();
    store
        .upsert_node(&Node::new(NodeType::Label, json!({"name": "b"})))
        .await
        .unwrap();

    let labels = store.get_nodes_by_type(NodeType::Label);
    assert_eq!(labels.len(), 2);
    let names: Vec<&str> = labels
        .iter()
        .map(|n| n.properties["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"a") && names.contains(&"b"));
}

#[tokio::test]
async fn get_relationships_from_empty_for_isolated_node() {
    let store = InMemoryGraphStore::new();
    let isolated = Node::new(NodeType::Feature, json!({"slug": "iso"}));
    store.upsert_node(&isolated).await.unwrap();

    assert!(store.get_relationships_from(isolated.id).is_empty());
}

#[tokio::test]
async fn get_relationships_to_empty_for_isolated_node() {
    let store = InMemoryGraphStore::new();
    let isolated = Node::new(NodeType::Feature, json!({"slug": "iso"}));
    store.upsert_node(&isolated).await.unwrap();

    assert!(store.get_relationships_to(isolated.id).is_empty());
}

#[tokio::test]
async fn upsert_node_same_id_twice_keeps_single_entry() {
    let store = InMemoryGraphStore::new();
    let node = Node::with_id(uuid::Uuid::new_v4(), NodeType::Feature, json!({"v": 1}));
    store.upsert_node(&node).await.unwrap();

    let updated = Node::with_id(node.id, NodeType::Feature, json!({"v": 2}));
    store.upsert_node(&updated).await.unwrap();

    assert_eq!(store.get_nodes_by_type(NodeType::Feature).len(), 1);
    assert_eq!(store.get_node(node.id).unwrap().properties["v"], 2);
}

#[tokio::test]
async fn get_dependencies_preserves_insertion_order() {
    let store = InMemoryGraphStore::new();
    let source = Node::new(NodeType::WorkPackage, json!({"t": "src"}));
    store.upsert_node(&source).await.unwrap();

    let mut expected = Vec::new();
    for i in 0..4 {
        let dep = Node::new(NodeType::WorkPackage, json!({"t": i}));
        store.upsert_node(&dep).await.unwrap();
        store
            .create_relationship(&Relationship::new(source.id, dep.id, RelType::DependsOn))
            .await
            .unwrap();
        expected.push(dep.id);
    }

    assert_eq!(store.get_dependencies(source.id).await.unwrap(), expected);
}

#[tokio::test]
async fn mixed_relationship_types_on_same_pair_are_independent() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::WorkPackage, json!({"t": "a"}));
    let b = Node::new(NodeType::WorkPackage, json!({"t": "b"}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();

    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::DependsOn))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::Blocks))
        .await
        .unwrap();

    // Dependency query sees only DEPENDS_ON, blocking query sees only BLOCKS.
    assert_eq!(store.get_dependencies(a.id).await.unwrap(), vec![b.id]);
    assert_eq!(store.get_blocking_path(b.id).await.unwrap(), vec![a.id]);
    assert!(store.get_blocking_path(a.id).await.unwrap().is_empty());
    assert_eq!(store.get_relationships_from(a.id).len(), 2);
}

#[tokio::test]
async fn get_blocking_path_is_direct_only_in_memory() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::WorkPackage, json!({"t": "a"}));
    let b = Node::new(NodeType::WorkPackage, json!({"t": "b"}));
    let c = Node::new(NodeType::WorkPackage, json!({"t": "c"}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store.upsert_node(&c).await.unwrap();

    // a blocks b, b blocks c.
    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::Blocks))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(b.id, c.id, RelType::Blocks))
        .await
        .unwrap();

    // The in-memory store reports only direct blockers, not the transitive chain.
    assert_eq!(store.get_blocking_path(c.id).await.unwrap(), vec![b.id]);
    assert_eq!(store.get_blocking_path(b.id).await.unwrap(), vec![a.id]);
}

#[tokio::test]
async fn get_dependencies_is_direct_only_in_memory() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::WorkPackage, json!({"t": "a"}));
    let b = Node::new(NodeType::WorkPackage, json!({"t": "b"}));
    let c = Node::new(NodeType::WorkPackage, json!({"t": "c"}));
    for n in [&a, &b, &c] {
        store.upsert_node(n).await.unwrap();
    }
    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::DependsOn))
        .await
        .unwrap();
    store
        .create_relationship(&Relationship::new(b.id, c.id, RelType::DependsOn))
        .await
        .unwrap();

    assert_eq!(store.get_dependencies(a.id).await.unwrap(), vec![b.id]);
}

#[tokio::test]
async fn delete_relationship_by_wrong_id_keeps_original() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({"s": "a"}));
    let b = Node::new(NodeType::Feature, json!({"s": "b"}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    let rel = Relationship::new(a.id, b.id, RelType::DependsOn);
    store.create_relationship(&rel).await.unwrap();

    store
        .delete_relationship(uuid::Uuid::new_v4())
        .await
        .unwrap();

    assert_eq!(store.get_dependencies(a.id).await.unwrap().len(), 1);
}

#[tokio::test]
async fn node_upsert_after_relationship_does_not_drop_relationship() {
    let store = InMemoryGraphStore::new();
    let a = Node::new(NodeType::Feature, json!({"s": "a"}));
    let b = Node::new(NodeType::Feature, json!({"s": "b"}));
    store.upsert_node(&a).await.unwrap();
    store.upsert_node(&b).await.unwrap();
    store
        .create_relationship(&Relationship::new(a.id, b.id, RelType::DependsOn))
        .await
        .unwrap();

    store
        .upsert_node(&Node::with_id(a.id, NodeType::Feature, json!({"s": "a2"})))
        .await
        .unwrap();

    assert_eq!(store.get_dependencies(a.id).await.unwrap(), vec![b.id]);
}

#[tokio::test]
async fn self_reference_dependency_is_reported() {
    let store = InMemoryGraphStore::new();
    let node = Node::new(NodeType::WorkPackage, json!({"t": "self"}));
    store.upsert_node(&node).await.unwrap();
    store
        .create_relationship(&Relationship::new(node.id, node.id, RelType::DependsOn))
        .await
        .unwrap();

    assert_eq!(
        store.get_dependencies(node.id).await.unwrap(),
        vec![node.id]
    );
}

#[tokio::test]
async fn multi_hop_dependency_chain_reports_each_hop() {
    let store = InMemoryGraphStore::new();
    let mut nodes = Vec::new();
    for i in 0..10 {
        let n = Node::new(NodeType::WorkPackage, json!({"t": i}));
        store.upsert_node(&n).await.unwrap();
        nodes.push(n);
    }
    for w in nodes.windows(2) {
        store
            .create_relationship(&Relationship::new(w[0].id, w[1].id, RelType::DependsOn))
            .await
            .unwrap();
    }

    for (i, w) in nodes.windows(2).enumerate() {
        assert_eq!(
            store.get_dependencies(w[0].id).await.unwrap(),
            vec![w[1].id],
            "hop {i} mismatch"
        );
    }
    assert!(
        store
            .get_dependencies(nodes[9].id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn concurrent_upserts_are_all_retained() {
    let store = std::sync::Arc::new(InMemoryGraphStore::new());
    let mut handles = Vec::new();
    for i in 0..20 {
        let store = store.clone();
        handles.push(tokio::spawn(async move {
            let node = Node::new(NodeType::WorkPackage, json!({"i": i}));
            store.upsert_node(&node).await.unwrap();
            node.id
        }));
    }
    let mut ids = Vec::new();
    for h in handles {
        ids.push(h.await.unwrap());
    }

    assert_eq!(store.get_nodes_by_type(NodeType::WorkPackage).len(), 20);
    for id in ids {
        assert!(store.get_node(id).is_some());
    }
}

#[tokio::test]
async fn concurrent_relationship_creates_are_all_retained() {
    let store = std::sync::Arc::new(InMemoryGraphStore::new());
    let a = Node::new(NodeType::Feature, json!({"s": "a"}));
    store.upsert_node(&a).await.unwrap();

    let mut handles = Vec::new();
    for i in 0..15 {
        let store = store.clone();
        let from = a.id;
        handles.push(tokio::spawn(async move {
            let to = Node::new(NodeType::WorkPackage, json!({"i": i}));
            store.upsert_node(&to).await.unwrap();
            store
                .create_relationship(&Relationship::new(from, to.id, RelType::Owns))
                .await
                .unwrap();
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    assert_eq!(store.get_relationships_from(a.id).len(), 15);
}

#[tokio::test]
async fn health_check_is_idempotent() {
    let store = InMemoryGraphStore::new();
    for _ in 0..5 {
        assert!(store.health_check().await.is_ok());
    }
}

#[tokio::test]
async fn store_is_usable_as_trait_object() {
    let boxed: Box<dyn GraphStore> = Box::new(InMemoryGraphStore::new());
    let node = Node::new(NodeType::Project, json!({"slug": "p"}));
    boxed.upsert_node(&node).await.unwrap();
    assert!(boxed.health_check().await.is_ok());

    let arc: std::sync::Arc<dyn GraphStore> = std::sync::Arc::new(InMemoryGraphStore::new());
    assert!(arc.health_check().await.is_ok());
}

#[tokio::test]
async fn large_chain_is_fully_retained() {
    let store = InMemoryGraphStore::new();
    let mut prev = Node::new(NodeType::WorkPackage, json!({"i": 0}));
    store.upsert_node(&prev).await.unwrap();
    for i in 1..100 {
        let next = Node::new(NodeType::WorkPackage, json!({"i": i}));
        store.upsert_node(&next).await.unwrap();
        store
            .create_relationship(&Relationship::new(prev.id, next.id, RelType::DependsOn))
            .await
            .unwrap();
        prev = next;
    }

    assert_eq!(store.get_nodes_by_type(NodeType::WorkPackage).len(), 100);
    assert_eq!(store.get_dependencies(prev.id).await.unwrap().len(), 0);
}

// ── Error trait conformance ─────────────────────────────────────────────────

#[test]
fn graph_error_implements_std_error() {
    fn assert_error<E: std::error::Error + Send + Sync + 'static>() {}
    assert_error::<GraphError>();
}

#[test]
fn graph_error_variants_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<GraphError>();
}
