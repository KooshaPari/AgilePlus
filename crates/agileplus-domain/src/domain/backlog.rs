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
}

impl fmt::Display for Intent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Intent::Bug => "bug",
            Intent::Feature => "feature",
            Intent::Idea => "idea",
            Intent::Task => "task",
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
            _ => Err(format!("unknown Intent: {s}")),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BacklogSort {
    Age,
    Priority,
    Impact,
}

impl Default for BacklogSort {
    fn default() -> Self {
        BacklogSort::Age
    }
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
