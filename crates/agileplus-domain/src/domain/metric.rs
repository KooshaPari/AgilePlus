// SPDX-License-Identifier: MIT OR Apache-2.0
//! Metric type — telemetry attached to features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A recorded metric for a feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub id: i64,
    pub feature_id: Option<i64>,
    pub command: String,
    pub duration_ms: i64,
    pub agent_runs: i32,
    pub review_cycles: i32,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_metric() -> Metric {
        Metric {
            id: 1,
            feature_id: Some(42),
            command: "cargo test".to_string(),
            duration_ms: 1500,
            agent_runs: 3,
            review_cycles: 1,
            metadata: Some(serde_json::json!({"key": "value"})),
            timestamp: DateTime::from_timestamp(0, 0).unwrap(),
        }
    }

    #[test]
    fn metric_fields_accessible() {
        let m = make_metric();
        assert_eq!(m.id, 1);
        assert_eq!(m.feature_id, Some(42));
        assert_eq!(m.command, "cargo test");
        assert_eq!(m.duration_ms, 1500);
        assert_eq!(m.agent_runs, 3);
        assert_eq!(m.review_cycles, 1);
    }

    #[test]
    fn metric_serializes_and_deserializes() {
        let m = make_metric();
        let json = serde_json::to_string(&m).unwrap();
        let back: Metric = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, m.id);
        assert_eq!(back.command, m.command);
        assert_eq!(back.duration_ms, m.duration_ms);
    }

    #[test]
    fn metric_optional_feature_id_can_be_none() {
        let mut m = make_metric();
        m.feature_id = None;
        let json = serde_json::to_string(&m).unwrap();
        let back: Metric = serde_json::from_str(&json).unwrap();
        assert!(back.feature_id.is_none());
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn metric() -> Metric {
        Metric {
            id: 7,
            feature_id: Some(3),
            command: "cargo build".into(),
            duration_ms: 250,
            agent_runs: 2,
            review_cycles: 1,
            metadata: None,
            timestamp: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        }
    }

    #[test]
    fn fields_are_accessible() {
        let m = metric();
        assert_eq!(m.id, 7);
        assert_eq!(m.feature_id, Some(3));
        assert_eq!(m.command, "cargo build");
        assert_eq!(m.duration_ms, 250);
        assert_eq!(m.agent_runs, 2);
        assert_eq!(m.review_cycles, 1);
        assert!(m.metadata.is_none());
    }

    #[test]
    fn serde_roundtrip_with_none_feature_and_metadata() {
        let mut m = metric();
        m.feature_id = None;
        let back: Metric = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
        assert!(back.feature_id.is_none());
        assert!(back.metadata.is_none());
        assert_eq!(back.duration_ms, 250);
    }

    #[test]
    fn serde_roundtrip_with_metadata() {
        let mut m = metric();
        m.metadata = Some(serde_json::json!({"k": [1, 2]}));
        let back: Metric = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
        assert_eq!(back.metadata, m.metadata);
    }

    #[test]
    fn negative_and_zero_values_allowed() {
        let mut m = metric();
        m.duration_ms = -1;
        m.agent_runs = 0;
        m.review_cycles = -5;
        let back: Metric = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
        assert_eq!(back.duration_ms, -1);
        assert_eq!(back.agent_runs, 0);
        assert_eq!(back.review_cycles, -5);
    }

    #[test]
    fn clone_and_debug() {
        let m = metric();
        let c = m.clone();
        assert_eq!(c.command, m.command);
        assert!(format!("{m:?}").contains("Metric"));
    }
}
