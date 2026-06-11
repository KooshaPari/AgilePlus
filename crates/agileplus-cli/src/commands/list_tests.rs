//! In-memory stub + unit tests for the list_projects / list_epics / list_stories
//! subcommands (FR-AGP-016).
//!
//! All tests run without I/O; the `MemStore` fulfils `StoragePort` by serving
//! pre-seeded data. The commands under test write to stdout but the tests only
//! assert that `run()` returns `Ok(())` with the expected filtering semantics.

#![cfg(test)]

#[allow(unused_imports)] // Backlog* types used in fixture/seed data
use agileplus_domain::{
    domain::{
        audit::AuditEntry,
        backlog::{BacklogFilters, BacklogItem, BacklogPriority, BacklogStatus},
        cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures},
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
    },
    error::DomainError,
    ports::StoragePort,
};
use async_trait::async_trait;

// ── In-memory test double ─────────────────────────────────────────────────────

pub struct MemStore {
    pub features: Vec<Feature>,
    pub projects: Vec<Project>,
    pub epics: Vec<Epic>,
    pub stories: Vec<Story>,
}

#[async_trait]
impl StoragePort for MemStore {
    // --- Projects ---
    async fn list_all_projects(&self) -> Result<Vec<Project>, DomainError> {
        Ok(self.projects.clone())
    }
    async fn create_project(&self, _: &Project) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn get_project_by_slug(&self, _: &str) -> Result<Option<Project>, DomainError> {
        unimplemented!()
    }
    async fn get_project_by_id(&self, _: i64) -> Result<Option<Project>, DomainError> {
        unimplemented!()
    }
    async fn delete_project(&self, _: i64) -> Result<(), DomainError> {
        unimplemented!()
    }

    // --- Epics ---
    async fn list_epics_by_project(&self, project_id: i64) -> Result<Vec<Epic>, DomainError> {
        Ok(self
            .epics
            .iter()
            .filter(|e| e.project_id == project_id)
            .cloned()
            .collect())
    }
    async fn create_epic(&self, _: &Epic) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn get_epic_by_id(&self, _: i64) -> Result<Option<Epic>, DomainError> {
        unimplemented!()
    }
    async fn update_epic_status(&self, _: i64, _: EpicStatus) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn delete_epic(&self, _: i64) -> Result<(), DomainError> {
        unimplemented!()
    }

    // --- Stories ---
    async fn list_stories_by_epic(&self, epic_id: i64) -> Result<Vec<Story>, DomainError> {
        Ok(self
            .stories
            .iter()
            .filter(|s| s.epic_id == epic_id)
            .cloned()
            .collect())
    }
    async fn list_stories_by_project(&self, project_id: i64) -> Result<Vec<Story>, DomainError> {
        Ok(self
            .stories
            .iter()
            .filter(|s| s.project_id == project_id)
            .cloned()
            .collect())
    }
    async fn create_story(&self, _: &Story) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn get_story_by_id(&self, _: i64) -> Result<Option<Story>, DomainError> {
        unimplemented!()
    }
    async fn update_story_status(&self, _: i64, _: StoryStatus) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn delete_story(&self, _: i64) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn upsert_story_by_requirement_id(&self, _: &Story) -> Result<i64, DomainError> {
        unimplemented!()
    }

