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
