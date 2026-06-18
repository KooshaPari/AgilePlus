use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{error::DomainError, DomainResult};

/// A project that owns modules, cycles, and features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    /// Construct a new `Project`. `name` must be non-empty; `slug` must be
    /// non-empty and consist only of lowercase ASCII alphanumerics and hyphens.
    pub fn new(name: &str, slug: &str) -> DomainResult<Self> {
        let name = name.trim();
        if name.is_empty() {
            return Err(DomainError::Validation(
                "project name must not be empty".to_string(),
            ));
        }
        let slug = slug.trim();
        if slug.is_empty() {
            return Err(DomainError::Validation(
                "project slug must not be empty".to_string(),
            ));
        }
        if !slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(DomainError::Validation(
                "project slug must contain only lowercase letters, digits, and hyphens".to_string(),
            ));
        }
        let now = Utc::now();
        Self {
            id: 0,
            slug: slug.into(),
            name: name.into(),
            description: description.into(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a project with a specific ID pre-assigned.
    pub fn with_id(id: i64, slug: &str, name: &str, description: &str) -> Self {
        let mut p = Self::new(slug, name, description);
        p.id = id;
        p
    }
}
