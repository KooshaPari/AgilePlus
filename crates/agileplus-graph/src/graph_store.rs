use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

use super::types::{Node, NodeType, Relationship, RelType};

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Query error: {0}")]
    QueryError(String),
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[async_trait]
pub trait GraphStore: Send + Sync {
    async fn upsert_node(&self, node: &Node) -> Result<(), GraphError>;
    async fn get_node(&self, node_id: uuid::Uuid) -> Result<Option<Node>, GraphError>;
    async fn get_nodes_by_type(&self, node_type: NodeType) -> Result<Vec<Node>, GraphError>;
    async fn create_relationship(&self, relationship: &Relationship) -> Result<(), GraphError>;
    async fn delete_relationship(&self, relationship_id: uuid::Uuid) -> Result<(), GraphError>;
    async fn get_relationships_from(&self, node_id: uuid::Uuid) -> Result<Vec<Relationship>, GraphError>;
    async fn get_relationships_to(&self, node_id: uuid::Uuid) -> Result<Vec<Relationship>, GraphError>;
    async fn get_dependencies(&self, node_id: uuid::Uuid) -> Result<Vec<uuid::Uuid>, GraphError>;
    async fn get_blocking_path(&self, node_id: uuid::Uuid) -> Result<Vec<uuid::Uuid>, GraphError>;
    async fn health_check(&self) -> Result<(), GraphError>;
}

pub struct InMemoryGraphStore {
    nodes: Mutex<HashMap<uuid::Uuid, Node>>,
    relationships: Mutex<Vec<Relationship>>,
}

impl InMemoryGraphStore {
    pub fn new() -> Self {
        InMemoryGraphStore {
            nodes: Mutex::new(HashMap::new()),
            relationships: Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemoryGraphStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphStore for InMemoryGraphStore {
    async fn upsert_node(&self, node: &Node) -> Result<(), GraphError> {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.insert(node.id, node.clone());
        Ok(())
    }

    async fn get_node(&self, node_id: uuid::Uuid) -> Result<Option<Node>, GraphError> {
        let nodes = self.nodes.lock().unwrap();
        Ok(nodes.get(&node_id).cloned())
    }

    async fn get_nodes_by_type(&self, node_type: NodeType) -> Result<Vec<Node>, GraphError> {
        let nodes = self.nodes.lock().unwrap();
        Ok(nodes.values().filter(|n| n.node_type == node_type).cloned().collect())
    }

    async fn create_relationship(&self, relationship: &Relationship) -> Result<(), GraphError> {
        let mut rels = self.relationships.lock().unwrap();
        rels.push(relationship.clone());
        Ok(())
    }

    async fn delete_relationship(&self, relationship_id: uuid::Uuid) -> Result<(), GraphError> {
        let mut rels = self.relationships.lock().unwrap();
        rels.retain(|r| r.id != relationship_id);
        Ok(())
    }

    async fn get_relationships_from(&self, node_id: uuid::Uuid) -> Result<Vec<Relationship>, GraphError> {
        let rels = self.relationships.lock().unwrap();
        Ok(rels.iter().filter(|r| r.from_node_id == node_id).cloned().collect())
    }

    async fn get_relationships_to(&self, node_id: uuid::Uuid) -> Result<Vec<Relationship>, GraphError> {
        let rels = self.relationships.lock().unwrap();
        Ok(rels.iter().filter(|r| r.to_node_id == node_id).cloned().collect())
    }

    async fn get_dependencies(&self, node_id: uuid::Uuid) -> Result<Vec<uuid::Uuid>, GraphError> {
        let rels = self.relationships.lock().unwrap();
        let deps: Vec<uuid::Uuid> = rels
            .iter()
            .filter(|r| r.from_node_id == node_id && r.rel_type == RelType::DependsOn)
            .map(|r| r.to_node_id)
            .collect();
        Ok(deps)
    }

    async fn get_blocking_path(&self, node_id: uuid::Uuid) -> Result<Vec<uuid::Uuid>, GraphError> {
        let rels = self.relationships.lock().unwrap();
        let blockers: Vec<uuid::Uuid> = rels
            .iter()
            .filter(|r| r.to_node_id == node_id && r.rel_type == RelType::Blocks)
            .map(|r| r.from_node_id)
            .collect();
        Ok(blockers)
    }

    async fn health_check(&self) -> Result<(), GraphError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_upsert_node() {
        let store = InMemoryGraphStore::new();
        let node = Node::feature("feat-1".into(), "open".into(), "Feature One".into());
        
        store.upsert_node(&node).await.unwrap();
        
        let retrieved = store.get_node(node.id).await.unwrap().unwrap();
        assert_eq!(retrieved.properties["slug"], "feat-1");
    }

    #[tokio::test]
    async fn test_create_and_delete_relationship() {
        let store = InMemoryGraphStore::new();
        let node1 = Node::feature("f1".into(), "open".into(), "Feature 1".into());
        let node2 = Node::feature("f2".into(), "open".into(), "Feature 2".into());
        
        store.upsert_node(&node1).await.unwrap();
        store.upsert_node(&node2).await.unwrap();
        
        let rel = Relationship::new(node1.id, node2.id, RelType::DependsOn);
        store.create_relationship(&rel).await.unwrap();
        
        let deps = store.get_dependencies(node1.id).await.unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], node2.id);
        
        store.delete_relationship(rel.id).await.unwrap();
        let deps = store.get_dependencies(node1.id).await.unwrap();
        assert!(deps.is_empty());
    }

    #[tokio::test]
    async fn test_get_blocking_path() {
        let store = InMemoryGraphStore::new();
        let blocker = Node::workpackage("blocker".into(), "todo".into(), 1);
        let blocked = Node::workpackage("blocked".into(), "todo".into(), 2);
        
        store.upsert_node(&blocker).await.unwrap();
        store.upsert_node(&blocked).await.unwrap();
        
        let rel = Relationship::new(blocker.id, blocked.id, RelType::Blocks);
        store.create_relationship(&rel).await.unwrap();
        
        let blockers = store.get_blocking_path(blocked.id).await.unwrap();
        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0], blocker.id);
    }

    #[tokio::test]
    async fn test_health_check() {
        let store = InMemoryGraphStore::new();
        assert!(store.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_get_nodes_by_type() {
        let store = InMemoryGraphStore::new();
        let feature = Node::feature("feat-1".into(), "open".into(), "Feature 1".into());
        let wp = Node::workpackage("wp-1".into(), "todo".into(), 1);
        
        store.upsert_node(&feature).await.unwrap();
        store.upsert_node(&wp).await.unwrap();
        
        let features = store.get_nodes_by_type(NodeType::Feature).await.unwrap();
        assert_eq!(features.len(), 1);
        
        let wps = store.get_nodes_by_type(NodeType::WorkPackage).await.unwrap();
        assert_eq!(wps.len(), 1);
    }
}
