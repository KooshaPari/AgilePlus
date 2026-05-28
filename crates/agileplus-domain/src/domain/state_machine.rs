//! Feature lifecycle state machine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::DomainError;

/// States in the feature lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
        write!(f, "{s}")
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
            _ => Err(format!("unknown FeatureState: {s}")),
        }
    }
}

/// A recorded state transition.
#[derive(Debug, Clone)]
pub struct Transition {
    pub from: FeatureState,
    pub to: FeatureState,
}

/// The result of a successful state machine transition.
#[derive(Debug, Clone)]
pub struct TransitionResult {
    pub transition: Transition,
    pub timestamp: DateTime<Utc>,
}

impl FeatureState {
    /// Attempt a transition to `target`.  Returns `Err(DomainError::InvalidTransition)`
    /// if the transition is not allowed by the linear lifecycle.
    pub fn transition(self, target: FeatureState) -> Result<TransitionResult, DomainError> {
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
        if !allowed {
            return Err(DomainError::InvalidTransition {
                from: self.to_string(),
                to: target.to_string(),
                reason: "not an allowed lifecycle step".to_string(),
            });
        }
        Ok(TransitionResult {
            transition: Transition { from: self, to: target },
            timestamp: Utc::now(),
        })
    }
}
