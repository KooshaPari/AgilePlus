//! Cycle (sprint/iteration) aggregate.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::feature::Feature;

/// Lifecycle state of a cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CycleState {
    Draft,
    Active,
    Completed,
    Cancelled,
}

impl fmt::Display for CycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CycleState::Draft => "draft",
            CycleState::Active => "active",
            CycleState::Completed => "completed",
            CycleState::Cancelled => "cancelled",
        };
        write!(f, "{s}")
    }
}

impl FromStr for CycleState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(CycleState::Draft),
            "active" => Ok(CycleState::Active),
            "completed" => Ok(CycleState::Completed),
            "cancelled" => Ok(CycleState::Cancelled),
            _ => Err(format!("unknown CycleState: {s}")),
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

/// A cycle with its associated features expanded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleWithFeatures {
    pub cycle: Cycle,
    pub features: Vec<Feature>,
    pub wp_progress: WpProgressSummary,
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
