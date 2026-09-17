//! Cycle lifecycle state and transitions.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// Lifecycle state for a Cycle.
///
/// Allowed transitions:
/// ```text
/// Draft   -> Active
/// Active  -> Review
/// Active  -> Draft      (revert)
/// Review  -> Shipped    (gate enforced in WP02/WP04, not here)
/// Review  -> Active     (changes requested)
/// Shipped -> Archived
/// ```
///
/// Traces to: FR-C02
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum CycleState {
    Draft,
    Active,
    Review,
    Shipped,
    Archived,
}

impl fmt::Display for CycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Draft => "Draft",
                Self::Active => "Active",
                Self::Review => "Review",
                Self::Shipped => "Shipped",
                Self::Archived => "Archived",
            }
        )
    }
}

impl FromStr for CycleState {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Draft" => Ok(Self::Draft),
            "Active" => Ok(Self::Active),
            "Review" => Ok(Self::Review),
            "Shipped" => Ok(Self::Shipped),
            "Archived" => Ok(Self::Archived),
            other => Err(DomainError::Other(format!("unknown cycle state: {other}"))),
        }
    }
}

impl CycleState {
    /// Validate a transition from `self` to `target`.
    ///
    /// Returns `Ok(())` for allowed edges, `Err(NoOpTransition)` for self-to-self,
    /// and `Err(InvalidTransition)` for all other pairs.
    pub fn transition(self, target: CycleState) -> Result<(), DomainError> {
        if self == target {
            return Err(DomainError::NoOpTransition);
        }
        let allowed = matches!(
            (self, target),
            (CycleState::Draft, CycleState::Active)
                | (CycleState::Active, CycleState::Review)
                | (CycleState::Active, CycleState::Draft)
                | (CycleState::Review, CycleState::Shipped)
                | (CycleState::Review, CycleState::Active)
                | (CycleState::Shipped, CycleState::Archived)
        );
        if allowed {
            Ok(())
        } else {
            Err(DomainError::InvalidTransition {
                from: self.to_string(),
                to: target.to_string(),
                reason: format!(
                    "transition from {self} to {target} is not a permitted edge in the cycle state graph"
                ),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_all_states() {
        assert_eq!(CycleState::Draft.to_string(), "Draft");
        assert_eq!(CycleState::Active.to_string(), "Active");
        assert_eq!(CycleState::Review.to_string(), "Review");
        assert_eq!(CycleState::Shipped.to_string(), "Shipped");
        assert_eq!(CycleState::Archived.to_string(), "Archived");
    }

    #[test]
    fn from_str_all_states() {
        assert_eq!("Draft".parse::<CycleState>().unwrap(), CycleState::Draft);
        assert_eq!("Active".parse::<CycleState>().unwrap(), CycleState::Active);
        assert_eq!("Review".parse::<CycleState>().unwrap(), CycleState::Review);
        assert_eq!(
            "Shipped".parse::<CycleState>().unwrap(),
            CycleState::Shipped
        );
        assert_eq!(
            "Archived".parse::<CycleState>().unwrap(),
            CycleState::Archived
        );
    }

    #[test]
    fn from_str_invalid() {
        assert!("Bogus".parse::<CycleState>().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        for state in [
            CycleState::Draft,
            CycleState::Active,
            CycleState::Review,
            CycleState::Shipped,
            CycleState::Archived,
        ] {
            let json = serde_json::to_string(&state).unwrap();
            let back: CycleState = serde_json::from_str(&json).unwrap();
            assert_eq!(back, state);
        }
    }

    #[test]
    fn valid_transitions() {
        assert!(CycleState::Draft.transition(CycleState::Active).is_ok());
        assert!(CycleState::Active.transition(CycleState::Review).is_ok());
        assert!(CycleState::Active.transition(CycleState::Draft).is_ok());
        assert!(CycleState::Review.transition(CycleState::Shipped).is_ok());
        assert!(CycleState::Review.transition(CycleState::Active).is_ok());
        assert!(CycleState::Shipped.transition(CycleState::Archived).is_ok());
    }

    #[test]
    fn invalid_transitions() {
        assert!(CycleState::Draft.transition(CycleState::Review).is_err());
        assert!(CycleState::Draft.transition(CycleState::Shipped).is_err());
        assert!(CycleState::Draft.transition(CycleState::Archived).is_err());
        assert!(CycleState::Active.transition(CycleState::Shipped).is_err());
        assert!(CycleState::Review.transition(CycleState::Draft).is_err());
        assert!(CycleState::Archived.transition(CycleState::Draft).is_err());
    }

    #[test]
    fn self_transition_is_err() {
        assert!(CycleState::Draft.transition(CycleState::Draft).is_err());
        assert!(CycleState::Active.transition(CycleState::Active).is_err());
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    const ALL: [CycleState; 5] = [
        CycleState::Draft,
        CycleState::Active,
        CycleState::Review,
        CycleState::Shipped,
        CycleState::Archived,
    ];

    fn is_allowed(from: CycleState, to: CycleState) -> bool {
        matches!(
            (from, to),
            (CycleState::Draft, CycleState::Active)
                | (CycleState::Active, CycleState::Review)
                | (CycleState::Active, CycleState::Draft)
                | (CycleState::Review, CycleState::Shipped)
                | (CycleState::Review, CycleState::Active)
                | (CycleState::Shipped, CycleState::Archived)
        )
    }

    #[test]
    fn full_25_pair_transition_matrix() {
        for from in ALL {
            for to in ALL {
                let r = from.transition(to);
                if from == to {
                    assert!(
                        matches!(r, Err(DomainError::NoOpTransition)),
                        "{from:?}->{to:?} should be NoOp, got {r:?}"
                    );
                } else if is_allowed(from, to) {
                    assert!(r.is_ok(), "{from:?}->{to:?} should be allowed");
                } else {
                    match r {
                        Err(DomainError::InvalidTransition { from: f, to: t, reason }) => {
                            assert_eq!(f, from.to_string());
                            assert_eq!(t, to.to_string());
                            assert!(!reason.is_empty());
                        }
                        other => panic!("{from:?}->{to:?} expected InvalidTransition, got {other:?}"),
                    }
                }
            }
        }
    }

    #[test]
    fn display_roundtrip_and_serde_wire() {
        for s in ALL {
            assert_eq!(s.to_string().parse::<CycleState>().unwrap(), s);
        }
        assert_eq!(serde_json::to_string(&CycleState::Draft).unwrap(), "\"Draft\"");
        assert_eq!(
            serde_json::to_string(&CycleState::Archived).unwrap(),
            "\"Archived\""
        );
    }

    #[test]
    fn from_str_rejects_unknown_and_lowercase() {
        assert!("draft".parse::<CycleState>().is_err());
        assert!("Bogus".parse::<CycleState>().is_err());
        assert!("".parse::<CycleState>().is_err());
    }

    #[test]
    fn from_str_error_mentions_input() {
        match "weird".parse::<CycleState>() {
            Err(DomainError::Other(msg)) => assert!(msg.contains("weird")),
            other => panic!("wrong: {other:?}"),
        }
    }

    #[test]
    fn terminal_archived_has_no_outgoing_edges() {
        for to in ALL {
            if to == CycleState::Archived {
                continue;
            }
            assert!(CycleState::Archived.transition(to).is_err(), "Archived->{to:?}");
        }
    }

    #[test]
    fn is_copy_hash_and_debug() {
        use std::collections::HashSet;
        let s = CycleState::Active;
        let c = s;
        assert_eq!(s, c);
        let mut set = HashSet::new();
        for st in ALL {
            set.insert(st);
        }
        assert_eq!(set.len(), 5);
        assert_eq!(format!("{:?}", CycleState::Review), "Review");
    }
}
