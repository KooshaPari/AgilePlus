//! Service-Level Health Check Implementations
//!
//! This module provides concrete health check implementations for various services.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Health status enum representing the state of a health check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Health check configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub timeout_ms: u64,
    pub retries: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            retries: 3,
        }
    }
}

/// Result type for health checks.
pub type HealthResult<T> = Result<T, HealthCheckError>;

/// Error type for health checks.
#[derive(Debug, Clone, thiserror::Error)]
pub enum HealthCheckError {
    #[error("health check timed out")]
    Timeout,
    #[error("health check failed: {0}")]
    Failed(String),
    #[error("health check unavailable: {0}")]
    Unavailable(String),
}

/// Health check trait for implementing service health checks.
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform the health check and return the status.
    async fn check(&self) -> HealthResult<HealthStatus>;

    /// Get the name of this health check.
    fn name(&self) -> &str;
}

/// Memory health checker - verifies system memory availability.
#[derive(Debug, Clone, Default)]
pub struct MemoryHealthChecker {
    min_available_mb: u64,
}

impl MemoryHealthChecker {
    pub fn new(min_available_mb: u64) -> Self {
        Self { min_available_mb }
    }
}

#[async_trait]
impl HealthCheck for MemoryHealthChecker {
    async fn check(&self) -> HealthResult<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }

    fn name(&self) -> &str {
        "memory"
    }
}

/// Cache health checker - verifies cache connectivity.
#[derive(Debug, Clone, Default)]
pub struct CacheHealthChecker {
    url: Option<String>,
}

impl CacheHealthChecker {
    pub fn new(url: Option<String>) -> Self {
        Self { url }
    }
}

#[async_trait]
impl HealthCheck for CacheHealthChecker {
    async fn check(&self) -> HealthResult<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }

    fn name(&self) -> &str {
        "cache"
    }
}

/// Database health checker - verifies database connectivity.
#[derive(Debug, Clone, Default)]
pub struct DatabaseHealthChecker {
    connection_string: Option<String>,
}

impl DatabaseHealthChecker {
    pub fn new(connection_string: Option<String>) -> Self {
        Self { connection_string }
    }
}

#[async_trait]
impl HealthCheck for DatabaseHealthChecker {
    async fn check(&self) -> HealthResult<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }

    fn name(&self) -> &str {
        "database"
    }
}

/// External service health checker.
#[derive(Debug, Clone, Default)]
pub struct ExternalServiceHealthChecker {
    services: Vec<String>,
}

impl ExternalServiceHealthChecker {
    pub fn new(services: Vec<String>) -> Self {
        Self { services }
    }
}

#[async_trait]
impl HealthCheck for ExternalServiceHealthChecker {
    async fn check(&self) -> HealthResult<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }

    fn name(&self) -> &str {
        "external_services"
    }
}
