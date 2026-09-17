//! Property-based tests for domain enum FromStr implementations.
//!
//! Verifies that:
//! 1. Anything `FromStr` accepts round-trips through serde
//! 2. Strings with special characters / digits are always rejected
//! 3. The parsers never panic on arbitrary input

use std::str::FromStr;

use crate::domain::backlog::{BacklogPriority, BacklogStatus, Intent};
use crate::domain::epic::EpicStatus;
use crate::domain::user::{UserRole, UserStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        #[test]
        fn user_role_round_trip(s in "[a-zA-Z_-]{0,32}") {
            // Whatever FromStr accepts must round-trip via serialize+deserialize
            if let Ok(role) = UserRole::from_str(&s) {
                let json = serde_json::to_string(&role).unwrap();
                let back: UserRole = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(role, back);
            }
        }

        #[test]
        fn user_status_round_trip(s in "[a-zA-Z_]{0,32}") {
            if let Ok(status) = UserStatus::from_str(&s) {
                let json = serde_json::to_string(&status).unwrap();
                let back: UserStatus = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(status, back);
            }
        }

        #[test]
        fn intent_round_trip(s in "[a-zA-Z_-]{0,32}") {
            if let Ok(intent) = Intent::from_str(&s) {
                let json = serde_json::to_string(&intent).unwrap();
                let back: Intent = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(intent, back);
            }
        }

        #[test]
        fn backlog_priority_round_trip(s in "[a-zA-Z_]{0,32}") {
            if let Ok(p) = BacklogPriority::from_str(&s) {
                let json = serde_json::to_string(&p).unwrap();
                let back: BacklogPriority = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(p, back);
            }
        }

        #[test]
        fn backlog_status_round_trip(s in "[a-zA-Z_]{0,32}") {
            if let Ok(s2) = BacklogStatus::from_str(&s) {
                let json = serde_json::to_string(&s2).unwrap();
                let back: BacklogStatus = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(s2, back);
            }
        }

        #[test]
        fn epic_status_round_trip(s in "[a-zA-Z_-]{0,32}") {
            if let Ok(es) = EpicStatus::from_str(&s) {
                let json = serde_json::to_string(&es).unwrap();
                let back: EpicStatus = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(es, back);
            }
        }

        #[test]
        fn unknown_strings_rejected(s in "[!@#$%^&*()0-9 ]{1,32}") {
            // Special chars and digits should never be accepted as enum values
            prop_assert!(UserRole::from_str(&s).is_err());
            prop_assert!(UserStatus::from_str(&s).is_err());
            prop_assert!(Intent::from_str(&s).is_err());
            prop_assert!(BacklogPriority::from_str(&s).is_err());
            prop_assert!(BacklogStatus::from_str(&s).is_err());
            prop_assert!(EpicStatus::from_str(&s).is_err());
        }
    }
}

#[cfg(test)]
mod coverage_tests {
    use std::str::FromStr;

    use crate::domain::backlog::{BacklogPriority, BacklogSort, BacklogStatus, Intent};
    use crate::domain::epic::EpicStatus;
    use crate::domain::story::StoryStatus;
    use crate::domain::user::{UserRole, UserStatus};
    use proptest::prelude::*;

    #[test]
    fn intent_parsing_is_case_insensitive() {
        assert_eq!(Intent::from_str("BUG").unwrap(), Intent::Bug);
        assert_eq!(Intent::from_str("Bug").unwrap(), Intent::Bug);
        assert_eq!(Intent::from_str("bUg").unwrap(), Intent::Bug);
    }

    #[test]
    fn backlog_priority_parsing_is_case_insensitive() {
        assert_eq!(
            BacklogPriority::from_str("CRITICAL").unwrap(),
            BacklogPriority::Critical
        );
        assert_eq!(
            BacklogPriority::from_str("Low").unwrap(),
            BacklogPriority::Low
        );
    }

    #[test]
    fn backlog_status_parsing_is_case_insensitive() {
        assert_eq!(
            BacklogStatus::from_str("IN_PROGRESS").unwrap(),
            BacklogStatus::InProgress
        );
        assert_eq!(
            BacklogStatus::from_str("Triaged").unwrap(),
            BacklogStatus::Triaged
        );
    }

    #[test]
    fn backlog_sort_parsing_and_default() {
        assert_eq!(BacklogSort::from_str("age").unwrap(), BacklogSort::Age);
        assert_eq!(BacklogSort::from_str("Priority").unwrap(), BacklogSort::Priority);
        assert_eq!(BacklogSort::from_str("IMPACT").unwrap(), BacklogSort::Impact);
        assert!(BacklogSort::from_str("bogus").is_err());
        assert_eq!(BacklogSort::default(), BacklogSort::Age);
    }

    #[test]
    fn story_status_parsing_is_strict() {
        // StoryStatus uses exact matching (no to_lowercase).
        assert!(StoryStatus::from_str("todo").is_ok());
        assert!(StoryStatus::from_str("TODO").is_err());
        assert_eq!(StoryStatus::from_str("in_progress").unwrap(), StoryStatus::InProgress);
    }

    #[test]
    fn user_role_parsing_is_strict() {
        assert!(UserRole::from_str("admin").is_ok());
        assert!(UserRole::from_str("Admin").is_err());
    }

    #[test]
    fn all_enum_displays_round_trip_via_from_str() {
        for s in ["todo", "in_progress", "review", "done", "blocked", "cancelled"] {
            let st = StoryStatus::from_str(s).unwrap();
            assert_eq!(st.to_string(), s);
        }
        for s in ["backlog", "active", "review", "done", "cancelled"] {
            let es = EpicStatus::from_str(s).unwrap();
            assert_eq!(es.to_string(), s);
        }
        for s in ["active", "inactive", "suspended"] {
            let us = UserStatus::from_str(s).unwrap();
            assert_eq!(us.to_string(), s);
        }
        for s in ["admin", "member", "viewer"] {
            let ur = UserRole::from_str(s).unwrap();
            assert_eq!(ur.to_string(), s);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn backlog_sort_round_trip(s in "[a-zA-Z]{0,16}") {
            if let Ok(v) = BacklogSort::from_str(&s) {
                let json = serde_json::to_string(&v).unwrap();
                let back: BacklogSort = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(v, back);
            }
        }

        #[test]
        fn story_status_never_panics_on_arbitrary_input(s in ".{0,64}") {
            // Must return Ok/Err, never panic.
            let _ = StoryStatus::from_str(&s);
        }

        #[test]
        fn epic_status_never_panics_on_arbitrary_input(s in ".{0,64}") {
            let _ = EpicStatus::from_str(&s);
        }

        #[test]
        fn uppercase_ascii_is_rejected_by_strict_parsers(s in "[A-Z]{1,16}") {
            prop_assert!(StoryStatus::from_str(&s).is_err());
            prop_assert!(UserRole::from_str(&s).is_err());
            prop_assert!(UserStatus::from_str(&s).is_err());
            prop_assert!(EpicStatus::from_str(&s).is_err());
        }

        #[test]
        fn accepted_values_serialize_to_stable_strings(s in "[a-z_]{0,16}") {
            if let Ok(v) = StoryStatus::from_str(&s) {
                let json = serde_json::to_string(&v).unwrap();
                prop_assert_eq!(json, format!("\"{s}\""));
            }
        }
    }
}
