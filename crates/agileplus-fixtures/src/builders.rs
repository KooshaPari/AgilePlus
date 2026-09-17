//! Builder patterns for constructing test fixtures.
//!
//! Provides fluent builders for creating features, work packages, and related
//! domain objects for testing. All builders produce valid, deterministic objects.

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use chrono::Utc;

/// Builder for constructing Feature test objects.
#[derive(Clone)]
pub struct FeatureBuilder {
    id: i64,
    slug: String,
    friendly_name: String,
    state: FeatureState,
    spec_hash: [u8; 32],
    target_branch: String,
    labels: Vec<String>,
    project_id: Option<i64>,
}

impl Default for FeatureBuilder {
    fn default() -> Self {
        Self::new("test-feature", "Test Feature")
    }
}

impl FeatureBuilder {
    /// Create a new feature builder with slug and friendly_name.
    pub fn new(slug: &str, friendly_name: &str) -> Self {
        Self {
            id: 1,
            slug: slug.to_string(),
            friendly_name: friendly_name.to_string(),
            state: FeatureState::Created,
            spec_hash: [0u8; 32],
            target_branch: "main".to_string(),
            labels: Vec::new(),
            project_id: None,
        }
    }

    /// Set the feature ID.
    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    /// Set the feature state.
    pub fn state(mut self, state: FeatureState) -> Self {
        self.state = state;
        self
    }

    /// Add a label to the feature.
    pub fn with_label(mut self, label: &str) -> Self {
        self.labels.push(label.to_string());
        self
    }

    /// Set multiple labels.
    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    /// Set the project ID.
    pub fn project_id(mut self, project_id: i64) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Set the spec hash.
    pub fn spec_hash(mut self, hash: [u8; 32]) -> Self {
        self.spec_hash = hash;
        self
    }

    /// Build the Feature.
    pub fn build(self) -> Feature {
        Feature {
            id: self.id,
            slug: self.slug,
            friendly_name: self.friendly_name,
            state: self.state,
            spec_hash: self.spec_hash,
            target_branch: self.target_branch,
            plane_issue_id: None,
            plane_state_id: None,
            labels: self.labels,
            module_id: None,
            project_id: self.project_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_at_commit: None,
            last_modified_commit: None,
        }
    }
}

/// Builder for constructing WorkPackage test objects.
#[derive(Clone)]
pub struct WorkPackageBuilder {
    id: i64,
    feature_id: i64,
    title: String,
    sequence: i32,
    acceptance_criteria: String,
    state: WpState,
    file_scope: Vec<String>,
}

impl WorkPackageBuilder {
    /// Create a new work package builder.
    pub fn new(feature_id: i64, title: &str, sequence: i32) -> Self {
        Self {
            id: 1,
            feature_id,
            title: title.to_string(),
            sequence,
            acceptance_criteria: String::new(),
            state: WpState::Planned,
            file_scope: Vec::new(),
        }
    }

    /// Set the work package ID.
    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    /// Set the work package state.
    pub fn state(mut self, state: WpState) -> Self {
        self.state = state;
        self
    }

    /// Set the acceptance criteria (previously `summary`; aligned with
    /// `agileplus_domain::WorkPackage::acceptance_criteria`).
    pub fn acceptance_criteria(mut self, acceptance_criteria: &str) -> Self {
        self.acceptance_criteria = acceptance_criteria.to_string();
        self
    }

    /// Deprecated alias for [`Self::acceptance_criteria`]; retained for
    /// backwards compatibility with older fixture callers.
    #[deprecated(note = "use `acceptance_criteria` to match `WorkPackage` field name")]
    pub fn summary(self, summary: &str) -> Self {
        self.acceptance_criteria(summary)
    }

    /// Add a file to the scope.
    pub fn with_file(mut self, file: &str) -> Self {
        self.file_scope.push(file.to_string());
        self
    }

