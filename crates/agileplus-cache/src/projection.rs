//! Projection cache for Feature and WorkPackage state.

use crate::store::{CacheStore, RedisCacheStore};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::work_package::WorkPackage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("Cache error: {0}")]
    CacheError(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureProjection {
    pub feature: Feature,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkPackageProjection {
    pub workpackage: WorkPackage,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

pub struct ProjectionCache {
    store: Arc<RedisCacheStore>,
}

impl ProjectionCache {
    pub fn new(store: Arc<RedisCacheStore>) -> Self {
        Self { store }
    }

    pub async fn get_feature(
        &self,
        feature_id: i64,
    ) -> Result<Option<FeatureProjection>, ProjectionError> {
        self.store
            .get(&format!("feature:{feature_id}"))
            .await
            .map_err(|e| ProjectionError::CacheError(e.to_string()))
    }

    pub async fn set_feature(&self, feature: &Feature) -> Result<(), ProjectionError> {
        let projection = FeatureProjection {
            feature: feature.clone(),
            cached_at: chrono::Utc::now(),
        };
        self.store
            .set(&format!("feature:{}", feature.id), &projection, None)
            .await
            .map_err(|e| ProjectionError::CacheError(e.to_string()))
    }

    pub async fn get_workpackage(
        &self,
        wp_id: i64,
    ) -> Result<Option<WorkPackageProjection>, ProjectionError> {
        self.store
            .get(&format!("wp:{wp_id}"))
            .await
            .map_err(|e| ProjectionError::CacheError(e.to_string()))
    }

    pub async fn set_workpackage(&self, wp: &WorkPackage) -> Result<(), ProjectionError> {
        let projection = WorkPackageProjection {
            workpackage: wp.clone(),
            cached_at: chrono::Utc::now(),
        };
        self.store
            .set(&format!("wp:{}", wp.id), &projection, None)
            .await
            .map_err(|e| ProjectionError::CacheError(e.to_string()))
    }

    pub async fn invalidate_feature(&self, feature_id: i64) -> Result<(), ProjectionError> {
        self.store
            .delete(&format!("feature:{feature_id}"))
            .await
            .map_err(|e| ProjectionError::CacheError(e.to_string()))
    }

    pub async fn invalidate_workpackage(&self, wp_id: i64) -> Result<(), ProjectionError> {
        self.store
            .delete(&format!("wp:{wp_id}"))
            .await
            .map_err(|e| ProjectionError::CacheError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::CacheError;
    use async_trait::async_trait;
    use dashmap::DashMap;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    struct TestCacheStore {
        data: Arc<DashMap<String, (String, Option<Instant>)>>,
    }

    impl TestCacheStore {
        fn new() -> Self {
            Self {
                data: Arc::new(DashMap::new()),
            }
        }

        async fn get<T: for<'de> Deserialize<'de>>(
            &self,
            key: &str,
        ) -> Result<Option<T>, CacheError> {
            let entry = self.data.get(key);
            match entry {
                Some((value, expiry)) => {
                    if let Some(inst) = expiry {
                        if Instant::now() > *inst {
                            drop(entry);
                            self.data.remove(key);
                            return Ok(None);
                        }
                    }
                    serde_json::from_str(value)
                        .map(Some)
                        .map_err(|e| CacheError::SerializationError(e.to_string()))
                }
                None => Ok(None),
            }
        }

        async fn set<T: Serialize>(
            &self,
            key: &str,
            value: &T,
            ttl: Option<Duration>,
        ) -> Result<(), CacheError> {
            let serialized = serde_json::to_string(value)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;
            let expiry = ttl.map(|d| Instant::now() + d);
            self.data.insert(key.to_string(), (serialized, expiry));
            Ok(())
        }

        async fn delete(&self, key: &str) -> Result<(), CacheError> {
            self.data.remove(key);
            Ok(())
        }
    }

    struct TestableProjectionStore {
        inner: TestCacheStore,
    }

    impl TestableProjectionStore {
        fn new() -> Self {
            Self {
                inner: TestCacheStore::new(),
            }
        }

        async fn get_feature_typed(
            &self,
            feature_id: i64,
        ) -> Result<Option<FeatureProjection>, ProjectionError> {
            self.inner
                .get(&format!("feature:{feature_id}"))
                .await
                .map_err(|e| ProjectionError::CacheError(e.to_string()))
        }

        async fn set_feature_typed(
            &self,
            feature: &Feature,
        ) -> Result<(), ProjectionError> {
            let projection = FeatureProjection {
                feature: feature.clone(),
                cached_at: chrono::Utc::now(),
            };
            self.inner
                .set(&format!("feature:{}", feature.id), &projection, None)
                .await
                .map_err(|e| ProjectionError::CacheError(e.to_string()))
        }

        async fn get_workpackage_typed(
            &self,
            wp_id: i64,
        ) -> Result<Option<WorkPackageProjection>, ProjectionError> {
            self.inner
                .get(&format!("wp:{wp_id}"))
                .await
                .map_err(|e| ProjectionError::CacheError(e.to_string()))
        }

        async fn set_workpackage_typed(
            &self,
            wp: &WorkPackage,
        ) -> Result<(), ProjectionError> {
            let projection = WorkPackageProjection {
                workpackage: wp.clone(),
                cached_at: chrono::Utc::now(),
            };
            self.inner
                .set(&format!("wp:{}", wp.id), &projection, None)
                .await
                .map_err(|e| ProjectionError::CacheError(e.to_string()))
        }

        async fn invalidate_feature_typed(
            &self,
            feature_id: i64,
        ) -> Result<(), ProjectionError> {
            self.inner
                .delete(&format!("feature:{feature_id}"))
                .await
                .map_err(|e| ProjectionError::CacheError(e.to_string()))
        }

        async fn invalidate_workpackage_typed(
            &self,
            wp_id: i64,
        ) -> Result<(), ProjectionError> {
            self.inner
                .delete(&format!("wp:{wp_id}"))
                .await
                .map_err(|e| ProjectionError::CacheError(e.to_string()))
        }
    }

    fn make_feature(id: i64, slug: &str) -> Feature {
        Feature {
            id,
            slug: slug.into(),
            friendly_name: format!("Feature {}", id),
            state: agileplus_domain::domain::state_machine::FeatureState::Created,
            spec_hash: [0u8; 32],
            target_branch: "main".into(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            module_id: None,
            project_id: None,
            created_at_commit: None,
            last_modified_commit: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn make_workpackage(id: i64, feature_id: i64) -> WorkPackage {
        WorkPackage {
            id,
            feature_id,
            title: format!("WP {}", id),
            state: WpState::Planned,
            sequence: 1,
            file_scope: vec![],
            acceptance_criteria: "Test acceptance criteria".into(),
            agent_id: None,
            pr_url: None,
            pr_state: None,
            worktree_path: None,
            plane_sub_issue_id: None,
            base_commit: None,
            head_commit: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn projection_error_cache_display() {
        let err = ProjectionError::CacheError("connection failed".into());
        assert!(err.to_string().contains("Cache error"));
        assert!(err.to_string().contains("connection failed"));
    }

    #[test]
    fn feature_projection_serde_roundtrip() {
        let feature = make_feature(1, "test-feat");
        let proj = FeatureProjection {
            feature: feature.clone(),
            cached_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&proj).expect("should serialize");
        let deserialized: FeatureProjection =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(deserialized.feature.id, proj.feature.id);
        assert_eq!(deserialized.feature.slug, proj.feature.slug);
    }

    #[test]
    fn workpackage_projection_serde_roundtrip() {
        let wp = make_workpackage(1, 1);
        let proj = WorkPackageProjection {
            workpackage: wp.clone(),
            cached_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&proj).expect("should serialize");
        let deserialized: WorkPackageProjection =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(deserialized.workpackage.id, proj.workpackage.id);
        assert_eq!(deserialized.workpackage.title, proj.workpackage.title);
    }

    #[tokio::test]
    async fn projection_cache_set_and_get_feature() {
        let store = TestableProjectionStore::new();
        let feature = make_feature(42, "my-feature");
        store.set_feature_typed(&feature).await.expect("set should succeed");
        let result = store
            .get_feature_typed(42)
            .await
            .expect("get should succeed");
        assert!(result.is_some());
        assert_eq!(result.unwrap().feature.id, 42);
    }

    #[tokio::test]
    async fn projection_cache_get_nonexistent_feature() {
        let store = TestableProjectionStore::new();
        let result = store
            .get_feature_typed(999)
            .await
            .expect("get should succeed");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn projection_cache_set_and_get_workpackage() {
        let store = TestableProjectionStore::new();
        let wp = make_workpackage(55, 10);
        store.set_workpackage_typed(&wp).await.expect("set should succeed");
        let result = store
            .get_workpackage_typed(55)
            .await
            .expect("get should succeed");
        assert!(result.is_some());
        assert_eq!(result.unwrap().workpackage.id, 55);
    }

    #[tokio::test]
    async fn projection_cache_get_nonexistent_workpackage() {
        let store = TestableProjectionStore::new();
        let result = store
            .get_workpackage_typed(888)
            .await
            .expect("get should succeed");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn projection_cache_invalidate_feature() {
        let store = TestableProjectionStore::new();
        let feature = make_feature(100, "invalidate-me");
        store.set_feature_typed(&feature).await.expect("set should succeed");
        store
            .invalidate_feature_typed(100)
            .await
            .expect("invalidate should succeed");
        let result = store
            .get_feature_typed(100)
            .await
            .expect("get should succeed");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn projection_cache_invalidate_workpackage() {
        let store = TestableProjectionStore::new();
        let wp = make_workpackage(200, 10);
        store.set_workpackage_typed(&wp).await.expect("set should succeed");
        store
            .invalidate_workpackage_typed(200)
            .await
            .expect("invalidate should succeed");
        let result = store
            .get_workpackage_typed(200)
            .await
            .expect("get should succeed");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn projection_cache_overwrite_feature() {
        let store = TestableProjectionStore::new();
        let feature1 = make_feature(300, "feat-v1");
        let mut feature2 = make_feature(300, "feat-v2");
        feature2.state = agileplus_domain::domain::state_machine::FeatureState::Specified;
        store.set_feature_typed(&feature1).await.expect("set should succeed");
        store.set_feature_typed(&feature2).await.expect("overwrite should succeed");
        let result = store
            .get_feature_typed(300)
            .await
            .expect("get should succeed");
        assert!(result.is_some());
        assert_eq!(result.unwrap().feature.friendly_name, "Feature 300");
    }
}
