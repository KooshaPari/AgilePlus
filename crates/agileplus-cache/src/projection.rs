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

    #[test]
    fn projection_error_implements_std_error() {
        fn assert_error<T: std::error::Error + Send + Sync>() {}
        assert_error::<ProjectionError>();
    }

    #[test]
    fn projection_error_wraps_message_without_source() {
        let error = ProjectionError::CacheError("backend down".to_string());
        assert!(error.to_string().contains("backend down"));
        assert!(std::error::Error::source(&error).is_none());
    }

    #[test]
    fn projection_key_layout_is_stable() {
        assert_eq!(format!("feature:{}", 7), "feature:7");
        assert_eq!(format!("wp:{}", 7), "wp:7");
    }
}
