//! WorkPackage entity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::entity::{Entity, EntityId};

/// A work package groups related tasks under a spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackage {
    id: EntityId,
    spec_id: EntityId,
    title: String,
    description: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl WorkPackage {
    /// Create a new work package linked to a spec
    pub fn new(
        spec_id: EntityId,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: EntityId::new(),
            spec_id,
            title: title.into(),
            description: description.into(),
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
    pub fn spec_id(&self) -> &EntityId {
        &self.spec_id
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Entity for WorkPackage {
    fn id(&self) -> &EntityId {
        &self.id
    }

    fn entity_type(&self) -> &'static str {
        "work_package"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Entity, EntityId};

    #[test]
    fn test_create_work_package() {
        let spec_id = EntityId::from_string("spec-1");
        let wp = WorkPackage::new(spec_id.clone(), "Implementation", "Description");
        assert_eq!(wp.title(), "Implementation");
        assert_eq!(wp.spec_id().as_str(), "spec-1");
        assert_eq!(wp.entity_type(), "work_package");
    }
}
