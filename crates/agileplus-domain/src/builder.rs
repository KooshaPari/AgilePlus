//! Builder patterns for constructing `Node` and `Edge` values safely.

use serde_json::Value;

use crate::intent_graph::{
    CanonicalMap, DagStage, Edge, Meta, Node, NodeType, RelationshipType, Status,
    ValidationError,
};

/// ---------------------------------------------------------------------------
/// Node Builder
/// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct NodeBuilder {
    id: Option<String>,
    node_type: NodeType,
    dag_stage: Option<DagStage>,
    title: Option<String>,
    description: Option<String>,
    status: Status,
    tags: Vec<String>,
    meta: Option<Meta>,
    properties: Option<Value>,
    table_ref: Option<String>,
    table_id: Option<String>,
}

impl NodeBuilder {
    /// Start building a node of the given type.
    pub fn new(node_type: NodeType) -> Self {
        Self {
            id: None,
            node_type,
            dag_stage: None,
            title: None,
            description: None,
            status: Status::Draft,
            tags: Vec::new(),
            meta: None,
            properties: None,
            table_ref: None,
            table_id: None,
        }
    }

    /// Set the node ID explicitly. If omitted, the ID is auto-generated from
    /// the node type and title slug.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the DAG stage. Defaults to the lowercase variant matching `node_type`.
    pub fn dag_stage(mut self, stage: DagStage) -> Self {
        self.dag_stage = Some(stage);
        self
    }

    /// Set the human-readable title (required).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set an optional description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the lifecycle status. Defaults to `Status::Draft`.
    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    /// Set the tag list.
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Add a single tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set the meta block (required).
    pub fn meta(mut self, meta: Meta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Set arbitrary JSON properties.
    pub fn properties(mut self, properties: Value) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Set the external table reference.
    pub fn table_ref(mut self, table_ref: impl Into<String>) -> Self {
        self.table_ref = Some(table_ref.into());
        self
    }

    /// Set the external table ID.
    pub fn table_id(mut self, table_id: impl Into<String>) -> Self {
        self.table_id = Some(table_id.into());
        self
    }

    /// Consume the builder and produce a `Node`.
    ///
    /// Returns `Err(ValidationError)` if a required field is missing or the
    /// generated / supplied ID does not match the ontology format.
    pub fn build(self) -> Result<Node, ValidationError> {
        let title = self
            .title
            .ok_or_else(|| ValidationError::MissingRequiredField("title".to_string()))?;

        let meta = self
            .meta
            .ok_or_else(|| ValidationError::MissingRequiredField("meta".to_string()))?;

        if meta.source.trim().is_empty() {
            return Err(ValidationError::MissingMeta("node: source is empty".to_string()));
        }

        let id = self.id.unwrap_or_else(|| {
            let slug = slugify(&title);
            format!("{}#{slug}", self.node_type)
        });

        if !is_valid_node_id(&id) {
            return Err(ValidationError::InvalidNodeId(id));
        }

        let dag_stage = self.dag_stage.unwrap_or(match self.node_type {
            NodeType::Intent => DagStage::Intent,
            NodeType::Plan => DagStage::Plan,
            NodeType::Feature => DagStage::Feature,
            NodeType::Story => DagStage::Story,
            NodeType::Task => DagStage::Task,
            NodeType::Spec => DagStage::Spec,
            NodeType::Commit => DagStage::Commit,
            NodeType::Test => DagStage::Test,
            NodeType::PR => DagStage::PR,
            NodeType::Bug => DagStage::Bug,
            NodeType::Artifact => DagStage::Artifact,
        });

        Ok(Node {
            id,
            node_type: self.node_type,
            dag_stage,
            title,
            description: self.description,
            status: self.status,
            tags: self.tags,
            meta,
            properties: self.properties,
            table_ref: self.table_ref,
            table_id: self.table_id,
        })
    }
}

/// ---------------------------------------------------------------------------
/// Edge Builder
/// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct EdgeBuilder {
    id: Option<String>,
    source: String,
    target: String,
    relationship_type: RelationshipType,
    canonical_map: Option<CanonicalMap>,
    meta: Option<Meta>,
    properties: Option<Value>,
}

impl EdgeBuilder {
    /// Start building an edge between `source` and `target`.
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        relationship_type: RelationshipType,
    ) -> Self {
        Self {
            id: None,
            source: source.into(),
            target: target.into(),
            relationship_type,
            canonical_map: None,
            meta: None,
            properties: None,
        }
    }

    /// Set the edge ID explicitly. If omitted, a new UUID v4 is generated.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the canonical link map.
    pub fn canonical_map(mut self, canonical_map: CanonicalMap) -> Self {
        self.canonical_map = Some(canonical_map);
        self
    }

    /// Set the meta block (required).
    pub fn meta(mut self, meta: Meta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Set arbitrary JSON properties.
    pub fn properties(mut self, properties: Value) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Consume the builder and produce an `Edge`.
    ///
    /// Returns `Err(ValidationError)` if a required field is missing.
    pub fn build(self) -> Result<Edge, ValidationError> {
        let meta = self
            .meta
            .ok_or_else(|| ValidationError::MissingRequiredField("meta".to_string()))?;

        if meta.source.trim().is_empty() {
            return Err(ValidationError::MissingMeta("edge: source is empty".to_string()));
        }

        let id = self.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        Ok(Edge {
            id,
            source: self.source,
            target: self.target,
            relationship_type: self.relationship_type,
            canonical_map: self.canonical_map,
            meta,
            properties: self.properties,
        })
    }
}

