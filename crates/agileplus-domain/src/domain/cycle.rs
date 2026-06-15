//! Cycle (sprint/iteration) aggregate.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::feature::Feature;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn valid_cycle_construction() {
        let c = Cycle::new("Sprint 1", date(2026, 1, 1), date(2026, 1, 14), None).unwrap();
        assert_eq!(c.name, "Sprint 1");
        assert_eq!(c.state, CycleState::Draft);
        assert_eq!(c.start_date, date(2026, 1, 1));
        assert_eq!(c.end_date, date(2026, 1, 14));
        assert!(c.module_scope_id.is_none());
    }

    #[test]
    fn cycle_with_module_scope() {
        let c = Cycle::new("Sprint 2", date(2026, 2, 1), date(2026, 2, 14), Some(42)).unwrap();
        assert_eq!(c.module_scope_id, Some(42));
    }

    #[test]
    fn rejects_end_before_start() {
        let err = Cycle::new("Bad", date(2026, 3, 10), date(2026, 3, 1), None).unwrap_err();
        assert!(err.contains("end date must be on or after start date"));
    }

    #[test]
    fn same_start_end_date_is_allowed() {
        let c = Cycle::new("One-Day", date(2026, 6, 15), date(2026, 6, 15), None).unwrap();
        assert_eq!(c.start_date, c.end_date);
    }

    #[test]
    fn cycle_feature_new() {
        let cf = CycleFeature::new(10, 20);
        assert_eq!(cf.cycle_id, 10);
        assert_eq!(cf.feature_id, 20);
    }

    #[test]
    fn cycle_state_from_str_roundtrips() {
        for s in &["draft", "active", "review", "completed", "cancelled", "shipped", "archived"] {
            let state: CycleState = s.parse().unwrap();
            assert_eq!(state.to_string(), *s);
        }
    }

    #[test]
    fn cycle_state_from_str_rejects_unknown() {
        assert!("running".parse::<CycleState>().is_err());
    }
}