    /// Set multiple files in scope.
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.file_scope = files;
        self
    }

    /// Build the WorkPackage.
    pub fn build(self) -> WorkPackage {
        let mut wp = WorkPackage::new(
            self.feature_id,
            &self.title,
            self.sequence,
            &self.acceptance_criteria,
        );
        wp.id = self.id;
        wp.state = self.state;
        wp.file_scope = self.file_scope;
        wp
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
            .acceptance_criteria("This is a test")
            .with_file("src/lib.rs")
            .build();

        assert_eq!(wp.id, 100);
        assert_eq!(wp.feature_id, 1);
        assert_eq!(wp.title, "Test WP");
        assert_eq!(wp.state, WpState::Done);
        assert_eq!(wp.acceptance_criteria, "This is a test");
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

    // ── FeatureBuilder ───────────────────────────────────────────────────────

    #[test]
    fn feature_builder_default_impl_uses_canonical_slug() {
        let feature = FeatureBuilder::default().build();
        assert_eq!(feature.slug, "test-feature");
        assert_eq!(feature.friendly_name, "Test Feature");
    }

    #[test]
    fn feature_builder_id_setter_is_applied() {
        assert_eq!(FeatureBuilder::new("s", "n").id(99).build().id, 99);
    }

    #[test]
    fn feature_builder_supports_every_state() {
        let states = [
            FeatureState::Created,
            FeatureState::Specified,
            FeatureState::Researched,
            FeatureState::Planned,
            FeatureState::Implementing,
            FeatureState::Validated,
            FeatureState::Shipped,
            FeatureState::Retrospected,
        ];
        for state in states {
            let feature = FeatureBuilder::new("s", "n").state(state).build();
            assert_eq!(feature.state, state, "state {state:?} not preserved");
        }
    }

    #[test]
    fn feature_builder_with_label_appends_in_order() {
        let feature = FeatureBuilder::new("s", "n")
            .with_label("first")
            .with_label("second")
            .build();
        assert_eq!(feature.labels, vec!["first", "second"]);
    }

    #[test]
    fn feature_builder_with_labels_replaces_existing() {
        let feature = FeatureBuilder::new("s", "n")
            .with_label("dropped")
            .with_labels(vec!["a".to_string(), "b".to_string()])
            .build();
        assert_eq!(feature.labels, vec!["a", "b"]);
    }

    #[test]
    fn feature_builder_project_id_is_optional() {
        assert!(FeatureBuilder::new("s", "n").build().project_id.is_none());
        assert_eq!(
            FeatureBuilder::new("s", "n")
                .project_id(7)
                .build()
                .project_id,
            Some(7)
        );
    }

    #[test]
    fn feature_builder_spec_hash_is_preserved() {
        let hash = [0xAB_u8; 32];
        assert_eq!(
            FeatureBuilder::new("s", "n")
                .spec_hash(hash)
                .build()
                .spec_hash,
            hash
        );
        assert_eq!(
            FeatureBuilder::new("s", "n").build().spec_hash,
            [0_u8; 32],
            "default spec hash must be zeroed"
        );
    }

    #[test]
    fn feature_builder_plane_fields_are_none() {
        let feature = FeatureBuilder::new("s", "n").build();
        assert!(feature.plane_issue_id.is_none());
        assert!(feature.plane_state_id.is_none());
        assert!(feature.module_id.is_none());
        assert!(feature.created_at_commit.is_none());
        assert!(feature.last_modified_commit.is_none());
    }

    #[test]
    fn feature_builder_timestamps_are_set_and_ordered() {
        let feature = FeatureBuilder::new("s", "n").build();
        assert!(feature.created_at <= feature.updated_at);
    }

    #[test]
    fn feature_builder_clone_is_independent() {
        let base = FeatureBuilder::new("s", "n");
        let a = base.clone().id(1).build();
        let b = base.clone().id(2).build();
        assert_eq!(a.id, 1);
        assert_eq!(b.id, 2);
    }

    #[test]
    fn feature_builder_does_not_leak_labels_between_builds() {
        let with_label = FeatureBuilder::new("s", "n").with_label("x").build();
        let fresh = FeatureBuilder::new("s", "n").build();
        assert_eq!(with_label.labels.len(), 1);
        assert!(fresh.labels.is_empty());
    }

    #[test]
    fn feature_builder_full_chain() {
        let feature = FeatureBuilder::new("slug", "Name")
            .id(5)
            .state(FeatureState::Shipped)
            .project_id(3)
            .spec_hash([1u8; 32])
            .with_labels(vec!["a".to_string()])
            .build();
        assert_eq!(feature.id, 5);
        assert_eq!(feature.slug, "slug");
        assert_eq!(feature.state, FeatureState::Shipped);
        assert_eq!(feature.project_id, Some(3));
        assert_eq!(feature.target_branch, "main");
    }

    // ── WorkPackageBuilder ───────────────────────────────────────────────────

    #[test]
    fn work_package_builder_defaults() {
        let wp = WorkPackageBuilder::new(4, "WP", 2).build();
        assert_eq!(wp.id, 1, "default id is 1");
        assert_eq!(wp.feature_id, 4);
        assert_eq!(wp.title, "WP");
        assert_eq!(wp.sequence, 2);
        assert_eq!(wp.state, WpState::Planned);
        assert!(wp.acceptance_criteria.is_empty());
        assert!(wp.file_scope.is_empty());
    }

    #[test]
    fn work_package_builder_id_setter() {
        assert_eq!(WorkPackageBuilder::new(1, "t", 1).id(77).build().id, 77);
    }

    #[test]
    fn work_package_builder_supports_every_state() {
        for state in [
            WpState::Planned,
            WpState::Doing,
            WpState::Review,
            WpState::Done,
            WpState::Blocked,
        ] {
            let wp = WorkPackageBuilder::new(1, "t", 1).state(state).build();
            assert_eq!(wp.state, state);
        }
    }

    #[test]
    fn work_package_builder_acceptance_criteria_is_stored() {
        let wp = WorkPackageBuilder::new(1, "t", 1)
            .acceptance_criteria("cargo test passes")
            .build();
        assert_eq!(wp.acceptance_criteria, "cargo test passes");
    }

    #[test]
    #[allow(deprecated)]
    fn work_package_builder_summary_alias_matches_acceptance_criteria() {
        let via_alias = WorkPackageBuilder::new(1, "t", 1)
            .summary("same text")
            .build();
        let via_canonical = WorkPackageBuilder::new(1, "t", 1)
            .acceptance_criteria("same text")
            .build();
        assert_eq!(
            via_alias.acceptance_criteria,
            via_canonical.acceptance_criteria
        );
    }

    #[test]
    fn work_package_builder_with_file_appends() {
        let wp = WorkPackageBuilder::new(1, "t", 1)
            .with_file("a.rs")
            .with_file("b.rs")
            .build();
        assert_eq!(wp.file_scope, vec!["a.rs", "b.rs"]);
    }

    #[test]
    fn work_package_builder_with_files_replaces() {
        let wp = WorkPackageBuilder::new(1, "t", 1)
            .with_file("dropped.rs")
            .with_files(vec!["x.rs".to_string(), "y.rs".to_string()])
            .build();
        assert_eq!(wp.file_scope, vec!["x.rs", "y.rs"]);
    }

    #[test]
    fn work_package_builder_leaves_agent_and_pr_unset() {
        let wp = WorkPackageBuilder::new(1, "t", 1).build();
        assert!(wp.agent_id.is_none());
        assert!(wp.pr_url.is_none());
        assert!(wp.pr_state.is_none());
        assert!(wp.worktree_path.is_none());
        assert!(wp.plane_sub_issue_id.is_none());
        assert!(wp.base_commit.is_none());
        assert!(wp.head_commit.is_none());
    }

    #[test]
    fn work_package_builder_sets_timestamps() {
        let wp = WorkPackageBuilder::new(1, "t", 1).build();
        assert!(wp.created_at <= wp.updated_at);
    }

    #[test]
    fn work_package_builder_clone_is_independent() {
        let base = WorkPackageBuilder::new(1, "t", 1);
        let a = base.clone().id(10).build();
        let b = base.clone().id(20).build();
        assert_eq!(a.id, 10);
        assert_eq!(b.id, 20);
    }

    #[test]
    fn builders_are_cloneable() {
        let fb = FeatureBuilder::new("s", "n").with_label("l");
        let fb2 = fb.clone();
        assert_eq!(fb.build().labels, fb2.build().labels);

        let wb = WorkPackageBuilder::new(1, "t", 1).with_file("f.rs");
        let wb2 = wb.clone();
        assert_eq!(wb.build().file_scope, wb2.build().file_scope);
    }
}
