//! Module domain entity and related types.
//!
//! Traces to: FR-M01, FR-M02, FR-M04, FR-M07

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Module groups Features into logical product areas and supports hierarchical organisation.
///
/// Traces to: FR-M01, FR-M02
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: i64,
    pub slug: String,
    pub friendly_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_module_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Module {
    /// Create a new Module with `id = 0` (populated by storage layer on insert).
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

    /// Derive a kebab-case slug from a display name using the same logic as `Feature::slug_from_name`.
    pub fn slug_from_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Update the module's display name, re-derive the slug, and touch `updated_at`.
    pub fn update_name(&mut self, new_name: &str) {
        self.friendly_name = new_name.to_string();
        self.slug = Self::slug_from_name(new_name);
        self.updated_at = Utc::now();
    }
}

/// Many-to-many tagging join between a Module and a Feature.
///
/// Traces to: FR-M04
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleFeatureTag {
    pub module_id: i64,
    pub feature_id: i64,
    pub created_at: DateTime<Utc>,
}

impl ModuleFeatureTag {
    pub fn new(module_id: i64, feature_id: i64) -> Self {
        Self {
            module_id,
            feature_id,
            created_at: Utc::now(),
        }
    }
}

/// View struct carrying a Module together with its associated features and children.
/// Populated by the storage/query layer in WP02.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleWithFeatures {
    pub module: Module,
    /// Features where `Feature.module_id == this module's id` (strict ownership).
    pub owned_features: Vec<crate::domain::feature::Feature>,
    /// Features linked via `module_feature_tags` (many-to-many tagging).
    pub tagged_features: Vec<crate::domain::feature::Feature>,
    /// Direct child modules (non-recursive).
    pub child_modules: Vec<Module>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_module_defaults() {
        let m = Module::new("My Module", None);
        assert_eq!(m.id, 0);
        assert_eq!(m.slug, "my-module");
        assert!(m.parent_module_id.is_none());
        assert!(m.description.is_none());
    }

    #[test]
    fn new_module_with_parent() {
        let m = Module::new("Child", Some(42));
        assert_eq!(m.parent_module_id, Some(42));
    }

    #[test]
    fn slug_derivation() {
        assert_eq!(Module::slug_from_name("OAuth Providers"), "oauth-providers");
        assert_eq!(Module::slug_from_name("Hello World"), "hello-world");
        assert_eq!(Module::slug_from_name("  Foo  Bar  "), "foo-bar");
        assert_eq!(Module::slug_from_name("a--b"), "a-b");
    }

    #[test]
    fn update_name_re_slugs() {
        let mut m = Module::new("Old Name", None);
        assert_eq!(m.slug, "old-name");
        let before = m.updated_at;
        // Ensure at least 1ns difference on fast systems.
        std::thread::sleep(std::time::Duration::from_millis(1));
        m.update_name("New Name");
        assert_eq!(m.friendly_name, "New Name");
        assert_eq!(m.slug, "new-name");
        assert!(m.updated_at >= before);
    }

    #[test]
    fn tag_new_stamps_created_at() {
        let before = Utc::now();
        let tag = ModuleFeatureTag::new(1, 2);
        let after = Utc::now();
        assert_eq!(tag.module_id, 1);
        assert_eq!(tag.feature_id, 2);
        assert!(tag.created_at >= before);
        assert!(tag.created_at <= after);
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn slug_from_name_table() {
        for (input, expected) in [
            ("OAuth Providers", "oauth-providers"),
            ("Hello World", "hello-world"),
            ("  Foo  Bar  ", "foo-bar"),
            ("a--b", "a-b"),
            ("", ""),
            ("---", ""),
            ("CamelCase", "camelcase"),
            ("v2 API", "v2-api"),
            ("unicode 功能", "unicode-功能"),
        ] {
            assert_eq!(Module::slug_from_name(input), expected, "input {input:?}");
        }
    }

    #[test]
    fn new_module_defaults_and_serde() {
        let m = Module::new("Payments", None);
        assert_eq!(m.id, 0);
        assert_eq!(m.slug, "payments");
        assert_eq!(m.friendly_name, "Payments");
        assert!(m.description.is_none());
        assert!(m.parent_module_id.is_none());
        assert_eq!(m.created_at, m.updated_at);

        let back: Module = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
        assert_eq!(back.slug, "payments");
        assert_eq!(back.id, 0);
    }

    #[test]
    fn serde_skips_none_optional_fields() {
        let m = Module::new("M", None);
        let v = serde_json::to_value(&m).unwrap();
        assert!(v.get("description").is_none());
        assert!(v.get("parent_module_id").is_none());
    }

    #[test]
    fn serde_includes_optional_fields_when_set() {
        let mut m = Module::new("M", Some(3));
        m.description = Some("desc".into());
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(v["description"], "desc");
        assert_eq!(v["parent_module_id"], 3);
    }

    #[test]
    fn update_name_re_slugs_and_preserves_other_fields() {
        let mut m = Module::new("Old Name", Some(9));
        m.description = Some("d".into());
        m.update_name("Brand New!");
        assert_eq!(m.friendly_name, "Brand New!");
        assert_eq!(m.slug, "brand-new");
        assert_eq!(m.parent_module_id, Some(9));
        assert_eq!(m.description.as_deref(), Some("d"));
    }

    #[test]
    fn module_feature_tag_new_and_serde() {
        let tag = ModuleFeatureTag::new(1, 2);
        assert_eq!(tag.module_id, 1);
        assert_eq!(tag.feature_id, 2);
        let back: ModuleFeatureTag =
            serde_json::from_str(&serde_json::to_string(&tag).unwrap()).unwrap();
        assert_eq!(back.module_id, 1);
        assert_eq!(back.feature_id, 2);
    }

    #[test]
    fn module_with_features_construction_and_serde() {
        let m = Module::new("Core", None);
        let feat = crate::domain::feature::Feature::new("f", "F", [1u8; 32], None);
        let child = Module::new("Child", Some(1));
        let view = ModuleWithFeatures {
            module: m,
            owned_features: vec![feat.clone()],
            tagged_features: vec![feat],
            child_modules: vec![child],
        };
        assert_eq!(view.owned_features.len(), 1);
        assert_eq!(view.tagged_features.len(), 1);
        assert_eq!(view.child_modules.len(), 1);
        let back: ModuleWithFeatures =
            serde_json::from_str(&serde_json::to_string(&view).unwrap()).unwrap();
        assert_eq!(back.child_modules.len(), 1);
    }

    #[test]
    fn module_clone_and_debug() {
        let m = Module::new("X", None);
        let c = m.clone();
        assert_eq!(c.slug, m.slug);
        assert!(format!("{m:?}").contains("Module"));
    }
}
