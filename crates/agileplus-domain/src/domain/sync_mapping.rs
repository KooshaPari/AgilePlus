// SPDX-License-Identifier: MIT OR Apache-2.0
//! Sync mapping — links internal entities to external plane issues.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Direction of a sync operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncDirection {
    Push,
    Pull,
    Bidirectional,
}

impl fmt::Display for SyncDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SyncDirection::Push => "push",
            SyncDirection::Pull => "pull",
            SyncDirection::Bidirectional => "bidirectional",
        };
        write!(f, "{s}")
    }
}

/// Maps an internal entity to an external plane issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMapping {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub plane_issue_id: String,
    pub content_hash: String,
    pub last_synced_at: DateTime<Utc>,
    pub sync_direction: SyncDirection,
    pub conflict_count: i32,
}

impl SyncMapping {
    /// Convenience constructor — sets defaults for `id`, `last_synced_at`,
    /// `sync_direction`, and `conflict_count`.
    pub fn new(
        entity_type: &str,
        entity_id: i64,
        plane_issue_id: &str,
        content_hash: &str,
    ) -> Self {
        Self {
            id: 0,
            entity_type: entity_type.to_string(),
            entity_id,
            plane_issue_id: plane_issue_id.to_string(),
            content_hash: content_hash.to_string(),
            last_synced_at: Utc::now(),
            sync_direction: SyncDirection::Bidirectional,
            conflict_count: 0,
        }
    }

    pub fn increment_conflict(&mut self) {
        self.conflict_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_mapping_defaults() {
        let m = SyncMapping::new("feature", 1, "plane-123", "abc");
        assert_eq!(m.id, 0);
        assert_eq!(m.entity_type, "feature");
        assert_eq!(m.entity_id, 1);
        assert_eq!(m.plane_issue_id, "plane-123");
        assert_eq!(m.content_hash, "abc");
        assert_eq!(m.sync_direction, SyncDirection::Bidirectional);
        assert_eq!(m.conflict_count, 0);
    }

    #[test]
    fn sync_direction_display() {
        assert_eq!(SyncDirection::Push.to_string(), "push");
        assert_eq!(SyncDirection::Pull.to_string(), "pull");
        assert_eq!(SyncDirection::Bidirectional.to_string(), "bidirectional");
    }

    #[test]
    fn sync_direction_equality() {
        assert_eq!(SyncDirection::Push, SyncDirection::Push);
        assert_ne!(SyncDirection::Push, SyncDirection::Pull);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_mapping_defaults() {
        let m = SyncMapping::new("feature", 1, "plane-123", "abc");
        assert_eq!(m.id, 0);
        assert_eq!(m.entity_type, "feature");
        assert_eq!(m.entity_id, 1);
        assert_eq!(m.plane_issue_id, "plane-123");
        assert_eq!(m.content_hash, "abc");
        assert_eq!(m.sync_direction, SyncDirection::Bidirectional);
        assert_eq!(m.conflict_count, 0);
    }

    #[test]
    fn sync_direction_display() {
        assert_eq!(SyncDirection::Push.to_string(), "push");
        assert_eq!(SyncDirection::Pull.to_string(), "pull");
        assert_eq!(SyncDirection::Bidirectional.to_string(), "bidirectional");
    }

    #[test]
    fn sync_direction_equality() {
        assert_eq!(SyncDirection::Push, SyncDirection::Push);
        assert_ne!(SyncDirection::Push, SyncDirection::Pull);
    }
}
