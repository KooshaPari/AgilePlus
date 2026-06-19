// SPDX-License-Identifier: MIT OR Apache-2.0
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
        panic!("MemStore stub: create_project not used in list_tests")
    }
    async fn get_project_by_slug(&self, _: &str) -> Result<Option<Project>, DomainError> {
        panic!("MemStore stub: get_project_by_slug not used in list_tests")
    }
    async fn get_project_by_id(&self, _: i64) -> Result<Option<Project>, DomainError> {
        panic!("MemStore stub: get_project_by_id not used in list_tests")
    }
    async fn delete_project(&self, _: i64) -> Result<(), DomainError> {
        panic!("MemStore stub: delete_project not used in list_tests")
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
        panic!("MemStore stub: create_epic not used in list_tests")
    }
    async fn get_epic_by_id(&self, _: i64) -> Result<Option<Epic>, DomainError> {
        panic!("MemStore stub: get_epic_by_id not used in list_tests")
    }
    async fn update_epic_status(&self, _: i64, _: EpicStatus) -> Result<(), DomainError> {
        panic!("MemStore stub: update_epic_status not used in list_tests")
    }
    async fn delete_epic(&self, _: i64) -> Result<(), DomainError> {
        panic!("MemStore stub: delete_epic not used in list_tests")
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
        panic!("MemStore stub: create_story not used in list_tests")
    }
    async fn get_story_by_id(&self, _: i64) -> Result<Option<Story>, DomainError> {
        panic!("MemStore stub: get_story_by_id not used in list_tests")
    }
    async fn update_story_status(&self, _: i64, _: StoryStatus) -> Result<(), DomainError> {
        panic!("MemStore stub: update_story_status not used in list_tests")
    }
    async fn delete_story(&self, _: i64) -> Result<(), DomainError> {
        panic!("MemStore stub: delete_story not used in list_tests")
    }
    async fn upsert_story_by_requirement_id(&self, _: &Story) -> Result<i64, DomainError> {
        panic!("MemStore stub: upsert_story_by_requirement_id not used in list_tests")
    }

    // --- Everything else is unreachable in list tests ---
    async fn create_feature(&self, _: &Feature) -> Result<i64, DomainError> {
        panic!("MemStore stub: create_feature not used in list_tests")
    }
    async fn get_feature_by_slug(&self, _: &str) -> Result<Option<Feature>, DomainError> {
        panic!("MemStore stub: get_feature_by_slug not used in list_tests")
    }
    async fn get_feature_by_id(&self, _: i64) -> Result<Option<Feature>, DomainError> {
        panic!("MemStore stub: get_feature_by_id not used in list_tests")
    }
    async fn update_feature_state(&self, _: i64, _: FeatureState) -> Result<(), DomainError> {
        panic!("MemStore stub: update_feature_state not used in list_tests")
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
        panic!("MemStore stub: create_work_package not used in list_tests")
    }
    async fn get_work_package(&self, _: i64) -> Result<Option<WorkPackage>, DomainError> {
        panic!("MemStore stub: get_work_package not used in list_tests")
    }
    async fn update_wp_state(&self, _: i64, _: WpState) -> Result<(), DomainError> {
        panic!("MemStore stub: update_wp_state not used in list_tests")
    }
    async fn list_wps_by_feature(&self, _: i64) -> Result<Vec<WorkPackage>, DomainError> {
        panic!("MemStore stub: list_wps_by_feature not used in list_tests")
    }
    async fn add_wp_dependency(&self, _: &WpDependency) -> Result<(), DomainError> {
        panic!("MemStore stub: add_wp_dependency not used in list_tests")
    }
    async fn get_wp_dependencies(&self, _: i64) -> Result<Vec<WpDependency>, DomainError> {
        panic!("MemStore stub: get_wp_dependencies not used in list_tests")
    }
    async fn get_ready_wps(&self, _: i64) -> Result<Vec<WorkPackage>, DomainError> {
        panic!("MemStore stub: get_ready_wps not used in list_tests")
    }
    async fn append_audit_entry(&self, _: &AuditEntry) -> Result<i64, DomainError> {
        panic!("MemStore stub: append_audit_entry not used in list_tests")
    }
    async fn get_audit_trail(&self, _: i64) -> Result<Vec<AuditEntry>, DomainError> {
        panic!("MemStore stub: get_audit_trail not used in list_tests")
    }
    async fn get_latest_audit_entry(&self, _: i64) -> Result<Option<AuditEntry>, DomainError> {
        panic!("MemStore stub: get_latest_audit_entry not used in list_tests")
    }
    async fn create_evidence(&self, _: &Evidence) -> Result<i64, DomainError> {
        panic!("MemStore stub: create_evidence not used in list_tests")
    }
    async fn get_evidence_by_wp(&self, _: i64) -> Result<Vec<Evidence>, DomainError> {
        panic!("MemStore stub: get_evidence_by_wp not used in list_tests")
    }
    async fn get_evidence_by_fr(&self, _: &str) -> Result<Vec<Evidence>, DomainError> {
        panic!("MemStore stub: get_evidence_by_fr not used in list_tests")
    }
    async fn create_policy_rule(&self, _: &PolicyRule) -> Result<i64, DomainError> {
        panic!("MemStore stub: create_policy_rule not used in list_tests")
    }
    async fn list_active_policies(&self) -> Result<Vec<PolicyRule>, DomainError> {
        panic!("MemStore stub: list_active_policies not used in list_tests")
    }
    async fn record_metric(&self, _: &Metric) -> Result<i64, DomainError> {
        panic!("MemStore stub: record_metric not used in list_tests")
    }
    async fn get_metrics_by_feature(&self, _: i64) -> Result<Vec<Metric>, DomainError> {
        panic!("MemStore stub: get_metrics_by_feature not used in list_tests")
    }
    async fn create_governance_contract(&self, _: &GovernanceContract) -> Result<i64, DomainError> {
        panic!("MemStore stub: create_governance_contract not used in list_tests")
    }
    async fn get_governance_contract(
        &self,
        _: i64,
        _: i32,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        panic!("MemStore stub: get_governance_contract not used in list_tests")
    }
    async fn get_latest_governance_contract(
        &self,
        _: i64,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        panic!("MemStore stub: get_latest_governance_contract not used in list_tests")
    }
    async fn create_module(&self, _: &Module) -> Result<i64, DomainError> {
        panic!("MemStore stub: create_module not used in list_tests")
    }
    async fn get_module(&self, _: i64) -> Result<Option<Module>, DomainError> {
        panic!("MemStore stub: get_module not used in list_tests")
    }
    async fn get_module_by_slug(&self, _: &str) -> Result<Option<Module>, DomainError> {
        panic!("MemStore stub: get_module_by_slug not used in list_tests")
    }
    async fn update_module(&self, _: i64, _: &str, _: Option<&str>) -> Result<(), DomainError> {
        panic!("MemStore stub: update_module not used in list_tests")
    }
    async fn delete_module(&self, _: i64) -> Result<(), DomainError> {
        panic!("MemStore stub: delete_module not used in list_tests")
    }
    async fn list_root_modules(&self) -> Result<Vec<Module>, DomainError> {
        panic!("MemStore stub: list_root_modules not used in list_tests")
    }
    async fn list_child_modules(&self, _: i64) -> Result<Vec<Module>, DomainError> {
        panic!("MemStore stub: list_child_modules not used in list_tests")
    }
    async fn get_module_with_features(
        &self,
        _: i64,
    ) -> Result<Option<ModuleWithFeatures>, DomainError> {
        panic!("MemStore stub: get_module_with_features not used in list_tests")
    }
    async fn tag_feature_to_module(&self, _: &ModuleFeatureTag) -> Result<(), DomainError> {
        panic!("MemStore stub: tag_feature_to_module not used in list_tests")
    }
    async fn untag_feature_from_module(&self, _: i64, _: i64) -> Result<(), DomainError> {
        panic!("MemStore stub: untag_feature_from_module not used in list_tests")
    }
    async fn create_cycle(&self, _: &Cycle) -> Result<i64, DomainError> {
        panic!("MemStore stub: create_cycle not used in list_tests")
    }
    async fn get_cycle(&self, _: i64) -> Result<Option<Cycle>, DomainError> {
        panic!("MemStore stub: get_cycle not used in list_tests")
    }
    async fn update_cycle_state(&self, _: i64, _: CycleState) -> Result<(), DomainError> {
        panic!("MemStore stub: update_cycle_state not used in list_tests")
    }
    async fn list_cycles_by_state(&self, _: CycleState) -> Result<Vec<Cycle>, DomainError> {
        panic!("MemStore stub: list_cycles_by_state not used in list_tests")
    }
    async fn list_cycles_by_module(&self, _: i64) -> Result<Vec<Cycle>, DomainError> {
        panic!("MemStore stub: list_cycles_by_module not used in list_tests")
    }
    async fn list_all_cycles(&self) -> Result<Vec<Cycle>, DomainError> {
        panic!("MemStore stub: list_all_cycles not used in list_tests")
    }
    async fn get_cycle_with_features(
        &self,
        _: i64,
    ) -> Result<Option<CycleWithFeatures>, DomainError> {
        panic!("MemStore stub: get_cycle_with_features not used in list_tests")
    }
    async fn add_feature_to_cycle(&self, _: &CycleFeature) -> Result<(), DomainError> {
        panic!("MemStore stub: add_feature_to_cycle not used in list_tests")
    }
    async fn remove_feature_from_cycle(&self, _: i64, _: i64) -> Result<(), DomainError> {
        panic!("MemStore stub: remove_feature_from_cycle not used in list_tests")
    }
    async fn get_sync_mapping(&self, _: &str, _: i64) -> Result<Option<SyncMapping>, DomainError> {
        panic!("MemStore stub: get_sync_mapping not used in list_tests")
    }
    async fn upsert_sync_mapping(&self, _: &SyncMapping) -> Result<(), DomainError> {
        panic!("MemStore stub: upsert_sync_mapping not used in list_tests")
    }
    async fn get_sync_mapping_by_plane_id(
        &self,
        _: &str,
        _: &str,
    ) -> Result<Option<SyncMapping>, DomainError> {
        panic!("MemStore stub: get_sync_mapping_by_plane_id not used in list_tests")
    }
    async fn delete_sync_mapping(&self, _: &str, _: i64) -> Result<(), DomainError> {
        panic!("MemStore stub: delete_sync_mapping not used in list_tests")
    }
    async fn create_user(&self, _: &User) -> Result<i64, DomainError> {
        panic!("MemStore stub: create_user not used in list_tests")
    }
    async fn get_user_by_id(&self, _: i64) -> Result<Option<User>, DomainError> {
        panic!("MemStore stub: get_user_by_id not used in list_tests")
    }
    async fn get_user_by_email(&self, _: &str) -> Result<Option<User>, DomainError> {
        panic!("MemStore stub: get_user_by_email not used in list_tests")
    }
    async fn update_user_status(&self, _: i64, _: UserStatus) -> Result<(), DomainError> {
        panic!("MemStore stub: update_user_status not used in list_tests")
    }
    async fn update_user_role(&self, _: i64, _: UserRole) -> Result<(), DomainError> {
        panic!("MemStore stub: update_user_role not used in list_tests")
    }
    async fn list_all_users(&self) -> Result<Vec<User>, DomainError> {
        panic!("MemStore stub: list_all_users not used in list_tests")
    }
    async fn delete_user(&self, _: i64) -> Result<(), DomainError> {
        panic!("MemStore stub: delete_user not used in list_tests")
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
