//! Backlog queue types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The intent/category of a backlog item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Intent {
    Bug,
    Feature,
    Idea,
    Task,
    /// Documentation work item (low priority by default).
    Docs,
}

impl fmt::Display for Intent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Intent::Bug => "bug",
            Intent::Feature => "feature",
            Intent::Idea => "idea",
            Intent::Task => "task",
            Intent::Docs => "docs",
        };
        write!(f, "{s}")
    }
}

impl FromStr for Intent {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bug" => Ok(Intent::Bug),
            "feature" => Ok(Intent::Feature),
            "idea" => Ok(Intent::Idea),
            "task" => Ok(Intent::Task),
            "docs" => Ok(Intent::Docs),
            _ => Err(format!("unknown Intent: {s}")),
        }
    }
}

impl Intent {
    /// Returns the default `BacklogPriority` for this intent category.
    pub fn default_priority(self) -> BacklogPriority {
        match self {
            Intent::Bug => BacklogPriority::High,
            Intent::Feature => BacklogPriority::Medium,
            Intent::Task => BacklogPriority::Medium,
            Intent::Idea => BacklogPriority::Low,
            Intent::Docs => BacklogPriority::Low,
        }
    }
}

/// Priority of a backlog item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BacklogPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl fmt::Display for BacklogPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BacklogPriority::Critical => "critical",
            BacklogPriority::High => "high",
            BacklogPriority::Medium => "medium",
            BacklogPriority::Low => "low",
        };
        write!(f, "{s}")
    }
}

impl FromStr for BacklogPriority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "critical" => Ok(BacklogPriority::Critical),
            "high" => Ok(BacklogPriority::High),
            "medium" => Ok(BacklogPriority::Medium),
            "low" => Ok(BacklogPriority::Low),
            _ => Err(format!("unknown BacklogPriority: {s}")),
        }
    }
}

/// Workflow status of a backlog item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BacklogStatus {
    New,
    Triaged,
    InProgress,
    Done,
    Dismissed,
}

impl fmt::Display for BacklogStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BacklogStatus::New => "new",
            BacklogStatus::Triaged => "triaged",
            BacklogStatus::InProgress => "in_progress",
            BacklogStatus::Done => "done",
            BacklogStatus::Dismissed => "dismissed",
        };
        write!(f, "{s}")
    }
}

impl FromStr for BacklogStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "new" => Ok(BacklogStatus::New),
            "triaged" => Ok(BacklogStatus::Triaged),
            "in_progress" => Ok(BacklogStatus::InProgress),
            "done" => Ok(BacklogStatus::Done),
            "dismissed" => Ok(BacklogStatus::Dismissed),
            _ => Err(format!("unknown BacklogStatus: {s}")),
        }
    }
}

/// Sort order for backlog queries.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BacklogSort {
    #[default]
    Age,
    Priority,
    Impact,
}

/// A single backlog item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklogItem {
    pub id: Option<i64>,
    pub title: String,
    pub description: String,
    pub intent: Intent,
    pub priority: BacklogPriority,
    pub status: BacklogStatus,
    pub source: String,
    pub feature_slug: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BacklogItem {
    /// Construct a new `BacklogItem` from a triage classification result.
    ///
    /// Priority defaults to `intent.default_priority()`.
    /// Status is set to `BacklogStatus::New`.
    pub fn from_triage(title: String, description: String, intent: Intent, source: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            title,
            description,
            priority: intent.default_priority(),
            intent,
            status: BacklogStatus::New,
            source,
            feature_slug: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Filter parameters for backlog list queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacklogFilters {
    pub intent: Option<Intent>,
    pub status: Option<BacklogStatus>,
    pub priority: Option<BacklogPriority>,
    pub feature_slug: Option<String>,
    pub source: Option<String>,
    pub sort: BacklogSort,
    pub limit: Option<usize>,
}
