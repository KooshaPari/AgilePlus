//! Feature lifecycle state machine — re-exported from `traceability-core`.

pub use traceability_core::lifecycle::{FeatureState, Transition, TransitionResult};

use crate::error::DomainError;

/// Transition a feature state, mapping core lifecycle errors to [`DomainError`].
pub fn transition(
    state: FeatureState,
    target: FeatureState,
) -> Result<TransitionResult, DomainError> {
    state
        .transition(target)
        .map_err(|e| DomainError::InvalidTransition {
            from: e.from,
            to: e.to,
            reason: e.reason,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_lifecycle_transition_succeeds() {
        let result =
            transition(FeatureState::Created, FeatureState::Specified).expect("domain operation");
        assert_eq!(result.transition.from, FeatureState::Created);
        assert_eq!(result.transition.to, FeatureState::Specified);
    }

    #[test]
    fn invalid_transition_returns_error() {
        let err = transition(FeatureState::Created, FeatureState::Shipped).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition { .. }));
    }

    #[test]
    fn backward_transition_rejected() {
        let err = transition(FeatureState::Specified, FeatureState::Created).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition { .. }));
    }

    #[test]
    fn full_happy_path_lifecycle() {
        let states = [
            FeatureState::Created,
            FeatureState::Specified,
            FeatureState::Researched,
            FeatureState::Planned,
            FeatureState::Implementing,
            FeatureState::Validated,
            FeatureState::Shipped,
            FeatureState::Retrospected,
        ];
        for window in states.windows(2) {
            let result = transition(window[0], window[1]);
            assert!(
                result.is_ok(),
                "transition {:?} -> {:?} should succeed",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn feature_state_from_str_roundtrips() {
        let all_states = [
            "created",
            "specified",
            "researched",
            "planned",
            "implementing",
            "validated",
            "shipped",
            "retrospected",
        ];
        for s in all_states {
            let state: FeatureState = s.parse().expect("domain operation");
            assert_eq!(state.to_string(), s);
        }
    }

    #[test]
    fn feature_state_from_str_rejects_unknown() {
        assert!("bogus".parse::<FeatureState>().is_err());
    }
}
