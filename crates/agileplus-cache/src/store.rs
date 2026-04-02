//! Typed cache store with serde serialization.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::pool::CachePool;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Redis error: {0}")]
    RedisError(String),
    #[error("Key not found")]
    NotFound,
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

#[async_trait]
pub trait CacheStore: Send + Sync {
    async fn get<T: for<'de> Deserialize<'de> + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError>;

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError>;

    async fn delete(&self, key: &str) -> Result<(), CacheError>;

    async fn exists(&self, key: &str) -> Result<bool, CacheError>;
}

/// Redis/Dragonfly-backed cache store.
pub struct RedisCacheStore {
    pool: CachePool,
    default_ttl: Duration,
}

impl RedisCacheStore {
    pub fn new(pool: CachePool, default_ttl_secs: u64) -> Self {
        Self {
            pool,
            default_ttl: Duration::from_secs(default_ttl_secs),
        }
    }
}

#[async_trait]
impl CacheStore for RedisCacheStore {
    async fn get<T: for<'de> Deserialize<'de> + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError> {
        let mut conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        let value: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| CacheError::RedisError(e.to_string()))?;

        match value {
            Some(v) => serde_json::from_str(&v)
                .map(Some)
                .map_err(|e| CacheError::SerializationError(e.to_string())),
            None => Ok(None),
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        let mut conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let ttl_secs = ttl.unwrap_or(self.default_ttl).as_secs() as i64;

        conn.set_ex::<_, _, ()>(key, &serialized, ttl_secs as u64)
            .await
            .map_err(|e| CacheError::RedisError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        conn.del::<_, ()>(key)
            .await
            .map_err(|e| CacheError::RedisError(e.to_string()))?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let mut conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        conn.exists(key)
            .await
            .map_err(|e| CacheError::RedisError(e.to_string()))
    }
}

pub struct InMemoryCacheStore {
    data: Arc<Mutex<HashMap<String, (String, Option<Duration>)>>>,
    default_ttl: Duration,
}

impl InMemoryCacheStore {
    pub fn new(default_ttl_secs: u64) -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            default_ttl: Duration::from_secs(default_ttl_secs),
        }
    }
}

#[async_trait]
impl CacheStore for InMemoryCacheStore {
    async fn get<T: for<'de> Deserialize<'de> + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError> {
        let data = self.data.lock().unwrap();
        if let Some((value, ttl)) = data.get(key) {
            if let Some(expire) = ttl {
                if std::time::Instant::now() > *expire {
                    return Ok(None);
                }
            }
            serde_json::from_str(value)
                .map(Some)
                .map_err(|e| CacheError::SerializationError(e.to_string()))
        } else {
            Ok(None)
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        let expire = ttl.map(|d| std::time::Instant::now() + d).unwrap_or_else(|| std::time::Instant::now() + self.default_ttl);
        self.data.lock().unwrap().insert(key.to_string(), (serialized, Some(expire)));
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.data.lock().unwrap().remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        Ok(self.data.lock().unwrap().contains_key(key))
    }
}
