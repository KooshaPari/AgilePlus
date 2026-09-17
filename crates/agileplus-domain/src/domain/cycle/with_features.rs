//! Cycle views with assigned feature data.

use serde::{Deserialize, Serialize};

use super::{Cycle, WpProgressSummary};
use crate::domain::feature::Feature;
use crate::domain::state_machine::FeatureState;

/// View struct carrying a Cycle together with its assigned features and WP progress.
/// Populated by the storage/query layer in WP02.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleWithFeatures {
    pub cycle: Cycle,
    pub features: Vec<Feature>,
    /// Aggregate count of WPs per WpState across all assigned features.
    pub wp_progress: WpProgressSummary,
}

impl CycleWithFeatures {
    /// Return `true` when the cycle is safe to ship.
    ///
    /// Per FR-C07: all assigned Features must be in `Validated` or `Shipped` state.
    /// An empty feature list is treated as vacuously shippable (no blocking features).
    /// A Cycle with no assigned Features is a planning placeholder -- callers may
    /// apply an additional "at least one feature" guard at the service layer.
    pub fn is_shippable(&self) -> bool {
        self.features
            .iter()
            .all(|f| matches!(f.state, FeatureState::Validated | FeatureState::Shipped))
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn cycle_with(states: &[FeatureState]) -> CycleWithFeatures {
        let cycle = Cycle::new(
            "C",
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            None,
        )
        .unwrap();
        let features = states
            .iter()
            .enumerate()
            .map(|(i, st)| {
                let mut f = Feature::new(&format!("f{i}"), "F", [0u8; 32], None);
                f.state = *st;
                f
            })
            .collect();
        CycleWithFeatures {
            cycle,
            features,
            wp_progress: WpProgressSummary::default(),
        }
    }

    #[test]
    fn empty_feature_list_is_vacuously_shippable() {
        assert!(cycle_with(&[]).is_shippable());
    }

    #[test]
    fn per_state_shippability_table() {
        let shippable = [FeatureState::Validated, FeatureState::Shipped];
        let not_shippable = [
            FeatureState::Created,
            FeatureState::Specified,
            FeatureState::Researched,
            FeatureState::Planned,
            FeatureState::Implementing,
            FeatureState::Retrospected,
        ];
        for st in shippable {
            assert!(cycle_with(&[st]).is_shippable(), "{st:?} should ship");
        }
        for st in not_shippable {
            assert!(!cycle_with(&[st]).is_shippable(), "{st:?} must not ship");
        }
    }

    #[test]
    fn mixed_validated_and_shipped_is_shippable() {
        assert!(cycle_with(&[FeatureState::Validated, FeatureState::Shipped]).is_shippable());
    }

    #[test]
    fn one_blocker_blocks_the_whole_cycle() {
        assert!(!cycle_with(&[
            FeatureState::Validated,
            FeatureState::Shipped,
            FeatureState::Implementing,
        ])
        .is_shippable());
    }

    #[test]
    fn serde_roundtrip() {
        let cwf = cycle_with(&[FeatureState::Validated]);
        let back: CycleWithFeatures =
            serde_json::from_str(&serde_json::to_string(&cwf).unwrap()).unwrap();
        assert_eq!(back.features.len(), 1);
        assert!(back.is_shippable());
    }
}
