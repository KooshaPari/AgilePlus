//! Snapshot type for fast aggregate rehydration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A point-in-time snapshot of an aggregate's state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub state: serde_json::Value,
    pub event_sequence: i64,
    pub taken_at: DateTime<Utc>,
}

impl Snapshot {
    /// Construct a new snapshot (id filled in by the store).
    pub fn new(
        entity_type: &str,
        entity_id: i64,
        state: serde_json::Value,
        event_sequence: i64,
    ) -> Self {
        Self {
            id: 0,
            entity_type: entity_type.to_string(),
            entity_id,
            state,
            event_sequence,
            taken_at: Utc::now(),
        }
    }
}
