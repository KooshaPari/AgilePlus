//! Domain event types — append-only event sourcing primitives.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An event-sourcing domain event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub actor: String,
    pub timestamp: DateTime<Utc>,
    pub prev_hash: [u8; 32],
    pub hash: [u8; 32],
    pub sequence: i64,
}

impl Event {
    /// Construct a new event (id/sequence/hashes filled in by the store).
    pub fn new(
        entity_type: &str,
        entity_id: i64,
        event_type: &str,
        payload: serde_json::Value,
        actor: &str,
    ) -> Self {
        Self {
            id: 0,
            entity_type: entity_type.to_string(),
            entity_id,
            event_type: event_type.to_string(),
            payload,
            actor: actor.to_string(),
            timestamp: Utc::now(),
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            sequence: 0,
        }
    }
}

/// Coarse category of an event for filtering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    Created,
    Updated,
    Deleted,
    StateChanged,
    Custom(String),
}

/// Metadata attached to an event that is not part of the payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub tags: Vec<String>,
}
