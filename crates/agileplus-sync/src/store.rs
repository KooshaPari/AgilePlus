//! SyncMappingStore trait for persistence abstraction.
//!
//! Traceability: FR-SYNC-STORE / WP09-T056

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use agileplus_domain::domain::sync_mapping::SyncMapping;

use crate::error::SyncError;

/// Persistence abstraction for sync mappings.
///
/// Implementations are expected to be cheaply cloneable (e.g., wrapping an
/// `Arc<dyn SyncMappingStore>`).
#[async_trait]
pub trait SyncMappingStore: Send + Sync {
    /// Persist a new sync mapping and return its assigned id.
    async fn create(&self, mapping: SyncMapping) -> Result<i64, SyncError>;

    /// Retrieve a mapping by entity type and local entity id.
    async fn get_by_entity(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Option<SyncMapping>, SyncError>;

    /// Update the stored content hash and last-synced timestamp for a mapping.
    async fn update_hash(
        &self,
        id: i64,
        new_hash: String,
        synced_at: DateTime<Utc>,
    ) -> Result<(), SyncError>;

    /// Increment the conflict counter for a mapping.
    async fn increment_conflict(&self, id: i64) -> Result<(), SyncError>;

    /// Return all stored mappings.
    async fn list_all(&self) -> Result<Vec<SyncMapping>, SyncError>;
}

// ---------------------------------------------------------------------------
// In-memory implementation for tests
// ---------------------------------------------------------------------------

#[cfg(test)]
pub mod mem {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    pub struct InMemoryStore {
        inner: Arc<Mutex<Vec<SyncMapping>>>,
        next_id: Arc<Mutex<i64>>,
    }

    #[async_trait]
    impl SyncMappingStore for InMemoryStore {
        async fn create(&self, mut mapping: SyncMapping) -> Result<i64, SyncError> {
            let mut id_lock = self.next_id.lock().unwrap();
            *id_lock += 1;
            mapping.id = *id_lock;
            self.inner.lock().unwrap().push(mapping);
            Ok(*id_lock)
        }

        async fn get_by_entity(
            &self,
            entity_type: &str,
            entity_id: i64,
        ) -> Result<Option<SyncMapping>, SyncError> {
            let lock = self.inner.lock().unwrap();
            Ok(lock
                .iter()
                .find(|m| m.entity_type == entity_type && m.entity_id == entity_id)
                .cloned())
        }

        async fn update_hash(
            &self,
            id: i64,
            new_hash: String,
            synced_at: DateTime<Utc>,
        ) -> Result<(), SyncError> {
            let mut lock = self.inner.lock().unwrap();
            if let Some(m) = lock.iter_mut().find(|m| m.id == id) {
                m.content_hash = new_hash;
                m.last_synced_at = synced_at;
                Ok(())
            } else {
                Err(SyncError::Store(format!("mapping {id} not found")))
            }
        }

        async fn increment_conflict(&self, id: i64) -> Result<(), SyncError> {
            let mut lock = self.inner.lock().unwrap();
            if let Some(m) = lock.iter_mut().find(|m| m.id == id) {
                m.increment_conflict();
                Ok(())
            } else {
                Err(SyncError::Store(format!("mapping {id} not found")))
            }
        }

