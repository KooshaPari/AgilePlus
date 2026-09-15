//! Cycle entity definition.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::CycleState;
use crate::error::DomainError;

/// A Cycle groups Features into a time-boxed delivery unit.
///
/// Traces to: FR-C01
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cycle {
    pub id: i64,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub state: CycleState,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    /// Optional Module scope; if set, only features owned/tagged to that Module may be assigned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_scope_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Cycle {
    /// Create a new Cycle in `Draft` state.
    ///
    /// Returns `Err` if `end_date` is not strictly after `start_date`.
    pub fn new(
        name: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        module_scope_id: Option<i64>,
    ) -> Result<Self, DomainError> {
        if end_date <= start_date {
            return Err(DomainError::Other(
                "end_date must be after start_date".to_string(),
            ));
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

    /// Transition this Cycle to `target`, updating `state` and `updated_at` on success.
    ///
    /// Note: the Review -> Shipped gate (all features validated) is enforced by the
    /// storage/service layer in WP02 and CLI in WP04 -- this method validates only
    /// the state graph edges.
    pub fn transition(&mut self, target: CycleState) -> Result<(), DomainError> {
        self.state.transition(target)?;
        self.state = target;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_new_happy_path() {
        let c = Cycle::new(
            "Sprint 1",
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 14).unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(c.name, "Sprint 1");
        assert_eq!(c.state, CycleState::Draft);
        assert_eq!(c.start_date, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        assert_eq!(c.end_date, NaiveDate::from_ymd_opt(2025, 1, 14).unwrap());
        assert!(c.module_scope_id.is_none());
        assert!(c.description.is_none());
    }

    #[test]
    fn cycle_new_rejects_same_date() {
        let d = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert!(Cycle::new("X", d, d, None).is_err());
    }

    #[test]
    fn cycle_new_rejects_end_before_start() {
        let start = NaiveDate::from_ymd_opt(2025, 1, 14).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert!(Cycle::new("X", start, end, None).is_err());
    }

    #[test]
    fn cycle_with_module_scope() {
        let c = Cycle::new(
            "S",
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            Some(42),
        )
        .unwrap();
        assert_eq!(c.module_scope_id, Some(42));
    }

    #[test]
    fn cycle_transition_updates_state() {
        let mut c = Cycle::new(
            "S",
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            None,
        )
        .unwrap();
        let before = c.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        c.transition(CycleState::Active).unwrap();
        assert_eq!(c.state, CycleState::Active);
        assert!(c.updated_at >= before);
    }

    #[test]
    fn cycle_transition_invalid() {
        let mut c = Cycle::new(
            "S",
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            None,
        )
        .unwrap();
        assert!(c.transition(CycleState::Review).is_err());
        assert_eq!(c.state, CycleState::Draft);
    }

    #[test]
    fn cycle_serde_roundtrip() {
        let c = Cycle::new(
            "Sprint",
            NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 6, 14).unwrap(),
            Some(3),
        )
        .unwrap();
        let json = serde_json::to_string(&c).unwrap();
        let back: Cycle = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Sprint");
        assert_eq!(back.module_scope_id, Some(3));
        assert_eq!(back.state, CycleState::Draft);
    }
}
