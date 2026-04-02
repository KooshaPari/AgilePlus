use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    Feature,
    WorkPackage,
    Agent,
    Label,
    Project,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Feature => "Feature",
            NodeType::WorkPackage => "WorkPackage",
            NodeType::Agent => "Agent",
            NodeType::Label => "Label",
            NodeType::Project => "Project",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelType {
    Owns,
    AssignedTo,
    DependsOn,
    Blocks,
    Tagged,
    InProject,
}

impl RelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelType::Owns => "OWNS",
            RelType::AssignedTo => "ASSIGNED_TO",
            RelType::DependsOn => "DEPENDS_ON",
            RelType::Blocks => "BLOCKS",
            RelType::Tagged => "TAGGED",
            RelType::InProject => "IN_PROJECT",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: uuid::Uuid,
    pub node_type: NodeType,
    pub properties: serde_json::Value,
}

impl Node {
    pub fn new(node_type: NodeType, properties: serde_json::Value) -> Self {
        Node {
            id: uuid::Uuid::new_v4(),
            node_type,
            properties,
        }
    }

    pub fn feature(slug: String, state: String, friendly_name: String) -> Self {
        Node::new(
            NodeType::Feature,
            serde_json::json!({"slug": slug, "state": state, "friendly_name": friendly_name}),
        )
    }

    pub fn workpackage(title: String, state: String, ordinal: i32) -> Self {
        Node::new(
            NodeType::WorkPackage,
            serde_json::json!({"title": title, "state": state, "ordinal": ordinal}),
        )
    }

    pub fn agent(name: String, agent_type: String) -> Self {
        Node::new(
            NodeType::Agent,
            serde_json::json!({"name": name, "type": agent_type}),
        )
    }

    pub fn label(name: String, color: String) -> Self {
        Node::new(
            NodeType::Label,
            serde_json::json!({"name": name, "color": color}),
        )
    }

    pub fn project(name: String, slug: String) -> Self {
        Node::new(
            NodeType::Project,
            serde_json::json!({"name": name, "slug": slug}),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: uuid::Uuid,
    pub from_node_id: uuid::Uuid,
    pub to_node_id: uuid::Uuid,
    pub rel_type: RelType,
    pub properties: serde_json::Value,
}

impl Relationship {
    pub fn new(from_node_id: uuid::Uuid, to_node_id: uuid::Uuid, rel_type: RelType) -> Self {
        Relationship {
            id: uuid::Uuid::new_v4(),
            from_node_id,
            to_node_id,
            rel_type,
            properties: serde_json::json!({}),
        }
    }
}
