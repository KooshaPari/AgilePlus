//! # Phenotype Ports Canonical
//!
//! Canonical port traits for the Phenotype hexagonal architecture ecosystem.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// =============================================================================
// Errors
// =============================================================================

/// Port operation errors
#[derive(Error, Debug)]
pub enum PortError {
    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Result type for port operations
pub type PortResult<T> = Result<T, PortError>;

// =============================================================================
// Repository Port
// =============================================================================

/// Repository port for data access
#[async_trait]
pub trait Repository<T: Send + Sync>: Send + Sync {
    /// Find entity by ID
    async fn find(&self, id: &str) -> PortResult<Option<T>>;

    /// Find all entities
    async fn find_all(&self) -> PortResult<Vec<T>>;

    /// Save entity
    async fn save(&self, entity: &T) -> PortResult<T>;

    /// Delete entity by ID
    async fn delete(&self, id: &str) -> PortResult<()>;

    /// Check if entity exists
    async fn exists(&self, id: &str) -> PortResult<bool>;
}

// =============================================================================
// Cache Port
// =============================================================================

/// Cache port for caching operations
#[async_trait]
pub trait CachePort<K: Send + Sync, V: Send + Sync>: Send + Sync {
    /// Get value by key
    async fn get(&self, key: &K) -> PortResult<Option<V>>;

    /// Set value with optional TTL
    async fn set(&self, key: &K, value: &V, ttl_secs: Option<u64>) -> PortResult<()>;

    /// Delete value by key
    async fn delete(&self, key: &K) -> PortResult<()>;

    /// Check if key exists
    async fn exists(&self, key: &K) -> PortResult<bool>;

    /// Clear all cache entries
    async fn clear(&self) -> PortResult<()>;
}

// =============================================================================
// Event Bus Port
// =============================================================================

/// Event bus port for publishing and subscribing to events
#[async_trait]
pub trait EventBus<P: Send + Sync>: Send + Sync {
    /// Event payload type
    type Event: Send + Sync + Serialize + for<'de> Deserialize<'de>;

    /// Publish an event
    async fn publish(&self, event: Self::Event) -> PortResult<()>;

    /// Subscribe to events
    async fn subscribe<F>(&self, handler: F) -> PortResult<()>
    where
        F: Fn(Self::Event) -> PortResult<()> + Send + Sync + 'static;
}

// =============================================================================
// Health Status
// =============================================================================

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is unhealthy
    Unhealthy,
    /// Health status unknown
    Unknown,
}

/// Health check port for service monitoring
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Health check result
    fn status(&self) -> HealthStatus;

    /// Check if healthy
    fn is_healthy(&self) -> bool {
        self.status() == HealthStatus::Healthy
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_port_error_display() {
        let err = PortError::NotFound("user:123".to_string());
        assert!(err.to_string().contains("user:123"));
    }
}
