//! Epic aggregate — a large body of work scoped to a project.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::DomainError;

/// Lifecycle status of an epic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EpicStatus {
    Backlog,
    Active,
    Review,
    Done,
    Cancelled,
}

impl fmt::Display for EpicStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EpicStatus::Backlog => "backlog",
            EpicStatus::Active => "active",
            EpicStatus::Review => "review",
            EpicStatus::Done => "done",
            EpicStatus::Cancelled => "cancelled",
        };
        write!(f, "{s}")
    }
}

impl FromStr for EpicStatus {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "backlog" => Ok(EpicStatus::Backlog),
            "active" => Ok(EpicStatus::Active),
            "review" => Ok(EpicStatus::Review),
            "done" => Ok(EpicStatus::Done),
            "cancelled" => Ok(EpicStatus::Cancelled),
            _ => Err(DomainError::Validation(format!("unknown EpicStatus: {s}"))),
        }
    }
}

impl EpicStatus {
    /// Returns `true` if a transition from `self` to `target` is allowed.
    pub fn can_transition_to(self, target: EpicStatus) -> bool {
        use EpicStatus::*;
        matches!(
            (self, target),
            (Backlog, Active)
                | (Active, Review)
                | (Active, Cancelled)
                | (Review, Done)
                | (Review, Active)
        )
    }
}

