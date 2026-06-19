// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cycle (sprint/iteration) aggregate.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::feature::Feature;
use super::state_machine::FeatureState;
use crate::error::DomainError;

/// Lifecycle state of a cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CycleState {
    Draft,
    Active,
    Review,
    Completed,
    Cancelled,
    Shipped,
    Archived,
}

impl fmt::Display for CycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CycleState::Draft => "draft",
            CycleState::Active => "active",
            CycleState::Review => "review",
            CycleState::Completed => "completed",
            CycleState::Cancelled => "cancelled",
            CycleState::Shipped => "shipped",
            CycleState::Archived => "archived",
        };
        write!(f, "{s}")
    }
}

impl FromStr for CycleState {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(CycleState::Draft),
            "active" => Ok(CycleState::Active),
            "review" => Ok(CycleState::Review),
            "completed" => Ok(CycleState::Completed),
            "cancelled" => Ok(CycleState::Cancelled),
            "shipped" => Ok(CycleState::Shipped),
            "archived" => Ok(CycleState::Archived),
            _ => Err(DomainError::Validation(format!("unknown CycleState: {s}"))),
        }
    }
}

/// A planning cycle (sprint or iteration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cycle {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub state: CycleState,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub module_scope_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Association between a cycle and a feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleFeature {
    pub cycle_id: i64,
    pub feature_id: i64,
}

impl CycleFeature {
    pub fn new(cycle_id: i64, feature_id: i64) -> Self {
        Self {
            cycle_id,
            feature_id,
        }
    }
}

/// A cycle with its associated features expanded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleWithFeatures {
    pub cycle: Cycle,
    pub features: Vec<Feature>,
    pub wp_progress: WpProgressSummary,
}

impl CycleWithFeatures {
    /// A cycle is shippable when it has at least one feature and every feature
    /// is in `Validated` or `Shipped` state (FR-C07).
    pub fn is_shippable(&self) -> bool {
        !self.features.is_empty()
            && self
                .features
                .iter()
                .all(|f| matches!(f.state, FeatureState::Validated | FeatureState::Shipped))
    }
}

/// Progress summary for work packages within a cycle.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WpProgressSummary {
    pub total: u32,
    pub planned: u32,
    pub done: u32,
    pub in_progress: u32,
    pub blocked: u32,
}

impl Cycle {
    pub fn new(
        name: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        module_scope_id: Option<i64>,
    ) -> Result<Self, String> {
        if end_date < start_date {
            return Err("end date must be on or after start date".into());
        }

        let now = Utc::now();
        Ok(Self {
            id: 0,
            name: name.to_string(),
            description: None,
            state: CycleState::Draft,
            start_date,
            end_date,
            module_scope_id,
            created_at: now,
            updated_at: now,
        })
    }

    /// Validate and apply a state transition for this cycle.
    /// Allowed edges: Draft→Active, Active→Review, Review→Shipped,
    /// any→Cancelled, Shipped→Archived.
    pub fn transition(&mut self, target: CycleState) -> Result<(), DomainError> {
        let allowed = match self.state {
            CycleState::Draft => matches!(target, CycleState::Active | CycleState::Cancelled),
            CycleState::Active => matches!(
                target,
                CycleState::Review | CycleState::Shipped | CycleState::Cancelled
            ),
            CycleState::Review => {
                matches!(target, CycleState::Shipped | CycleState::Cancelled)
            }
            CycleState::Shipped => matches!(target, CycleState::Archived),
            CycleState::Completed => matches!(target, CycleState::Archived | CycleState::Cancelled),
            CycleState::Cancelled | CycleState::Archived => false,
        };
        if allowed {
            self.state = target;
            Ok(())
        } else {
            Err(DomainError::InvalidTransition {
                from: format!("{:?}", self.state),
                to: format!("{target:?}"),
                reason: "edge not permitted by cycle state machine".to_string(),
            })
        }
    }
}

impl CycleWithFeatures {
    /// Returns true if every feature in this cycle is Validated or Shipped
    /// (the "shipped gate" required before a cycle can be moved to Shipped).
    pub fn is_shippable(&self) -> bool {
        use crate::domain::state_machine::FeatureState;
        self.features.iter().all(|f| {
            matches!(f.state, FeatureState::Validated | FeatureState::Shipped)
        })
    }
}
