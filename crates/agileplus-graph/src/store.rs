use crate::config::GraphConfig;
use async_trait::async_trait;
use serde_json::Value;

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

/// Trait abstracting the graph database backend. The default implementation
/// is an in-memory store suitable for testing; a Neo4j-backed implementation
/// can be provided when a real database is available.
#[async_trait]
pub trait GraphBackend: Send + Sync {
    async fn run_cypher(&self, query: &str, params: &Value) -> Result<(), GraphError>;
    async fn query_cypher(&self, query: &str, params: &Value) -> Result<Vec<Value>, GraphError>;
    async fn health_check(&self) -> Result<(), GraphError>;
}

/// The primary graph store wrapping a backend implementation.
pub struct GraphStore {
    backend: Box<dyn GraphBackend>,
    #[allow(dead_code)]
    config: GraphConfig,
}

impl GraphStore {
    /// Create a new `GraphStore` with the given backend.
    pub fn new(config: GraphConfig, backend: Box<dyn GraphBackend>) -> Self {
        GraphStore { backend, config }
    }

    /// Create a `GraphStore` backed by the in-memory implementation (for tests).
    pub fn in_memory(config: GraphConfig) -> Self {
        GraphStore {
            backend: Box::new(InMemoryBackend::new()),
            config,
        }
    }

    pub fn backend(&self) -> &dyn GraphBackend {
        &*self.backend
    }

    /// Initialize uniqueness constraints. With a real Neo4j backend this
    /// executes Cypher constraint statements. The in-memory backend treats
    /// this as a no-op.
    pub async fn init_constraints(&self) -> Result<(), GraphError> {
        let constraints = [
            "CREATE CONSTRAINT feature_id IF NOT EXISTS FOR (f:Feature) REQUIRE f.id IS UNIQUE",
            "CREATE CONSTRAINT workpackage_id IF NOT EXISTS FOR (w:WorkPackage) REQUIRE w.id IS UNIQUE",
            "CREATE CONSTRAINT agent_name IF NOT EXISTS FOR (a:Agent) REQUIRE a.name IS UNIQUE",
            "CREATE CONSTRAINT label_name IF NOT EXISTS FOR (l:Label) REQUIRE l.name IS UNIQUE",
            "CREATE CONSTRAINT project_slug IF NOT EXISTS FOR (p:Project) REQUIRE p.slug IS UNIQUE",
        ];

        for cypher in &constraints {
            // Ignore errors (constraint may already exist)
            let _ = self
                .backend
                .run_cypher(cypher, &serde_json::json!({}))
                .await;
        }

        Ok(())
    }

    pub async fn health_check(&self) -> Result<(), GraphError> {
        self.backend.health_check().await
    }

    pub(crate) async fn run_cypher(&self, query: &str, params: &Value) -> Result<(), GraphError> {
        self.backend.run_cypher(query, params).await
    }

    pub(crate) async fn query_cypher(
        &self,
        query: &str,
        params: &Value,
    ) -> Result<Vec<Value>, GraphError> {
        self.backend.query_cypher(query, params).await
    }
}

// ---------------------------------------------------------------------------
// In-memory backend for testing
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::sync::Mutex;

