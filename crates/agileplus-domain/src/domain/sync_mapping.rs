//! Sync mapping domain type — tracks entity↔remote system mappings.
//!
//! Traceability: FR-006 / WP01-T003

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteSystem {
    PlaneSo,
    Git,
    P2P,
}

impl std::fmt::Display for RemoteSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlaneSo => write!(f, "plane_so"),
            Self::Git => write!(f, "git"),
            Self::P2P => write!(f, "p2p"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncState {
    Synced,
    Pending,
    Conflicted,
}

impl std::fmt::Display for SyncState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Synced => write!(f, "synced"),
            Self::Pending => write!(f, "pending"),
            Self::Conflicted => write!(f, "conflicted"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMapping {
    pub id: Uuid,
    pub local_entity_id: String,
    pub local_entity_type: String,
    pub remote_id: String,
    pub remote_system: RemoteSystem,
    pub content_hash: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub sync_state: SyncState,
}

impl SyncMapping {
    pub fn new(
        local_entity_id: impl Into<String>,
        local_entity_type: impl Into<String>,
        remote_id: impl Into<String>,
        remote_system: RemoteSystem,
        content_hash: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            local_entity_id: local_entity_id.into(),
            local_entity_type: local_entity_type.into(),
            remote_id: remote_id.into(),
            remote_system,
            content_hash: content_hash.into(),
            last_synced_at: None,
            sync_state: SyncState::Pending,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.local_entity_id.is_empty() {
            return Err(ValidationError::empty_field("local_entity_id"));
        }
        if self.local_entity_type.is_empty() {
            return Err(ValidationError::empty_field("local_entity_type"));
        }
        if self.remote_id.is_empty() {
            return Err(ValidationError::empty_field("remote_id"));
        }
        if self.content_hash.is_empty() {
            return Err(ValidationError::empty_field("content_hash"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sync_mapping() {
        let m = SyncMapping::new(
            "entity-1".to_string(),
            "feature".to_string(),
            "plane-123".to_string(),
            RemoteSystem::PlaneSo,
            "abc123".to_string(),
        );
        assert_eq!(m.local_entity_type, "feature");
        assert_eq!(m.remote_id, "plane-123");
        assert_eq!(m.remote_system, RemoteSystem::PlaneSo);
        assert_eq!(m.sync_state, SyncState::Pending);
    }

    #[test]
    fn remote_system_display() {
        assert_eq!(RemoteSystem::PlaneSo.to_string(), "plane_so");
        assert_eq!(RemoteSystem::Git.to_string(), "git");
        assert_eq!(RemoteSystem::P2P.to_string(), "p2p");
    }

    #[test]
    fn sync_state_display() {
        assert_eq!(SyncState::Synced.to_string(), "synced");
        assert_eq!(SyncState::Pending.to_string(), "pending");
        assert_eq!(SyncState::Conflicted.to_string(), "conflicted");
    }

    #[test]
    fn sync_mapping_validation_empty_remote_id() {
        let m = SyncMapping::new(
            "entity-1".to_string(),
            "feature".to_string(),
            "".to_string(),
            RemoteSystem::Git,
            "abc123".to_string(),
        );
        assert!(m.validate().is_err());
    }

    #[test]
    fn sync_mapping_serde_roundtrip() {
        let m = SyncMapping::new(
            "entity-2".to_string(),
            "wp".to_string(),
            "plane-456".to_string(),
            RemoteSystem::PlaneSo,
            "def456".to_string(),
        );
        let json = serde_json::to_string(&m).unwrap();
        let m2: SyncMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(m2.local_entity_id, "entity-2");
        assert_eq!(m2.remote_system, RemoteSystem::PlaneSo);
    }
}
