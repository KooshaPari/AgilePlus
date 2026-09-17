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

#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn cyc() -> Cycle {
        Cycle::new("C", d(2026, 1, 1), d(2026, 2, 1), None).unwrap()
    }

    #[test]
    fn new_sets_draft_and_optional_fields() {
        let c = Cycle::new("Q1", d(2026, 1, 1), d(2026, 3, 31), Some(4)).unwrap();
        assert_eq!(c.id, 0);
        assert_eq!(c.name, "Q1");
        assert_eq!(c.state, CycleState::Draft);
        assert_eq!(c.module_scope_id, Some(4));
        assert!(c.description.is_none());
        assert_eq!(c.created_at, c.updated_at);
    }

    #[test]
    fn new_accepts_single_day_apart() {
        let c = Cycle::new("S", d(2026, 1, 1), d(2026, 1, 2), None);
        assert!(c.is_ok());
    }

    #[test]
    fn new_rejects_equal_and_reversed_dates() {
        let same = Cycle::new("S", d(2026, 1, 1), d(2026, 1, 1), None);
        assert!(matches!(same, Err(DomainError::Other(_))));
        let reversed = Cycle::new("S", d(2026, 3, 1), d(2026, 1, 1), None);
        assert!(reversed.is_err());
    }

    #[test]
    fn new_accepts_multiyear_span() {
        assert!(Cycle::new("Long", d(2020, 1, 1), d(2030, 1, 1), None).is_ok());
    }

    #[test]
    fn transition_updates_state_and_timestamp() {
        let mut c = cyc();
        let before = c.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(2));
        c.transition(CycleState::Active).unwrap();
        assert_eq!(c.state, CycleState::Active);
        assert!(c.updated_at >= before);
    }

    #[test]
    fn transition_failure_leaves_state_untouched() {
        let mut c = cyc();
        let before = c.updated_at;
        assert!(c.transition(CycleState::Shipped).is_err());
        assert_eq!(c.state, CycleState::Draft);
        assert_eq!(c.updated_at, before);
    }

    #[test]
    fn transition_noop_is_error() {
        let mut c = cyc();
        assert!(matches!(
            c.transition(CycleState::Draft),
            Err(DomainError::NoOpTransition)
        ));
    }

    #[test]
    fn serde_roundtrip_preserves_dates_and_scope() {
        let c = Cycle::new("S", d(2026, 6, 1), d(2026, 6, 14), Some(3)).unwrap();
        let back: Cycle = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(back.name, "S");
        assert_eq!(back.start_date, c.start_date);
        assert_eq!(back.end_date, c.end_date);
        assert_eq!(back.module_scope_id, Some(3));
    }

    #[test]
    fn serde_omits_none_optionals() {
        let v = serde_json::to_value(cyc()).unwrap();
        assert!(v.get("description").is_none());
        assert!(v.get("module_scope_id").is_none());
    }

    #[test]
    fn clone_and_debug() {
        let c = cyc();
        let cl = c.clone();
        assert_eq!(cl.name, c.name);
        assert!(format!("{c:?}").contains("Cycle"));
    }
}
