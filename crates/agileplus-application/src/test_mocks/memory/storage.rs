// SPDX-License-Identifier: MIT OR Apache-2.0
//! In-memory `StoragePort` double, with optional fault injection.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use tokio::sync::RwLock;

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
use agileplus_domain::domain::user::{User, UserRole, UserStatus};
use agileplus_domain::domain::work_package::{WorkPackage, WpDependency, WpState};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::StoragePort;

/// In-memory Feature store.
///
/// # Fault injection
///
/// The `failing_*` builders flip a flag so a single operation starts
/// returning [`DomainError::Storage`]. This mirrors the
/// `MockStoragePort` convention used elsewhere in the workspace and lets
/// use-case error paths be driven without a bespoke `StoragePort` impl.
#[derive(Default)]
pub struct InMemoryFeatureRepo {
    store: RwLock<HashMap<i64, Feature>>,
    next_id: RwLock<i64>,
    fail_create: AtomicBool,
    fail_read: AtomicBool,
    fail_write: AtomicBool,
}

impl InMemoryFeatureRepo {
    /// The error every injected fault returns.
    fn offline() -> DomainError {
        DomainError::Storage("in-memory store offline".to_string())
    }

    /// Make `create_feature` fail.
    pub fn failing_create(self) -> Self {
        self.fail_create.store(true, Ordering::SeqCst);
        self
    }

    /// Make `get_feature_by_id` fail.
    pub fn failing_read(self) -> Self {
        self.fail_read.store(true, Ordering::SeqCst);
        self
    }

    /// Make `update_feature_state` fail.
    pub fn failing_state_write(self) -> Self {
        self.fail_write.store(true, Ordering::SeqCst);
        self
    }
}

#[async_trait]
impl StoragePort for InMemoryFeatureRepo {
    async fn create_feature(&self, feature: &Feature) -> Result<i64, DomainError> {
        if self.fail_create.load(Ordering::SeqCst) {
            return Err(Self::offline());
        }
        let mut next = self.next_id.write().await;
        *next += 1;
        let id = *next;
        let mut f = feature.clone();
        f.id = id;
        self.store.write().await.insert(id, f);
        Ok(id)
    }

    async fn get_feature_by_id(&self, id: i64) -> Result<Option<Feature>, DomainError> {
        if self.fail_read.load(Ordering::SeqCst) {
            return Err(Self::offline());
        }
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn update_feature_state(&self, id: i64, state: FeatureState) -> Result<(), DomainError> {
        if self.fail_write.load(Ordering::SeqCst) {
            return Err(Self::offline());
        }
        let mut store = self.store.write().await;
        if let Some(f) = store.get_mut(&id) {
            f.state = state;
            Ok(())
        } else {
            Err(DomainError::FeatureNotFound(id.to_string()))
        }
    }

    async fn update_feature(&self, feature: &Feature) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if let std::collections::hash_map::Entry::Occupied(mut e) = store.entry(feature.id) {
            e.insert(feature.clone());
            Ok(())
        } else {
            Err(DomainError::FeatureNotFound(feature.id.to_string()))
        }
    }

