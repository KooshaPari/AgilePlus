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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_state() -> serde_json::Value {
        serde_json::json!({
            "title": "Aggregate state at sequence 100",
            "status": "in_progress",
        })
    }

    #[test]
    fn snapshot_new_preserves_identity_state_sequence() {
        let state = sample_state();
        let snapshot = Snapshot::new("story", 42, state.clone(), 100);

        assert_eq!(snapshot.entity_type, "story");
        assert_eq!(snapshot.entity_id, 42);
        assert_eq!(snapshot.event_sequence, 100);
        assert_eq!(snapshot.state, state);
    }

    #[test]
    fn snapshot_new_initializes_id_to_zero() {
        let snapshot = Snapshot::new("story", 42, sample_state(), 100);

        assert_eq!(snapshot.id, 0);
    }

    #[test]
    fn snapshot_new_timestamp_is_non_future() {
        let before = Utc::now();
        let snapshot = Snapshot::new("story", 42, sample_state(), 100);
        let after = Utc::now();

        assert!(
            snapshot.taken_at <= after,
            "snapshot taken_at {} must be <= post-construction clock {}",
            snapshot.taken_at,
            after
        );
        assert!(snapshot.taken_at >= before);
    }

    #[test]
    fn snapshot_round_trips_through_serde_json() {
        let snapshot = Snapshot::new("story", 42, sample_state(), 100);

        let json = serde_json::to_string(&snapshot).expect("serialize snapshot");
        let decoded: Snapshot = serde_json::from_str(&json).expect("deserialize snapshot");

        assert_eq!(decoded.entity_type, snapshot.entity_type);
        assert_eq!(decoded.entity_id, snapshot.entity_id);
        assert_eq!(decoded.event_sequence, snapshot.event_sequence);
        assert_eq!(decoded.state, snapshot.state);
    }
}
