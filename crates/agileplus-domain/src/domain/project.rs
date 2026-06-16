//! Project aggregate — top-level organisational unit.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use agileplus_validate::{name_required, slug_format};

use crate::{error::DomainError, DomainResult};

/// A project that owns modules, cycles, and features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    /// URL-safe slug — must be non-empty and contain only `[a-z0-9-]`.
    pub slug: String,
    /// Human-readable name — must be non-empty.
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    /// Construct a new `Project`. `name` must be non-empty; `slug` must be
    /// non-empty and consist only of lowercase ASCII alphanumerics and hyphens.
    pub fn new(name: &str, slug: &str) -> DomainResult<Self> {
        let name = name.trim();
        name_required(name)
            .map_err(|message| DomainError::Validation(format!("project {message}")))?;
        let slug = slug.trim();
        slug_format(slug)
            .map_err(|message| DomainError::Validation(format!("project {message}")))?;
        let now = Utc::now();
        Ok(Self {
            id: 0,
            slug: slug.to_string(),
            name: name.to_string(),
            description: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Derive a slug from a human-readable name (same algorithm as `Module`).
    pub fn slug_from_name(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_project_construction() {
        let p = Project::new("My Project", "my-project").expect("domain operation");
        assert_eq!(p.name, "My Project");
        assert_eq!(p.slug, "my-project");
    }

    #[test]
    fn rejects_empty_name() {
        let err = Project::new("  ", "my-project").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn rejects_invalid_slug() {
        let err = Project::new("My Project", "My Project").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn slug_from_name_helper() {
        assert_eq!(Project::slug_from_name("Hello World!"), "hello-world");
    }
}
