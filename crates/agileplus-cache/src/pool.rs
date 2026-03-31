//! Connection pool for Dragonfly/Redis.

use crate::config::CacheConfig;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Thread-safe connection pool to Dragonfly (Redis-compatible).
pub struct CachePool {
    pool: Pool<RedisConnectionManager>,
}

impl CachePool {
    pub async fn new(config: &CacheConfig) -> Result<Self, PoolError> {
        let manager = RedisConnectionManager::new(config.redis_url())
            .map_err(|e| PoolError::ConnectionError(e.to_string()))?;

        let pool = Pool::builder()
            .max_size(config.pool_size)
            .connection_timeout(Duration::from_secs(config.connection_timeout_secs))
            .build(manager)
            .await
            .map_err(|e| PoolError::Timeout(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn get_connection(
        &self,
    ) -> Result<bb8::PooledConnection<'_, RedisConnectionManager>, PoolError> {
        self.pool
            .get()
            .await
            .map_err(|e| PoolError::Timeout(e.to_string()))
    }

    pub fn raw_pool(&self) -> &Pool<RedisConnectionManager> {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_error_connection_display() {
        let err = PoolError::ConnectionError("redis connection refused".into());
        assert!(err.to_string().contains("Connection error"));
        assert!(err.to_string().contains("redis connection refused"));
    }

    #[test]
    fn pool_error_timeout_display() {
        let err = PoolError::Timeout("connection timed out".into());
        assert!(err.to_string().contains("Timeout"));
        assert!(err.to_string().contains("connection timed out"));
    }

    #[test]
    fn pool_error_eq() {
        assert_eq!(
            PoolError::ConnectionError("e1".into()),
            PoolError::ConnectionError("e1".into())
        );
        assert_ne!(
            PoolError::ConnectionError("e1".into()),
            PoolError::ConnectionError("e2".into())
        );
        assert_ne!(
            PoolError::ConnectionError("e1".into()),
            PoolError::Timeout("e1".into())
        );
    }

    #[tokio::test]
    async fn cache_pool_new_invalid_url() {
        let config = CacheConfig::new("invalid-host-that-does-not-exist".into(), 6379);
        let result = CachePool::new(&config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cache_pool_new_connection_timeout() {
        let config = CacheConfig {
            host: "10.255.255.1".into(),
            port: 6379,
            pool_size: 1,
            default_ttl_secs: 3600,
            connection_timeout_secs: 1,
        };
        let result = CachePool::new(&config).await;
        assert!(result.is_err());
    }
}
