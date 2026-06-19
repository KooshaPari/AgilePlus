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
}
