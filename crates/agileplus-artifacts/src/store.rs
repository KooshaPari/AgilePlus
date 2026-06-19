// SPDX-License-Identifier: MIT OR Apache-2.0
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use bytes::Bytes;
use std::collections::HashMap;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("artifact not found: {bucket}/{key}")]
    ArtifactNotFound { bucket: String, key: String },
    #[error("S3 operation failed: {0}")]
    S3(String),
}

pub type Result<T> = std::result::Result<T, ArtifactError>;

#[async_trait]
pub trait ArtifactStore: Send + Sync {
    async fn ensure_buckets(&self, buckets: &[&str]) -> Result<()>;
    async fn upload(
        &self,
        bucket: &str,
        key: &str,
        data: Bytes,
        content_type: Option<&str>,
    ) -> Result<()>;
    async fn download(&self, bucket: &str, key: &str) -> Result<Bytes>;
    async fn archive_old_events(&self, older_than_days: u32) -> Result<u64>;
    async fn health_check(&self) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct InMemoryArtifactStore {
    buckets: RwLock<HashMap<String, HashMap<String, Bytes>>>,
}

impl InMemoryArtifactStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn buckets(&self) -> Vec<String> {
        self.buckets.read().await.keys().cloned().collect()
    }
}

#[async_trait]
impl ArtifactStore for InMemoryArtifactStore {
    async fn ensure_buckets(&self, buckets: &[&str]) -> Result<()> {
        let mut store = self.buckets.write().await;
        for bucket in buckets {
            store.entry((*bucket).to_string()).or_default();
        }
        Ok(())
    }

    async fn upload(
        &self,
        bucket: &str,
        key: &str,
        data: Bytes,
        _content_type: Option<&str>,
    ) -> Result<()> {
        let mut store = self.buckets.write().await;
        store
            .entry(bucket.to_string())
            .or_default()
            .insert(key.to_string(), data);
        Ok(())
    }

    async fn download(&self, bucket: &str, key: &str) -> Result<Bytes> {
        let store = self.buckets.read().await;
        store
            .get(bucket)
            .and_then(|bucket_data| bucket_data.get(key))
            .cloned()
            .ok_or_else(|| ArtifactError::ArtifactNotFound {
                bucket: bucket.to_string(),
                key: key.to_string(),
            })
    }

    async fn archive_old_events(&self, older_than_days: u32) -> Result<u64> {
        let mut store = self.buckets.write().await;
        let Some(events) = store.get_mut("events-archive") else {
            return Ok(0);
        };
        let before = chrono::Utc::now() - chrono::Duration::days(older_than_days as i64);
        let before_ts = before.timestamp();
        let keys_to_remove: Vec<_> = events
            .keys()
            .filter(|key| {
                key.parse::<i64>()
                    .is_ok_and(|timestamp| timestamp < before_ts)
            })
            .cloned()
            .collect();
        let count = keys_to_remove.len() as u64;
        for key in keys_to_remove {
            events.remove(&key);
        }
        Ok(count)
    }

    async fn health_check(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct S3ArtifactStore {
    endpoint: String,
    access_key: String,
    secret_key: String,
    region: String,
    bucket_prefix: String,
    client: reqwest::Client,
}

impl S3ArtifactStore {
    pub fn new(
        endpoint: String,
        access_key: String,
        secret_key: String,
        region: String,
        bucket_prefix: String,
    ) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        Ok(Self {
            endpoint,
            access_key,
            secret_key,
            region,
            bucket_prefix,
            client,
        })
    }
}

#[async_trait]
impl ArtifactStore for S3ArtifactStore {
    async fn ensure_buckets(&self, buckets: &[&str]) -> Result<()> {
        info!(
            endpoint = %self.endpoint,
            region = %self.region,
            bucket_prefix = %self.bucket_prefix,
            bucket_count = buckets.len(),
            access_key_configured = !self.access_key.is_empty(),
            secret_key_configured = !self.secret_key.is_empty(),
            "S3ArtifactStore.ensure_buckets called"
        );
        Ok(())
    }

