//! Error types for analytics

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalyticsError {
    #[error("tracking failed: {0}")]
    TrackingFailed(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AnalyticsError>;
