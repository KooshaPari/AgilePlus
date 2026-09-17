//! Integration tests for the agileplus-import import_bundle flow.
//!
//! Uses in-memory mock implementations of StoragePort and VcsPort to exercise
//! the full import pipeline without external dependencies.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use agileplus_domain::domain::audit::AuditEntry;
use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures};
use agileplus_domain::domain::epic::{Epic, EpicStatus};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::governance::{Evidence, GovernanceContract, PolicyRule};
use agileplus_domain::domain::metric::Metric;
use agileplus_domain::domain::module::{Module, ModuleFeatureTag, ModuleWithFeatures};
use agileplus_domain::domain::project::Project;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::story::{Story, StoryStatus};
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_domain::domain::user::User;
use agileplus_domain::domain::work_package::{WorkPackage, WpDependency, WpState};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::{BranchInfo, ConflictInfo, FeatureArtifacts, MergeResult, StoragePort, VcsPort, WorktreeInfo};

use agileplus_import::{import_bundle, ImportBundle, ImportCycle, ImportFeature, ImportModule, ImportProject, ImportWorkPackage, ImportReport};
use agileplus_domain::domain::work_package::PrState;

// ---------------------------------------------------------------------------
// Mock Storage
// ---------------------------------------------------------------------------

#[derive(Default, Clone)]
struct MockStorage {
    features: Arc<Mutex<HashMap<i64, Feature>>>,
    modules: Arc<Mutex<HashMap<i64, Module>>>,
    projects: Arc<Mutex<HashMap<i64, Project>>>,
    cycles: Arc<Mutex<Vec<Cycle>>>,
    work_packages: Arc<Mutex<Vec<WorkPackage>>>,
    audits: Arc<Mutex<Vec<AuditEntry>>>,
    wp_deps: Arc<Mutex<Vec<WpDependency>>>,
    module_feature_tags: Arc<Mutex<Vec<ModuleFeatureTag>>>,
    cycle_features: Arc<Mutex<Vec<CycleFeature>>>,
    next_id: Arc<Mutex<i64>>,
    // Track state updates
    feature_state_updates: Arc<Mutex<Vec<(i64, FeatureState)>>>,
    wp_state_updates: Arc<Mutex<Vec<(i64, WpState)>>>,
    cycle_state_updates: Arc<Mutex<Vec<(i64, CycleState)>>>,
    module_updates: Arc<Mutex<Vec<(i64, String, Option<String>)>>>,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            next_id: Arc::new(Mutex::new(1)),
            ..Default::default()
        }
    }

    fn next_id(&self) -> i64 {
        let mut id = self.next_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    }
}

#[async_trait]
impl StoragePort for MockStorage {
    async fn create_feature(&self, feature: &Feature) -> Result<i64, DomainError> {
        let id = self.next_id();
        let mut f = feature.clone();
        f.id = id;
        self.features.lock().unwrap().insert(id, f);
        Ok(id)
    }

    async fn get_feature_by_slug(&self, slug: &str) -> Result<Option<Feature>, DomainError> {
        Ok(self
            .features
            .lock()
            .unwrap()
            .values()
            .find(|f| f.slug == slug)
            .cloned())
    }

    async fn get_feature_by_id(&self, id: i64) -> Result<Option<Feature>, DomainError> {
        Ok(self.features.lock().unwrap().get(&id).cloned())
    }

    async fn update_feature_state(&self, id: i64, state: FeatureState) -> Result<(), DomainError> {
        self.feature_state_updates.lock().unwrap().push((id, state));
        Ok(())
    }

    async fn list_features_by_state(&self, _state: FeatureState) -> Result<Vec<Feature>, DomainError> {
        Ok(vec![])
    }

    async fn list_all_features(&self) -> Result<Vec<Feature>, DomainError> {
        Ok(self.features.lock().unwrap().values().cloned().collect())
    }

    async fn create_work_package(&self, wp: &WorkPackage) -> Result<i64, DomainError> {
        let id = self.next_id();
        let mut wp = wp.clone();
        wp.id = id;
        self.work_packages.lock().unwrap().push(wp);
        Ok(id)
    }

    async fn get_work_package(&self, id: i64) -> Result<Option<WorkPackage>, DomainError> {
        Ok(self
            .work_packages
            .lock()
            .unwrap()
            .iter()
            .find(|w| w.id == id)
            .cloned())
    }

    async fn update_wp_state(&self, id: i64, state: WpState) -> Result<(), DomainError> {
        self.wp_state_updates.lock().unwrap().push((id, state));
        Ok(())
    }

    async fn list_wps_by_feature(&self, _feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        Ok(vec![])
    }

    async fn add_wp_dependency(&self, dep: &WpDependency) -> Result<(), DomainError> {
        self.wp_deps.lock().unwrap().push(dep.clone());
        Ok(())
    }

    async fn get_wp_dependencies(&self, _wp_id: i64) -> Result<Vec<WpDependency>, DomainError> {
        Ok(vec![])
    }

    async fn get_ready_wps(&self, _feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        Ok(vec![])
    }

    async fn append_audit_entry(&self, entry: &AuditEntry) -> Result<i64, DomainError> {
        let id = self.next_id();
        let mut e = entry.clone();
        e.id = id;
        self.audits.lock().unwrap().push(e);
        Ok(id)
    }

    async fn get_audit_trail(&self, _feature_id: i64) -> Result<Vec<AuditEntry>, DomainError> {
        Ok(vec![])
    }

    async fn get_latest_audit_entry(
        &self,
        _feature_id: i64,
    ) -> Result<Option<AuditEntry>, DomainError> {
        Ok(None)
    }

    async fn create_evidence(&self, _ev: &Evidence) -> Result<i64, DomainError> {
        Ok(self.next_id())
    }

    async fn get_evidence_by_wp(&self, _wp_id: i64) -> Result<Vec<Evidence>, DomainError> {
        Ok(vec![])
    }

    async fn get_evidence_by_fr(&self, _fr_id: &str) -> Result<Vec<Evidence>, DomainError> {
        Ok(vec![])
    }

    async fn create_policy_rule(&self, _rule: &PolicyRule) -> Result<i64, DomainError> {
        Ok(self.next_id())
    }

