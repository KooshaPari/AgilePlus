//! Health monitoring strategies

use crate::check::{HealthCheckResult, HealthStatus};
use serde::{Deserialize, Serialize};

/// Actions to take based on health evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthAction {
    #[default]
    Continue,
    Warn,
    Degrade,
    Recover,
    Fail,
}

/// Strategy for handling health check failures
pub trait HealthStrategy: Send + Sync {
    fn evaluate(&self, result: &HealthCheckResult) -> HealthAction;
    fn should_recover(&self, result: &HealthCheckResult) -> bool;
}

/// Threshold-based health strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdStrategy {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub slow_response_threshold_ms: u64,
}

impl Default for ThresholdStrategy {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 2,
            slow_response_threshold_ms: 1000,
        }
    }
}

impl HealthStrategy for ThresholdStrategy {
    fn evaluate(&self, result: &HealthCheckResult) -> HealthAction {
        if let Some(rt) = result.response_time_ms {
            if rt > self.slow_response_threshold_ms {
                return HealthAction::Degrade;
            }
        }

        match result.status {
            HealthStatus::Healthy => HealthAction::Continue,
            HealthStatus::Degraded => HealthAction::Warn,
            HealthStatus::Unhealthy => {
                if result.failure_count >= self.failure_threshold {
                    HealthAction::Fail
                } else {
                    HealthAction::Recover
                }
            }
            HealthStatus::Unknown => HealthAction::Warn,
        }
    }

    fn should_recover(&self, result: &HealthCheckResult) -> bool {
        result.status == HealthStatus::Healthy && result.failure_count < self.success_threshold
    }
}

/// Retry strategy with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStrategy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
        }
    }
}

impl RetryStrategy {
    pub fn calculate_delay(&self, attempt: u32) -> std::time::Duration {
        let exponential = self.base_delay_ms * 2u64.pow(attempt);
        let capped = exponential.min(self.max_delay_ms);
        std::time::Duration::from_millis(capped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_strategy_healthy() {
        let strategy = ThresholdStrategy::default();
        let result = HealthCheckResult::healthy("test");
        assert_eq!(strategy.evaluate(&result), HealthAction::Continue);
    }

    #[test]
    fn test_retry_strategy_delay() {
        let strategy = RetryStrategy::default();
        let delay1 = strategy.calculate_delay(0);
        let delay2 = strategy.calculate_delay(1);
        assert!(delay2 > delay1);
    }
}