    async fn get_feature_by_slug(&self, slug: &str) -> Result<Option<Feature>, DomainError> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .find(|f| f.slug == slug)
            .cloned())
    }
    async fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> Result<Vec<Feature>, DomainError> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|f| f.state == state)
            .cloned()
            .collect())
    }
    async fn list_all_features(&self) -> Result<Vec<Feature>, DomainError> {
        Ok(self.store.read().await.values().cloned().collect())
    }
    async fn list_features_by_label(&self, label: &str) -> Result<Vec<Feature>, DomainError> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|f| f.labels.iter().any(|l| l == label))
            .cloned()
            .collect())
    }

    async fn create_work_package(&self, _: &WorkPackage) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_work_package(&self, _: i64) -> Result<Option<WorkPackage>, DomainError> {
        Ok(None)
    }
    async fn update_wp_state(&self, _: i64, _: WpState) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_wps_by_feature(&self, _: i64) -> Result<Vec<WorkPackage>, DomainError> {
        Ok(vec![])
    }
    async fn add_wp_dependency(&self, _: &WpDependency) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_wp_dependencies(&self, _: i64) -> Result<Vec<WpDependency>, DomainError> {
        Ok(vec![])
    }
    async fn get_ready_wps(&self, _: i64) -> Result<Vec<WorkPackage>, DomainError> {
        Ok(vec![])
    }
    async fn append_audit_entry(&self, _: &AuditEntry) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_audit_trail(&self, _: i64) -> Result<Vec<AuditEntry>, DomainError> {
        Ok(vec![])
    }
    async fn get_latest_audit_entry(&self, _: i64) -> Result<Option<AuditEntry>, DomainError> {
        Ok(None)
    }
    async fn create_evidence(&self, _: &Evidence) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_evidence_by_wp(&self, _: i64) -> Result<Vec<Evidence>, DomainError> {
        Ok(vec![])
    }
    async fn get_evidence_by_fr(&self, _: &str) -> Result<Vec<Evidence>, DomainError> {
        Ok(vec![])
    }
    async fn create_policy_rule(&self, _: &PolicyRule) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn list_active_policies(&self) -> Result<Vec<PolicyRule>, DomainError> {
        Ok(vec![])
    }
    async fn record_metric(&self, _: &Metric) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_metrics_by_feature(&self, _: i64) -> Result<Vec<Metric>, DomainError> {
        Ok(vec![])
    }
    async fn create_governance_contract(&self, _: &GovernanceContract) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_governance_contract(
        &self,
        _: i64,
        _: i32,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        Ok(None)
    }
    async fn get_latest_governance_contract(
        &self,
        _: i64,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        Ok(None)
    }
    async fn create_module(&self, _: &Module) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_module(&self, _: i64) -> Result<Option<Module>, DomainError> {
        Ok(None)
    }
    async fn get_module_by_slug(&self, _: &str) -> Result<Option<Module>, DomainError> {
        Ok(None)
    }
    async fn update_module(&self, _: i64, _: &str, _: Option<&str>) -> Result<(), DomainError> {
        Ok(())
    }
    async fn delete_module(&self, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_root_modules(&self) -> Result<Vec<Module>, DomainError> {
        Ok(vec![])
    }
    async fn list_child_modules(&self, _: i64) -> Result<Vec<Module>, DomainError> {
        Ok(vec![])
    }
    async fn get_module_with_features(
        &self,
        _: i64,
    ) -> Result<Option<ModuleWithFeatures>, DomainError> {
        Ok(None)
    }
    async fn tag_feature_to_module(&self, _: &ModuleFeatureTag) -> Result<(), DomainError> {
        Ok(())
    }
    async fn untag_feature_from_module(&self, _: i64, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn create_cycle(&self, _: &Cycle) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_cycle(&self, _: i64) -> Result<Option<Cycle>, DomainError> {
        Ok(None)
    }
    async fn update_cycle_state(&self, _: i64, _: CycleState) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_cycles_by_state(&self, _: CycleState) -> Result<Vec<Cycle>, DomainError> {
        Ok(vec![])
    }
    async fn list_cycles_by_module(&self, _: i64) -> Result<Vec<Cycle>, DomainError> {
        Ok(vec![])
    }
    async fn list_all_cycles(&self) -> Result<Vec<Cycle>, DomainError> {
        Ok(vec![])
    }
    async fn get_cycle_with_features(
        &self,
        _: i64,
    ) -> Result<Option<CycleWithFeatures>, DomainError> {
        Ok(None)
    }
    async fn add_feature_to_cycle(&self, _: &CycleFeature) -> Result<(), DomainError> {
        Ok(())
    }
    async fn remove_feature_from_cycle(&self, _: i64, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_sync_mapping(&self, _: &str, _: i64) -> Result<Option<SyncMapping>, DomainError> {
        Ok(None)
    }
    async fn upsert_sync_mapping(&self, _: &SyncMapping) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_sync_mapping_by_plane_id(
        &self,
        _: &str,
        _: &str,
    ) -> Result<Option<SyncMapping>, DomainError> {
        Ok(None)
    }
    async fn delete_sync_mapping(&self, _: &str, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn create_project(&self, _: &Project) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_project_by_slug(&self, _: &str) -> Result<Option<Project>, DomainError> {
        Ok(None)
    }
    async fn get_project_by_id(&self, _: i64) -> Result<Option<Project>, DomainError> {
        Ok(None)
    }
    async fn list_all_projects(&self) -> Result<Vec<Project>, DomainError> {
        Ok(vec![])
    }
    async fn delete_project(&self, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn create_user(&self, _: &User) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_user(&self, _: i64) -> Result<Option<User>, DomainError> {
        Ok(None)
    }
    async fn get_user_by_email(&self, _: &str) -> Result<Option<User>, DomainError> {
        Ok(None)
    }
    async fn update_user_status(&self, _: i64, _: UserStatus) -> Result<(), DomainError> {
        Ok(())
    }
    async fn update_user_role(&self, _: i64, _: UserRole) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_all_users(&self) -> Result<Vec<User>, DomainError> {
        Ok(vec![])
    }
    async fn delete_user(&self, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn create_epic(&self, _: &Epic) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_epic(&self, _: i64) -> Result<Option<Epic>, DomainError> {
        Ok(None)
    }
    async fn update_epic_status(&self, _: i64, _: EpicStatus) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_epics_by_project(&self, _: i64) -> Result<Vec<Epic>, DomainError> {
        Ok(vec![])
    }
    async fn delete_epic(&self, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn create_story(&self, _: &Story) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_story(&self, _: i64) -> Result<Option<Story>, DomainError> {
        Ok(None)
    }
    async fn update_story_status(&self, _: i64, _: StoryStatus) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_stories_by_epic(&self, _: i64) -> Result<Vec<Story>, DomainError> {
        Ok(vec![])
    }
    async fn list_stories_by_project(&self, _: i64) -> Result<Vec<Story>, DomainError> {
        Ok(vec![])
    }
    async fn delete_story(&self, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn upsert_story_by_requirement_id(&self, _: &Story) -> Result<i64, DomainError> {
        Ok(0)
    }
}
