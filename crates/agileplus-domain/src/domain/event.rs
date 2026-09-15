//! `Event` — the core domain event type for AgilePlus event sourcing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An append-only domain event in the AgilePlus event store.
///
/// All events carry a SHA-256 hash that chains to the previous event,
/// enabling tamper-evident audit trails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event ID (assigned by the event store on append).
    pub id: i64,
    /// Type of entity this event belongs to (e.g., "Feature", "WorkPackage").
    pub entity_type: String,
    /// ID of the entity this event belongs to.
    pub entity_id: i64,
    /// Kind of event (e.g., "created", "transitioned", "assigned").
    pub event_type: String,
    /// JSON payload with event-specific data.
    pub payload: serde_json::Value,
    /// Actor who caused this event.
    pub actor: String,
    /// When this event occurred.
    pub timestamp: DateTime<Utc>,
    /// SHA-256 hash of the previous event in the chain.
    pub prev_hash: [u8; 32],
    /// SHA-256 hash of this event (computed over event fields + prev_hash).
    pub hash: [u8; 32],
    /// Monotonically increasing sequence number within this entity's stream.
    pub sequence: i64,
}

impl Event {
    /// Create a new unsaved event.  Hash and sequence are assigned by the store.
    pub fn new(
        entity_type: &str,
        entity_id: i64,
        event_type: &str,
        payload: serde_json::Value,
        actor: &str,
    ) -> Self {
        Self {
            id: 0,
            entity_type: entity_type.to_owned(),
            entity_id,
            event_type: event_type.to_owned(),
            payload,
            actor: actor.to_owned(),
            timestamp: Utc::now(),
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            sequence: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_new_defaults() {
        let e = Event::new("Feature", 1, "created", serde_json::json!({}), "agent-1");
        assert_eq!(e.id, 0);
        assert_eq!(e.entity_type, "Feature");
        assert_eq!(e.entity_id, 1);
        assert_eq!(e.event_type, "created");
        assert_eq!(e.actor, "agent-1");
        assert_eq!(e.sequence, 0);
        assert_eq!(e.prev_hash, [0u8; 32]);
        assert_eq!(e.hash, [0u8; 32]);
    }

    #[test]
    fn event_serde_roundtrip() {
        let e = Event::new("WorkPackage", 42, "transitioned", serde_json::json!({"state": "doing"}), "user-1");
        let json = serde_json::to_string(&e).unwrap();
        let back: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(back.entity_type, "WorkPackage");
        assert_eq!(back.entity_id, 42);
        assert_eq!(back.event_type, "transitioned");
        assert_eq!(back.actor, "user-1");
        assert_eq!(back.payload, serde_json::json!({"state": "doing"}));
    }

    #[test]
    fn event_timestamp_is_recent() {
        let before = Utc::now();
        let e = Event::new("Feature", 1, "created", serde_json::json!(null), "agent");
        let after = Utc::now();
        assert!(e.timestamp >= before);
        assert!(e.timestamp <= after);
    }

    #[test]
    fn event_clone() {
        let e = Event::new("Feature", 1, "created", serde_json::json!({}), "agent");
        let e2 = e.clone();
        assert_eq!(e.entity_type, e2.entity_type);
        assert_eq!(e.entity_id, e2.entity_id);
    }

    #[test]
    fn event_with_complex_payload() {
        let payload = serde_json::json!({
            "from": "created",
            "to": "implementing",
            "details": {"author": "alice"}
        });
        let e = Event::new("Feature", 5, "transitioned", payload.clone(), "bob");
        assert_eq!(e.payload, payload);
    }
}
