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
