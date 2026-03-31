//! Snapshot domain type — periodic materialized state for fast reads.
//!
//! Traceability: FR-022 / WP01-T002

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: Uuid,
    pub entity_id: String,
    pub entity_type: String,
    pub state: serde_json::Value,
    pub sequence: u64,
    pub created_at: DateTime<Utc>,
    pub hash: String,
}

impl Snapshot {
    pub fn new(
        entity_id: impl Into<String>,
        entity_type: impl Into<String>,
        state: serde_json::Value,
        sequence: u64,
        hash: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_id: entity_id.into(),
            entity_type: entity_type.into(),
            state,
            sequence,
            created_at: Utc::now(),
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
    fn new_snapshot() {
        let s = Snapshot::new(
            "entity-1".to_string(),
            "feature".to_string(),
            serde_json::json!({"state": "implementing"}),
            100,
            "abc123def456abc123def456abc123def456abc123def456abc123def456abc123def456",
        );
        assert_eq!(s.entity_type, "feature");
        assert_eq!(s.sequence, 100);
    }

    #[test]
    fn snapshot_validation_empty_entity_id() {
        let s = Snapshot::new(
            "".to_string(),
            "feature".to_string(),
            serde_json::json!({"state": "done"}),
            50,
            "abc123def456abc123def456abc123def456abc123def456abc123def456abc123def456",
        );
        assert!(s.validate().is_err());
    }

    #[test]
    fn snapshot_serde_roundtrip() {
        let s = Snapshot::new(
            "entity-3".to_string(),
            "wp".to_string(),
            serde_json::json!({"state": "doing"}),
            50,
            "abc123def456abc123def456abc123def456abc123def456abc123def456abc123def456",
        );
        let json = serde_json::to_string(&s).unwrap();
        let s2: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(s2.entity_id, "entity-3");
        assert_eq!(s2.sequence, 50);
    }
}
