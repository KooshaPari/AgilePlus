//! Module aggregate — hierarchical grouping of features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::feature::Feature;

/// A module groups features into a logical scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: i64,
    pub slug: String,
    pub friendly_name: String,
    pub description: Option<String>,
    pub parent_module_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Module {
    /// Construct a new module with a derived slug and default timestamps.
    pub fn new(friendly_name: &str, parent_module_id: Option<i64>) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            slug: Self::slug_from_name(friendly_name),
            friendly_name: friendly_name.to_string(),
            description: None,
            parent_module_id,
            created_at: now,
            updated_at: now,
        }
    }

    /// Derive a URL-safe slug from a human-readable name.
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

/// A module together with its owned features, tagged features, and child modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleWithFeatures {
    pub module: Module,
    pub owned_features: Vec<Feature>,
    pub tagged_features: Vec<Feature>,
    pub child_modules: Vec<Module>,
}

/// Association record linking a feature to a module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleFeatureTag {
    pub module_id: i64,
    pub feature_id: i64,
}

impl ModuleFeatureTag {
    pub fn new(module_id: i64, feature_id: i64) -> Self {
        Self {
            module_id,
            feature_id,
        }
    }
}
