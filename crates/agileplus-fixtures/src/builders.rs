use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use chrono::Utc;

/// Builder for constructing Feature test objects.
#[derive(Clone)]
pub struct FeatureBuilder {
    pub id: i64,
    pub slug: String,
    pub friendly_name: String,
    pub project_id: Option<i64>,
    pub state: FeatureState,
    pub labels: Vec<String>,
    pub target_branch: String,
}

impl FeatureBuilder {
    /// Create a new builder with required fields.
    pub fn new(slug: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: 0,
            slug: slug.into(),
            friendly_name: name.into(),
            project_id: None,
            state: FeatureState::Created,
            labels: Vec::new(),
            target_branch: "main".to_string(),
        }
    }

    /// Set the feature ID.
    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    /// Set the feature project ID.
    pub fn project_id(mut self, project_id: i64) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Set the feature state.
    pub fn state(mut self, state: FeatureState) -> Self {
        self.state = state;
        self
    }

    /// Add a label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }

    /// Set the target branch.
    pub fn target_branch(mut self, branch: impl Into<String>) -> Self {
        self.target_branch = branch.into();
        self
    }

    /// Build the Feature.
    pub fn build(&self) -> Feature {
        Feature {
            id: self.id,
            slug: self.slug.clone(),
            friendly_name: self.friendly_name.clone(),
            project_id: self.project_id,
            state: self.state,
            labels: self.labels.clone(),
            target_branch: self.target_branch.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_at_commit: None,
            last_modified_commit: None,
            module_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            spec_hash: [0u8; 32],
        }
    }
}

impl Default for FeatureBuilder {
    fn default() -> Self {
        Self::new("default-feature", "Default Feature")
    }
}

/// Builder for constructing WorkPackage test objects.
#[derive(Clone)]
pub struct WorkPackageBuilder {
    pub id: i64,
    pub feature_id: i64,
    pub title: String,
    pub sequence: i32,
    pub state: WpState,
    pub file_scope: Vec<String>,
}

impl WorkPackageBuilder {
    /// Create a new builder with required fields.
    pub fn new(feature_id: i64, title: impl Into<String>, sequence: i32) -> Self {
        Self {
            id: 0,
            feature_id,
            title: title.into(),
            sequence,
            state: WpState::Planned,
            file_scope: Vec::new(),
        }
    }

    /// Set the WP ID.
    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    /// Set the WP state.
    pub fn state(mut self, state: WpState) -> Self {
        self.state = state;
        self
    }

    /// Add a file to scope.
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file_scope.push(file.into());
        self
    }

    /// Set multiple files in scope.
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.file_scope = files;
        self
    }

    /// Build the WorkPackage.
    pub fn build(&self) -> WorkPackage {
        WorkPackage {
            id: self.id,
            feature_id: self.feature_id,
            title: self.title.clone(),
            state: self.state,
            sequence: self.sequence,
            file_scope: self.file_scope.clone(),
            acceptance_criteria: String::new(),
            agent_id: None,
            pr_url: None,
            pr_state: None,
            worktree_path: None,
            plane_sub_issue_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            base_commit: None,
            head_commit: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_builder_creates_valid_feature() {
        let feature = FeatureBuilder::new("test-slug", "Test Name")
            .id(1)
            .state(FeatureState::Specified)
            .with_label("test")
            .build();

        assert_eq!(feature.id, 1);
        assert_eq!(feature.slug, "test-slug");
        assert_eq!(feature.friendly_name, "Test Name");
        assert_eq!(feature.state, FeatureState::Specified);
        assert_eq!(feature.labels, vec!["test"]);
    }

    #[test]
    fn feature_builder_default_values() {
        let feature = FeatureBuilder::default().build();

        assert_eq!(feature.state, FeatureState::Created);
        assert_eq!(feature.target_branch, "main");
        assert!(feature.labels.is_empty());
        assert!(feature.project_id.is_none());
    }

    #[test]
    fn work_package_builder_creates_valid_wp() {
        let wp = WorkPackageBuilder::new(1, "Test WP", 1)
            .id(100)
            .state(WpState::Done)
            .with_file("src/lib.rs")
            .build();

        assert_eq!(wp.id, 100);
        assert_eq!(wp.feature_id, 1);
        assert_eq!(wp.title, "Test WP");
        assert_eq!(wp.state, WpState::Done);
        assert_eq!(wp.file_scope, vec!["src/lib.rs"]);
    }

    #[test]
    fn work_package_builder_multiple_files() {
        let files = vec!["src/lib.rs".to_string(), "tests/unit.rs".to_string()];
        let wp = WorkPackageBuilder::new(1, "Multi-file WP", 2)
            .with_files(files.clone())
            .build();

        assert_eq!(wp.file_scope, files);
    }
}