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
    fn pool_error_implements_std_error() {
        fn assert_error<T: std::error::Error + Send + Sync>() {}
        assert_error::<PoolError>();
    }

    #[test]
    fn pool_error_variants_have_no_source() {
        let errors = [
            PoolError::ConnectionError("x".to_string()),
            PoolError::Timeout("x".to_string()),
        ];
        for error in &errors {
            assert!(std::error::Error::source(error).is_none());
        }
    }
}
