// SPDX-License-Identifier: MIT OR Apache-2.0
//! In-memory test doubles for the application layer.
//!
//! These are used by the inline `#[cfg(test)]` tests in `lib.rs` and
//! can be imported by other crates' test suites via
//! `agileplus_application::test_mocks::*`.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use tokio::sync::RwLock;

use agileplus_domain::domain::epic::{Epic, EpicStatus};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::story::{Story, StoryStatus};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::StoragePort;
use agileplus_domain::ports::epic::EpicRepository;
use agileplus_domain::ports::events::{DomainEvent, DomainEventPublisher};
use agileplus_domain::ports::story::StoryRepository;

/// In-memory Feature store.
#[derive(Default)]
pub struct InMemoryFeatureRepo {
    store: RwLock<HashMap<i64, Feature>>,
    next_id: RwLock<i64>,
}

#[async_trait]
impl StoragePort for InMemoryFeatureRepo {
    async fn create_feature(&self, feature: &Feature) -> Result<i64, DomainError> {
        let mut next = self.next_id.write().await;
        *next += 1;
        let id = *next;
        let mut f = feature.clone();
        f.id = id;
        self.store.write().await.insert(id, f);
        Ok(id)
    }

    async fn get_feature_by_id(&self, id: i64) -> Result<Option<Feature>, DomainError> {
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn update_feature_state(&self, id: i64, state: FeatureState) -> Result<(), DomainError> {
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
    async fn list_features_by_state(&self, state: FeatureState) -> Result<Vec<Feature>, DomainError> {
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
    async fn create_work_package(
        &self,
        _: &agileplus_domain::domain::work_package::WorkPackage,
    ) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_work_package(
        &self,
        _: i64,
    ) -> Result<Option<agileplus_domain::domain::work_package::WorkPackage>, DomainError> {
        Ok(None)
    }
    async fn update_wp_state(
        &self,
        _: i64,
        _: agileplus_domain::domain::work_package::WpState,
    ) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_wps_by_feature(
        &self,
        _: i64,
    ) -> Result<Vec<agileplus_domain::domain::work_package::WorkPackage>, DomainError> {
        Ok(vec![])
    }
    async fn add_wp_dependency(
        &self,
        _: &agileplus_domain::domain::work_package::WpDependency,
    ) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_wp_dependencies(
        &self,
        _: i64,
    ) -> Result<Vec<agileplus_domain::domain::work_package::WpDependency>, DomainError> {
        Ok(vec![])
    }
    async fn get_ready_wps(
        &self,
        _: i64,
    ) -> Result<Vec<agileplus_domain::domain::work_package::WorkPackage>, DomainError> {
        Ok(vec![])
    }
    async fn append_audit_entry(
        &self,
        _: &agileplus_domain::domain::audit::AuditEntry,
    ) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_audit_trail(
        &self,
        _: i64,
    ) -> Result<Vec<agileplus_domain::domain::audit::AuditEntry>, DomainError> {
        Ok(vec![])
    }
    async fn get_latest_audit_entry(
        &self,
        _: i64,
    ) -> Result<Option<agileplus_domain::domain::audit::AuditEntry>, DomainError> {
        Ok(None)
    }
    async fn create_evidence(
        &self,
        _: &agileplus_domain::domain::governance::Evidence,
    ) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_evidence_by_wp(
        &self,
        _: i64,
    ) -> Result<Vec<agileplus_domain::domain::governance::Evidence>, DomainError> {
        Ok(vec![])
    }
    async fn get_evidence_by_fr(
        &self,
        _: &str,
    ) -> Result<Vec<agileplus_domain::domain::governance::Evidence>, DomainError> {
        Ok(vec![])
    }
    async fn create_policy_rule(
        &self,
        _: &agileplus_domain::domain::governance::PolicyRule,
    ) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn list_active_policies(
        &self,
    ) -> Result<Vec<agileplus_domain::domain::governance::PolicyRule>, DomainError> {
        Ok(vec![])
    }
    async fn record_metric(
        &self,
        _: &agileplus_domain::domain::metric::Metric,
    ) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_metrics_by_feature(
        &self,
        _: i64,
    ) -> Result<Vec<agileplus_domain::domain::metric::Metric>, DomainError> {
        Ok(vec![])
    }
    async fn create_governance_contract(
        &self,
        _: &agileplus_domain::domain::governance::GovernanceContract,
    ) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_governance_contract(
        &self,
        _: i64,
        _: i32,
    ) -> Result<Option<agileplus_domain::domain::governance::GovernanceContract>, DomainError> {
        Ok(None)
    }
    async fn get_latest_governance_contract(
        &self,
        _: i64,
    ) -> Result<Option<agileplus_domain::domain::governance::GovernanceContract>, DomainError> {
        Ok(None)
    }
    async fn create_module(
        &self,
        _: &agileplus_domain::domain::module::Module,
    ) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_module(&self, _: i64) -> Result<Option<agileplus_domain::domain::module::Module>, DomainError> {
        Ok(None)
    }
    async fn get_module_by_slug(&self, _: &str) -> Result<Option<agileplus_domain::domain::module::Module>, DomainError> {
        Ok(None)
    }
    async fn update_module(&self, _: i64, _: &str, _: Option<&str>) -> Result<(), DomainError> {
        Ok(())
    }
    async fn delete_module(&self, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_root_modules(&self) -> Result<Vec<agileplus_domain::domain::module::Module>, DomainError> {
        Ok(vec![])
    }
    async fn list_child_modules(&self, _: i64) -> Result<Vec<agileplus_domain::domain::module::Module>, DomainError> {
        Ok(vec![])
    }
    async fn get_module_with_features(&self, _: i64) -> Result<Option<agileplus_domain::domain::module::ModuleWithFeatures>, DomainError> {
        Ok(None)
    }
    async fn tag_feature_to_module(&self, _: &agileplus_domain::domain::module::ModuleFeatureTag) -> Result<(), DomainError> {
        Ok(())
    }
    async fn untag_feature_from_module(&self, _: i64, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn create_cycle(&self, _: &agileplus_domain::domain::cycle::Cycle) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_cycle(&self, _: i64) -> Result<Option<agileplus_domain::domain::cycle::Cycle>, DomainError> {
        Ok(None)
    }
    async fn update_cycle_state(&self, _: i64, _: agileplus_domain::domain::cycle::CycleState) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_cycles_by_state(&self, _: agileplus_domain::domain::cycle::CycleState) -> Result<Vec<agileplus_domain::domain::cycle::Cycle>, DomainError> {
        Ok(vec![])
    }
    async fn list_cycles_by_module(&self, _: i64) -> Result<Vec<agileplus_domain::domain::cycle::Cycle>, DomainError> {
        Ok(vec![])
    }
    async fn list_all_cycles(&self) -> Result<Vec<agileplus_domain::domain::cycle::Cycle>, DomainError> {
        Ok(vec![])
    }
    async fn get_cycle_with_features(&self, _: i64) -> Result<Option<agileplus_domain::domain::cycle::CycleWithFeatures>, DomainError> {
        Ok(None)
    }
    async fn add_feature_to_cycle(&self, _: &agileplus_domain::domain::cycle::CycleFeature) -> Result<(), DomainError> {
        Ok(())
    }
    async fn remove_feature_from_cycle(&self, _: i64, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_sync_mapping(&self, _: &str, _: i64) -> Result<Option<agileplus_domain::domain::sync_mapping::SyncMapping>, DomainError> {
        Ok(None)
    }
    async fn upsert_sync_mapping(&self, _: &agileplus_domain::domain::sync_mapping::SyncMapping) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_sync_mapping_by_plane_id(&self, _: &str, _: &str) -> Result<Option<agileplus_domain::domain::sync_mapping::SyncMapping>, DomainError> {
        Ok(None)
    }
    async fn delete_sync_mapping(&self, _: &str, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn create_project(&self, _: &agileplus_domain::domain::project::Project) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_project_by_slug(&self, _: &str) -> Result<Option<agileplus_domain::domain::project::Project>, DomainError> {
        Ok(None)
    }
    async fn get_project_by_id(&self, _: i64) -> Result<Option<agileplus_domain::domain::project::Project>, DomainError> {
        Ok(None)
    }
    async fn list_all_projects(&self) -> Result<Vec<agileplus_domain::domain::project::Project>, DomainError> {
        Ok(vec![])
    }
    async fn delete_project(&self, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn create_user(&self, _: &agileplus_domain::domain::user::User) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_user(&self, _: i64) -> Result<Option<agileplus_domain::domain::user::User>, DomainError> {
        Ok(None)
    }
    async fn get_user_by_email(&self, _: &str) -> Result<Option<agileplus_domain::domain::user::User>, DomainError> {
        Ok(None)
    }
    async fn update_user_status(&self, _: i64, _: agileplus_domain::domain::user::UserStatus) -> Result<(), DomainError> {
        Ok(())
    }
    async fn update_user_role(&self, _: i64, _: agileplus_domain::domain::user::UserRole) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_all_users(&self) -> Result<Vec<agileplus_domain::domain::user::User>, DomainError> {
        Ok(vec![])
    }
    async fn delete_user(&self, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn create_epic(&self, _: &agileplus_domain::domain::epic::Epic) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_epic(&self, _: i64) -> Result<Option<agileplus_domain::domain::epic::Epic>, DomainError> {
        Ok(None)
    }
    async fn update_epic_status(&self, _: i64, _: agileplus_domain::domain::epic::EpicStatus) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_epics_by_project(&self, _: i64) -> Result<Vec<agileplus_domain::domain::epic::Epic>, DomainError> {
        Ok(vec![])
    }
    async fn delete_epic(&self, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn create_story(&self, _: &agileplus_domain::domain::story::Story) -> Result<i64, DomainError> {
        Ok(0)
    }
    async fn get_story(&self, _: i64) -> Result<Option<agileplus_domain::domain::story::Story>, DomainError> {
        Ok(None)
    }
    async fn update_story_status(&self, _: i64, _: agileplus_domain::domain::story::StoryStatus) -> Result<(), DomainError> {
        Ok(())
    }
    async fn list_stories_by_epic(&self, _: i64) -> Result<Vec<agileplus_domain::domain::story::Story>, DomainError> {
        Ok(vec![])
    }
    async fn list_stories_by_project(&self, _: i64) -> Result<Vec<agileplus_domain::domain::story::Story>, DomainError> {
        Ok(vec![])
    }
    async fn delete_story(&self, _: i64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn upsert_story_by_requirement_id(&self, _: &agileplus_domain::domain::story::Story) -> Result<i64, DomainError> {
        Ok(0)
    }
}

/// In-memory Story store.
#[derive(Default)]
pub struct InMemoryStoryRepo {
    store: RwLock<HashMap<i64, Story>>,
    next_id: RwLock<i64>,
}

#[async_trait]
impl StoryRepository for InMemoryStoryRepo {
    async fn create(&self, story: &Story) -> Result<i64, DomainError> {
        let mut next = self.next_id.write().await;
        *next += 1;
        let id = *next;
        let mut s = story.clone();
        s.id = id;
        self.store.write().await.insert(id, s);
        Ok(id)
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Story>, DomainError> {
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn update_status(&self, id: i64, status: StoryStatus) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if let Some(s) = store.get_mut(&id) {
            s.status = status;
            Ok(())
        } else {
            Err(DomainError::NotFound(id.to_string()))
        }
    }

    async fn list_by_epic(&self, epic_id: i64) -> Result<Vec<Story>, DomainError> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|s| s.epic_id == epic_id)
            .cloned()
            .collect())
    }
}

/// In-memory Epic store.
#[derive(Default)]
pub struct InMemoryEpicRepo {
    store: RwLock<HashMap<i64, Epic>>,
    next_id: RwLock<i64>,
}

#[async_trait]
impl EpicRepository for InMemoryEpicRepo {
    async fn create(&self, epic: &Epic) -> Result<i64, DomainError> {
        let mut next = self.next_id.write().await;
        *next += 1;
        let id = *next;
        let mut e = epic.clone();
        e.id = id;
        self.store.write().await.insert(id, e);
        Ok(id)
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Epic>, DomainError> {
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn update_status(&self, id: i64, status: EpicStatus) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if let Some(e) = store.get_mut(&id) {
            e.status = status;
            Ok(())
        } else {
            Err(DomainError::NotFound(id.to_string()))
        }
    }

    async fn list_by_project(&self, project_id: i64) -> Result<Vec<Epic>, DomainError> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|e| e.project_id == project_id)
            .cloned()
            .collect())
    }
}

/// Spy publisher — records emitted events.
#[derive(Default)]
pub struct SpyPublisher {
    events: Mutex<Vec<DomainEvent>>,
}

#[async_trait]
impl DomainEventPublisher for SpyPublisher {
    fn publish(&self, event: DomainEvent) -> Result<(), DomainError> {
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

impl SpyPublisher {
    pub fn emitted(&self) -> Vec<DomainEvent> {
        self.events.lock().unwrap().clone()
    }
}
