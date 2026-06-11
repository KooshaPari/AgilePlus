//! Feature aggregate — the central planning unit.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::state_machine::FeatureState;

/// A software feature tracked through the planning lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: i64,
    pub slug: String,
    pub friendly_name: String,
    pub state: FeatureState,
    pub spec_hash: [u8; 32],
    pub target_branch: String,
    pub plane_issue_id: Option<String>,
    pub plane_state_id: Option<String>,
    pub labels: Vec<String>,
    pub module_id: Option<i64>,
    pub project_id: Option<i64>,
    pub created_at_commit: Option<String>,
    pub last_modified_commit: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Feature {
    /// Attempt a state transition. Returns an error string if the transition is not allowed.
    pub fn transition(&mut self, target: FeatureState) -> Result<(), String> {
        use FeatureState::*;
        let allowed = match self.state {
            Created => matches!(target, Specified),
            Specified => matches!(target, Researched),
            Researched => matches!(target, Planned),
            Planned => matches!(target, Implementing),
            Implementing => matches!(target, Validated),
            Validated => matches!(target, Shipped),
            Shipped => matches!(target, Retrospected),
            Retrospected => false,
        };
        if allowed {
            self.state = target;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!(
                "invalid transition {:?} -> {:?}",
                self.state, target
            ))
        }
    }

    /// Construct a new Feature with sensible defaults.
    pub fn new(
        slug: &str,
        friendly_name: &str,
        spec_hash: [u8; 32],
        target_branch: Option<&str>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            slug: slug.to_string(),
            friendly_name: friendly_name.to_string(),
            state: FeatureState::Created,
            spec_hash,
            target_branch: target_branch.unwrap_or("main").to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: Vec::new(),
            module_id: None,
            project_id: None,
            created_at_commit: None,
            last_modified_commit: None,
            created_at: now,
            updated_at: now,
        }
    }
}
