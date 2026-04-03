//! # Phenotype Rate Limit
//!
//! Rate limiting utilities for the Phenotype ecosystem.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::time::{Duration, Instant};
use thiserror::Error;

// =============================================================================
// Errors
// =============================================================================

/// Rate limit errors
#[derive(Error, Debug)]
pub enum RateLimitError {
    #[error("Rate limit exceeded")]
    Exceeded,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Result type for rate limit operations
pub type RateLimitResult<T> = Result<T, RateLimitError>;

// =============================================================================
// Rate Limit Configuration
// =============================================================================

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests allowed
    pub max_requests: u32,
    /// Time window duration
    pub window: Duration,
}

impl RateLimitConfig {
    /// Create a new config with requests per window
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
        }
    }

    /// Create a config for requests per second
    pub fn per_second(requests: u32) -> Self {
        Self::new(requests, Duration::from_secs(1))
    }

    /// Create a config for requests per minute
    pub fn per_minute(requests: u32) -> Self {
        Self::new(requests, Duration::from_secs(60))
    }

    /// Create a config for requests per hour
    pub fn per_hour(requests: u32) -> Self {
        Self::new(requests, Duration::from_secs(3600))
    }
}

// =============================================================================
// Token Bucket
// =============================================================================

/// Token bucket rate limiter
#[derive(Debug)]
pub struct TokenBucket {
    /// Maximum tokens
    max_tokens: u32,
    /// Current tokens
    tokens: u32,
    /// Refill rate (tokens per second)
    refill_rate: f64,
    /// Last refill time
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    pub fn new(max_tokens: u32, refill_rate: f64) -> Self {
        Self {
            max_tokens,
            tokens: max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Try to acquire a token
    pub fn try_acquire(&mut self) -> bool {
        self.refill();
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Get current available tokens
    pub fn available(&self) -> u32 {
        self.tokens
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        let tokens_to_add = (elapsed * self.refill_rate) as u32;

        if tokens_to_add > 0 {
            self.tokens = (self.tokens + tokens_to_add).min(self.max_tokens);
            self.last_refill = Instant::now();
        }
    }
}

// =============================================================================
// Sliding Window
// =============================================================================

/// Sliding window rate limiter
#[derive(Debug)]
pub struct SlidingWindow {
    /// Maximum requests in window
    max_requests: u32,
    /// Request timestamps
    requests: Vec<Instant>,
    /// Window duration
    window: Duration,
}

impl SlidingWindow {
    /// Create a new sliding window
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            requests: Vec::new(),
            window,
        }
    }

    /// Check if request is allowed and record it
    pub fn check_and_record(&mut self) -> bool {
        let now = Instant::now();

        // Remove expired requests
        let cutoff = now - self.window;
        self.requests.retain(|&t| t > cutoff);

        // Check if we can add a new request
        if self.requests.len() < self.max_requests as usize {
            self.requests.push(now);
            true
        } else {
            false
        }
    }

    /// Get remaining requests in current window
    pub fn remaining(&self) -> u32 {
        let now = Instant::now();
        let cutoff = now - self.window;
        let active = self.requests.iter().filter(|&&t| t > cutoff).count();
        (self.max_requests as usize).saturating_sub(active) as u32
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_per_second() {
        let config = RateLimitConfig::per_second(100);
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window, Duration::from_secs(1));
    }

    #[test]
    fn test_config_per_minute() {
        let config = RateLimitConfig::per_minute(1000);
        assert_eq!(config.max_requests, 1000);
        assert_eq!(config.window, Duration::from_secs(60));
    }

    #[test]
    fn test_token_bucket_acquire() {
        let mut bucket = TokenBucket::new(5, 1.0);
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        assert_eq!(bucket.available(), 3);
    }

    #[test]
    fn test_token_bucket_empty() {
        let mut bucket = TokenBucket::new(1, 0.0);
        assert!(bucket.try_acquire());
        assert!(!bucket.try_acquire());
    }

    #[test]
    fn test_sliding_window() {
        let mut window = SlidingWindow::new(3, Duration::from_secs(1));

        assert!(window.check_and_record());
        assert!(window.check_and_record());
        assert!(window.check_and_record());
        assert!(!window.check_and_record());

        assert_eq!(window.remaining(), 0);
    }

    #[test]
    fn test_sliding_window_refill() {
        let mut window = SlidingWindow::new(2, Duration::from_millis(50));

        assert!(window.check_and_record());
        assert!(window.check_and_record());
        assert!(!window.check_and_record());

        // Wait for window to clear
        std::thread::sleep(Duration::from_millis(60));

        assert!(window.check_and_record());
    }
}