    async fn upload(
        &self,
        bucket: &str,
        key: &str,
        data: Bytes,
        content_type: Option<&str>,
    ) -> Result<()> {
        let url = format!("{}/{}/{}", self.endpoint, bucket, key);
        info!(
            url = %url,
            bucket,
            key,
            size = data.len(),
            "S3ArtifactStore.upload: PUT object"
        );

        let mut req = self.client.put(&url).body(data);
        if let Some(ct) = content_type {
            req = req.header("Content-Type", ct);
        }
        // Basic auth header (access_key as username, secret_key as password)
        let auth_value = format!("{}:{}", self.access_key, self.secret_key);
        req = req.header(
            "Authorization",
            format!("Basic {}", STANDARD.encode(&auth_value)),
        );

        let resp = req
            .send()
            .await
            .map_err(|e| ArtifactError::S3(format!("upload request failed: {e}")))?;

        let status = resp.status();
        if status.is_success() {
            info!(bucket, key, status = %status, "upload succeeded");
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_else(|_| "(unreadable)".into());
            Err(ArtifactError::S3(format!(
                "upload failed: status={status} body={body}"
            )))
        }
    }

    async fn download(&self, bucket: &str, key: &str) -> Result<Bytes> {
        let url = format!("{}/{}/{}", self.endpoint, bucket, key);
        info!(
            url = %url,
            bucket,
            key,
            "S3ArtifactStore.download: GET object"
        );

        let auth_value = format!("{}:{}", self.access_key, self.secret_key);
        let resp = self
            .client
            .get(&url)
            .header(
                "Authorization",
                format!("Basic {}", STANDARD.encode(&auth_value)),
            )
            .send()
            .await
            .map_err(|e| ArtifactError::S3(format!("download request failed: {e}")))?;

        let status = resp.status();
        if status.is_success() {
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| ArtifactError::S3(format!("failed to read body: {e}")))?;
            info!(bucket, key, size = bytes.len(), "download succeeded");
            Ok(bytes)
        } else if status == reqwest::StatusCode::NOT_FOUND {
            Err(ArtifactError::ArtifactNotFound {
                bucket: bucket.to_string(),
                key: key.to_string(),
            })
        } else {
            let body = resp.text().await.unwrap_or_else(|_| "(unreadable)".into());
            Err(ArtifactError::S3(format!(
                "download failed: status={status} body={body}"
            )))
        }
    }

    async fn archive_old_events(&self, older_than_days: u32) -> Result<u64> {
        warn!(
            older_than_days,
            "S3ArtifactStore.archive_old_events: not yet implemented, returning 0"
        );
        Ok(0)
    }

    async fn health_check(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_upload_download_round_trip() {
        let store = InMemoryArtifactStore::new();
        store.ensure_buckets(&["artifacts"]).await.unwrap();

        let body = Bytes::from_static(b"artifact-body");
        store
            .upload(
                "artifacts",
                "evidence/report.md",
                body.clone(),
                Some("text/markdown"),
            )
            .await
            .unwrap();

        assert_eq!(
            store
                .download("artifacts", "evidence/report.md")
                .await
                .unwrap(),
            body
        );
    }

    #[tokio::test]
    async fn in_memory_download_missing_key_returns_error() {
        let store = InMemoryArtifactStore::new();

        let err = store.download("artifacts", "missing").await.unwrap_err();

        assert!(matches!(err, ArtifactError::ArtifactNotFound { .. }));
    }

    #[tokio::test]
    async fn in_memory_archive_old_timestamp_keys() {
        let store = InMemoryArtifactStore::new();
        store.ensure_buckets(&["events-archive"]).await.unwrap();
        store
            .upload(
                "events-archive",
                "1",
                Bytes::from_static(b"old"),
                Some("application/json"),
            )
            .await
            .unwrap();

        assert_eq!(store.archive_old_events(1).await.unwrap(), 1);
    }
}