        async fn list_all(&self) -> Result<Vec<SyncMapping>, SyncError> {
            Ok(self.inner.lock().unwrap().clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mem::InMemoryStore;
    use super::*;
    use agileplus_domain::domain::sync_mapping::SyncMapping;

    #[tokio::test]
    async fn create_and_retrieve() {
        let store = InMemoryStore::default();
        let m = SyncMapping::new("feature", 10, "plane-001", "hash-aaa");
        let id = store.create(m).await.unwrap();
        assert_eq!(id, 1);

        let found = store.get_by_entity("feature", 10).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().plane_issue_id, "plane-001");
    }

    #[tokio::test]
    async fn update_hash() {
        let store = InMemoryStore::default();
        let m = SyncMapping::new("wp", 5, "plane-999", "old-hash");
        let id = store.create(m).await.unwrap();

        store
            .update_hash(id, "new-hash".to_string(), Utc::now())
            .await
            .unwrap();

        let found = store.get_by_entity("wp", 5).await.unwrap().unwrap();
        assert_eq!(found.content_hash, "new-hash");
    }

    #[tokio::test]
    async fn increment_conflict() {
        let store = InMemoryStore::default();
        let m = SyncMapping::new("feature", 7, "plane-777", "h");
        let id = store.create(m).await.unwrap();

        store.increment_conflict(id).await.unwrap();
        store.increment_conflict(id).await.unwrap();

        let found = store.get_by_entity("feature", 7).await.unwrap().unwrap();
        assert_eq!(found.conflict_count, 2);
    }

    #[tokio::test]
    async fn list_all() {
        let store = InMemoryStore::default();
        store
            .create(SyncMapping::new("a", 1, "p1", "h1"))
            .await
            .unwrap();
        store
            .create(SyncMapping::new("b", 2, "p2", "h2"))
            .await
            .unwrap();
        let all = store.list_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn create_assigns_incrementing_ids() {
        let store = InMemoryStore::default();
        let id1 = store
            .create(SyncMapping::new("feature", 1, "p1", "h1"))
            .await
            .unwrap();
        let id2 = store
            .create(SyncMapping::new("feature", 2, "p2", "h2"))
            .await
            .unwrap();
        let id3 = store
            .create(SyncMapping::new("feature", 3, "p3", "h3"))
            .await
            .unwrap();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[tokio::test]
    async fn get_by_entity_returns_none_when_missing() {
        let store = InMemoryStore::default();
        store
            .create(SyncMapping::new("feature", 1, "p1", "h1"))
            .await
            .unwrap();
        assert!(store.get_by_entity("feature", 999).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn get_by_entity_returns_none_when_wrong_type() {
        let store = InMemoryStore::default();
        store
            .create(SyncMapping::new("feature", 1, "p1", "h1"))
            .await
            .unwrap();
        assert!(store
            .get_by_entity("work_package", 1)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn get_by_entity_distinguishes_same_id_different_type() {
        let store = InMemoryStore::default();
        store
            .create(SyncMapping::new("feature", 7, "p-f", "h-f"))
            .await
            .unwrap();
        store
            .create(SyncMapping::new("work_package", 7, "p-w", "h-w"))
            .await
            .unwrap();

        let f = store.get_by_entity("feature", 7).await.unwrap().unwrap();
        let w = store
            .get_by_entity("work_package", 7)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(f.plane_issue_id, "p-f");
        assert_eq!(w.plane_issue_id, "p-w");
    }

    #[tokio::test]
    async fn update_hash_sets_last_synced_timestamp() {
        let store = InMemoryStore::default();
        let id = store
            .create(SyncMapping::new("feature", 1, "p1", "old"))
            .await
            .unwrap();

        let ts = Utc::now();
        store.update_hash(id, "new-hash".into(), ts).await.unwrap();

        let found = store.get_by_entity("feature", 1).await.unwrap().unwrap();
        assert_eq!(found.content_hash, "new-hash");
        assert_eq!(found.last_synced_at, ts);
    }

    #[tokio::test]
    async fn update_hash_nonexistent_returns_error() {
        let store = InMemoryStore::default();
        let err = store
            .update_hash(42, "h".into(), Utc::now())
            .await
            .unwrap_err();
        assert!(matches!(err, SyncError::Store(_)));
    }

    #[tokio::test]
    async fn increment_conflict_nonexistent_returns_error() {
        let store = InMemoryStore::default();
        let err = store.increment_conflict(99).await.unwrap_err();
        assert!(matches!(err, SyncError::Store(_)));
    }

    #[tokio::test]
    async fn list_all_empty_when_no_mappings() {
        let store = InMemoryStore::default();
        assert!(store.list_all().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn store_clone_shares_state() {
        let store = InMemoryStore::default();
        store
            .create(SyncMapping::new("feature", 1, "p1", "h1"))
            .await
            .unwrap();

        let clone = store.clone();
        let found = clone.get_by_entity("feature", 1).await.unwrap();
        assert!(found.is_some());

        // Mutation through one handle is visible on the other.
        let id = found.unwrap().id;
        store.increment_conflict(id).await.unwrap();
        let updated = clone.get_by_entity("feature", 1).await.unwrap().unwrap();
        assert_eq!(updated.conflict_count, 1);
    }

    #[tokio::test]
    async fn full_store_lifecycle() {
        let store = InMemoryStore::default();

        // Create.
        let id = store
            .create(SyncMapping::new("feature", 100, "PLN-100", "hash-v1"))
            .await
            .unwrap();

        // Retrieve.
        let m = store.get_by_entity("feature", 100).await.unwrap().unwrap();
        assert_eq!(m.content_hash, "hash-v1");
        assert_eq!(m.conflict_count, 0);

        // Update hash.
        store
            .update_hash(id, "hash-v2".into(), Utc::now())
            .await
            .unwrap();
        let updated = store.get_by_entity("feature", 100).await.unwrap().unwrap();
        assert_eq!(updated.content_hash, "hash-v2");

        // Increment conflict.
        store.increment_conflict(id).await.unwrap();
        store.increment_conflict(id).await.unwrap();
        let after = store.get_by_entity("feature", 100).await.unwrap().unwrap();
        assert_eq!(after.conflict_count, 2);
    }
}