/// ---------------------------------------------------------------------------
/// Helpers
/// ---------------------------------------------------------------------------
/// Simple slugify: lowercase, ASCII alphanumeric + hyphens, trimmed.
fn slugify(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|seg| !seg.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn is_valid_node_id(id: &str) -> bool {
    use regex::Regex;
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^[A-Z][a-z]+#[a-z0-9\-]+$").unwrap());
    re.is_match(id)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use super::*;

    fn sample_meta() -> Meta {
        Meta {
            confidence: Some(0.95),
            source: "test".to_string(),
            timestamp: Utc::now(),
            agent_id: None,
        }
    }

    #[test]
    fn node_builder_ok() {
        let node = NodeBuilder::new(NodeType::Feature)
            .title("OAuth2 Login")
            .meta(sample_meta())
            .build()
            .unwrap();

        assert_eq!(node.node_type, NodeType::Feature);
        assert_eq!(node.title, "OAuth2 Login");
        assert_eq!(node.id, "Feature#oauth2-login");
        assert_eq!(node.dag_stage, DagStage::Feature);
    }

    #[test]
    fn node_builder_with_explicit_id() {
        let node = NodeBuilder::new(NodeType::Bug)
            .id("Bug#memory-leak")
            .title("Memory leak in parser")
            .meta(sample_meta())
            .build()
            .unwrap();

        assert_eq!(node.id, "Bug#memory-leak");
    }

    #[test]
    fn node_builder_rejects_missing_title() {
        let err = NodeBuilder::new(NodeType::Task)
            .meta(sample_meta())
            .build()
            .unwrap_err();
        assert!(matches!(err, ValidationError::MissingRequiredField(_)));
    }

    #[test]
    fn node_builder_rejects_missing_meta() {
        let err = NodeBuilder::new(NodeType::Task)
            .title("Fix auth")
            .build()
            .unwrap_err();
        assert!(matches!(err, ValidationError::MissingRequiredField(_)));
    }

    #[test]
    fn node_builder_rejects_invalid_id() {
        let err = NodeBuilder::new(NodeType::Task)
            .id("bad-id")
            .title("Fix auth")
            .meta(sample_meta())
            .build()
            .unwrap_err();
        assert!(matches!(err, ValidationError::InvalidNodeId(_)));
    }

    #[test]
    fn node_builder_custom_fields() {
        let node = NodeBuilder::new(NodeType::Story)
            .title("User Dashboard")
            .description("A story about the dashboard")
            .status(Status::Active)
            .tag("frontend")
            .tag("ui")
            .meta(sample_meta())
            .table_ref("stories")
            .table_id("ST-42")
            .build()
            .unwrap();

        assert_eq!(node.status, Status::Active);
        assert_eq!(node.tags, vec!["frontend", "ui"]);
        assert_eq!(node.description, Some("A story about the dashboard".to_string()));
        assert_eq!(node.table_ref, Some("stories".to_string()));
        assert_eq!(node.table_id, Some("ST-42".to_string()));
    }

    #[test]
    fn edge_builder_ok() {
        let edge = EdgeBuilder::new(
            "Intent#auth",
            "Feature#oauth2",
            RelationshipType::Implements,
        )
        .meta(sample_meta())
        .build()
        .unwrap();

        assert_eq!(edge.source, "Intent#auth");
        assert_eq!(edge.target, "Feature#oauth2");
        assert_eq!(edge.relationship_type, RelationshipType::Implements);
        // UUID should be generated automatically
        assert!(!edge.id.is_empty());
    }

    #[test]
    fn edge_builder_with_explicit_id() {
        let edge = EdgeBuilder::new(
            "Intent#auth",
            "Feature#oauth2",
            RelationshipType::Implements,
        )
        .id("edge-001")
        .meta(sample_meta())
        .build()
        .unwrap();

        assert_eq!(edge.id, "edge-001");
    }

    #[test]
    fn edge_builder_rejects_missing_meta() {
        let err = EdgeBuilder::new("A#1", "B#2", RelationshipType::TracesTo)
            .build()
            .unwrap_err();
        assert!(matches!(err, ValidationError::MissingRequiredField(_)));
    }

    #[test]
    fn slugify_helper() {
        assert_eq!(slugify("  Hello World  "), "hello-world");
        assert_eq!(slugify("Auth & OAuth2!!!"), "auth-oauth2");
        assert_eq!(slugify("---weird---"), "weird");
    }
}
