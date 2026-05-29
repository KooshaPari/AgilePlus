//! Hexagonal-architecture ports — async traits implemented by adapters.

pub mod epic;
pub mod events;
pub mod observability;
pub mod storage;
pub mod story;
pub mod vcs;

pub use epic::EpicRepository;
pub use events::{DomainEvent, DomainEventPublisher};
pub use story::StoryRepository;

use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::domain::{
    audit::AuditEntry,
    backlog::{BacklogFilters, BacklogItem, BacklogPriority, BacklogStatus},
    cycle::{Cycle, CycleFeature, CycleWithFeatures, CycleState},
    epic::{Epic, EpicStatus},
    feature::Feature,
    governance::{Evidence, GovernanceContract, PolicyRule},
    metric::Metric,
    module::{Module, ModuleFeatureTag, ModuleWithFeatures},
    project::Project,
    state_machine::FeatureState,
    story::{Story, StoryStatus},
    sync_mapping::SyncMapping,
    user::{User, UserRole, UserStatus},
    work_package::{WorkPackage, WpDependency, WpState},
};
use crate::error::DomainError;

use self::vcs::{BranchInfo, ConflictInfo, FeatureArtifacts, MergeResult, WorktreeInfo};

/// Primary storage port — full CRUD across all domain aggregates.
#[async_trait]
pub trait StoragePort: Send + Sync {
    // --- Features ---
    async fn create_feature(&self, feature: &Feature) -> Result<i64, DomainError>;
    async fn get_feature_by_slug(&self, slug: &str) -> Result<Option<Feature>, DomainError>;
    async fn get_feature_by_id(&self, id: i64) -> Result<Option<Feature>, DomainError>;
    async fn update_feature_state(&self, id: i64, state: FeatureState) -> Result<(), DomainError>;
    async fn list_features_by_state(&self, state: FeatureState) -> Result<Vec<Feature>, DomainError>;
    async fn list_all_features(&self) -> Result<Vec<Feature>, DomainError>;