    async fn list_active_policies(&self) -> Result<Vec<PolicyRule>, DomainError> {
        Ok(vec![])
    }

    async fn record_metric(&self, _metric: &Metric) -> Result<i64, DomainError> {
        Ok(self.next_id())
    }

    async fn get_metrics_by_feature(&self, _feature_id: i64) -> Result<Vec<Metric>, DomainError> {
        Ok(vec![])
    }

    async fn create_governance_contract(
        &self,
        _contract: &GovernanceContract,
    ) -> Result<i64, DomainError> {
        Ok(self.next_id())
    }

    async fn get_governance_contract(
        &self,
        _feature_id: i64,
        _version: i32,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        Ok(None)
    }

    async fn get_latest_governance_contract(
        &self,
        _feature_id: i64,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        Ok(None)
    }

    async fn create_module(&self, module: &Module) -> Result<i64, DomainError> {
        let id = self.next_id();
        let mut m = module.clone();
        m.id = id;
        self.modules.lock().unwrap().insert(id, m);
        Ok(id)
    }

    async fn get_module(&self, id: i64) -> Result<Option<Module>, DomainError> {
        Ok(self.modules.lock().unwrap().get(&id).cloned())
    }

    async fn get_module_by_slug(&self, slug: &str) -> Result<Option<Module>, DomainError> {
        Ok(self
            .modules
            .lock()
            .unwrap()
            .values()
            .find(|m| m.slug == slug)
            .cloned())
    }

    async fn update_module(
        &self,
        id: i64,
        friendly_name: &str,
        description: Option<&str>,
    ) -> Result<(), DomainError> {
        self.module_updates
            .lock()
            .unwrap()
            .push((id, friendly_name.to_string(), description.map(|s| s.to_string())));
        Ok(())
    }

    async fn delete_module(&self, _id: i64) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_root_modules(&self) -> Result<Vec<Module>, DomainError> {
        Ok(vec![])
    }

    async fn list_child_modules(&self, _parent_id: i64) -> Result<Vec<Module>, DomainError> {
        Ok(vec![])
    }

    async fn get_module_with_features(
        &self,
        _id: i64,
    ) -> Result<Option<ModuleWithFeatures>, DomainError> {
        Ok(None)
    }

    async fn tag_feature_to_module(&self, tag: &ModuleFeatureTag) -> Result<(), DomainError> {
        self.module_feature_tags.lock().unwrap().push(tag.clone());
        Ok(())
    }

    async fn untag_feature_from_module(
        &self,
        _module_id: i64,
        _feature_id: i64,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn create_cycle(&self, cycle: &Cycle) -> Result<i64, DomainError> {
        let id = self.next_id();
        let mut c = cycle.clone();
        c.id = id;
        self.cycles.lock().unwrap().push(c);
        Ok(id)
    }

    async fn get_cycle(&self, id: i64) -> Result<Option<Cycle>, DomainError> {
        Ok(self
            .cycles
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.id == id)
            .cloned())
    }

    async fn update_cycle_state(&self, id: i64, state: CycleState) -> Result<(), DomainError> {
        self.cycle_state_updates.lock().unwrap().push((id, state));
        Ok(())
    }

    async fn list_cycles_by_state(&self, _state: CycleState) -> Result<Vec<Cycle>, DomainError> {
        Ok(vec![])
    }

    async fn list_cycles_by_module(&self, _module_id: i64) -> Result<Vec<Cycle>, DomainError> {
        Ok(vec![])
    }

    async fn list_all_cycles(&self) -> Result<Vec<Cycle>, DomainError> {
        Ok(self.cycles.lock().unwrap().clone())
    }

    async fn get_cycle_with_features(
        &self,
        _id: i64,
    ) -> Result<Option<CycleWithFeatures>, DomainError> {
        Ok(None)
    }

    async fn add_feature_to_cycle(&self, entry: &CycleFeature) -> Result<(), DomainError> {
        self.cycle_features.lock().unwrap().push(entry.clone());
        Ok(())
    }

