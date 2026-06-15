//! Module aggregate — hierarchical grouping of features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use agileplus_validate::slug_format;

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
        let slug = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        debug_assert!(slug_format(&slug).is_ok() || slug.is_empty());
        slug
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_new_derives_slug() {
        let m = Module::new("My Feature Area", None);
        assert_eq!(m.slug, "my-feature-area");
        assert_eq!(m.friendly_name, "My Feature Area");
        assert!(m.parent_module_id.is_none());
        assert_eq!(m.id, 0);
    }

    #[test]
    fn module_new_with_parent() {
        let m = Module::new("Sub Module", Some(42));
        assert_eq!(m.parent_module_id, Some(42));
    }

    #[test]
    fn slug_from_name_lowercases_and_hyphenates() {
        assert_eq!(Module::slug_from_name("Hello World"), "hello-world");
        assert_eq!(Module::slug_from_name("  spaces  "), "spaces");
        assert_eq!(Module::slug_from_name("multi---dashes"), "multi-dashes");
    }

    #[test]
    fn slug_from_name_strips_special_chars() {
        assert_eq!(Module::slug_from_name("Foo/Bar"), "foo-bar");
        assert_eq!(Module::slug_from_name("a@b#c"), "a-b-c");
    }

    #[test]
    fn module_feature_tag_new() {
        let tag = ModuleFeatureTag::new(10, 20);
        assert_eq!(tag.module_id, 10);
        assert_eq!(tag.feature_id, 20);
    }
}
