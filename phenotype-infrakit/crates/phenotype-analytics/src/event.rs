//! Analytics event types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents an analytics event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event name
    pub name: String,
    /// Event properties
    pub properties: serde_json::Value,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// User identifier (optional)
    pub user_id: Option<String>,
}

impl Event {
    /// Create a new event
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            properties: serde_json::json!({}),
            timestamp: Utc::now(),
            user_id: None,
        }
    }

    /// Add a property to the event
    pub fn with_property(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(val) = serde_json::to_value(value) {
            self.properties.as_object_mut().map(|m| m.insert(key.into(), val));
        }
        self
    }

    /// Set the user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}
