//! Error types for health monitoring

use thiserror::Error;

/// Errors that can occur in health monitoring
#[derive(Error, Debug)]
pub enum HealthMonitorError {
    #[error("health check failed for '{0}': {1}")]
    CheckFailed(String, String),

    #[error("target '{0}' not found")]
    TargetNotFound(String),

    #[error("monitoring error: {0}")]
    MonitoringError(String),

    #[error("recovery action failed: {0}")]
    RecoveryFailed(String),

    #[error("timeout waiting for health check")]
    Timeout,

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}