/// An epic — a large, named unit of work belonging to a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epic {
    pub id: i64,
    /// Owning project.
    pub project_id: i64,
    /// Human-readable title — must be non-empty.
    pub title: String,
    pub description: Option<String>,
    pub status: EpicStatus,
    /// Optional owner (user id).
    pub owner_id: Option<i64>,
    /// Epic/requirement id this epic maps to.
    #[serde(default)]
    pub requirement_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Epic {
    /// Construct a new `Epic`. `title` must be non-empty.
    pub fn new(project_id: i64, title: &str) -> Result<Self, DomainError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(DomainError::Validation(
                "epic title must not be empty".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: 0,
            project_id,
            title: title.to_string(),
            description: None,
            status: EpicStatus::Backlog,
            owner_id: None,
            requirement_id: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Attempt a status transition. Returns `Err(DomainError::InvalidTransition)`
    /// if the transition is not permitted.
    pub fn transition_status(&mut self, target: EpicStatus) -> Result<(), DomainError> {
        if !self.status.can_transition_to(target) {
            return Err(DomainError::InvalidTransition {
                from: self.status.to_string(),
                to: target.to_string(),
                reason: "not an allowed epic status transition".to_string(),
            });
        }
        self.status = target;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_epic_construction() {
        let e = Epic::new(1, "Authentication Overhaul").unwrap();
        assert_eq!(e.title, "Authentication Overhaul");
        assert_eq!(e.project_id, 1);
        assert_eq!(e.status, EpicStatus::Backlog);
    }

    #[test]
    fn rejects_empty_title() {
        let err = Epic::new(1, "  ").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn valid_status_transition() {
        let mut e = Epic::new(1, "Big Feature").unwrap();
        e.transition_status(EpicStatus::Active).unwrap();
        assert_eq!(e.status, EpicStatus::Active);
        e.transition_status(EpicStatus::Review).unwrap();
        assert_eq!(e.status, EpicStatus::Review);
    }

    #[test]
    fn invalid_status_transition_rejected() {
        let mut e = Epic::new(1, "Skipped Epic").unwrap();
        // Backlog -> Done is not allowed
        let err = e.transition_status(EpicStatus::Done).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition { .. }));
    }

    #[test]
    fn title_trimmed_on_construction() {
        let e = Epic::new(2, "  Trimmed  ").unwrap();
        assert_eq!(e.title, "Trimmed");
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    const ALL: [EpicStatus; 5] = [
        EpicStatus::Backlog,
        EpicStatus::Active,
        EpicStatus::Review,
        EpicStatus::Done,
        EpicStatus::Cancelled,
    ];

    fn allowed(a: EpicStatus, b: EpicStatus) -> bool {
        matches!(
            (a, b),
            (EpicStatus::Backlog, EpicStatus::Active)
                | (EpicStatus::Active, EpicStatus::Review)
                | (EpicStatus::Active, EpicStatus::Cancelled)
                | (EpicStatus::Review, EpicStatus::Done)
                | (EpicStatus::Review, EpicStatus::Active)
        )
    }

    #[test]
    fn can_transition_to_full_matrix() {
        for from in ALL {
            for to in ALL {
                assert_eq!(from.can_transition_to(to), allowed(from, to), "{from:?}->{to:?}");
            }
        }
    }

    #[test]
    fn transition_status_matrix() {
        for from in ALL {
            for to in ALL {
                let mut e = Epic::new(1, "T").unwrap();
                e.status = from;
                let r = e.transition_status(to);
                if allowed(from, to) {
                    assert!(r.is_ok(), "{from:?}->{to:?}");
                    assert_eq!(e.status, to);
                } else {
                    assert!(r.is_err(), "{from:?}->{to:?}");
                    assert_eq!(e.status, from);
                }
            }
        }
    }

    #[test]
    fn invalid_transition_error_fields() {
        let mut e = Epic::new(1, "T").unwrap();
        match e.transition_status(EpicStatus::Done).unwrap_err() {
            DomainError::InvalidTransition { from, to, reason } => {
                assert_eq!(from, "backlog");
                assert_eq!(to, "done");
                assert!(reason.contains("epic"));
            }
            other => panic!("wrong: {other:?}"),
        }
    }

    #[test]
    fn new_trims_title_and_defaults() {
        let e = Epic::new(9, "  Big Epic  ").unwrap();
        assert_eq!(e.title, "Big Epic");
        assert_eq!(e.project_id, 9);
        assert_eq!(e.id, 0);
        assert_eq!(e.status, EpicStatus::Backlog);
        assert!(e.description.is_none());
        assert!(e.owner_id.is_none());
        assert!(e.requirement_id.is_none());
    }

    #[test]
    fn new_rejects_empty_and_whitespace_title() {
        for t in ["", "   ", "\t"] {
            assert!(matches!(Epic::new(1, t), Err(DomainError::Validation(_))));
        }
    }

    #[test]
    fn display_roundtrip_and_case_sensitivity() {
        for s in ALL {
            assert_eq!(s.to_string().parse::<EpicStatus>().unwrap(), s);
        }
        assert!("Backlog".parse::<EpicStatus>().is_err());
        assert!("bogus".parse::<EpicStatus>().is_err());
    }

    #[test]
    fn serde_roundtrip_and_wire_strings() {
        for s in ALL {
            let back: EpicStatus =
                serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
            assert_eq!(back, s);
        }
        assert_eq!(serde_json::to_string(&EpicStatus::Backlog).unwrap(), "\"backlog\"");
        assert_eq!(serde_json::to_string(&EpicStatus::Cancelled).unwrap(), "\"cancelled\"");
    }

    #[test]
    fn requirement_id_defaults_none_when_absent() {
        let e = Epic::new(1, "T").unwrap();
        let mut v = serde_json::to_value(&e).unwrap();
        v.as_object_mut().unwrap().remove("requirement_id");
        let back: Epic = serde_json::from_value(v).unwrap();
        assert!(back.requirement_id.is_none());
    }

    #[test]
    fn epic_serde_roundtrip_preserves_fields() {
        let mut e = Epic::new(3, "E").unwrap();
        e.description = Some("d".into());
        e.owner_id = Some(7);
        e.requirement_id = Some("EP-1".into());
        let back: Epic = serde_json::from_str(&serde_json::to_string(&e).unwrap()).unwrap();
        assert_eq!(back.project_id, 3);
        assert_eq!(back.description.as_deref(), Some("d"));
        assert_eq!(back.owner_id, Some(7));
        assert_eq!(back.requirement_id.as_deref(), Some("EP-1"));
    }

    #[test]
    fn epic_clone_and_debug() {
        let e = Epic::new(1, "T").unwrap();
        let c = e.clone();
        assert_eq!(c.title, e.title);
        assert!(format!("{e:?}").contains("Epic"));
    }
}