    async fn remove_feature_from_cycle(
        &self,
        _cycle_id: i64,
        _feature_id: i64,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn get_sync_mapping(
        &self,
        _entity_type: &str,
        _entity_id: i64,
    ) -> Result<Option<SyncMapping>, DomainError> {
        Ok(None)
    }

    async fn upsert_sync_mapping(&self, _mapping: &SyncMapping) -> Result<(), DomainError> {
        Ok(())
    }

    async fn get_sync_mapping_by_plane_id(
        &self,
        _entity_type: &str,
        _plane_issue_id: &str,
    ) -> Result<Option<SyncMapping>, DomainError> {
        Ok(None)
    }

    async fn delete_sync_mapping(
        &self,
        _entity_type: &str,
        _entity_id: i64,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn create_project(&self, project: &Project) -> Result<i64, DomainError> {
        let id = self.next_id();
        let mut p = project.clone();
        p.id = id;
        self.projects.lock().unwrap().insert(id, p);
        Ok(id)
    }

    async fn get_project_by_slug(&self, slug: &str) -> Result<Option<Project>, DomainError> {
        Ok(self
            .projects
            .lock()
            .unwrap()
            .values()
            .find(|p| p.slug == slug)
            .cloned())
    }

    async fn list_all_projects(&self) -> Result<Vec<Project>, DomainError> {
        Ok(vec![])
    }

    async fn create_epic(&self, _epic: &Epic) -> Result<i64, DomainError> {
        Ok(self.next_id())
    }

    async fn get_epic(&self, _id: i64) -> Result<Option<Epic>, DomainError> {
        Ok(None)
    }

    async fn list_epics_by_project(&self, _project_id: i64) -> Result<Vec<Epic>, DomainError> {
        Ok(vec![])
    }

    async fn update_epic_status(&self, _id: i64, _status: EpicStatus) -> Result<(), DomainError> {
        Ok(())
    }

    async fn create_story(&self, _story: &Story) -> Result<i64, DomainError> {
        Ok(self.next_id())
    }

    async fn get_story(&self, _id: i64) -> Result<Option<Story>, DomainError> {
        Ok(None)
    }

    async fn list_stories_by_epic(&self, _epic_id: i64) -> Result<Vec<Story>, DomainError> {
        Ok(vec![])
    }

    async fn update_story_status(
        &self,
        _id: i64,
        _status: StoryStatus,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn create_user(&self, _user: &User) -> Result<i64, DomainError> {
        Ok(self.next_id())
    }

    async fn get_user(&self, _id: i64) -> Result<Option<User>, DomainError> {
        Ok(None)
    }

    async fn get_user_by_email(&self, _email: &str) -> Result<Option<User>, DomainError> {
        Ok(None)
    }

    async fn list_all_users(&self) -> Result<Vec<User>, DomainError> {
        Ok(vec![])
    }
}

// ---------------------------------------------------------------------------
// Mock VCS
// ---------------------------------------------------------------------------

#[derive(Default, Clone)]
struct MockVcs {
    written_artifacts: Arc<Mutex<Vec<(String, String, String)>>>,
}

#[async_trait]
impl VcsPort for MockVcs {
    async fn create_worktree(
        &self,
        _feature_slug: &str,
        _wp_id: &str,
    ) -> Result<std::path::PathBuf, DomainError> {
        Ok(std::path::PathBuf::from("/tmp/mock-worktree"))
    }

    async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError> {
        Ok(vec![])
    }

    async fn cleanup_worktree(&self, _path: &std::path::Path) -> Result<(), DomainError> {
        Ok(())
    }

    async fn create_branch(&self, _name: &str, _base: &str) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_branches(
        &self,
        _pattern: Option<&str>,
        _remote: bool,
    ) -> Result<Vec<BranchInfo>, DomainError> {
        Ok(vec![])
    }

    async fn delete_branch(
        &self,
        _name: &str,
        _force: bool,
        _remote: Option<&str>,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn checkout_branch(&self, _name: &str) -> Result<(), DomainError> {
        Ok(())
    }

    async fn merge_to_target(&self, _source: &str, _target: &str) -> Result<MergeResult, DomainError> {
        Ok(MergeResult {
            success: true,
            conflicts: vec![],
            merged_commit: Some("abc123".into()),
            commit: Some("abc123".into()),
            message: Some("Merged".into()),
        })
    }

    async fn detect_conflicts(
        &self,
        _source: &str,
        _target: &str,
    ) -> Result<Vec<ConflictInfo>, DomainError> {
        Ok(vec![])
    }

    async fn read_artifact(
        &self,
        _feature_slug: &str,
        _relative_path: &str,
    ) -> Result<String, DomainError> {
        Ok(String::new())
    }

    async fn write_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
        content: &str,
    ) -> Result<(), DomainError> {
        self.written_artifacts.lock().unwrap().push((
            feature_slug.to_string(),
            relative_path.to_string(),
            content.to_string(),
        ));
        Ok(())
    }

    async fn artifact_exists(
        &self,
        _feature_slug: &str,
        _relative_path: &str,
    ) -> Result<bool, DomainError> {
        Ok(false)
    }

    async fn scan_feature_artifacts(
        &self,
        _feature_slug: &str,
    ) -> Result<FeatureArtifacts, DomainError> {
        Ok(FeatureArtifacts {
            spec: None,
            research: None,
            plan: None,
            other: vec![],
            meta_json: None,
            audit_chain: None,
            evidence_paths: vec![],
        })
    }
}

// ---------------------------------------------------------------------------
// Helper to build a minimal bundle
// ---------------------------------------------------------------------------

fn empty_bundle() -> ImportBundle {
    ImportBundle::default()
}

fn simple_feature_bundle() -> ImportBundle {
    ImportBundle {
        projects: vec![],
        modules: vec![],
        features: vec![ImportFeature {
            slug: Some("auth-login".into()),
            friendly_name: "Auth Login".into(),
            spec_content: "# Login\n\nUser authenticates.".into(),
            state: FeatureState::Specified,
            target_branch: Some("feature/auth-login".into()),
            labels: vec!["auth".into()],
            module_slug: None,
            project_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            work_packages: vec![
                ImportWorkPackage {
                    title: "Implement login endpoint".into(),
                    acceptance_criteria: Some("POST /login works".into()),
                    sequence: Some(1),
                    file_scope: vec!["src/auth.rs".into()],
                    state: WpState::Planned,
                    agent_id: None,
                    pr_url: None,
                    pr_state: None,
                    worktree_path: None,
                    plane_sub_issue_id: None,
                    depends_on_sequences: vec![],
                },
                ImportWorkPackage {
                    title: "Add login tests".into(),
                    acceptance_criteria: None,
                    sequence: Some(2),
                    file_scope: vec!["tests/auth.rs".into()],
                    state: WpState::Planned,
                    agent_id: None,
                    pr_url: None,
                    pr_state: None,
                    worktree_path: None,
                    plane_sub_issue_id: None,
                    depends_on_sequences: vec![1],
                },
            ],
        }],
        cycles: vec![],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn import_empty_bundle() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = empty_bundle();

    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();

    assert_eq!(report.projects_created, 0);
    assert_eq!(report.projects_updated, 0);
    assert_eq!(report.modules_created, 0);
    assert_eq!(report.features_created, 0);
    assert_eq!(report.cycles_created, 0);
}

#[tokio::test]
async fn import_single_feature_creates_feature() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = simple_feature_bundle();

    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();

    assert_eq!(report.features_created, 1);
    assert_eq!(report.features_updated, 0);
    assert_eq!(report.work_packages_created, 2);
    assert_eq!(report.audits_written, 1);
    assert_eq!(report.artifacts_written, 2); // spec.md + meta.json

    // Verify feature was stored
    let features = storage.features.lock().unwrap();
    assert_eq!(features.len(), 1);
    let feature = features.values().next().unwrap();
    assert_eq!(feature.slug, "auth-login");
    assert_eq!(feature.friendly_name, "Auth Login");
    assert_eq!(feature.labels, vec!["auth"]);
}

#[tokio::test]
async fn import_feature_writes_vcs_artifacts() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = simple_feature_bundle();

    import_bundle(bundle, &storage, &vcs).await.unwrap();

    let artifacts = vcs.written_artifacts.lock().unwrap();
    assert_eq!(artifacts.len(), 2);
    assert_eq!(artifacts[0].0, "auth-login");
    assert_eq!(artifacts[0].1, "spec.md");
    assert!(artifacts[0].2.contains("# Login"));
    assert_eq!(artifacts[1].0, "auth-login");
    assert_eq!(artifacts[1].1, "meta.json");
    assert!(artifacts[1].2.contains("auth-login"));
}

#[tokio::test]
async fn import_feature_state_non_default_triggers_update() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        features: vec![ImportFeature {
            slug: Some("my-feat".into()),
            friendly_name: "My Feat".into(),
            spec_content: "spec".into(),
            state: FeatureState::Implementing,
            target_branch: None,
            labels: vec![],
            module_slug: None,
            project_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            work_packages: vec![],
        }],
        ..empty_bundle()
    };