/// A simple in-memory graph backend that stores nodes and relationships as
/// JSON values. Cypher is not actually parsed; instead, operations are
/// performed through the higher-level `NodeStore` / `RelationshipStore`
/// APIs which call `run_cypher`/`query_cypher` with well-known query strings
/// that this backend pattern-matches against.
pub struct InMemoryBackend {
    /// Label -> id-key -> Vec<Value>
    nodes: Mutex<HashMap<String, Vec<Value>>>,
    /// (from_label, rel_type, to_label, from_id, to_id, props)
    #[allow(clippy::type_complexity)]
    relationships: Mutex<Vec<(String, String, String, Value, Value, Value)>>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        InMemoryBackend {
            nodes: Mutex::new(HashMap::new()),
            relationships: Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphBackend for InMemoryBackend {
    async fn run_cypher(&self, query: &str, params: &Value) -> Result<(), GraphError> {
        // Pattern-match well-known queries used by the store modules
        let q = query.trim();

        // CREATE node patterns
        if q.starts_with("CREATE (f:Feature") {
            let mut nodes = self.nodes.lock().unwrap();
            let list = nodes.entry("Feature".to_string()).or_default();
            list.push(params.clone());
            return Ok(());
        }
        if q.starts_with("CREATE (w:WorkPackage") {
            let mut nodes = self.nodes.lock().unwrap();
            let list = nodes.entry("WorkPackage".to_string()).or_default();
            list.push(params.clone());
            return Ok(());
        }
        if q.starts_with("CREATE (a:Agent") {
            let mut nodes = self.nodes.lock().unwrap();
            let list = nodes.entry("Agent".to_string()).or_default();
            list.push(params.clone());
            return Ok(());
        }
        if q.starts_with("CREATE (l:Label") {
            let mut nodes = self.nodes.lock().unwrap();
            let list = nodes.entry("Label".to_string()).or_default();
            list.push(params.clone());
            return Ok(());
        }
        if q.starts_with("CREATE (p:Project") {
            let mut nodes = self.nodes.lock().unwrap();
            let list = nodes.entry("Project".to_string()).or_default();
            list.push(params.clone());
            return Ok(());
        }

        // SET state
        if q.contains("SET f.state") {
            let mut nodes = self.nodes.lock().unwrap();
            if let Some(list) = nodes.get_mut("Feature") {
                for node in list.iter_mut() {
                    if node.get("id") == params.get("id") {
                        node.as_object_mut()
                            .unwrap()
                            .insert("state".to_string(), params["state"].clone());
                    }
                }
            }
            return Ok(());
        }

        // DETACH DELETE
        if q.contains("DETACH DELETE") {
            if q.contains(":Feature") {
                let mut nodes = self.nodes.lock().unwrap();
                if let Some(list) = nodes.get_mut("Feature") {
                    list.retain(|n| n.get("id") != params.get("id"));
                }
                return Ok(());
            }
            if q.contains(":WorkPackage") {
                let mut nodes = self.nodes.lock().unwrap();
                if let Some(list) = nodes.get_mut("WorkPackage") {
                    list.retain(|n| n.get("id") != params.get("id"));
                }
                return Ok(());
            }
        }

        // Relationship creation: MATCH ... CREATE ...->...
        if q.contains("CREATE") && q.contains("->") && q.contains("MATCH") {
            // Extract relationship type from pattern like [:OWNS], [:DEPENDS_ON], etc.
            if let Some(start) = q.find("[:")
                && let Some(end) = q[start..].find(']')
            {
                let rel_type = &q[start + 2..start + end];
                let mut rels = self.relationships.lock().unwrap();
                rels.push((
                    String::new(), // from_label (unused for simple matching)
                    rel_type.to_string(),
                    String::new(), // to_label
                    params.clone(),
                    serde_json::json!(null),
                    serde_json::json!(null),
                ));
            }
            return Ok(());
        }

        // DELETE relationship
        if q.starts_with("MATCH") && q.contains("DELETE r") {
            let mut rels = self.relationships.lock().unwrap();
            rels.retain(|r| {
                !(r.3.get("from_id") == params.get("from_id")
                    && r.3.get("to_id") == params.get("to_id"))
            });
            return Ok(());
        }

        // Constraint/index DDL - no-op
        if q.starts_with("CREATE CONSTRAINT")
            || q.starts_with("CREATE INDEX")
            || q.starts_with("DROP INDEX")
        {
            return Ok(());
        }

        // RETURN 1 (health check)
        if q == "RETURN 1" {
            return Ok(());
        }

        Ok(())
    }

    async fn query_cypher(&self, query: &str, params: &Value) -> Result<Vec<Value>, GraphError> {
        let q = query.trim();

        // MATCH (f:Feature {id: $id}) RETURN f
        if q.contains("MATCH (f:Feature {id:") && q.contains("RETURN f") && !q.contains("->") {
            let nodes = self.nodes.lock().unwrap();
            if let Some(list) = nodes.get("Feature") {
                for node in list {
                    if node.get("id") == params.get("id") {
                        return Ok(vec![node.clone()]);
                    }
                }
            }
            return Ok(vec![]);
        }

        // MATCH (w:WorkPackage {id: $id}) RETURN w
        if q.contains("MATCH (w:WorkPackage {id:") && q.contains("RETURN w") && !q.contains("->") {
            let nodes = self.nodes.lock().unwrap();
            if let Some(list) = nodes.get("WorkPackage") {
                for node in list {
                    if node.get("id") == params.get("id") {
                        return Ok(vec![node.clone()]);
                    }
                }
            }
            return Ok(vec![]);
        }

        // MATCH (a:Agent {name: $name}) RETURN a
        if q.contains("MATCH (a:Agent {name:") && q.contains("RETURN a") {
            let nodes = self.nodes.lock().unwrap();
            if let Some(list) = nodes.get("Agent") {
                for node in list {
                    if node.get("name") == params.get("name") {
                        return Ok(vec![node.clone()]);
                    }
                }
            }
            return Ok(vec![]);
        }

        // MATCH (p:Project {slug: $slug}) RETURN p
        if q.contains("MATCH (p:Project {slug:") && q.contains("RETURN p") {
            let nodes = self.nodes.lock().unwrap();
            if let Some(list) = nodes.get("Project") {
                for node in list {
                    if node.get("slug") == params.get("slug") {
                        return Ok(vec![node.clone()]);
                    }
                }
            }
            return Ok(vec![]);
        }

        // For graph traversal queries, return empty results in the in-memory backend
        // (full traversal would require implementing a Cypher engine)
        Ok(vec![])
    }

    async fn health_check(&self) -> Result<(), GraphError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingBackend;
    #[async_trait]
    impl GraphBackend for FailingBackend {
        async fn run_cypher(&self, _: &str, _: &Value) -> Result<(), GraphError> {
            Err(GraphError::ConnectionError("simulated".into()))
        }
        async fn query_cypher(&self, _: &str, _: &Value) -> Result<Vec<Value>, GraphError> {
            Err(GraphError::ConnectionError("simulated".into()))
        }
        async fn health_check(&self) -> Result<(), GraphError> {
            Err(GraphError::ConnectionError("simulated".into()))
        }
    }

    #[tokio::test]
    async fn test_in_memory_health_check() {
        let store = GraphStore::in_memory(GraphConfig::default());
        assert!(store.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_init_constraints() {
        let store = GraphStore::in_memory(GraphConfig::default());
        assert!(store.init_constraints().await.is_ok());
    }

    #[tokio::test]
    async fn test_graph_store_new_with_custom_backend() {
        let backend = Box::new(FailingBackend);
        let store = GraphStore::new(GraphConfig::default(), backend);
        let err = store.health_check().await.unwrap_err();
        assert!(matches!(err, GraphError::ConnectionError(_)));
    }

    #[tokio::test]
    async fn test_graph_store_backend_accessor() {
        let store = GraphStore::in_memory(GraphConfig::default());
        let backend = store.backend();
        assert!(backend.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_in_memory_run_cypher_creates_agent() {
        let backend = InMemoryBackend::new();
        backend
            .run_cypher(
                "CREATE (a:Agent {name: $name, type: $type})",
                &serde_json::json!({"name": "test", "type": "human"}),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_in_memory_run_cypher_creates_label() {
        let backend = InMemoryBackend::new();
        backend
            .run_cypher(
                "CREATE (l:Label {name: $name, color: $color})",
                &serde_json::json!({"name": "bug", "color": "#red"}),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_in_memory_run_cypher_creates_project() {
        let backend = InMemoryBackend::new();
        backend
            .run_cypher(
                "CREATE (p:Project {name: $name, slug: $slug})",
                &serde_json::json!({"name": "Test", "slug": "test"}),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_in_memory_run_cypher_sets_feature_state() {
        let backend = InMemoryBackend::new();
        backend
            .run_cypher(
                "CREATE (f:Feature {id: 1, slug: 'f1', state: 'open', friendly_name: 'F1'})",
                &serde_json::json!({}),
            )
            .await
            .unwrap();
        backend
            .run_cypher(
                "MATCH (f:Feature {id: $id}) SET f.state = $state",
                &serde_json::json!({"id": 1, "state": "closed"}),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_in_memory_run_cypher_constraint_noop() {
        let backend = InMemoryBackend::new();
        let result = backend
            .run_cypher(
                "CREATE CONSTRAINT feature_id IF NOT EXISTS FOR (f:Feature) REQUIRE f.id IS UNIQUE",
                &serde_json::json!({}),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_in_memory_run_cypher_create_index_noop() {
        let backend = InMemoryBackend::new();
        let result = backend
            .run_cypher(
                "CREATE INDEX feature_slug IF NOT EXISTS FOR (f:Feature) ON (f.slug)",
                &serde_json::json!({}),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_in_memory_run_cypher_drop_index_noop() {
        let backend = InMemoryBackend::new();
        let result = backend
            .run_cypher("DROP INDEX feature_slug IF EXISTS", &serde_json::json!({}))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_in_memory_run_cypher_health_check() {
        let backend = InMemoryBackend::new();
        let result = backend.run_cypher("RETURN 1", &serde_json::json!({})).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_in_memory_query_cypher_feature_found() {
        let backend = InMemoryBackend::new();
        backend
            .run_cypher(
                "CREATE (f:Feature {id: 1, slug: 'f1', state: 'open', friendly_name: 'F1'})",
                &serde_json::json!({}),
            )
            .await
            .unwrap();
        let result = backend
            .query_cypher(
                "MATCH (f:Feature {id: $id}) RETURN f",
                &serde_json::json!({"id": 1}),
            )
            .await
            .unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_query_cypher_workpackage_found() {
        let backend = InMemoryBackend::new();
        backend
            .run_cypher(
                "CREATE (w:WorkPackage {id: 10, title: 'WP10', state: 'todo', ordinal: 1})",
                &serde_json::json!({}),
            )
            .await
            .unwrap();
        let result = backend
            .query_cypher(
                "MATCH (w:WorkPackage {id: $id}) RETURN w",
                &serde_json::json!({"id": 10}),
            )
            .await
            .unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_query_cypher_agent_found() {
        let backend = InMemoryBackend::new();
        backend
            .run_cypher(
                "CREATE (a:Agent {name: $name, type: $type})",
                &serde_json::json!({"name": "alice", "type": "human"}),
            )
            .await
            .unwrap();
        let result = backend
            .query_cypher(
                "MATCH (a:Agent {name: $name}) RETURN a",
                &serde_json::json!({"name": "alice"}),
            )
            .await
            .unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_query_cypher_project_found() {
        let backend = InMemoryBackend::new();
        backend
            .run_cypher(
                "CREATE (p:Project {name: $name, slug: $slug})",
                &serde_json::json!({"name": "TestProj", "slug": "testproj"}),
            )
            .await
            .unwrap();
        let result = backend
            .query_cypher(
                "MATCH (p:Project {slug: $slug}) RETURN p",
                &serde_json::json!({"slug": "testproj"}),
            )
            .await
            .unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_query_cypher_traversal_returns_empty() {
        let backend = InMemoryBackend::new();
        let result = backend
            .query_cypher(
                "MATCH (f:Feature {id: $id})-[:DEPENDS_ON*]->(dep:Feature) RETURN dep.id AS id",
                &serde_json::json!({"id": 1}),
            )
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    fn test_graph_error_display() {
        let err = GraphError::ConnectionError("oops".into());
        assert!(err.to_string().contains("Connection error"));
        let err = GraphError::QueryError("bad".into());
        assert!(err.to_string().contains("Query error"));
        let err = GraphError::ConstraintViolation("cv".into());
        assert!(err.to_string().contains("Constraint violation"));
        let err = GraphError::NotFound("nf".into());
        assert!(err.to_string().contains("Not found"));
        let err = GraphError::InvalidInput("ii".into());
        assert!(err.to_string().contains("Invalid input"));
    }

    #[tokio::test]
    async fn test_in_memory_backend_run_cypher_unknown_query() {
        let backend = InMemoryBackend::new();
        let result = backend
            .run_cypher("UNKNOWN QUERY", &serde_json::json!({}))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_graph_store_run_cypher_error_propagates() {
        let backend = Box::new(FailingBackend);
        let store = GraphStore::new(GraphConfig::default(), backend);
        let result = store
            .run_cypher("CREATE (f:Feature {id: 1})", &serde_json::json!({}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_graph_store_query_cypher_error_propagates() {
        let backend = Box::new(FailingBackend);
        let store = GraphStore::new(GraphConfig::default(), backend);
        let result = store
            .query_cypher("MATCH (f:Feature) RETURN f", &serde_json::json!({}))
            .await;
        assert!(result.is_err());
    }
}
