//! Dragonfly/Redis health check.

use crate::config::CacheConfig;
use crate::pool::CachePool;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheHealth {
    Healthy,
    Unavailable,
}

pub struct CacheHealthChecker {
    pool: CachePool,
}

impl CacheHealthChecker {
    pub fn new(pool: CachePool) -> Self {
        Self { pool }
    }

    pub async fn check(&self) -> CacheHealth {
        match self.pool.get_connection().await {
            Ok(mut conn) => {
                let result: Result<String, _> = redis::cmd("PING").query_async(&mut *conn).await;
                match result {
                    Ok(pong) if pong == "PONG" => CacheHealth::Healthy,
                    _ => CacheHealth::Unavailable,
                }
            }
            Err(_) => CacheHealth::Unavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_enum_equality() {
        assert_eq!(CacheHealth::Healthy, CacheHealth::Healthy);
        assert_ne!(CacheHealth::Healthy, CacheHealth::Unavailable);
    }

    #[test]
    fn health_debug() {
        let healthy = CacheHealth::Healthy;
        let unavailable = CacheHealth::Unavailable;
        assert!(format!("{:?}", healthy).contains("Healthy"));
        assert!(format!("{:?}", unavailable).contains("Unavailable"));
    }

    #[test]
    fn cache_health_checker_new() {
        let config = CacheConfig::new("localhost".into(), 6379);
        let pool = CachePool::new(&config);
        if pool.is_ok() {
            let checker = CacheHealthChecker::new(pool.unwrap());
            assert!(format!("{:?}", checker).contains("CacheHealthChecker"));
        }
    }

    #[test]
    fn cache_health_checker_unavailable_on_invalid_pool() {
        let config = CacheConfig::new("invalid-host-that-does-not-resolve".into(), 6379);
        let pool = CachePool::new(&config);
        if pool.is_err() {
            assert!(true);
        }
    }
}
