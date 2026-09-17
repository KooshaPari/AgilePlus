//! Story aggregate — a user-facing deliverable belonging to an Epic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::DomainError;

/// Lifecycle status of a story.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoryStatus {
    Todo,
    InProgress,
    Review,
    Done,
    Blocked,
    Cancelled,
}

impl fmt::Display for StoryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            StoryStatus::Todo => "todo",
            StoryStatus::InProgress => "in_progress",
            StoryStatus::Review => "review",
            StoryStatus::Done => "done",
            StoryStatus::Blocked => "blocked",
            StoryStatus::Cancelled => "cancelled",
        };
        write!(f, "{s}")
    }
}

impl FromStr for StoryStatus {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "todo" => Ok(StoryStatus::Todo),
            "in_progress" => Ok(StoryStatus::InProgress),
            "review" => Ok(StoryStatus::Review),
            "done" => Ok(StoryStatus::Done),
            "blocked" => Ok(StoryStatus::Blocked),
            "cancelled" => Ok(StoryStatus::Cancelled),
            _ => Err(DomainError::Validation(format!("unknown StoryStatus: {s}"))),
        }
    }
}

impl StoryStatus {
    /// Returns `true` if a transition from `self` to `target` is allowed.
    pub fn can_transition_to(self, target: StoryStatus) -> bool {
        use StoryStatus::*;
        matches!(
            (self, target),
            (Todo, InProgress)
                | (Todo, Cancelled)
                | (InProgress, Review)
                | (InProgress, Blocked)
                | (InProgress, Cancelled)
                | (Blocked, InProgress)
                | (Review, Done)
                | (Review, InProgress)
        )
    }
}

