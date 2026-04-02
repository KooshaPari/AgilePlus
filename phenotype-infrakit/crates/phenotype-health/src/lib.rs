//! # Phenotype Health
//!
//! This crate provides health checking capabilities for services and infrastructure components.
//!
//! ## Features
//!
//! - Service-level health checks with async support
//! - Project-level health metrics and scoring
//! - Health status aggregation and reporting
//!
//! ## Usage
//!
//! ```rust,ignore
//! use phenotype_health::{HealthCheck, HealthStatus};
//! use phenotype_health::checkers::MemoryHealthChecker;
//!
//! #[tokio::main]
//! async fn main() {
//!     let checker = MemoryHealthChecker::new(100);
//!     let status = checker.check().await.unwrap();
//!     println!("Health status: {:?}", status);
//! }
//! ```

pub mod checkers;
pub mod project;

pub use checkers::{
    CacheHealthChecker, DatabaseHealthChecker, ExternalServiceHealthChecker, HealthCheck,
    HealthCheckConfig, HealthResult, HealthStatus, MemoryHealthChecker,
};
pub use project::{HealthBand, HealthDimension, LanguageStack, ProjectHealth};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health response containing aggregated status of all checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub checks: HashMap<String, HealthStatus>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthResponse {
    /// Create a new health response with the given status.
    pub fn new(status: HealthStatus) -> Self {
        Self {
            status,
            checks: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add a check result to the response.
    pub fn with_check(mut self, name: String, status: HealthStatus) -> Self {
        self.checks.insert(name, status);
        self.status = self.aggregate_status();
        self
    }

    /// Aggregate the overall status from all checks.
    fn aggregate_status(&self) -> HealthStatus {
        if self.checks.is_empty() {
            return HealthStatus::Unknown;
        }

        let has_degraded = self.checks.values().any(|s| *s == HealthStatus::Degraded);
        let has_unhealthy = self.checks.values().any(|s| *s == HealthStatus::Unhealthy);

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}
