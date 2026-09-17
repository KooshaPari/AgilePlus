//! Cycle and feature association types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Many-to-many assignment join between a Cycle and a Feature.
///
/// Traces to: FR-C03
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleFeature {
    pub cycle_id: i64,
    pub feature_id: i64,
    pub added_at: DateTime<Utc>,
}

impl CycleFeature {
    pub fn new(cycle_id: i64, feature_id: i64) -> Self {
        Self {
            cycle_id,
            feature_id,
            added_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_feature_new() {
        let cf = CycleFeature::new(1, 42);
        assert_eq!(cf.cycle_id, 1);
        assert_eq!(cf.feature_id, 42);
    }

    #[test]
    fn cycle_feature_serde_roundtrip() {
        let cf = CycleFeature::new(5, 10);
        let json = serde_json::to_string(&cf).unwrap();
        let back: CycleFeature = serde_json::from_str(&json).unwrap();
        assert_eq!(back.cycle_id, 5);
        assert_eq!(back.feature_id, 10);
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn new_stores_ids_and_timestamp_window() {
        let before = Utc::now();
        let cf = CycleFeature::new(7, 8);
        let after = Utc::now();
        assert_eq!(cf.cycle_id, 7);
        assert_eq!(cf.feature_id, 8);
        assert!(cf.added_at >= before && cf.added_at <= after);
    }

    #[test]
    fn accepts_zero_and_negative_ids() {
        let cf = CycleFeature::new(0, -1);
        assert_eq!(cf.cycle_id, 0);
        assert_eq!(cf.feature_id, -1);
    }

    #[test]
    fn serde_roundtrip_all_fields() {
        let cf = CycleFeature::new(3, 4);
        let back: CycleFeature =
            serde_json::from_str(&serde_json::to_string(&cf).unwrap()).unwrap();
        assert_eq!(back.cycle_id, 3);
        assert_eq!(back.feature_id, 4);
        assert_eq!(back.added_at, cf.added_at);
    }

    #[test]
    fn clone_and_debug() {
        let cf = CycleFeature::new(1, 2);
        let c = cf.clone();
        assert_eq!(c.cycle_id, cf.cycle_id);
        assert!(format!("{cf:?}").contains("CycleFeature"));
    }
}
