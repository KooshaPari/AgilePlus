use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPolicy {
    /// The main integration branch (usually "main")
    pub main_branch: String,
    /// Release branches pattern
    pub release_pattern: String,
    /// Feature branch prefix
    pub feature_prefix: String,
    /// WP branch format: feature/{slug}/WP{id}
    pub wp_branch_format: String,
    /// Whether direct commits to main are blocked
    pub block_direct_main_commits: bool,
}

impl Default for BranchPolicy {
    fn default() -> Self {
        Self {
            main_branch: "main".to_string(),
            release_pattern: "release/*".to_string(),
            feature_prefix: "feature/".to_string(),
            wp_branch_format: "feature/{slug}/WP{id}".to_string(),
            block_direct_main_commits: true,
        }
    }
}

#[derive(Debug)]
pub enum TopologyError {
    DirectMainCommit,
    InvalidBranchName { name: String, reason: String },
    PolicyViolation { message: String },
}

impl std::fmt::Display for TopologyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopologyError::DirectMainCommit => write!(f, "Direct commits to main are blocked"),
            TopologyError::InvalidBranchName { name, reason } => {
                write!(f, "Invalid branch name '{}': {}", name, reason)
            }
            TopologyError::PolicyViolation { message } => {
                write!(f, "Policy violation: {}", message)
            }
        }
    }
}

impl std::error::Error for TopologyError {}

pub struct BranchTopology {
    policy: BranchPolicy,
}

impl BranchTopology {
    pub fn new(policy: BranchPolicy) -> Self {
        Self { policy }
    }

    pub fn from_config_file(path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = std::fs::read_to_string(path)?;
        let policy: BranchPolicy = toml::from_str(&content)?;
        Ok(Self::new(policy))
    }

    /// Generate the expected branch name for a feature
    pub fn feature_branch(&self, slug: &str) -> String {
        format!("{}{}", self.policy.feature_prefix, slug)
    }

    /// Generate the expected branch name for a WP within a feature
    pub fn wp_branch(&self, slug: &str, wp_id: &str) -> String {
        self.policy
            .wp_branch_format
            .replace("{slug}", slug)
            .replace("{id}", wp_id)
    }

    /// Validate that a branch name conforms to policy
    pub fn validate_branch(&self, branch: &str) -> Result<(), TopologyError> {
        if branch == self.policy.main_branch {
            return Ok(()); // main itself is valid
        }

        if branch.starts_with(&self.policy.feature_prefix) {
            return Ok(());
        }

        // Check release pattern (simple glob)
        let release_prefix = self.policy.release_pattern.trim_end_matches('*');
        if branch.starts_with(release_prefix) {
            return Ok(());
        }

        Err(TopologyError::InvalidBranchName {
            name: branch.to_string(),
            reason: format!(
                "Branch must start with '{}' for features, '{}' for releases, or be '{}'",
                self.policy.feature_prefix, release_prefix, self.policy.main_branch
            ),
        })
    }

