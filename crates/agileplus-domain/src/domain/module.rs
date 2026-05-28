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