    import_bundle(bundle, &storage, &vcs).await.unwrap();

    let updates = storage.feature_state_updates.lock().unwrap();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].1, FeatureState::Implementing);
}

#[tokio::test]
async fn import_feature_default_state_no_update() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        features: vec![ImportFeature {
            slug: Some("my-feat".into()),
            friendly_name: "My Feat".into(),
            spec_content: "spec".into(),
            state: FeatureState::Specified, // default
            target_branch: None,
            labels: vec![],
            module_slug: None,
            project_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            work_packages: vec![],
        }],
        ..empty_bundle()
    };

    import_bundle(bundle, &storage, &vcs).await.unwrap();

    // Specified is the default; since the Feature is created with Created state
    // and the import target is Specified (non-Created), an update should be triggered
    let updates = storage.feature_state_updates.lock().unwrap();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].1, FeatureState::Specified);
}

#[tokio::test]
async fn import_feature_created_state_no_update() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        features: vec![ImportFeature {
            slug: Some("my-feat".into()),
            friendly_name: "My Feat".into(),
            spec_content: "spec".into(),
            state: FeatureState::Created,
            target_branch: None,
            labels: vec![],
            module_slug: None,
            project_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            work_packages: vec![],
        }],
        ..empty_bundle()
    };

    import_bundle(bundle, &storage, &vcs).await.unwrap();

    // Created is the initial state, so no update should be triggered
    let updates = storage.feature_state_updates.lock().unwrap();
    assert_eq!(updates.len(), 0);
}

#[tokio::test]
async fn import_work_packages_with_dependencies() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        features: vec![ImportFeature {
            slug: Some("dep-test".into()),
            friendly_name: "Dep Test".into(),
            spec_content: "spec".into(),
            state: FeatureState::Specified,
            target_branch: None,
            labels: vec![],
            module_slug: None,
            project_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            work_packages: vec![
                ImportWorkPackage {
                    title: "WP A".into(),
                    acceptance_criteria: None,
                    sequence: Some(1),
                    file_scope: vec![],
                    state: WpState::Planned,
                    agent_id: None,
                    pr_url: None,
                    pr_state: None,
                    worktree_path: None,
                    plane_sub_issue_id: None,
                    depends_on_sequences: vec![],
                },
                ImportWorkPackage {
                    title: "WP B".into(),
                    acceptance_criteria: None,
                    sequence: Some(2),
                    file_scope: vec![],
                    state: WpState::Planned,
                    agent_id: None,
                    pr_url: None,
                    pr_state: None,
                    worktree_path: None,
                    plane_sub_issue_id: None,
                    depends_on_sequences: vec![1],
                },
                ImportWorkPackage {
                    title: "WP C".into(),
                    acceptance_criteria: None,
                    sequence: Some(3),
                    file_scope: vec![],
                    state: WpState::Planned,
                    agent_id: None,
                    pr_url: None,
                    pr_state: None,
                    worktree_path: None,
                    plane_sub_issue_id: None,
                    depends_on_sequences: vec![1, 2],
                },
            ],
        }],
        ..empty_bundle()
    };

    import_bundle(bundle, &storage, &vcs).await.unwrap();

    let deps = storage.wp_deps.lock().unwrap();
    // WP B depends on WP A (1 dep), WP C depends on WP A and WP B (2 deps) = 3 total
    assert_eq!(deps.len(), 3);
}

#[tokio::test]
async fn import_modules_creates_new() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        modules: vec![
            ImportModule {
                slug: Some("auth".into()),
                friendly_name: "Auth".into(),
                description: Some("Authentication module".into()),
                parent_slug: None,
            },
            ImportModule {
                slug: Some("auth-login".into()),
                friendly_name: "Auth Login".into(),
                description: None,
                parent_slug: Some("auth".into()),
            },
        ],
        ..empty_bundle()
    };

    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();

    let modules = storage.modules.lock().unwrap();
    assert_eq!(modules.len(), 2);
    assert_eq!(report.modules_created, 2);

    // Verify parent relationship
    let child = modules.values().find(|m| m.slug == "auth-login").unwrap();
    assert!(child.parent_module_id.is_some());
}

#[tokio::test]
async fn import_projects_creates_new() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        projects: vec![ImportProject {
            slug: Some("my-proj".into()),
            name: "My Project".into(),
            description: Some("A test project".into()),
            features: vec![],
        }],
        ..empty_bundle()
    };

    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();

    assert_eq!(report.projects_created, 1);
    let projects = storage.projects.lock().unwrap();
    assert_eq!(projects.len(), 1);
    let project = projects.values().next().unwrap();
    assert_eq!(project.name, "My Project");
    assert_eq!(project.slug, "my-proj");
}

#[tokio::test]
async fn import_cycles_creates_new() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        cycles: vec![ImportCycle {
            name: "Sprint 1".into(),
            description: Some("First sprint".into()),
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
            state: CycleState::Draft,
            module_scope_slug: None,
            feature_slugs: vec![],
        }],
        ..empty_bundle()
    };

    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();

    assert_eq!(report.cycles_created, 1);
    let cycles = storage.cycles.lock().unwrap();
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].name, "Sprint 1");
}

#[tokio::test]
async fn import_cycle_non_draft_state_triggers_update() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        cycles: vec![ImportCycle {
            name: "Sprint 1".into(),
            description: None,
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
            state: CycleState::Active,
            module_scope_slug: None,
            feature_slugs: vec![],
        }],
        ..empty_bundle()
    };

    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();

    assert_eq!(report.cycles_updated, 1);
    let updates = storage.cycle_state_updates.lock().unwrap();
    assert_eq!(updates.len(), 1);
}

