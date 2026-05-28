//! Feature lifecycle state machine.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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