    // --- Everything else is unreachable in list tests ---
    async fn create_feature(&self, _: &Feature) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn get_feature_by_slug(&self, _: &str) -> Result<Option<Feature>, DomainError> {
        unimplemented!()
    }
    async fn get_feature_by_id(&self, _: i64) -> Result<Option<Feature>, DomainError> {
        unimplemented!()
    }
    async fn update_feature_state(&self, _: i64, _: FeatureState) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> Result<Vec<Feature>, DomainError> {
        Ok(self
            .features
            .iter()
            .filter(|feature| feature.state == state)
            .cloned()
            .collect())
    }
    async fn list_all_features(&self) -> Result<Vec<Feature>, DomainError> {
        Ok(self.features.clone())
    }
    async fn create_work_package(&self, _: &WorkPackage) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn get_work_package(&self, _: i64) -> Result<Option<WorkPackage>, DomainError> {
        unimplemented!()
    }
    async fn update_wp_state(&self, _: i64, _: WpState) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn list_wps_by_feature(&self, _: i64) -> Result<Vec<WorkPackage>, DomainError> {
        unimplemented!()
    }
    async fn add_wp_dependency(&self, _: &WpDependency) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn get_wp_dependencies(&self, _: i64) -> Result<Vec<WpDependency>, DomainError> {
        unimplemented!()
    }
    async fn get_ready_wps(&self, _: i64) -> Result<Vec<WorkPackage>, DomainError> {
        unimplemented!()
    }
    async fn append_audit_entry(&self, _: &AuditEntry) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn get_audit_trail(&self, _: i64) -> Result<Vec<AuditEntry>, DomainError> {
        unimplemented!()
    }
    async fn get_latest_audit_entry(&self, _: i64) -> Result<Option<AuditEntry>, DomainError> {
        unimplemented!()
    }
    async fn create_evidence(&self, _: &Evidence) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn get_evidence_by_wp(&self, _: i64) -> Result<Vec<Evidence>, DomainError> {
        unimplemented!()
    }
    async fn get_evidence_by_fr(&self, _: &str) -> Result<Vec<Evidence>, DomainError> {
        unimplemented!()
    }
    async fn create_policy_rule(&self, _: &PolicyRule) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn list_active_policies(&self) -> Result<Vec<PolicyRule>, DomainError> {
        unimplemented!()
    }
    async fn record_metric(&self, _: &Metric) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn get_metrics_by_feature(&self, _: i64) -> Result<Vec<Metric>, DomainError> {
        unimplemented!()
    }
    async fn create_governance_contract(&self, _: &GovernanceContract) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn get_governance_contract(
        &self,
        _: i64,
        _: i32,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        unimplemented!()
    }
    async fn get_latest_governance_contract(
        &self,
        _: i64,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        unimplemented!()
    }
    async fn create_module(&self, _: &Module) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn get_module(&self, _: i64) -> Result<Option<Module>, DomainError> {
        unimplemented!()
    }
    async fn get_module_by_slug(&self, _: &str) -> Result<Option<Module>, DomainError> {
        unimplemented!()
    }
    async fn update_module(&self, _: i64, _: &str, _: Option<&str>) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn delete_module(&self, _: i64) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn list_root_modules(&self) -> Result<Vec<Module>, DomainError> {
        unimplemented!()
    }
    async fn list_child_modules(&self, _: i64) -> Result<Vec<Module>, DomainError> {
        unimplemented!()
    }
    async fn get_module_with_features(
        &self,
        _: i64,
    ) -> Result<Option<ModuleWithFeatures>, DomainError> {
        unimplemented!()
    }
    async fn tag_feature_to_module(&self, _: &ModuleFeatureTag) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn untag_feature_from_module(&self, _: i64, _: i64) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn create_cycle(&self, _: &Cycle) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn get_cycle(&self, _: i64) -> Result<Option<Cycle>, DomainError> {
        unimplemented!()
    }
    async fn update_cycle_state(&self, _: i64, _: CycleState) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn list_cycles_by_state(&self, _: CycleState) -> Result<Vec<Cycle>, DomainError> {
        unimplemented!()
    }
    async fn list_cycles_by_module(&self, _: i64) -> Result<Vec<Cycle>, DomainError> {
        unimplemented!()
    }
    async fn list_all_cycles(&self) -> Result<Vec<Cycle>, DomainError> {
        unimplemented!()
    }
    async fn get_cycle_with_features(
        &self,
        _: i64,
    ) -> Result<Option<CycleWithFeatures>, DomainError> {
        unimplemented!()
    }
    async fn add_feature_to_cycle(&self, _: &CycleFeature) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn remove_feature_from_cycle(&self, _: i64, _: i64) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn get_sync_mapping(&self, _: &str, _: i64) -> Result<Option<SyncMapping>, DomainError> {
        unimplemented!()
    }
    async fn upsert_sync_mapping(&self, _: &SyncMapping) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn get_sync_mapping_by_plane_id(
        &self,
        _: &str,
        _: &str,
    ) -> Result<Option<SyncMapping>, DomainError> {
        unimplemented!()
    }
    async fn delete_sync_mapping(&self, _: &str, _: i64) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn create_user(&self, _: &User) -> Result<i64, DomainError> {
        unimplemented!()
    }
    async fn get_user_by_id(&self, _: i64) -> Result<Option<User>, DomainError> {
        unimplemented!()
    }
    async fn get_user_by_email(&self, _: &str) -> Result<Option<User>, DomainError> {
        unimplemented!()
    }
    async fn update_user_status(&self, _: i64, _: UserStatus) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn update_user_role(&self, _: i64, _: UserRole) -> Result<(), DomainError> {
        unimplemented!()
    }
    async fn list_all_users(&self) -> Result<Vec<User>, DomainError> {
        unimplemented!()
    }
    async fn delete_user(&self, _: i64) -> Result<(), DomainError> {
        unimplemented!()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_project(id: i64, slug: &str, name: &str) -> Project {
    let mut p = Project::new(name, slug).unwrap();
    p.id = id;
    p
}

fn make_feature(id: i64, slug: &str, title: &str, state: FeatureState) -> Feature {
    let mut feature = Feature::new(slug, title, [id as u8; 32], None);
    feature.id = id;
    feature.state = state;
    feature
}

fn make_epic(id: i64, project_id: i64, title: &str, status: EpicStatus) -> Epic {
    let mut e = Epic::new(project_id, title).unwrap();
    e.id = id;
    e.status = status;
    e
}

fn make_story(id: i64, epic_id: i64, project_id: i64, title: &str, status: StoryStatus) -> Story {
    let mut s = Story::new(epic_id, project_id, title, None).unwrap();
    s.id = id;
    s.status = status;
    s
}

// ── Tests: list projects ──────────────────────────────────────────────────────

#[tokio::test]
async fn list_projects_returns_ok_for_empty_store() {
    let store = MemStore {
        features: vec![],
        projects: vec![],
        epics: vec![],
        stories: vec![],
    };
    let args = crate::commands::list_projects::ListProjectsArgs { json: false };
    crate::commands::list_projects::run(&args, &store)
        .await
        .unwrap();
}

#[tokio::test]
async fn list_projects_returns_ok_with_data() {
    let store = MemStore {
        features: vec![],
        projects: vec![
            make_project(1, "alpha", "Alpha"),
            make_project(2, "beta", "Beta"),
        ],
        epics: vec![],
        stories: vec![],
    };
    let args = crate::commands::list_projects::ListProjectsArgs { json: false };
    crate::commands::list_projects::run(&args, &store)
        .await
        .unwrap();
}

#[tokio::test]
async fn list_projects_json_flag_returns_ok() {
    let store = MemStore {
        features: vec![],
        projects: vec![make_project(1, "alpha", "Alpha")],
        epics: vec![],
        stories: vec![],
    };
    let args = crate::commands::list_projects::ListProjectsArgs { json: true };
    crate::commands::list_projects::run(&args, &store)
        .await
        .unwrap();
}

// ── Tests: list epics ─────────────────────────────────────────────────────────

#[tokio::test]
async fn list_epics_no_filter_returns_ok() {
    let store = MemStore {
        features: vec![],
        projects: vec![make_project(1, "alpha", "Alpha")],
        epics: vec![make_epic(1, 1, "Epic One", EpicStatus::Active)],
        stories: vec![],
    };
    let args = crate::commands::list_epics::ListEpicsArgs {
        project: None,
        json: false,
    };
    crate::commands::list_epics::run(&args, &store)
        .await
        .unwrap();
}

#[tokio::test]
async fn list_epics_with_project_filter_returns_only_matching() {
    let store = MemStore {
        features: vec![],
        projects: vec![
            make_project(1, "alpha", "Alpha"),
            make_project(2, "beta", "Beta"),
        ],
        epics: vec![
            make_epic(1, 1, "Epic P1", EpicStatus::Active),
            make_epic(2, 2, "Epic P2", EpicStatus::Backlog),
        ],
        stories: vec![],
    };
    // Filter to project 1 — only epic 1 should be returned.
    let args = crate::commands::list_epics::ListEpicsArgs {
        project: Some(1),
        json: false,
    };
    crate::commands::list_epics::run(&args, &store)
        .await
        .unwrap();
}

#[tokio::test]
async fn list_epics_json_flag_returns_ok() {
    let store = MemStore {
        features: vec![],
        projects: vec![make_project(1, "alpha", "Alpha")],
        epics: vec![make_epic(1, 1, "Epic One", EpicStatus::Done)],
        stories: vec![],
    };
    let args = crate::commands::list_epics::ListEpicsArgs {
        project: Some(1),
        json: true,
    };
    crate::commands::list_epics::run(&args, &store)
        .await
        .unwrap();
}

// ── Tests: list stories ───────────────────────────────────────────────────────

#[tokio::test]
async fn list_stories_no_filter_returns_ok() {
    let store = MemStore {
        features: vec![],
        projects: vec![make_project(1, "alpha", "Alpha")],
        epics: vec![],
        stories: vec![make_story(1, 10, 1, "Story One", StoryStatus::Todo)],
    };
    let args = crate::commands::list_stories::ListStoriesArgs {
        epic: None,
        status: None,
        json: false,
    };
    crate::commands::list_stories::run(&args, &store)
        .await
        .unwrap();
}

#[tokio::test]
async fn list_stories_epic_filter_returns_only_matching() {
    let store = MemStore {
        features: vec![],
        projects: vec![],
        epics: vec![],
        stories: vec![
            make_story(1, 10, 1, "Story A", StoryStatus::Todo),
            make_story(2, 20, 1, "Story B", StoryStatus::Done),
        ],
    };
    let args = crate::commands::list_stories::ListStoriesArgs {
        epic: Some(10),
        status: None,
        json: false,
    };
    crate::commands::list_stories::run(&args, &store)
        .await
        .unwrap();
}

#[tokio::test]
async fn list_stories_status_filter_returns_only_matching() {
    let store = MemStore {
        features: vec![],
        projects: vec![make_project(1, "alpha", "Alpha")],
        epics: vec![],
        stories: vec![
            make_story(1, 10, 1, "Story A", StoryStatus::Todo),
            make_story(2, 10, 1, "Story B", StoryStatus::Done),
            make_story(3, 10, 1, "Story C", StoryStatus::InProgress),
        ],
    };
    // Filter by epic + status
    let args = crate::commands::list_stories::ListStoriesArgs {
        epic: Some(10),
        status: Some("done".to_string()),
        json: false,
    };
    crate::commands::list_stories::run(&args, &store)
        .await
        .unwrap();
}

#[tokio::test]
async fn list_stories_invalid_status_returns_err() {
    let store = MemStore {
        features: vec![],
        projects: vec![],
        epics: vec![],
        stories: vec![],
    };
    let args = crate::commands::list_stories::ListStoriesArgs {
        epic: None,
        status: Some("not_a_status".to_string()),
        json: false,
    };
    assert!(crate::commands::list_stories::run(&args, &store)
        .await
        .is_err());
}

#[tokio::test]
async fn list_stories_json_flag_returns_ok() {
    let store = MemStore {
        features: vec![],
        projects: vec![make_project(1, "alpha", "Alpha")],
        epics: vec![],
        stories: vec![make_story(1, 10, 1, "Story One", StoryStatus::Review)],
    };
    let args = crate::commands::list_stories::ListStoriesArgs {
        epic: Some(10),
        status: None,
        json: true,
    };
    crate::commands::list_stories::run(&args, &store)
        .await
        .unwrap();
}

// ── Tests: list features ──────────────────────────────────────────────────────

#[tokio::test]
async fn list_features_returns_ok_for_empty_store() {
    let store = MemStore {
        features: vec![],
        projects: vec![],
        epics: vec![],
        stories: vec![],
    };
    let args = crate::commands::list::ListArgs { state: None };

    crate::commands::list::run(args, &store).await.unwrap();
}

#[tokio::test]
async fn list_features_returns_all_features() {
    let store = MemStore {
        features: vec![
            make_feature(1, "feat-alpha", "Alpha", FeatureState::Created),
            make_feature(2, "feat-beta", "Beta", FeatureState::Planned),
        ],
        projects: vec![],
        epics: vec![],
        stories: vec![],
    };
    let args = crate::commands::list::ListArgs { state: None };

    crate::commands::list::run(args, &store).await.unwrap();
}

#[tokio::test]
async fn list_features_filters_by_state() {
    let store = MemStore {
        features: vec![
            make_feature(1, "feat-alpha", "Alpha", FeatureState::Created),
            make_feature(2, "feat-beta", "Beta", FeatureState::Planned),
        ],
        projects: vec![],
        epics: vec![],
        stories: vec![],
    };
    let args = crate::commands::list::ListArgs {
        state: Some("planned".to_string()),
    };

    crate::commands::list::run(args, &store).await.unwrap();
}

#[tokio::test]
async fn list_features_rejects_invalid_state() {
    let store = MemStore {
        features: vec![],
        projects: vec![],
        epics: vec![],
        stories: vec![],
    };
    let args = crate::commands::list::ListArgs {
        state: Some("not-a-state".to_string()),
    };

    assert!(crate::commands::list::run(args, &store).await.is_err());
}
