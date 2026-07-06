//! Feature lifecycle state machine.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureState {
    Created,
    Specified,
    Researched,
    Planned,
    Implementing,
    Validated,
    Shipped,
    Retrospected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub from: FeatureState,
    pub to: FeatureState,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionResult {
    pub transition: Transition,
}

impl FeatureState {
    pub fn transition(self, target: FeatureState) -> Result<TransitionResult, Transition> {
        let allowed = matches!(
            (self, target),
            (FeatureState::Created, FeatureState::Specified)
                | (FeatureState::Specified, FeatureState::Researched)
                | (FeatureState::Researched, FeatureState::Planned)
                | (FeatureState::Planned, FeatureState::Implementing)
                | (FeatureState::Implementing, FeatureState::Validated)
                | (FeatureState::Validated, FeatureState::Shipped)
                | (FeatureState::Shipped, FeatureState::Retrospected)
        );
        let transition = Transition {
            from: self,
            to: target,
            reason: if allowed { String::new() } else { "invalid state transition".into() },
        };
        if allowed {
            Ok(TransitionResult { transition })
        } else {
            Err(transition)
        }
    }
}

impl fmt::Display for FeatureState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FeatureState::Created => "created",
            FeatureState::Specified => "specified",
            FeatureState::Researched => "researched",
            FeatureState::Planned => "planned",
            FeatureState::Implementing => "implementing",
            FeatureState::Validated => "validated",
            FeatureState::Shipped => "shipped",
            FeatureState::Retrospected => "retrospected",
        };
        f.write_str(s)
    }
}

impl FromStr for FeatureState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created" => Ok(FeatureState::Created),
            "specified" => Ok(FeatureState::Specified),
            "researched" => Ok(FeatureState::Researched),
            "planned" => Ok(FeatureState::Planned),
            "implementing" => Ok(FeatureState::Implementing),
            "validated" => Ok(FeatureState::Validated),
            "shipped" => Ok(FeatureState::Shipped),
            "retrospected" => Ok(FeatureState::Retrospected),
            other => Err(format!("unknown FeatureState: {other}")),
        }
    }
}

use crate::error::DomainError;

/// Transition a feature state, mapping core lifecycle errors to [`DomainError`].
pub fn transition(
    state: FeatureState,
    target: FeatureState,
) -> Result<TransitionResult, DomainError> {
    state.transition(target).map_err(|e| DomainError::InvalidTransition {
        from: format!("{:?}", e.from),
        to: format!("{:?}", e.to),
        reason: e.reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_lifecycle_transition_succeeds() {
        let result = transition(FeatureState::Created, FeatureState::Specified).expect("domain operation");
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

