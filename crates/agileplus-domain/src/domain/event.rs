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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_payload() -> serde_json::Value {
        serde_json::json!({
            "title": "Add FR-DOMAIN-001 traceability",
            "status": "draft",
        })
    }

    #[test]
    fn event_new_preserves_identity_payload_actor() {
        let payload = sample_payload();
        let event = Event::new("story", 42, "Created", payload.clone(), "user@host");

        assert_eq!(event.entity_type, "story");
        assert_eq!(event.entity_id, 42);
        assert_eq!(event.event_type, "Created");
        assert_eq!(event.actor, "user@host");
        assert_eq!(event.payload, payload);
    }

    #[test]
    fn event_new_initializes_store_managed_fields() {
        let event = Event::new("story", 42, "Created", sample_payload(), "user@host");

        assert_eq!(event.id, 0);
        assert_eq!(event.sequence, 0);
        assert_eq!(event.prev_hash, [0u8; 32]);
        assert_eq!(event.hash, [0u8; 32]);
    }

    #[test]
    fn event_new_timestamp_is_non_future() {
        let before = Utc::now();
        let event = Event::new("story", 42, "Created", sample_payload(), "user@host");
        let after = Utc::now();

        assert!(
            event.timestamp <= after,
            "event timestamp {} must be <= post-construction clock {}",
            event.timestamp,
            after
        );
        assert!(event.timestamp >= before);
    }

    #[test]
    fn event_round_trips_through_serde_json() {
        let event = Event::new("story", 42, "Created", sample_payload(), "user@host");

        let json = serde_json::to_string(&event).expect("serialize event");
        let decoded: Event = serde_json::from_str(&json).expect("deserialize event");

        assert_eq!(decoded.entity_type, event.entity_type);
        assert_eq!(decoded.entity_id, event.entity_id);
        assert_eq!(decoded.event_type, event.event_type);
        assert_eq!(decoded.actor, event.actor);
        assert_eq!(decoded.payload, event.payload);
        assert_eq!(decoded.sequence, event.sequence);
    }
}
