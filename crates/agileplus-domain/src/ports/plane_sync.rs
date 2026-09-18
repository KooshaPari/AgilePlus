// SPDX-License-Identifier: MIT OR Apache-2.0
//! Plane.so sync port.

use crate::domain::story::{Story, StoryStatus};
use crate::error::DomainError;

/// Minimal Plane project representation used by the sync adapter.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlaneProject {
    pub id: String,
    pub name: String,
    pub identifier: String,
}

/// Minimal Plane issue representation used by the sync adapter.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlaneIssue {
    pub id: String,
    pub name: String,
    pub state: Option<String>,
    pub priority: Option<i32>,
    pub sequence_id: Option<i64>,
}

impl PlaneIssue {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        state: Option<String>,
        priority: Option<i32>,
        sequence_id: Option<i64>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            state,
            priority,
            sequence_id,
        }
    }
}

/// Hexagonal port for Plane.so synchronization.
pub trait PlaneSyncPort: Send + Sync {
    fn list_projects(&self) -> Result<Vec<PlaneProject>, DomainError>;

    fn sync_story_to_plane(
        &self,
        project_identifier: &str,
        story: &Story,
    ) -> Result<PlaneIssue, DomainError>;

    fn sync_from_plane(
        &self,
        project_id: i64,
        epic_id: i64,
        issue: &PlaneIssue,
    ) -> Result<Story, DomainError>;
}

pub fn story_status_to_plane_state(status: StoryStatus) -> &'static str {
    match status {
        StoryStatus::Todo => "todo",
        StoryStatus::InProgress => "in_progress",
        StoryStatus::Review => "review",
        StoryStatus::Done => "done",
        StoryStatus::Blocked => "blocked",
        StoryStatus::Cancelled => "cancelled",
    }
}

pub fn plane_state_to_story_status(state: &str) -> Result<StoryStatus, DomainError> {
    match state {
        "todo" => Ok(StoryStatus::Todo),
        "in_progress" => Ok(StoryStatus::InProgress),
        "review" => Ok(StoryStatus::Review),
        "done" => Ok(StoryStatus::Done),
        "blocked" => Ok(StoryStatus::Blocked),
        "cancelled" => Ok(StoryStatus::Cancelled),
        other => Err(DomainError::Validation(format!(
            "unknown Plane story state: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_STATUSES: [StoryStatus; 6] = [
        StoryStatus::Todo,
        StoryStatus::InProgress,
        StoryStatus::Review,
        StoryStatus::Done,
        StoryStatus::Blocked,
        StoryStatus::Cancelled,
    ];

    #[test]
    fn status_to_plane_state_covers_every_variant() {
        assert_eq!(story_status_to_plane_state(StoryStatus::Todo), "todo");
        assert_eq!(
            story_status_to_plane_state(StoryStatus::InProgress),
            "in_progress"
        );
        assert_eq!(story_status_to_plane_state(StoryStatus::Review), "review");
        assert_eq!(story_status_to_plane_state(StoryStatus::Done), "done");
        assert_eq!(story_status_to_plane_state(StoryStatus::Blocked), "blocked");
        assert_eq!(
            story_status_to_plane_state(StoryStatus::Cancelled),
            "cancelled"
        );
    }

    #[test]
    fn plane_state_round_trips_through_story_status() {
        for status in ALL_STATUSES {
            let wire = story_status_to_plane_state(status);
            assert_eq!(plane_state_to_story_status(wire).unwrap(), status);
        }
    }

    #[test]
    fn plane_state_parsing_is_exact_and_case_sensitive() {
        for label in ["TODO", "Todo", "in progress", "in-progress", "", " backlog"] {
            let error = plane_state_to_story_status(label).unwrap_err();
            assert!(
                matches!(
                    error,
                    DomainError::Validation(ref message)
                        if message == &format!("unknown Plane story state: {label}")
                ),
                "unexpected result for {label:?}"
            );
        }
    }

    #[test]
    fn plane_issue_new_stores_all_fields_verbatim() {
        let issue = PlaneIssue::new(
            "issue-1",
            "Fix login",
            Some("done".into()),
            Some(2),
            Some(41),
        );
        assert_eq!(issue.id, "issue-1");
        assert_eq!(issue.name, "Fix login");
        assert_eq!(issue.state.as_deref(), Some("done"));
        assert_eq!(issue.priority, Some(2));
        assert_eq!(issue.sequence_id, Some(41));

        let bare = PlaneIssue::new(
            String::from("issue-2"),
            String::from("Bare"),
            None,
            None,
            None,
        );
        assert_eq!(bare.state, None);
        assert_eq!(bare.priority, None);
        assert_eq!(bare.sequence_id, None);
    }

    #[test]
    fn plane_project_and_issue_serde_round_trip() {
        let project = PlaneProject {
            id: "p-1".into(),
            name: "AgilePlus".into(),
            identifier: "AGP".into(),
        };
        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains(r#""identifier":"AGP""#));
        assert_eq!(
            serde_json::from_str::<PlaneProject>(&json).unwrap(),
            project
        );

        let issue = PlaneIssue::new("i-1", "Story", Some("review".into()), None, Some(7));
        let json = serde_json::to_string(&issue).unwrap();
        assert_eq!(serde_json::from_str::<PlaneIssue>(&json).unwrap(), issue);
    }
}