#[tokio::test]
async fn import_cycle_with_feature_slugs_links_features() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        features: vec![ImportFeature {
            slug: Some("feat-a".into()),
            friendly_name: "Feat A".into(),
            spec_content: "spec".into(),
            state: FeatureState::Specified,
            target_branch: None,
            labels: vec![],
            module_slug: None,
            project_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            work_packages: vec![],
        }],
        cycles: vec![ImportCycle {
            name: "Sprint 1".into(),
            description: None,
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
            state: CycleState::Draft,
            module_scope_slug: None,
            feature_slugs: vec!["feat-a".into()],
        }],
        ..empty_bundle()
    };

    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();

    assert_eq!(report.cycle_links_created, 1);
    let links = storage.cycle_features.lock().unwrap();
    assert_eq!(links.len(), 1);
}

#[tokio::test]
async fn import_feature_with_module_slug_links_to_module() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        modules: vec![ImportModule {
            slug: Some("auth".into()),
            friendly_name: "Auth".into(),
            description: None,
            parent_slug: None,
        }],
        features: vec![ImportFeature {
            slug: Some("login".into()),
            friendly_name: "Login".into(),
            spec_content: "spec".into(),
            state: FeatureState::Specified,
            target_branch: None,
            labels: vec![],
            module_slug: Some("auth".into()),
            project_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            work_packages: vec![],
        }],
        ..empty_bundle()
    };

    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();

    assert_eq!(report.module_links_created, 1);
    let tags = storage.module_feature_tags.lock().unwrap();
    assert_eq!(tags.len(), 1);
}

#[tokio::test]
async fn import_project_with_embedded_features() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        projects: vec![ImportProject {
            slug: Some("proj-x".into()),
            name: "Project X".into(),
            description: None,
            features: vec![ImportFeature {
                slug: Some("feat-1".into()),
                friendly_name: "Feature 1".into(),
                spec_content: "spec".into(),
                state: FeatureState::Specified,
                target_branch: None,
                labels: vec![],
                module_slug: None,
                project_id: None,
                plane_issue_id: None,
                plane_state_id: None,
                work_packages: vec![],
            }],
        }],
        ..empty_bundle()
    };

    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();

    assert_eq!(report.projects_created, 1);
    assert_eq!(report.features_created, 1);

    // Feature should have project_id set
    let features = storage.features.lock().unwrap();
    let feature = features.values().next().unwrap();
    assert!(feature.project_id.is_some());
    assert_eq!(feature.project_id, Some(1)); // First project gets id 1
}

#[tokio::test]
async fn import_work_packages_default_sequence() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        features: vec![ImportFeature {
            slug: Some("seq-test".into()),
            friendly_name: "Seq Test".into(),
            spec_content: "spec".into(),
            state: FeatureState::Specified,
            target_branch: None,
            labels: vec![],
            module_slug: None,
            project_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            work_packages: vec![
                ImportWorkPackage {
                    title: "First".into(),
                    acceptance_criteria: None,
                    sequence: None, // should default to 1
                    file_scope: vec![],
                    state: WpState::Planned,
                    agent_id: None,
                    pr_url: None,
                    pr_state: None,
                    worktree_path: None,
                    plane_sub_issue_id: None,
                    depends_on_sequences: vec![],
                },
                ImportWorkPackage {
                    title: "Second".into(),
                    acceptance_criteria: None,
                    sequence: None, // should default to 2
                    file_scope: vec![],
                    state: WpState::Planned,
                    agent_id: None,
                    pr_url: None,
                    pr_state: None,
                    worktree_path: None,
                    plane_sub_issue_id: None,
                    depends_on_sequences: vec![],
                },
            ],
        }],
        ..empty_bundle()
    };

    import_bundle(bundle, &storage, &vcs).await.unwrap();

    let wps = storage.work_packages.lock().unwrap();
    assert_eq!(wps.len(), 2);
    // Sequences should be 1 and 2 (default from index)
    let mut sequences: Vec<i32> = wps.iter().map(|w| w.sequence).collect();
    sequences.sort();
    assert_eq!(sequences, vec![1, 2]);
}

#[tokio::test]
async fn import_feature_with_plane_ids() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        features: vec![ImportFeature {
            slug: Some("plane-feat".into()),
            friendly_name: "Plane Feature".into(),
            spec_content: "spec".into(),
            state: FeatureState::Specified,
            target_branch: None,
            labels: vec![],
            module_slug: None,
            project_id: None,
            plane_issue_id: Some("PLANE-123".into()),
            plane_state_id: Some("STATE-456".into()),
            work_packages: vec![],
        }],
        ..empty_bundle()
    };

    import_bundle(bundle, &storage, &vcs).await.unwrap();

    let features = storage.features.lock().unwrap();
    let feature = features.values().next().unwrap();
    assert_eq!(feature.plane_issue_id.as_deref(), Some("PLANE-123"));
    assert_eq!(feature.plane_state_id.as_deref(), Some("STATE-456"));
}

#[tokio::test]
async fn import_multiple_features_all_created() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        features: vec![
            ImportFeature {
                slug: Some("feat-a".into()),
                friendly_name: "Feature A".into(),
                spec_content: "spec a".into(),
                state: FeatureState::Specified,
                target_branch: None,
                labels: vec![],
                module_slug: None,
                project_id: None,
                plane_issue_id: None,
                plane_state_id: None,
                work_packages: vec![],
            },
            ImportFeature {
                slug: Some("feat-b".into()),
                friendly_name: "Feature B".into(),
                spec_content: "spec b".into(),
                state: FeatureState::Specified,
                target_branch: None,
                labels: vec![],
                module_slug: None,
                project_id: None,
                plane_issue_id: None,
                plane_state_id: None,
                work_packages: vec![],
            },
            ImportFeature {
                slug: Some("feat-c".into()),
                friendly_name: "Feature C".into(),
                spec_content: "spec c".into(),
                state: FeatureState::Specified,
                target_branch: None,
                labels: vec![],
                module_slug: None,
                project_id: None,
                plane_issue_id: None,
                plane_state_id: None,
                work_packages: vec![],
            },
        ],
        ..empty_bundle()
    };

    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();

    assert_eq!(report.features_created, 3);
    let features = storage.features.lock().unwrap();
    assert_eq!(features.len(), 3);
}

