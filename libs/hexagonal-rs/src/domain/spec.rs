//! Spec entity - represents a feature or requirement

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::entity::{Entity, EntityId};

/// Spec status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpecStatus {
    #[default]
    Draft,
    Active,
    Completed,
    Archived,
}

/// Spec priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpecPriority {
    Critical,
    #[default]
    Medium,
    Low,
}

/// A specification represents a feature, requirement, or epic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    id: EntityId,
    title: String,
    description: String,
    status: SpecStatus,
    priority: SpecPriority,
    acceptance_criteria: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Spec {
    /// Create a new spec
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            title: title.into(),
            description: description.into(),
            status: SpecStatus::default(),
            priority: SpecPriority::default(),
            acceptance_criteria: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create with explicit ID (for repositories)
    pub fn with_id(id: EntityId, title: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            description,
            status: SpecStatus::default(),
            priority: SpecPriority::default(),
            acceptance_criteria: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Update title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.updated_at = Utc::now();
    }

    /// Update status
    pub fn set_status(&mut self, status: SpecStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Add acceptance criterion
    pub fn add_acceptance_criterion(&mut self, criterion: impl Into<String>) {
        self.acceptance_criteria.push(criterion.into());
        self.updated_at = Utc::now();
    }

    // Getters
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn status(&self) -> SpecStatus {
        self.status
    }
    pub fn priority(&self) -> SpecPriority {
        self.priority
    }
    pub fn acceptance_criteria(&self) -> &[String] {
        &self.acceptance_criteria
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Entity for Spec {
    fn id(&self) -> &EntityId {
        &self.id
    }

    fn entity_type(&self) -> &'static str {
        "spec"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::Entity;

    #[test]
    fn test_create_spec() {
        let spec = Spec::new("Feature A", "Description of feature A");
        assert_eq!(spec.title(), "Feature A");
        assert_eq!(spec.status(), SpecStatus::Draft);
    }

    #[test]
    fn test_spec_id() {
        let spec = Spec::new("Test", "Test desc");
        let id = spec.id();
        assert!(!id.as_str().is_empty());
    }
}
