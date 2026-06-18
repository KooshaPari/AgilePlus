//! Intent types for service capability description

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A semantic capability that a service provides
///
/// Intents are high-level descriptions of what a service can do,
/// allowing consumers to find services by capability rather than name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    /// The primary category of capability (e.g., "vcs", "storage", "auth")
    pub category: String,

    /// Specific actions or features within the category
    pub actions: HashSet<String>,
}

impl std::hash::Hash for Capability {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.category.hash(state);
        // Sort actions for deterministic hashing
        let mut sorted: Vec<_> = self.actions.iter().collect();
        sorted.sort();
        for action in sorted {
            action.hash(state);
        }
    }
}

impl Capability {
    /// Create a new capability with the given category and actions
    pub fn new(category: impl Into<String>, actions: impl IntoIterator<Item = String>) -> Self {
        Self {
            category: category.into(),
            actions: actions.into_iter().collect(),
        }
    }

    /// Add an action to this capability
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.actions.insert(action.into());
        self
    }

    /// Check if this capability matches the given category
    pub fn matches_category(&self, category: &str) -> bool {
        self.category == category
    }

    /// Check if this capability has the given action
    pub fn has_action(&self, action: &str) -> bool {
        self.actions.contains(action)
    }
}

/// A service intent - the registration of what a service intends to provide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceIntent {
    /// The primary capability category
    pub capability: Capability,

    /// Version constraints (semver format)
    pub version_constraint: Option<String>,

    /// Priority for resolution (higher = preferred)
    #[serde(default)]
    pub priority: u32,
}

impl ServiceIntent {
    /// Create a new service intent
    pub fn new(category: impl Into<String>, actions: impl IntoIterator<Item = String>) -> Self {
        Self {
            capability: Capability::new(category, actions),
            version_constraint: None,
            priority: 0,
        }
    }

    /// Create with a specific version constraint
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version_constraint = Some(version.into());
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_creation() {
        let cap = Capability::new("vcs", vec!["git".to_string(), "commit".to_string()]);
        assert_eq!(cap.category, "vcs");
        assert!(cap.has_action("git"));
        assert!(cap.has_action("commit"));
        assert!(!cap.has_action("push"));
    }

    #[test]
    fn test_service_intent_creation() {
        let intent = ServiceIntent::new("storage", vec!["read".to_string(), "write".to_string()])
            .with_version(">=1.0.0")
            .with_priority(10);

        assert_eq!(intent.capability.category, "storage");
        assert_eq!(intent.version_constraint, Some(">=1.0.0".to_string()));
        assert_eq!(intent.priority, 10);
    }
}
