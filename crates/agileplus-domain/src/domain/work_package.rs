//! Work package types — sub-units of features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// State of a work package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WpState {
    Planned,
    Doing,
    Review,
    Done,
    Blocked,
}

/// State of an associated pull request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrState {
    Open,
    Review,
    ChangesRequested,
    Approved,
    Merged,
}

/// How one work package depends on another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    Explicit,
    FileOverlap,
    Data,
}

/// A work package — concrete implementation unit under a Feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackage {
    pub id: i64,
    pub feature_id: i64,
    pub title: String,
    pub state: WpState,
    pub sequence: i32,
    pub file_scope: Vec<String>,
    pub acceptance_criteria: String,
    pub agent_id: Option<String>,
    pub pr_url: Option<String>,
    pub pr_state: Option<PrState>,
    pub worktree_path: Option<String>,
    pub plane_sub_issue_id: Option<String>,
    pub base_commit: Option<String>,
    pub head_commit: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Directed dependency between two work packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpDependency {
    pub wp_id: i64,
    pub depends_on: i64,
    pub dep_type: DependencyType,
}

impl WpState {
    /// Returns `true` if a transition from `self` to `target` is allowed.
    pub fn can_transition_to(self, target: WpState) -> bool {
        use WpState::*;
        matches!(
            (self, target),
            (Planned, Doing)
                | (Doing, Review)
                | (Doing, Blocked)
                | (Blocked, Doing)
                | (Review, Done)
                | (Review, Doing)
        )
    }
}

impl WorkPackage {
    /// Construct a new WorkPackage with sensible defaults.
    pub fn new(feature_id: i64, title: &str, sequence: i32, acceptance_criteria: &str) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: 0,
            feature_id,
            title: title.to_string(),
            state: WpState::Planned,
            sequence,
            file_scope: Vec::new(),
            acceptance_criteria: acceptance_criteria.to_string(),
            agent_id: None,
            pr_url: None,
            pr_state: None,
            worktree_path: None,
            plane_sub_issue_id: None,
            base_commit: None,
            head_commit: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_work_package_starts_planned_with_empty_scope() {
        let wp = WorkPackage::new(42, "Implement login", 3, "User can sign in");

        assert_eq!(wp.feature_id, 42);
        assert_eq!(wp.title, "Implement login");
        assert_eq!(wp.state, WpState::Planned);
        assert_eq!(wp.sequence, 3);
        assert!(wp.file_scope.is_empty());
        assert_eq!(wp.acceptance_criteria, "User can sign in");
    }

    #[test]
    fn work_package_state_machine_rejects_skipped_lifecycle_steps() {
        assert!(!WpState::Planned.can_transition_to(WpState::Done));
        assert!(!WpState::Review.can_transition_to(WpState::Blocked));
        assert!(!WpState::Done.can_transition_to(WpState::Doing));
    }

    #[test]
    fn work_package_state_machine_allows_blocked_recovery_path() {
        assert!(WpState::Doing.can_transition_to(WpState::Blocked));
        assert!(WpState::Blocked.can_transition_to(WpState::Doing));
    }
}