    /// Check if a commit to a branch is allowed
    pub fn check_commit(&self, branch: &str) -> Result<(), TopologyError> {
        if branch == self.policy.main_branch && self.policy.block_direct_main_commits {
            return Err(TopologyError::DirectMainCommit);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_topology() -> BranchTopology {
        BranchTopology::new(BranchPolicy::default())
    }

    #[test]
    fn feature_branch_format() {
        let t = default_topology();
        assert_eq!(t.feature_branch("my-feature"), "feature/my-feature");
    }

    #[test]
    fn wp_branch_format() {
        let t = default_topology();
        assert_eq!(t.wp_branch("my-feature", "01"), "feature/my-feature/WP01");
    }

    #[test]
    fn validate_main_branch() {
        let t = default_topology();
        assert!(t.validate_branch("main").is_ok());
    }

    #[test]
    fn validate_feature_branch() {
        let t = default_topology();
        assert!(t.validate_branch("feature/foo").is_ok());
    }

    #[test]
    fn validate_release_branch() {
        let t = default_topology();
        assert!(t.validate_branch("release/1.0").is_ok());
    }

    #[test]
    fn validate_invalid_branch() {
        let t = default_topology();
        assert!(t.validate_branch("hotfix/oops").is_err());
    }

    #[test]
    fn commit_to_main_blocked() {
        let t = default_topology();
        assert!(t.check_commit("main").is_err());
    }

    #[test]
    fn commit_to_feature_allowed() {
        let t = default_topology();
        assert!(t.check_commit("feature/foo").is_ok());
    }

    #[test]
    fn commit_to_main_allowed_when_unblocked() {
        let policy = BranchPolicy {
            block_direct_main_commits: false,
            ..BranchPolicy::default()
        };
        let t = BranchTopology::new(policy);
        assert!(t.check_commit("main").is_ok());
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;

    #[test]
    fn default_policy_values() {
        let p = BranchPolicy::default();
        assert_eq!(p.main_branch, "main");
        assert_eq!(p.release_pattern, "release/*");
        assert_eq!(p.feature_prefix, "feature/");
        assert_eq!(p.wp_branch_format, "feature/{slug}/WP{id}");
        assert!(p.block_direct_main_commits);
    }

    #[test]
    fn policy_serde_roundtrip() {
        let p = BranchPolicy::default();
        let t = toml::to_string(&p).unwrap();
        let back: BranchPolicy = toml::from_str(&t).unwrap();
        assert_eq!(back.main_branch, p.main_branch);
        assert_eq!(back.release_pattern, p.release_pattern);
    }

    #[test]
    fn from_config_file_reads_policy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("topology.toml");
        std::fs::write(
            &path,
            "main_branch = \"trunk\"\nrelease_pattern = \"rel/*\"\nfeature_prefix = \"feat/\"\nwp_branch_format = \"feat/{slug}/{id}\"\nblock_direct_main_commits = false\n",
        )
        .unwrap();
        let t = BranchTopology::from_config_file(&path).unwrap();
        assert_eq!(t.feature_branch("x"), "feat/x");
        assert_eq!(t.wp_branch("x", "9"), "feat/x/9");
        assert!(t.check_commit("trunk").is_ok());
    }

    #[test]
    fn from_config_file_missing_errors() {
        assert!(BranchTopology::from_config_file(Path::new("/nope/topology.toml")).is_err());
    }

    #[test]
    fn from_config_file_invalid_toml_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.toml");
        std::fs::write(&path, "not = valid = toml").unwrap();
        assert!(BranchTopology::from_config_file(&path).is_err());
    }

    #[test]
    fn validate_exactly_feature_prefix_is_ok() {
        // Anything starting with the prefix passes, including the bare prefix.
        let t = BranchTopology::new(BranchPolicy::default());
        assert!(t.validate_branch("feature/").is_ok());
    }

    #[test]
    fn validate_custom_main_branch() {
        let policy = BranchPolicy {
            main_branch: "trunk".into(),
            ..BranchPolicy::default()
        };
        let t = BranchTopology::new(policy);
        assert!(t.validate_branch("trunk").is_ok());
        assert!(t.validate_branch("main").is_err());
    }

    #[test]
    fn validate_custom_release_pattern() {
        let policy = BranchPolicy {
            release_pattern: "rel-*".into(),
            ..BranchPolicy::default()
        };
        let t = BranchTopology::new(policy);
        assert!(t.validate_branch("rel-2024").is_ok());
        assert!(t.validate_branch("release/1").is_err());
    }

    #[test]
    fn validate_empty_branch_is_err() {
        let t = BranchTopology::new(BranchPolicy::default());
        assert!(t.validate_branch("").is_err());
    }

    #[test]
    fn check_commit_allows_release_branch() {
        let t = BranchTopology::new(BranchPolicy::default());
        assert!(t.check_commit("release/1.0").is_ok());
    }

    #[test]
    fn direct_main_commit_error_display() {
        let e = TopologyError::DirectMainCommit;
        assert_eq!(format!("{e}"), "Direct commits to main are blocked");
    }

    #[test]
    fn invalid_branch_error_display_includes_name() {
        let e = TopologyError::InvalidBranchName {
            name: "hotfix/x".into(),
            reason: "bad".into(),
        };
        let s = format!("{e}");
        assert!(s.contains("hotfix/x"));
        assert!(s.contains("bad"));
    }

    #[test]
    fn policy_violation_error_display() {
        let e = TopologyError::PolicyViolation {
            message: "nope".into(),
        };
        assert_eq!(format!("{e}"), "Policy violation: nope");
    }

    #[test]
    fn topology_error_implements_error_trait() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<TopologyError>();
    }

    #[test]
    fn feature_and_wp_branch_are_distinct() {
        let t = BranchTopology::new(BranchPolicy::default());
        assert_ne!(t.feature_branch("x"), t.wp_branch("x", "1"));
    }
}
