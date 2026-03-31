//! Event sourcing domain types — immutable events with hash-chain integrity.
//!
//! Traceability: FR-020..FR-025 / WP01-T001

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub entity_id: String,
    pub entity_type: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub actor: String,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
    pub previous_hash: Option<String>,
    pub hash: String,
}

impl Event {
    pub fn new(
        entity_id: impl Into<String>,
        entity_type: impl Into<String>,
        event_type: impl Into<String>,
        payload: serde_json::Value,
        actor: impl Into<String>,
        sequence: u64,
        previous_hash: Option<String>,
        hash: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_id: entity_id.into(),
            entity_type: entity_type.into(),
            event_type: event_type.into(),
            payload,
            actor: actor.into(),
            timestamp: Utc::now(),
            sequence,
            previous_hash,
            hash: hash.into(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.entity_id.is_empty() {
            return Err(ValidationError::empty_field("entity_id"));
        }
        if self.entity_type.is_empty() {
            return Err(ValidationError::empty_field("entity_type"));
        }
        if self.event_type.is_empty() {
            return Err(ValidationError::empty_field("event_type"));
        }
        if self.actor.is_empty() {
            return Err(ValidationError::empty_field("actor"));
        }
        if self.hash.is_empty() {
            return Err(ValidationError::empty_field("hash"));
        }
        if self.hash.len() != 64 {
            return Err(ValidationError::invalid_value("hash", "must be 64 characters (SHA-256 hex)"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_event() {
        let e = Event::new(
            "entity-1".to_string(),
            "feature".to_string(),
            "state_transitioned".to_string(),
            serde_json::json!({}),
            "system".to_string(),
            1,
            None,
            "abc123def456".to_string(),
        );
        assert_eq!(e.entity_id, "entity-1");
        assert_eq!(e.entity_type, "feature");
        assert_eq!(e.sequence, 1);
        assert!(e.previous_hash.is_none());
    }

    #[test]
    fn event_with_previous_hash() {
        let e = Event::new(
            "entity-1".to_string(),
            "feature".to_string(),
            "created".to_string(),
            serde_json::json!({"title": "WP05"}),
            "agent".to_string(),
            2,
            Some("previous_hash_value".to_string()),
            "current_hash_value".to_string(),
        );
        assert!(e.previous_hash.is_some());
        assert_eq!(e.previous_hash.as_deref(), Some("previous_hash_value"));
    }

    #[test]
    fn event_validation_empty_entity_id() {
        let e = Event::new(
            "".to_string(),
            "feature".to_string(),
            "created".to_string(),
            serde_json::json!({}),
            "agent".to_string(),
            1,
            None,
            "abc123def456abc123def456abc123def456abc123def456abc123def456abc123def456",
        );
        assert!(e.validate().is_err());
    }

    #[test]
    fn event_validation_empty_hash() {
        let e = Event::new(
            "entity-1".to_string(),
            "feature".to_string(),
            "created".to_string(),
            serde_json::json!({}),
            "agent".to_string(),
            1,
            None,
            "".to_string(),
        );
        assert!(e.validate().is_err());
    }

    #[test]
    fn event_serde_roundtrip() {
        let e = Event::new(
            "entity-5".to_string(),
            "wp".to_string(),
            "created".to_string(),
            serde_json::json!({"title": "WP05"}),
            "agent".to_string(),
            1,
            None,
            "abc123def456abc123def456abc123def456abc123def456abc123def456abc123def456",
        );
        let json = serde_json::to_string(&e).unwrap();
        let e2: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(e2.entity_type, "wp");
        assert_eq!(e2.entity_id, "entity-5");
    }
}