/// A story — a user-facing unit of work owned by an Epic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    pub id: i64,
    /// Owning epic.
    pub epic_id: i64,
    /// Owning project (denormalised for queries).
    pub project_id: i64,
    /// Title — must be non-empty.
    pub title: String,
    /// "As a <role>, I want <goal> so that <benefit>." Free-form narrative.
    pub description: Option<String>,
    pub status: StoryStatus,
    /// Story-point estimate — must be positive when set.
    pub points: Option<u32>,
    /// Assignee (user id).
    pub assignee_id: Option<i64>,
    /// FR/requirement id this story satisfies — the traceability key used by
    /// `upsert_by_requirement_id`. `None` for ad-hoc stories.
    #[serde(default)]
    pub requirement_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Story {
    /// Construct a new `Story`. `title` must be non-empty; `points` must be > 0 when given.
    pub fn new(
        epic_id: i64,
        project_id: i64,
        title: &str,
        points: Option<u32>,
    ) -> Result<Self, DomainError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(DomainError::Validation(
                "story title must not be empty".to_string(),
            ));
        }
        if points == Some(0) {
            return Err(DomainError::Validation(
                "story points must be greater than zero".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: 0,
            epic_id,
            project_id,
            title: title.to_string(),
            description: None,
            status: StoryStatus::Todo,
            points,
            assignee_id: None,
            requirement_id: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Attempt a status transition. Returns `Err(DomainError::InvalidTransition)`
    /// if the transition is not permitted.
    pub fn transition_status(&mut self, target: StoryStatus) -> Result<(), DomainError> {
        if !self.status.can_transition_to(target) {
            return Err(DomainError::InvalidTransition {
                from: self.status.to_string(),
                to: target.to_string(),
                reason: "not an allowed story status transition".to_string(),
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
    fn valid_story_construction() {
        let s = Story::new(1, 10, "User can log in", Some(3)).unwrap();
        assert_eq!(s.title, "User can log in");
        assert_eq!(s.epic_id, 1);
        assert_eq!(s.project_id, 10);
        assert_eq!(s.points, Some(3));
        assert_eq!(s.status, StoryStatus::Todo);
    }

    #[test]
    fn rejects_empty_title() {
        let err = Story::new(1, 10, "", None).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn rejects_zero_points() {
        let err = Story::new(1, 10, "Has zero points", Some(0)).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn valid_status_transition() {
        let mut s = Story::new(1, 10, "Login flow", None).unwrap();
        s.transition_status(StoryStatus::InProgress).unwrap();
        assert_eq!(s.status, StoryStatus::InProgress);
        s.transition_status(StoryStatus::Review).unwrap();
        assert_eq!(s.status, StoryStatus::Review);
        s.transition_status(StoryStatus::Done).unwrap();
        assert_eq!(s.status, StoryStatus::Done);
    }

    #[test]
    fn invalid_status_transition_rejected() {
        let mut s = Story::new(1, 10, "Skip ahead", None).unwrap();
        // Todo -> Done is not allowed
        let err = s.transition_status(StoryStatus::Done).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition { .. }));
    }

    #[test]
    fn blocked_unblocked_cycle() {
        let mut s = Story::new(2, 20, "Blocked story", Some(5)).unwrap();
        s.transition_status(StoryStatus::InProgress).unwrap();
        s.transition_status(StoryStatus::Blocked).unwrap();
        s.transition_status(StoryStatus::InProgress).unwrap();
        assert_eq!(s.status, StoryStatus::InProgress);
    }

    // --- Additional coverage ---

    #[test]
    fn story_status_serde_roundtrip() {
        for s in [
            StoryStatus::Todo,
            StoryStatus::InProgress,
            StoryStatus::Review,
            StoryStatus::Done,
            StoryStatus::Blocked,
            StoryStatus::Cancelled,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: StoryStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, s);
        }
    }

    #[test]
    fn story_serde_roundtrip() {
        let mut s = Story::new(1, 2, "Test", Some(3)).unwrap();
        s.description = Some("A test story".to_string());
        s.requirement_id = Some("FR-001".to_string());
        let json = serde_json::to_string(&s).unwrap();
        let back: Story = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "Test");
        assert_eq!(back.points, Some(3));
        assert_eq!(back.requirement_id, Some("FR-001".to_string()));
    }

    #[test]
    fn story_status_display_all() {
        assert_eq!(StoryStatus::Todo.to_string(), "todo");
        assert_eq!(StoryStatus::InProgress.to_string(), "in_progress");
        assert_eq!(StoryStatus::Review.to_string(), "review");
        assert_eq!(StoryStatus::Done.to_string(), "done");
        assert_eq!(StoryStatus::Blocked.to_string(), "blocked");
        assert_eq!(StoryStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn story_status_from_str_invalid() {
        assert!("unknown".parse::<StoryStatus>().is_err());
    }

    #[test]
    fn story_new_no_points() {
        let s = Story::new(1, 2, "Title", None).unwrap();
        assert!(s.points.is_none());
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    const ALL: [StoryStatus; 6] = [
        StoryStatus::Todo,
        StoryStatus::InProgress,
        StoryStatus::Review,
        StoryStatus::Done,
        StoryStatus::Blocked,
        StoryStatus::Cancelled,
    ];

    fn allowed(a: StoryStatus, b: StoryStatus) -> bool {
        matches!(
            (a, b),
            (StoryStatus::Todo, StoryStatus::InProgress)
                | (StoryStatus::Todo, StoryStatus::Cancelled)
                | (StoryStatus::InProgress, StoryStatus::Review)
                | (StoryStatus::InProgress, StoryStatus::Blocked)
                | (StoryStatus::InProgress, StoryStatus::Cancelled)
                | (StoryStatus::Blocked, StoryStatus::InProgress)
                | (StoryStatus::Review, StoryStatus::Done)
                | (StoryStatus::Review, StoryStatus::InProgress)
        )
    }

    #[test]
    fn can_transition_to_full_matrix() {
        for from in ALL {
            for to in ALL {
                assert_eq!(
                    from.can_transition_to(to),
                    allowed(from, to),
                    "{from:?} -> {to:?}"
                );
            }
        }
    }

    #[test]
    fn transition_status_matrix_matches_can_transition() {
        for from in ALL {
            for to in ALL {
                let mut s = Story::new(1, 1, "T", None).unwrap();
                s.status = from;
                let before = s.updated_at;
                let r = s.transition_status(to);
                if allowed(from, to) {
                    assert!(r.is_ok(), "{from:?} -> {to:?}");
                    assert_eq!(s.status, to);
                } else {
                    assert!(r.is_err(), "{from:?} -> {to:?}");
                    assert_eq!(s.status, from);
                    assert_eq!(s.updated_at, before);
                }
            }
        }
    }

    #[test]
    fn invalid_transition_error_fields() {
        let mut s = Story::new(1, 1, "T", None).unwrap();
        let err = s.transition_status(StoryStatus::Done).unwrap_err();
        match err {
            DomainError::InvalidTransition { from, to, reason } => {
                assert_eq!(from, "todo");
                assert_eq!(to, "done");
                assert!(!reason.is_empty());
            }
            other => panic!("wrong error: {other:?}"),
        }
    }

    #[test]
    fn self_transition_rejected_for_all_states() {
        for st in ALL {
            assert!(!st.can_transition_to(st), "{st:?} self");
        }
    }

    #[test]
    fn new_trims_title_and_rejects_whitespace_only() {
        let s = Story::new(1, 2, "  Trim me  ", None).unwrap();
        assert_eq!(s.title, "Trim me");
        assert!(Story::new(1, 2, "   \t\n", None).is_err());
    }

    #[test]
    fn new_rejects_zero_points_but_allows_none_and_positive() {
        assert!(Story::new(1, 1, "T", Some(0)).is_err());
        assert_eq!(Story::new(1, 1, "T", Some(1)).unwrap().points, Some(1));
        assert_eq!(Story::new(1, 1, "T", Some(u32::MAX)).unwrap().points, Some(u32::MAX));
        assert!(Story::new(1, 1, "T", None).unwrap().points.is_none());
    }

    #[test]
    fn new_sets_todo_status_and_zeroed_optionals() {
        let s = Story::new(5, 6, "T", Some(2)).unwrap();
        assert_eq!(s.id, 0);
        assert_eq!(s.epic_id, 5);
        assert_eq!(s.project_id, 6);
        assert_eq!(s.status, StoryStatus::Todo);
        assert!(s.description.is_none());
        assert!(s.assignee_id.is_none());
        assert!(s.requirement_id.is_none());
        assert_eq!(s.created_at, s.updated_at);
    }

    #[test]
    fn display_round_trips_for_all_statuses() {
        for st in ALL {
            let s = st.to_string();
            assert_eq!(s.parse::<StoryStatus>().unwrap(), st);
        }
    }

    #[test]
    fn from_str_rejects_unknown_and_is_case_sensitive() {
        assert!("unknown".parse::<StoryStatus>().is_err());
        assert!("TODO".parse::<StoryStatus>().is_err());
        assert!("".parse::<StoryStatus>().is_err());
    }

    #[test]
    fn from_str_error_is_validation() {
        match "nope".parse::<StoryStatus>() {
            Err(DomainError::Validation(msg)) => assert!(msg.contains("nope")),
            other => panic!("wrong: {other:?}"),
        }
    }

    #[test]
    fn serde_roundtrip_all_statuses() {
        for st in ALL {
            let back: StoryStatus =
                serde_json::from_str(&serde_json::to_string(&st).unwrap()).unwrap();
            assert_eq!(back, st);
        }
    }

    #[test]
    fn requirement_id_defaults_to_none_when_absent_in_json() {
        let s = Story::new(1, 1, "T", None).unwrap();
        let mut v = serde_json::to_value(&s).unwrap();
        v.as_object_mut().unwrap().remove("requirement_id");
        let back: Story = serde_json::from_value(v).unwrap();
        assert!(back.requirement_id.is_none());
    }

    #[test]
    fn story_clone_and_debug() {
        let s = Story::new(1, 1, "T", Some(1)).unwrap();
        let c = s.clone();
        assert_eq!(c.title, s.title);
        assert!(format!("{s:?}").contains("Story"));
    }
}
