use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::{ArtifactError, Result};

#[async_trait]
pub trait ArtifactStore: Send + Sync {
    async fn ensure_buckets(&self) -> Result<()>;
    async fn upload(
        &self,
        bucket: &str,
        key: &str,
        data: Bytes,
        content_type: &str,
    ) -> Result<String>;
    async fn download(&self, bucket: &str, key: &str) -> Result<Bytes>;
    async fn archive_old_events(
        &self,
        events: &[agileplus_domain::domain::event::Event],
        before: DateTime<Utc>,
    ) -> Result<u64>;
    async fn health_check(&self) -> Result<()>;
}

pub struct InMemoryArtifactStore {
    storage: HashMap<String, HashMap<String, Bytes>>,
}

impl InMemoryArtifactStore {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn buckets(&self) -> Vec<String> {
        self.storage.keys().cloned().collect()
    }

    pub fn keys_for_bucket(&self, bucket: &str) -> Vec<String> {
        self.storage
            .get(bucket)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for InMemoryArtifactStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ArtifactStore for InMemoryArtifactStore {
    async fn ensure_buckets(&self) -> Result<()> {
        let buckets = ["events-archive", "audit-archive", "specs", "artifacts"];
        for bucket in buckets {
            self.storage.entry(bucket.to_string()).or_default();
            info!(bucket, "Bucket ensured");
        }
        Ok(())
    }

    async fn upload(
        &self,
        bucket: &str,
        key: &str,
        data: Bytes,
        _content_type: &str,
    ) -> Result<String> {
        let bucket_storage = self.storage.get(bucket).ok_or_else(|| {
            warn!(bucket, "Upload to non-existent bucket");
            ArtifactError::BucketNotFound(bucket.to_string())
        })?;
        bucket_storage.contains_key(key);
        let url = format!("mem://{}/{}", bucket, key);
        drop(bucket_storage);
        self.storage
            .get_mut(bucket)
            .ok_or(ArtifactError::BucketNotFound(bucket.to_string()))?
            .insert(key.to_string(), data);
        info!(bucket, key, "Upload complete");
        Ok(url)
    }

    async fn download(&self, bucket: &str, key: &str) -> Result<Bytes> {
        self.storage
            .get(bucket)
            .and_then(|m| m.get(key))
            .cloned()
            .ok_or_else(|| {
                warn!(bucket, key, "Download failed - key not found");
                ArtifactError::KeyNotFound(format!("{}/{}", bucket, key))
            })
    }

    async fn archive_old_events(
        &self,
        events: &[agileplus_domain::domain::event::Event],
        before: DateTime<Utc>,
    ) -> Result<u64> {
        let bucket = "events-archive";
        if !self.storage.contains_key(bucket) {
            self.storage.entry(bucket.to_string()).or_default();
        }
        let mut count = 0u64;
        let bucket_storage = self.storage.get_mut(bucket).unwrap();
        for event in events {
            if event.timestamp < before {
                let key = format!(
                    "{}/{}/{}.json",
                    event.entity_type,
                    event.entity_id,
                    event.id
                );
                let data = serde_json::to_vec(event).map_err(|e| {
                    ArtifactError::UploadFailed(format!("Serialization failed: {}", e))
                })?;
                bucket_storage.insert(key, Bytes::from(data));
                count += 1;
            }
        }
        info!(count, "Archived old events");
        Ok(count)
    }

    async fn health_check(&self) -> Result<()> {
        if self.storage.contains_key("events-archive") {
            Ok(())
        } else {
            Err(ArtifactError::HealthCheckFailed(
                "InMemoryArtifactStore not initialized".to_string(),
            ))
        }
    }
}

pub struct S3ArtifactStore {
    bucket: String,
}

impl S3ArtifactStore {
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
        }
    }
}

#[async_trait]
impl ArtifactStore for S3ArtifactStore {
    async fn ensure_buckets(&self) -> Result<()> {
        info!(bucket = %self.bucket, "S3ArtifactStore.ensure_buckets called - S3 stub");
        Ok(())
    }

    async fn upload(
        &self,
        bucket: &str,
        key: &str,
        _data: Bytes,
        _content_type: &str,
    ) -> Result<String> {
        info!(bucket, key, "S3ArtifactStore.upload called - S3 stub");
        Err(ArtifactError::S3Error(
            "S3 implementation not yet available".to_string(),
        ))
    }

    async fn download(&self, _bucket: &str, _key: &str) -> Result<Bytes> {
        Err(ArtifactError::S3Error(
            "S3 implementation not yet available".to_string(),
        ))
    }

    async fn archive_old_events(
        &self,
        _events: &[agileplus_domain::domain::event::Event],
        _before: DateTime<Utc>,
    ) -> Result<u64> {
        Err(ArtifactError::S3Error(
            "S3 implementation not yet available".to_string(),
        ))
    }

    async fn health_check(&self) -> Result<()> {
        Err(ArtifactError::S3Error(
            "S3 implementation not yet available".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::event::Event;
    use chrono::Utc;

    fn create_test_event(id: i64, hours_ago: i64) -> Event {
        let timestamp = Utc::now() - chrono::Duration::hours(hours_ago);
        Event {
            id,
            entity_type: "feature".to_string(),
            entity_id: 1,
            event_type: "created".to_string(),
            payload: serde_json::json!({}),
            actor: "test".to_string(),
            timestamp,
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            sequence: id,
        }
    }

    #[tokio::test]
    async fn test_ensure_buckets() {
        let store = InMemoryArtifactStore::new();
        store.ensure_buckets().await.unwrap();
        assert_eq!(store.buckets().len(), 4);
        assert!(store.buckets().contains(&"events-archive".to_string()));
        assert!(store.buckets().contains(&"audit-archive".to_string()));
        assert!(store.buckets().contains(&"specs".to_string()));
        assert!(store.buckets().contains(&"artifacts".to_string()));
    }

    #[tokio::test]
    async fn test_upload_download() {
        let store = InMemoryArtifactStore::new();
        store.ensure_buckets().await.unwrap();
        let data = Bytes::from("test content");
        let url = store
            .upload("artifacts", "test/key.txt", data.clone(), "text/plain")
            .await
            .unwrap();
        assert_eq!(url, "mem://artifacts/test/key.txt");
        let retrieved = store.download("artifacts", "test/key.txt").await.unwrap();
        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn test_download_missing_key() {
        let store = InMemoryArtifactStore::new();
        store.ensure_buckets().await.unwrap();
        let result = store.download("artifacts", "nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_archive_old_events() {
        let store = InMemoryArtifactStore::new();
        store.ensure_buckets().await.unwrap();
        let events = vec![
            create_test_event(1, 100),
            create_test_event(2, 50),
            create_test_event(3, 10),
        ];
        let cutoff = Utc::now() - chrono::Duration::hours(48);
        let archived = store.archive_old_events(&events, cutoff).await.unwrap();
        assert_eq!(archived, 2);
        let keys = store.keys_for_bucket("events-archive");
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_health_check() {
        let store = InMemoryArtifactStore::new();
        store.ensure_buckets().await.unwrap();
        assert!(store.health_check().await.is_ok());
    }
}