    // --- Work Packages ---
    async fn create_work_package(&self, wp: &WorkPackage) -> Result<i64, DomainError>;
    async fn get_work_package(&self, id: i64) -> Result<Option<WorkPackage>, DomainError>;
    async fn update_wp_state(&self, id: i64, state: WpState) -> Result<(), DomainError>;
    async fn list_wps_by_feature(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError>;
    async fn add_wp_dependency(&self, dep: &WpDependency) -> Result<(), DomainError>;
    async fn get_wp_dependencies(&self, wp_id: i64) -> Result<Vec<WpDependency>, DomainError>;
    async fn get_ready_wps(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError>;

    // --- Audit ---
    async fn append_audit_entry(&self, entry: &AuditEntry) -> Result<i64, DomainError>;
    async fn get_audit_trail(&self, feature_id: i64) -> Result<Vec<AuditEntry>, DomainError>;
    async fn get_latest_audit_entry(&self, feature_id: i64) -> Result<Option<AuditEntry>, DomainError>;

    // --- Evidence ---
    async fn create_evidence(&self, ev: &Evidence) -> Result<i64, DomainError>;
    async fn get_evidence_by_wp(&self, wp_id: i64) -> Result<Vec<Evidence>, DomainError>;
    async fn get_evidence_by_fr(&self, fr_id: &str) -> Result<Vec<Evidence>, DomainError>;

    // --- Policy / Governance ---
    async fn create_policy_rule(&self, rule: &PolicyRule) -> Result<i64, DomainError>;
    async fn list_active_policies(&self) -> Result<Vec<PolicyRule>, DomainError>;
    async fn record_metric(&self, metric: &Metric) -> Result<i64, DomainError>;
    async fn get_metrics_by_feature(&self, feature_id: i64) -> Result<Vec<Metric>, DomainError>;
    async fn create_governance_contract(&self, contract: &GovernanceContract) -> Result<i64, DomainError>;
    async fn get_governance_contract(&self, feature_id: i64, version: i32) -> Result<Option<GovernanceContract>, DomainError>;
    async fn get_latest_governance_contract(&self, feature_id: i64) -> Result<Option<GovernanceContract>, DomainError>;

    // --- Modules ---
    async fn create_module(&self, module: &Module) -> Result<i64, DomainError>;
    async fn get_module(&self, id: i64) -> Result<Option<Module>, DomainError>;
    async fn get_module_by_slug(&self, slug: &str) -> Result<Option<Module>, DomainError>;
    async fn update_module(&self, id: i64, friendly_name: &str, description: Option<&str>) -> Result<(), DomainError>;
    async fn delete_module(&self, id: i64) -> Result<(), DomainError>;
    async fn list_root_modules(&self) -> Result<Vec<Module>, DomainError>;
    async fn list_child_modules(&self, parent_id: i64) -> Result<Vec<Module>, DomainError>;
    async fn get_module_with_features(&self, id: i64) -> Result<Option<ModuleWithFeatures>, DomainError>;
    async fn tag_feature_to_module(&self, tag: &ModuleFeatureTag) -> Result<(), DomainError>;
    async fn untag_feature_from_module(&self, module_id: i64, feature_id: i64) -> Result<(), DomainError>;

    // --- Cycles ---
    async fn create_cycle(&self, cycle: &Cycle) -> Result<i64, DomainError>;
    async fn get_cycle(&self, id: i64) -> Result<Option<Cycle>, DomainError>;
    async fn update_cycle_state(&self, id: i64, state: CycleState) -> Result<(), DomainError>;
    async fn list_cycles_by_state(&self, state: CycleState) -> Result<Vec<Cycle>, DomainError>;
    async fn list_cycles_by_module(&self, module_id: i64) -> Result<Vec<Cycle>, DomainError>;
    async fn list_all_cycles(&self) -> Result<Vec<Cycle>, DomainError>;
    async fn get_cycle_with_features(&self, id: i64) -> Result<Option<CycleWithFeatures>, DomainError>;
    async fn add_feature_to_cycle(&self, entry: &CycleFeature) -> Result<(), DomainError>;
    async fn remove_feature_from_cycle(&self, cycle_id: i64, feature_id: i64) -> Result<(), DomainError>;

    // --- Sync Mappings ---
    async fn get_sync_mapping(&self, entity_type: &str, entity_id: i64) -> Result<Option<SyncMapping>, DomainError>;
    async fn upsert_sync_mapping(&self, mapping: &SyncMapping) -> Result<(), DomainError>;
    async fn get_sync_mapping_by_plane_id(&self, entity_type: &str, plane_issue_id: &str) -> Result<Option<SyncMapping>, DomainError>;
    async fn delete_sync_mapping(&self, entity_type: &str, entity_id: i64) -> Result<(), DomainError>;

    // --- Projects ---
    async fn create_project(&self, project: &Project) -> Result<i64, DomainError>;
    async fn get_project_by_slug(&self, slug: &str) -> Result<Option<Project>, DomainError>;
    async fn get_project_by_id(&self, id: i64) -> Result<Option<Project>, DomainError>;
    async fn list_all_projects(&self) -> Result<Vec<Project>, DomainError>;
    async fn delete_project(&self, id: i64) -> Result<(), DomainError>;

    // --- Users ---
    async fn create_user(&self, user: &User) -> Result<i64, DomainError>;
    async fn get_user_by_id(&self, id: i64) -> Result<Option<User>, DomainError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    async fn update_user_status(&self, id: i64, status: UserStatus) -> Result<(), DomainError>;
    async fn update_user_role(&self, id: i64, role: UserRole) -> Result<(), DomainError>;
    async fn list_all_users(&self) -> Result<Vec<User>, DomainError>;
    async fn delete_user(&self, id: i64) -> Result<(), DomainError>;

    // --- Epics ---
    async fn create_epic(&self, epic: &Epic) -> Result<i64, DomainError>;
    async fn get_epic_by_id(&self, id: i64) -> Result<Option<Epic>, DomainError>;
    async fn update_epic_status(&self, id: i64, status: EpicStatus) -> Result<(), DomainError>;
    async fn list_epics_by_project(&self, project_id: i64) -> Result<Vec<Epic>, DomainError>;
    async fn delete_epic(&self, id: i64) -> Result<(), DomainError>;

    // --- Stories ---
    async fn create_story(&self, story: &Story) -> Result<i64, DomainError>;
    async fn get_story_by_id(&self, id: i64) -> Result<Option<Story>, DomainError>;
    async fn update_story_status(&self, id: i64, status: StoryStatus) -> Result<(), DomainError>;
    async fn list_stories_by_epic(&self, epic_id: i64) -> Result<Vec<Story>, DomainError>;
    async fn list_stories_by_project(&self, project_id: i64) -> Result<Vec<Story>, DomainError>;
    async fn delete_story(&self, id: i64) -> Result<(), DomainError>;
    /// Upsert a story keyed by `story.requirement_id` — see
    /// [`StoryRepository::upsert_by_requirement_id`] for semantics.
    async fn upsert_story_by_requirement_id(&self, story: &Story) -> Result<i64, DomainError>;
}

// ── Blanket impls ─────────────────────────────────────────────────────────────
//
// Any type that implements `StoragePort` automatically satisfies the focused
// repository sub-ports.  This lets `AppState<S, …>` pass `Arc<S>` directly
// to use-cases that only depend on the narrower port trait.

#[async_trait]
impl<T: StoragePort> StoryRepository for T {
    async fn create(&self, story: &Story) -> Result<i64, DomainError> {
        self.create_story(story).await
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Story>, DomainError> {
        self.get_story_by_id(id).await
    }
    async fn update_status(&self, id: i64, status: StoryStatus) -> Result<(), DomainError> {
        self.update_story_status(id, status).await
    }
    async fn list_by_epic(&self, epic_id: i64) -> Result<Vec<Story>, DomainError> {
        self.list_stories_by_epic(epic_id).await
    }
    async fn upsert_by_requirement_id(&self, story: &Story) -> Result<i64, DomainError> {
        self.upsert_story_by_requirement_id(story).await
    }
}

#[async_trait]
impl<T: StoragePort> EpicRepository for T {
    async fn create(&self, epic: &Epic) -> Result<i64, DomainError> {
        self.create_epic(epic).await
    }
    async fn get_by_id(&self, id: i64) -> Result<Option<Epic>, DomainError> {
        self.get_epic_by_id(id).await
    }
    async fn update_status(&self, id: i64, status: EpicStatus) -> Result<(), DomainError> {
        self.update_epic_status(id, status).await
    }
    async fn list_by_project(&self, project_id: i64) -> Result<Vec<Epic>, DomainError> {
        self.list_epics_by_project(project_id).await
    }
}

/// Content storage port — subset used by the dashboard/content layer.
#[async_trait]
pub trait ContentStoragePort: Send + Sync {
    // Features
    async fn create_feature(&self, feature: &Feature) -> Result<i64, DomainError>;
    async fn get_feature_by_slug(&self, slug: &str) -> Result<Option<Feature>, DomainError>;
    async fn get_feature_by_id(&self, id: i64) -> Result<Option<Feature>, DomainError>;
    async fn update_feature_state(&self, id: i64, state: FeatureState) -> Result<(), DomainError>;
    async fn update_feature(&self, feature: &Feature) -> Result<(), DomainError>;
    async fn list_features_by_state(&self, state: FeatureState) -> Result<Vec<Feature>, DomainError>;
    async fn list_all_features(&self) -> Result<Vec<Feature>, DomainError>;
    // Work packages
    async fn create_work_package(&self, wp: &WorkPackage) -> Result<i64, DomainError>;
    async fn get_work_package(&self, id: i64) -> Result<Option<WorkPackage>, DomainError>;
    async fn update_wp_state(&self, id: i64, state: WpState) -> Result<(), DomainError>;
    async fn update_work_package(&self, wp: &WorkPackage) -> Result<(), DomainError>;
    async fn list_wps_by_feature(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError>;
    async fn add_wp_dependency(&self, dep: &WpDependency) -> Result<(), DomainError>;
    async fn get_wp_dependencies(&self, wp_id: i64) -> Result<Vec<WpDependency>, DomainError>;
    async fn get_ready_wps(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError>;
    // Backlog
    async fn create_backlog_item(&self, item: &BacklogItem) -> Result<i64, DomainError>;
    async fn get_backlog_item(&self, id: i64) -> Result<Option<BacklogItem>, DomainError>;
    async fn list_backlog_items(&self, filters: &BacklogFilters) -> Result<Vec<BacklogItem>, DomainError>;
    async fn update_backlog_status(&self, id: i64, status: BacklogStatus) -> Result<(), DomainError>;
    async fn update_backlog_priority(&self, id: i64, priority: BacklogPriority) -> Result<(), DomainError>;
    async fn pop_next_backlog_item(&self) -> Result<Option<BacklogItem>, DomainError>;
}

/// VCS port — git operations needed by the domain.
#[async_trait]
pub trait VcsPort: Send + Sync {
    async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> Result<PathBuf, DomainError>;
    async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError>;
    async fn cleanup_worktree(&self, worktree_path: &Path) -> Result<(), DomainError>;
    async fn create_branch(&self, branch_name: &str, base: &str) -> Result<(), DomainError>;
    async fn list_branches(&self, pattern: Option<&str>, remote: bool) -> Result<Vec<BranchInfo>, DomainError>;
    async fn delete_branch(&self, branch_name: &str, force: bool, remote: Option<&str>) -> Result<(), DomainError>;
    async fn checkout_branch(&self, branch_name: &str) -> Result<(), DomainError>;
    async fn merge_to_target(&self, source: &str, target: &str) -> Result<MergeResult, DomainError>;
    async fn detect_conflicts(&self, source: &str, target: &str) -> Result<Vec<ConflictInfo>, DomainError>;
    async fn read_artifact(&self, feature_slug: &str, relative_path: &str) -> Result<String, DomainError>;
    async fn write_artifact(&self, feature_slug: &str, relative_path: &str, content: &str) -> Result<(), DomainError>;
    async fn artifact_exists(&self, feature_slug: &str, relative_path: &str) -> Result<bool, DomainError>;
    async fn scan_feature_artifacts(&self, feature_slug: &str) -> Result<FeatureArtifacts, DomainError>;
}