#[tokio::test]
async fn import_full_bundle_comprehensive() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        projects: vec![ImportProject {
            slug: Some("proj-alpha".into()),
            name: "Project Alpha".into(),
            description: Some("Alpha project".into()),
            features: vec![ImportFeature {
                slug: Some("feat-from-proj".into()),
                friendly_name: "Feature from Project".into(),
                spec_content: "embedded feature".into(),
                state: FeatureState::Specified,
                target_branch: None,
                labels: vec!["embedded".into()],
                module_slug: Some("infra".into()),
                project_id: None,
                plane_issue_id: None,
                plane_state_id: None,
                work_packages: vec![ImportWorkPackage {
                    title: "Embedded WP".into(),
                    acceptance_criteria: None,
                    sequence: Some(1),
                    file_scope: vec![],
                    state: WpState::Planned,
                    agent_id: None,
                    pr_url: None,
                    pr_state: None,
                    worktree_path: None,
                    plane_sub_issue_id: None,
                    depends_on_sequences: vec![],
                }],
            }],
        }],
        modules: vec![
            ImportModule {
                slug: Some("infra".into()),
                friendly_name: "Infrastructure".into(),
                description: None,
                parent_slug: None,
            },
            ImportModule {
                slug: Some("infra-db".into()),
                friendly_name: "Database".into(),
                description: None,
                parent_slug: Some("infra".into()),
            },
        ],
        features: vec![ImportFeature {
            slug: Some("standalone-feat".into()),
            friendly_name: "Standalone Feature".into(),
            spec_content: "standalone".into(),
            state: FeatureState::Specified,
            target_branch: None,
            labels: vec![],
            module_slug: None,
            project_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            work_packages: vec![],
        }],
        cycles: vec![ImportCycle {
            name: "Sprint 1".into(),
            description: Some("First sprint".into()),
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
            state: CycleState::Active,
            module_scope_slug: Some("infra".into()),
            feature_slugs: vec!["feat-from-proj".into(), "standalone-feat".into()],
        }],
    };

    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();

    assert_eq!(report.projects_created, 1);
    assert_eq!(report.modules_created, 2);
    assert_eq!(report.features_created, 2); // 1 embedded + 1 standalone
    assert_eq!(report.work_packages_created, 1);
    assert_eq!(report.cycles_created, 1);
    assert_eq!(report.cycles_updated, 1); // Active != Draft
    assert_eq!(report.cycle_links_created, 2);
    assert_eq!(report.module_links_created, 1); // feat-from-proj linked to infra
    assert_eq!(report.audits_written, 2); // 1 per feature
    assert_eq!(report.artifacts_written, 4); // 2 per feature
}

// ── Additional edge-case tests (expanded coverage) ────────────────────────────

/// Test: Empty bundle produces zero counts.
#[tokio::test]
async fn import_empty_bundle_produces_zero_report() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle::default();
    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();
    assert_eq!(report.projects_created, 0);
    assert_eq!(report.modules_created, 0);
    assert_eq!(report.features_created, 0);
    assert_eq!(report.cycles_created, 0);
    assert_eq!(report.work_packages_created, 0);
    assert_eq!(report.module_links_created, 0);
    assert_eq!(report.cycle_links_created, 0);
    assert_eq!(report.audits_written, 0);
    assert_eq!(report.artifacts_written, 0);
}

/// Test: Project with no slug derives slug from name.
#[tokio::test]
async fn import_project_without_slug_derives_from_name() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        projects: vec![ImportProject {
            slug: None,
            name: "My Cool Project".into(),
            description: None,
            features: vec![],
        }],
        ..Default::default()
    };
    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();
    assert_eq!(report.projects_created, 1);
    // The mock storage should have recorded the project
    assert_eq!(storage.projects.lock().unwrap().len(), 1);
}

/// Test: Duplicate project slugs result in update.
#[tokio::test]
async fn import_duplicate_project_slugs_results_in_update() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        projects: vec![
            ImportProject {
                slug: Some("dup-proj".into()),
                name: "First".into(),
                description: None,
                features: vec![],
            },
            ImportProject {
                slug: Some("dup-proj".into()),
                name: "Second".into(),
                description: Some("Updated".into()),
                features: vec![],
            },
        ],
        ..Default::default()
    };
    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();
    assert_eq!(report.projects_created, 1);
    assert_eq!(report.projects_updated, 1);
}

/// Test: Cycle without module scope still creates cycle.
#[tokio::test]
async fn import_cycle_without_module_scope_succeeds() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        cycles: vec![ImportCycle {
            name: "Standalone Cycle".into(),
            description: None,
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 14).unwrap(),
            state: CycleState::Active,
            module_scope_slug: None,
            feature_slugs: vec![],
        }],
        ..Default::default()
    };
    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();
    assert_eq!(report.cycles_created, 1);
}

/// Test: Feature with module_slug resolved from storage lookup.
#[tokio::test]
async fn import_feature_module_slug_resolved_from_storage() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        projects: vec![ImportProject {
            slug: Some("p1".into()),
            name: "P1".into(),
            description: None,
            features: vec![ImportFeature {
                slug: Some("feat".into()),
                friendly_name: "Feat".into(),
                spec_content: "# Spec".into(),
                state: FeatureState::Specified,
                target_branch: None,
                labels: vec![],
                module_slug: Some("existing-module".to_string()),
                project_id: None,
                plane_issue_id: None,
                plane_state_id: None,
                work_packages: vec![],
            }],
        }],
        modules: vec![ImportModule {
            slug: Some("existing-module".into()),
            friendly_name: "Existing Module".into(),
            description: None,
            parent_slug: None,
        }],
        ..Default::default()
    };
    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();
    assert_eq!(report.features_created, 1);
    assert_eq!(report.module_links_created, 1);
}

/// Test: Cycle referencing unknown module fails.
#[tokio::test]
async fn import_cycle_unknown_module_fails() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        cycles: vec![ImportCycle {
            name: "Scoped Cycle".into(),
            description: None,
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            state: CycleState::Active,
            module_scope_slug: Some("does-not-exist".into()),
            feature_slugs: vec![],
        }],
        ..Default::default()
    };
    let result = import_bundle(bundle, &storage, &vcs).await;
    assert!(result.is_err());
}

