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
