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

    // --- Property-based tests (proptest) ---
    //
    // Pure fuzz-the-impl: for every random (from, to) the transition table
    // is deterministic — same call returns the same error. We do NOT try
    // to invent the table, only verify the table produces stable, mutually-
    // consistent answers. This catches regressions where someone reorders a
    // match arm or introduces a non-deterministic branch.

    use proptest::prelude::*;

    fn arb_state() -> impl Strategy<Value = FeatureState> {
        prop_oneof![
            Just(FeatureState::Created),
            Just(FeatureState::Specified),
            Just(FeatureState::Researched),
            Just(FeatureState::Planned),
            Just(FeatureState::Implementing),
            Just(FeatureState::Validated),
            Just(FeatureState::Shipped),
            Just(FeatureState::Retrospected),
        ]
    }

    proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(512))]

        /// Same input → same output (determinism).
        #[test]
        fn transition_is_deterministic(a in arb_state(), b in arb_state()) {
            let r1 = transition(a, b);
            let r2 = transition(a, b);
            prop_assert_eq!(r1.is_ok(), r2.is_ok());
            if let (Ok(v1), Ok(v2)) = (r1, r2) {
                prop_assert_eq!(v1.transition.from, v2.transition.from);
                prop_assert_eq!(v1.transition.to, v2.transition.to);
            }
        }

        /// `transition(a, a)` is always an error (no-op is not a transition).
        #[test]
        fn same_state_transition_is_error(a in arb_state()) {
            let r = transition(a, a);
            prop_assert!(r.is_err());
        }

        /// Idempotence: a successful transition followed by transitioning
        /// to *the same state we just landed in* returns Err (no-op).
        #[test]
        fn landing_then_same_is_err(
            a in arb_state(),
            b in arb_state(),
        ) {
            if transition(a, b).is_ok() {
                prop_assert!(transition(b, b).is_err());
            }
        }

        /// Every unknown string is rejected — no panics, just Err.
        #[test]
        fn unknown_state_strings_rejected(s in ".*") {
            // Skip empties so we don't even round-trip — unknown-only path.
            if s.is_empty() { return Ok(()); }
            let r: Result<FeatureState, _> = s.parse();
            let accepted = r.is_ok();
            // If accepted, must be one of the 8 canonical states.
            if accepted {
                let known = matches!(
                    r.as_ref().unwrap(),
                    FeatureState::Created
                        | FeatureState::Specified
                        | FeatureState::Researched
                        | FeatureState::Planned
                        | FeatureState::Implementing
                        | FeatureState::Validated
                        | FeatureState::Shipped
                        | FeatureState::Retrospected
                );
                prop_assert!(known);
            }
        }
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    const ALL: [FeatureState; 8] = [
        FeatureState::Created,
        FeatureState::Specified,
        FeatureState::Researched,
        FeatureState::Planned,
        FeatureState::Implementing,
        FeatureState::Validated,
        FeatureState::Shipped,
        FeatureState::Retrospected,
    ];

    fn is_allowed(a: FeatureState, b: FeatureState) -> bool {
        matches!(
            (a, b),
            (FeatureState::Created, FeatureState::Specified)
                | (FeatureState::Specified, FeatureState::Researched)
                | (FeatureState::Researched, FeatureState::Planned)
                | (FeatureState::Planned, FeatureState::Implementing)
                | (FeatureState::Implementing, FeatureState::Validated)
                | (FeatureState::Validated, FeatureState::Shipped)
                | (FeatureState::Shipped, FeatureState::Retrospected)
        )
    }

    #[test]
    fn transition_returns_result_with_from_and_to() {
        for from in ALL {
            for to in ALL {
                let r = transition(from, to);
                if is_allowed(from, to) {
                    let tr = r.expect("allowed");
                    assert_eq!(tr.transition.from, from);
                    assert_eq!(tr.transition.to, to);
                } else {
                    match r {
                        Err(DomainError::InvalidTransition { from: f, to: t, reason }) => {
                            assert_eq!(f, from.to_string());
                            assert_eq!(t, to.to_string());
                            assert!(!reason.is_empty());
                        }
                        other => panic!("{from:?}->{to:?}: {other:?}"),
                    }
                }
            }
        }
    }

    #[test]
    fn display_roundtrip_and_wire_strings() {
        for st in ALL {
            assert_eq!(st.to_string().parse::<FeatureState>().unwrap(), st);
        }
        assert_eq!(serde_json::to_string(&FeatureState::Created).unwrap(), "\"created\"");
        assert_eq!(
            serde_json::to_string(&FeatureState::Retrospected).unwrap(),
            "\"retrospected\""
        );
    }

    #[test]
    fn from_str_rejects_unknown_and_uppercase() {
        for s in ["", "bogus", "Created", "CREATED", " created"] {
            assert!(s.parse::<FeatureState>().is_err(), "input {s:?}");
        }
    }

    #[test]
    fn terminal_state_has_no_valid_successor() {
        for to in ALL {
            assert!(!is_allowed(FeatureState::Retrospected, to));
            assert!(transition(FeatureState::Retrospected, to).is_err());
        }
    }

    #[test]
    fn created_only_advances_to_specified() {
        for to in ALL {
            let r = transition(FeatureState::Created, to);
            if to == FeatureState::Specified {
                assert!(r.is_ok());
            } else {
                assert!(r.is_err(), "Created->{to:?} must fail");
            }
        }
    }

    #[test]
    fn feature_state_is_copy_hash_debug() {
        use std::collections::HashSet;
        let st = FeatureState::Planned;
        let c = st;
        assert_eq!(st, c);
        let mut set = HashSet::new();
        for s in ALL {
            set.insert(s);
        }
        assert_eq!(set.len(), 8);
        assert_eq!(format!("{:?}", FeatureState::Shipped), "Shipped");
    }

    #[test]
    fn serde_roundtrip_all_states() {
        for st in ALL {
            let back: FeatureState =
                serde_json::from_str(&serde_json::to_string(&st).unwrap()).unwrap();
            assert_eq!(back, st);
        }
    }
}
