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
