//! AgilePlus cache layer — Dragonfly (Redis-compatible) adapter.
//!
//! Provides connection pooling, typed cache operations, projection caching,
//! rate limiting, and health checks.
//! Traceability: FR-CACHE / WP04

pub mod config;
pub mod health;
pub mod limiter;
pub mod pool;
pub mod projection;
pub mod store;

pub use config::CacheConfig;
pub use health::{CacheHealth, CacheHealthChecker};
pub use limiter::RateLimiter;
pub use pool::CachePool;
pub use projection::ProjectionCache;
pub use store::{CacheError, CacheStore, RedisCacheStore};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),
    #[error("Config error: {0}")]
    Config(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }

    #[test]
    fn cache_variant_exposes_source() {
        let error = Error::from(CacheError::RedisError("boom".to_string()));
        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn config_variant_has_no_source() {
        let error = Error::Config("bad config".to_string());
        assert!(std::error::Error::source(&error).is_none());
    }
}
