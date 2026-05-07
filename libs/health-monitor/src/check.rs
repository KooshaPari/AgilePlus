//! Health check types

use serde::{Deserialize, Serialize};

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    #[default]
    Unknown,
}

/// Result of a health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub target: String,
    pub status: HealthStatus,
    pub response_time_ms: Option<u64>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub failure_count: u32,
}

impl HealthCheckResult {
    pub fn healthy(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            status: HealthStatus::Healthy,
            response_time_ms: None,
            message: None,
            failure_count: 0,
        }
    }

    pub fn with_response_time(mut self, ms: u64) -> Self {
        self.response_time_ms = Some(ms);
        self
    }

    pub fn unhealthy(target: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            status: HealthStatus::Unhealthy,
            response_time_ms: None,
            message: Some(message.into()),
            failure_count: 1,
        }
    }

    pub fn with_failures(mut self, count: u32) -> Self {
        self.failure_count = count;
        self
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_default() {
        assert_eq!(HealthStatus::default(), HealthStatus::Unknown);
    }

    #[test]
    fn test_health_check_result_healthy() {
        let result = HealthCheckResult::healthy("test-service");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.failure_count, 0);
    }
}
