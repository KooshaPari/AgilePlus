//! Artifact storage for AgilePlus — MinIO/S3 object storage operations.
//!
//! Provides an `ArtifactStore` trait for uploading, downloading, and archiving
//! artifacts with support for both in-memory (testing) and S3/MinIO backends.
//!
//! Traceability: FR-ARTIFACT-* / WP06

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use thiserror::Error;

pub use crate::store::{ArtifactStore, InMemoryArtifactStore, S3ArtifactStore};

mod store;

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("Bucket not found: {0}")]
    BucketNotFound(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Upload failed: {0}")]
    UploadFailed(String),
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),
    #[error("S3 operation failed: {0}")]
    S3Error(String),
}

pub type Result<T> = std::result::Result<T, ArtifactError>;
