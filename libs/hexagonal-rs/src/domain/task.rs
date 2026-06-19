//! Task entity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::entity::{Entity, EntityId};

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    #[default]
    Todo,
    InProgress,
    Done,
    Blocked,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Critical,
    High,
    #[default]
    Medium,
    Low,
}

/// A task is the smallest work unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    id: EntityId,
    work_package_id: EntityId,
    title: String,
    description: String,
    status: TaskStatus,
    priority: TaskPriority,
    estimated_hours: Option<f32>,
    actual_hours: Option<f32>,
    assignee: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Task {
    /// Create a new task
    pub fn new(work_package_id: EntityId, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            work_package_id,
            title: title.into(),
            description: String::new(),
            status: TaskStatus::default(),
            priority: TaskPriority::default(),
            estimated_hours: None,
            actual_hours: None,
            assignee: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn status(&self) -> TaskStatus {
        self.status
    }
    pub fn priority(&self) -> TaskPriority {
        self.priority
    }
    pub fn estimated_hours(&self) -> Option<f32> {
        self.estimated_hours
    }
    pub fn actual_hours(&self) -> Option<f32> {
        self.actual_hours
    }
    pub fn assignee(&self) -> Option<&str> {
        self.assignee.as_deref()
    }
    pub fn work_package_id(&self) -> &EntityId {
        &self.work_package_id
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn set_assignee(&mut self, assignee: Option<String>) {
        self.assignee = assignee;
        self.updated_at = Utc::now();
    }
}

impl Entity for Task {
    fn id(&self) -> &EntityId {
        &self.id
    }

    fn entity_type(&self) -> &'static str {
        "task"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task() {
        let wp_id = EntityId::new();
        let task = Task::new(wp_id.clone(), "Implement feature");
        assert_eq!(task.title(), "Implement feature");
        assert_eq!(task.status(), TaskStatus::Todo);
    }
}