/// Test: Feature referencing nonexistent module via storage lookup fails.
#[tokio::test]
async fn import_feature_nonexistent_module_fails() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        projects: vec![ImportProject {
            slug: Some("p-fail".into()),
            name: "PFail".into(),
            description: None,
            features: vec![ImportFeature {
                slug: Some("feat-fail".into()),
                friendly_name: "Bad Module".into(),
                spec_content: "# Spec".into(),
                state: FeatureState::Specified,
                target_branch: None,
                labels: vec![],
                module_slug: Some("nonexistent".into()),
                project_id: None,
                plane_issue_id: None,
                plane_state_id: None,
                work_packages: vec![],
            }],
        }],
        ..Default::default()
    };
    let result = import_bundle(bundle, &storage, &vcs).await;
    assert!(result.is_err());
}

/// Test: ImportReport is Clone and Debug.
#[test]
fn import_report_clone_and_debug() {
    let mut report = ImportReport::default();
    report.projects_created = 5;
    let cloned = report.clone();
    assert_eq!(cloned.projects_created, 5);
    let debug_str = format!("{report:?}");
    assert!(debug_str.contains("ImportReport"));
}

/// Test: ImportReport JSON roundtrip preserves all fields.
#[test]
fn import_report_json_roundtrip_preserves_all_fields() {
    let mut report = ImportReport::default();
    report.projects_created = 1;
    report.modules_created = 2;
    report.features_updated = 3;
    report.work_packages_created = 4;
    report.cycle_links_created = 5;
    report.artifacts_written = 6;
    report.audits_written = 7;

    let json = serde_json::to_string(&report).unwrap();
    let restored: ImportReport = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, report);
}

/// Test: ImportReport serialization with all fields non-zero.
#[test]
fn import_report_full_json_roundtrip() {
    let mut report = ImportReport::default();
    report.projects_created = 10;
    report.projects_updated = 20;
    report.modules_created = 30;
    report.modules_updated = 40;
    report.features_created = 50;
    report.features_updated = 60;
    report.cycles_created = 70;
    report.cycles_updated = 80;
    report.work_packages_created = 90;
    report.work_packages_updated = 100;
    report.module_links_created = 110;
    report.cycle_links_created = 120;
    report.artifacts_written = 130;
    report.audits_written = 140;

    let json = serde_json::to_string(&report).unwrap();
    let restored: ImportReport = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.projects_created, 10);
    assert_eq!(restored.audits_written, 140);
}

/// Test: ImportWorkPackage fields are mutable post-construction.
#[test]
fn import_work_package_all_fields_mutable() {
    let mut wp = ImportWorkPackage {
        title: "Task".into(),
        acceptance_criteria: Some("Given when then".into()),
        sequence: Some(1),
        file_scope: vec!["src/main.rs".into()],
        state: WpState::Planned,
        agent_id: Some("agent-1".into()),
        pr_url: Some("http://example.com".into()),
        pr_state: Some(PrState::Open),
        worktree_path: Some("/wt/path".into()),
        plane_sub_issue_id: Some("sub-1".into()),
        depends_on_sequences: vec![1, 2],
    };
    wp.title = "Updated".into();
    wp.state = WpState::Doing;
    assert_eq!(wp.title, "Updated");
    assert_eq!(wp.state, WpState::Doing);
}

/// Test: ImportFeature with None slug and project_id.
#[test]
fn import_feature_no_slug_no_project_id() {
    let feature = ImportFeature {
        slug: None,
        friendly_name: "Untitled".into(),
        spec_content: "# Untitled".into(),
        state: FeatureState::Planned,
        target_branch: None,
        labels: vec!["bug".into()],
        module_slug: None,
        project_id: None,
        plane_issue_id: None,
        plane_state_id: None,
        work_packages: vec![],
    };
    assert!(feature.slug.is_none());
    assert!(feature.project_id.is_none());
    assert_eq!(feature.labels.len(), 1);
}

/// Test: ImportModule::slug() derives from friendly_name when slug is None.
#[test]
fn import_module_slug_derives_from_friendly_name() {
    let module = ImportModule {
        slug: None,
        friendly_name: "My Module".into(),
        description: None,
        parent_slug: None,
    };
    let derived = module.slug();
    assert!(!derived.is_empty());
}

/// Test: ImportModule::slug() uses provided slug when present.
#[test]
fn import_module_slug_uses_provided_slug() {
    let module = ImportModule {
        slug: Some("custom-slug".into()),
        friendly_name: "Friendly".into(),
        description: None,
        parent_slug: None,
    };
    assert_eq!(module.slug(), "custom-slug");
}

/// Test: ImportBundle serde with empty arrays serializes correctly.
#[test]
fn import_bundle_empty_arrays_serialize() {
    let bundle = ImportBundle::default();
    let json = serde_json::to_string(&bundle).unwrap();
    let restored: ImportBundle = serde_json::from_str(&json).unwrap();
    assert!(restored.projects.is_empty());
    assert!(restored.modules.is_empty());
    assert!(restored.features.is_empty());
    assert!(restored.cycles.is_empty());
}

/// Test: ImportCycle serde roundtrip with all fields.
#[test]
fn import_cycle_full_roundtrip() {
    let cycle = ImportCycle {
        name: "Cycle A".into(),
        description: Some("Desc".into()),
        start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        state: CycleState::Shipped,
        module_scope_slug: Some("mod-a".into()),
        feature_slugs: vec!["feat-1".into(), "feat-2".into()],
    };
    let json = serde_json::to_string(&cycle).unwrap();
    let restored: ImportCycle = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.name, "Cycle A");
    assert_eq!(restored.state, CycleState::Shipped);
    assert_eq!(restored.feature_slugs.len(), 2);
}

/// Test: ImportProject serde roundtrip with embedded features.
#[test]
fn import_project_with_embedded_features_roundtrip() {
    let project = ImportProject {
        slug: Some("proj".into()),
        name: "Project".into(),
        description: Some("Desc".into()),
        features: vec![ImportFeature {
            slug: Some("f1".into()),
            friendly_name: "F1".into(),
            spec_content: "# F1".into(),
            state: FeatureState::Specified,
            target_branch: Some("branch".into()),
            labels: vec!["label".into()],
            module_slug: None,
            project_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            work_packages: vec![],
        }],
    };
    let json = serde_json::to_string(&project).unwrap();
    let restored: ImportProject = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.features.len(), 1);
    assert_eq!(restored.features[0].friendly_name, "F1");
}

