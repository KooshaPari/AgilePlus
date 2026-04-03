//! # Phenotype Retry
//!
//! Retry utilities with exponential backoff for the Phenotype ecosystem.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::time::Duration;
use thiserror::Error;

// =============================================================================
// Errors
// =============================================================================

/// Retry operation errors
#[derive(Error, Debug)]
pub enum RetryError {
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Retry not allowed for this error")]
    NotRetryable,
}

/// Result type for retry operations
pub type RetryResult<T> = Result<T, RetryError>;

// =============================================================================
// Retry Configuration
// =============================================================================

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub multiplier: f64,
    /// Jitter factor (0.0 to 1.0)
    pub jitter: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: 0.1,
        }
    }
}

impl RetryConfig {
    /// Calculate delay for a given attempt
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_ms = self.base_delay.as_millis() as f64;
        let exponential_delay_ms = base_ms * self.multiplier.powi(attempt as i32);
        let capped_delay_ms = exponential_delay_ms.min(self.max_delay.as_millis() as f64);

        // Apply jitter
        let jitter_range = capped_delay_ms * self.jitter;
        let jitter = (rand_simple() * jitter_range * 2.0) - jitter_range;

        Duration::from_secs_f64((capped_delay_ms + jitter) / 1000.0)
    }
}

/// Simple random number generator (0.0 to 1.0)
fn rand_simple() -> f64 {
    use std::time::Instant;
    let seed = Instant::now().elapsed().as_nanos() as u64;
    (seed % 1000) as f64 / 1000.0
}

// =============================================================================
// Retry Operations
// =============================================================================

/// Execute an operation with retry
pub async fn retry_with_backoff<F, Fut, T>(
    config: &RetryConfig,
    mut operation: F,
) -> RetryResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = RetryResult<T>>,
{
    let mut last_error = RetryError::MaxRetriesExceeded;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = e;
                if attempt < config.max_retries {
                    tokio::time::sleep(config.delay_for_attempt(attempt)).await;
                }
            }
        }
    }

    Err(last_error)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay, Duration::from_millis(100));
    }

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter: 0.0,
        };

        // Attempt 0: 100ms * 2^0 = 100ms
        let delay0 = config.delay_for_attempt(0);
        assert!(delay0 >= Duration::from_millis(100));
        assert!(delay0 <= Duration::from_millis(110));

        // Attempt 1: 100ms * 2^1 = 200ms
        let delay1 = config.delay_for_attempt(1);
        assert!(delay1 >= Duration::from_millis(200));
        assert!(delay1 <= Duration::from_millis(220));
    }

    #[test]
    fn test_max_delay_cap() {
        let config = RetryConfig {
            max_retries: 10,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            multiplier: 2.0,
            jitter: 0.0,
        };

        // Should be capped at 1 second
        let delay = config.delay_for_attempt(10);
        assert!(delay <= Duration::from_secs(1));
    }
}
