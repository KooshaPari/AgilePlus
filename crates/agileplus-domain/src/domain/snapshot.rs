//! `Snapshot` — point-in-time state capture for fast aggregate reloading.

use serde::{Deserialize, Serialize};

/// A snapshot captures aggregate state at a specific event sequence,
/// enabling fast reloading without replaying the full event history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Type of entity this snapshot belongs to.
    pub entity_type: String,
    /// ID of the entity.
    pub entity_id: i64,
    /// Serialized aggregate state.
    pub state: serde_json::Value,
    /// The last event sequence included in this snapshot.
    pub event_sequence: i64,
}

impl Snapshot {
    pub fn new(
        entity_type: &str,
        entity_id: i64,
        state: serde_json::Value,
        event_sequence: i64,
    ) -> Self {
        Self {
            entity_type: entity_type.to_owned(),
            entity_id,
            state,
            event_sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_new() {
        let s = Snapshot::new("Feature", 42, serde_json::json!({"state": "active"}), 100);
        assert_eq!(s.entity_type, "Feature");
        assert_eq!(s.entity_id, 42);
        assert_eq!(s.event_sequence, 100);
        assert_eq!(s.state, serde_json::json!({"state": "active"}));
    }

    #[test]
    fn snapshot_serde_roundtrip() {
        let s = Snapshot::new("WorkPackage", 7, serde_json::json!({"title": "test"}), 5);
        let json = serde_json::to_string(&s).unwrap();
        let back: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.entity_type, "WorkPackage");
        assert_eq!(back.entity_id, 7);
        assert_eq!(back.event_sequence, 5);
    }

    #[test]
    fn snapshot_clone() {
        let s = Snapshot::new("Feature", 1, serde_json::json!(null), 0);
        let s2 = s.clone();
        assert_eq!(s.entity_type, s2.entity_type);
        assert_eq!(s.entity_id, s2.entity_id);
    }

    #[test]
    fn snapshot_debug() {
        let s = Snapshot::new("Feature", 1, serde_json::json!({}), 0);
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("Snapshot"));
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn new_stores_all_fields() {
        let s = Snapshot::new("Feature", 42, serde_json::json!({"k": "v"}), 7);
        assert_eq!(s.entity_type, "Feature");
        assert_eq!(s.entity_id, 42);
        assert_eq!(s.event_sequence, 7);
        assert_eq!(s.state, serde_json::json!({"k": "v"}));
    }

    #[test]
    fn new_accepts_empty_type_and_null_state() {
        let s = Snapshot::new("", 0, serde_json::Value::Null, 0);
        assert_eq!(s.entity_type, "");
        assert!(s.state.is_null());
    }

    #[test]
    fn serde_roundtrip_nested_state() {
        let s = Snapshot::new(
            "WorkPackage",
            9,
            serde_json::json!({"a": [1, 2], "b": {"c": true}}),
            -1,
        );
        let back: Snapshot = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(back.state, s.state);
        assert_eq!(back.event_sequence, -1);
    }

    #[test]
    fn clone_is_independent() {
        let mut s = Snapshot::new("F", 1, serde_json::json!({"n": 1}), 1);
        let c = s.clone();
        s.state = serde_json::json!({"n": 2});
        assert_eq!(c.state, serde_json::json!({"n": 1}));
    }

    #[test]
    fn debug_and_json_shape() {
        let s = Snapshot::new("F", 1, serde_json::json!({}), 3);
        assert!(format!("{s:?}").contains("Snapshot"));
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["entity_type"], "F");
        assert_eq!(v["entity_id"], 1);
        assert_eq!(v["event_sequence"], 3);
    }
}