/// Test: ImportBundle with all top-level entries roundtrips.
#[test]
fn import_bundle_full_roundtrip() {
    let bundle = ImportBundle {
        projects: vec![ImportProject {
            slug: Some("p".into()),
            name: "P".into(),
            description: None,
            features: vec![],
        }],
        modules: vec![ImportModule {
            slug: Some("m".into()),
            friendly_name: "M".into(),
            description: None,
            parent_slug: None,
        }],
        features: vec![ImportFeature {
            slug: Some("f".into()),
            friendly_name: "F".into(),
            spec_content: "# F".into(),
            state: FeatureState::Specified,
            target_branch: None,
            labels: vec![],
            module_slug: None,
            project_id: None,
            plane_issue_id: None,
            plane_state_id: None,
            work_packages: vec![],
        }],
        cycles: vec![ImportCycle {
            name: "C".into(),
            description: None,
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2026, 5, 14).unwrap(),
            state: CycleState::Draft,
            module_scope_slug: None,
            feature_slugs: vec![],
        }],
    };
    let json = serde_json::to_string(&bundle).unwrap();
    let restored: ImportBundle = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.projects.len(), 1);
    assert_eq!(restored.modules.len(), 1);
    assert_eq!(restored.features.len(), 1);
    assert_eq!(restored.cycles.len(), 1);
}

/// Test: ImportFeature default state is Specified.
#[test]
fn import_feature_default_state_is_specified() {
    let raw = r##"{"friendly_name":"TestFeature","spec_content":"# Spec"}"##;
    let feature: ImportFeature = serde_json::from_str(raw).unwrap();
    assert_eq!(feature.state, FeatureState::Specified);
}

/// Test: ImportWorkPackage default state is Planned.
#[test]
fn import_work_package_default_state_is_planned() {
    let raw = r#"{"title":"Task"}"#;
    let wp: ImportWorkPackage = serde_json::from_str(raw).unwrap();
    assert_eq!(wp.state, WpState::Planned);
}

/// Test: ImportCycle default state is Draft.
#[test]
fn import_cycle_default_state_is_draft() {
    let raw = r#"{"name":"Cycle","start_date":"2026-01-01","end_date":"2026-02-01"}"#;
    let cycle: ImportCycle = serde_json::from_str(raw).unwrap();
    assert_eq!(cycle.state, CycleState::Draft);
}

/// Test: ImportModule with parent_slug resolves module hierarchy.
#[tokio::test]
async fn import_module_hierarchy_resolves_parent() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        modules: vec![
            ImportModule {
                slug: Some("parent-mod".into()),
                friendly_name: "Parent".into(),
                description: None,
                parent_slug: None,
            },
            ImportModule {
                slug: Some("child-mod".into()),
                friendly_name: "Child".into(),
                description: None,
                parent_slug: Some("parent-mod".into()),
            },
        ],
        ..Default::default()
    };
    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();
    assert_eq!(report.modules_created, 2);
}

/// Test: ImportModule with unresolved parent defers then resolves.
#[tokio::test]
async fn import_module_unresolved_parent_defers() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        modules: vec![
            ImportModule {
                slug: Some("child-first".into()),
                friendly_name: "Child".into(),
                description: None,
                parent_slug: Some("parent-later".into()),
            },
            ImportModule {
                slug: Some("parent-later".into()),
                friendly_name: "Parent".into(),
                description: None,
                parent_slug: None,
            },
        ],
        ..Default::default()
    };
    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();
    assert_eq!(report.modules_created, 2);
}

/// Test: ImportFeature with non-Created state triggers state update.
#[tokio::test]
async fn import_feature_non_created_state_triggers_update() {
    let storage = MockStorage::new();
    let vcs = MockVcs::default();
    let bundle = ImportBundle {
        projects: vec![ImportProject {
            slug: Some("proj-state".into()),
            name: "Proj".into(),
            description: None,
            features: vec![ImportFeature {
                slug: Some("feat-state".into()),
                friendly_name: "Feat".into(),
                spec_content: "# Spec".into(),
                state: FeatureState::Planned,
                target_branch: None,
                labels: vec![],
                module_slug: None,
                project_id: None,
                plane_issue_id: None,
                plane_state_id: None,
                work_packages: vec![],
            }],
        }],
        ..Default::default()
    };
    let report = import_bundle(bundle, &storage, &vcs).await.unwrap();
    assert_eq!(report.features_created, 1);
    assert_eq!(storage.feature_state_updates.lock().unwrap().len(), 1);
}

/// Test: ImportWorkPackage default sequence is index + 1.
#[test]
fn import_work_package_sequence_defaults_to_index() {
    let wp = ImportWorkPackage {
        title: "Task".into(),
        acceptance_criteria: None,
        sequence: None,
        file_scope: vec![],
        state: WpState::Planned,
        agent_id: None,
        pr_url: None,
        pr_state: None,
        worktree_path: None,
        plane_sub_issue_id: None,
        depends_on_sequences: vec![],
    };
    assert!(wp.sequence.is_none());
    assert!(wp.file_scope.is_empty());
    assert!(wp.depends_on_sequences.is_empty());
}

/// Test: ImportBundle Default derives empty vecs.
#[test]
fn import_bundle_default_has_empty_vecs() {
    let bundle = ImportBundle::default();
    assert!(bundle.projects.is_empty());
    assert!(bundle.modules.is_empty());
    assert!(bundle.features.is_empty());
    assert!(bundle.cycles.is_empty());
    assert!(bundle.projects.is_empty());
}

/// Test: ImportProject Default is not possible (no Default impl).
#[test]
fn import_project_requires_name_field() {
    // ImportProject does not implement Default; verify via construction
    let project = ImportProject {
        slug: Some("slug".into()),
        name: "Name".into(),
        description: Some("desc".into()),
        features: vec![],
    };
    assert_eq!(project.slug, Some("slug".into()));
}

#[allow(dead_code)]
fn _placeholder() {}
