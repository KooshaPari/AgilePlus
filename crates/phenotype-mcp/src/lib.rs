//! # Phenotype MCP
//!
//! Model Context Protocol implementation for AI agent communication.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

// =============================================================================
// Errors
// =============================================================================

/// MCP protocol errors
#[derive(Error, Debug)]
pub enum McpError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Unknown method: {0}")]
    UnknownMethod(String),
}

/// Result type for MCP operations
pub type McpResult<T> = Result<T, McpError>;

// =============================================================================
// Message Types
// =============================================================================

/// MCP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageType {
    /// Initialize connection
    Initialize,
    /// Request
    Request,
    /// Response
    Response,
    /// Notification (no response expected)
    Notification,
    /// Error
    Error,
}

/// MCP protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
}

impl Version {
    /// Create version 1.0
    #[must_use]
    pub fn v1_0() -> Self {
        Self { major: 1, minor: 0 }
    }
}

/// MCP message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message type
    #[serde(rename = "msgType")]
    pub msg_type: String,
    /// Message ID for correlation
    #[serde(rename = "msgId")]
    pub msg_id: String,
    /// Payload
    pub payload: serde_json::Value,
}

impl Message {
    /// Create a new message
    #[must_use]
    pub fn new(msg_type: MessageType, msg_id: String, payload: serde_json::Value) -> Self {
        let msg_type = match msg_type {
            MessageType::Initialize => "initialize",
            MessageType::Request => "request",
            MessageType::Response => "response",
            MessageType::Notification => "notification",
            MessageType::Error => "error",
        };

        Self {
            msg_type: msg_type.to_string(),
            msg_id,
            payload,
        }
    }
}

// =============================================================================
// Initialize
// =============================================================================

/// Initialize request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    /// Protocol version
    pub version: Version,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
    /// Client info
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Client capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Roots support
    pub roots: bool,
    /// Sampling support
    pub sampling: bool,
    /// Window support
    pub window: bool,
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Initialize response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResponse {
    pub version: Version,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// Server capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub roots: bool,
    pub sampling: bool,
    pub window: bool,
    pub resources: bool,
    pub tools: bool,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

// =============================================================================
// Resources
// =============================================================================

/// Resource item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub mime_type: Option<String>,
    pub description: Option<String>,
}

/// Resource list request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesRequest {
    pub cursor: Option<String>,
}

/// Resource list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResponse {
    pub resources: Vec<Resource>,
    pub next_cursor: Option<String>,
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: String,
    pub content: String,
}

// =============================================================================
// Tools
// =============================================================================

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<ToolContent>,
    pub is_error: bool,
}

/// Tool content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    /// Text content
    #[serde(rename = "text")]
    Text { text: String },
    /// Image content
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    /// Resource content
    #[serde(rename = "resource")]
    Resource { resource: ResourceContent },
}

// =============================================================================
// Prompts
// =============================================================================

/// Prompt message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptContent,
}

/// Prompt content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptContent {
    /// Text content
    #[serde(rename = "text")]
    Text { text: String },
    /// Resource content
    #[serde(rename = "resource")]
    Resource { resource: ResourceContent },
}

/// Prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

/// Prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = Version::v1_0();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::new(
            MessageType::Request,
            "123".to_string(),
            serde_json::json!({"data": "test"}),
        );

        assert_eq!(msg.msg_type, "request");
        assert_eq!(msg.msg_id, "123");
    }

    #[test]
    fn test_client_capabilities() {
        let caps = ClientCapabilities {
            roots: true,
            sampling: false,
            window: true,
        };

        assert!(caps.roots);
        assert!(!caps.sampling);
        assert!(caps.window);
    }

    #[test]
    fn test_tool_serialization() {
        let tool = Tool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "arg1": {"type": "string"}
                }
            }),
        };

        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: Tool = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, tool.name);
        assert_eq!(deserialized.description, tool.description);
    }

    #[test]
    fn test_call_tool_result() {
        let result = CallToolResult {
            content: vec![
                ToolContent::Text {
                    text: "Hello, world!".to_string(),
                },
            ],
            is_error: false,
        };

        assert!(!result.is_error);
        match &result.content[0] {
            ToolContent::Text { text } => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected Text content"),
        }
    }

    #[test]
    fn test_prompt_message() {
        let msg = PromptMessage {
            role: "user".to_string(),
            content: PromptContent::Text {
                text: "Test prompt".to_string(),
            },
        };

        assert_eq!(msg.role, "user");
        match msg.content {
            PromptContent::Text { text } => assert_eq!(text, "Test prompt"),
            _ => panic!("Expected Text content"),
        }
    }
}
