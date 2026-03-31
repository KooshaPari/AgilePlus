//! Typed cache store with serde serialization.

use crate::pool::CachePool;
use async_trait::async_trait;
use dashmap::DashMap;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

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

#[cfg(test)]
mod tests {
    use super::*;

    struct InMemoryCacheStore {
        data: Arc<DashMap<String, (String, Option<Instant>)>>,
        default_ttl: Duration,
    }

    impl InMemoryCacheStore {
        fn new(default_ttl_secs: u64) -> Self {
            Self {
                data: Arc::new(DashMap::new()),
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
            let entry = self.data.get(key);
            match entry {
                Some((value, expiry)) => {
                    if let Some(inst) = expiry {
                        if Instant::now() > *inst {
                            drop(entry);
                            self.data.remove(key);
                            return Ok(None);
                        }
                    }
                    serde_json::from_str(value)
                        .map(Some)
                        .map_err(|e| CacheError::SerializationError(e.to_string()))
                }
                None => Ok(None),
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
            let expiry = ttl.map(|d| Instant::now() + d);
            self.data.insert(key.to_string(), (serialized, expiry));
            Ok(())
        }

        async fn delete(&self, key: &str) -> Result<(), CacheError> {
            self.data.remove(key);
            Ok(())
        }

        async fn exists(&self, key: &str) -> Result<bool, CacheError> {
            let entry = self.data.get(key);
            match entry {
                Some((_, expiry)) => {
                    if let Some(inst) = expiry {
                        if Instant::now() > *inst {
                            drop(entry);
                            self.data.remove(key);
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                None => Ok(false),
            }
        }
    }

    #[test]
    fn cache_error_serialization_display() {
        let err = CacheError::SerializationError("bad json".into());
        assert!(err.to_string().contains("Serialization error"));
        assert!(err.to_string().contains("bad json"));
    }

    #[test]
    fn cache_error_redis_display() {
        let err = CacheError::RedisError("connection refused".into());
        assert!(err.to_string().contains("Redis error"));
        assert!(err.to_string().contains("connection refused"));
    }

    #[test]
    fn cache_error_not_found_display() {
        let err = CacheError::NotFound;
        assert!(err.to_string().contains("Key not found"));
    }

    #[test]
    fn cache_error_connection_display() {
        let err = CacheError::ConnectionError("timeout".into());
        assert!(err.to_string().contains("Connection error"));
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn cache_error_not_found_eq() {
        assert_eq!(CacheError::NotFound, CacheError::NotFound);
        assert_ne!(CacheError::NotFound, CacheError::SerializationError("x".into()));
    }

    #[tokio::test]
    async fn in_memory_cache_set_and_get() {
        let store = InMemoryCacheStore::new(3600);
        store
            .set("key1", &"value1", None)
            .await
            .expect("set should succeed");
        let result: Option<String> = store.get("key1").await.expect("get should succeed");
        assert_eq!(result, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn in_memory_cache_get_nonexistent() {
        let store = InMemoryCacheStore::new(3600);
        let result: Option<String> = store.get("nonexistent").await.expect("get should succeed");
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn in_memory_cache_delete() {
        let store = InMemoryCacheStore::new(3600);
        store.set("key1", &"value1", None).await.expect("set should succeed");
        store.delete("key1").await.expect("delete should succeed");
        let result: Option<String> = store.get("key1").await.expect("get should succeed");
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn in_memory_cache_exists() {
        let store = InMemoryCacheStore::new(3600);
        assert!(!store.exists("key1").await.expect("exists should succeed"));
        store.set("key1", &"value1", None).await.expect("set should succeed");
        assert!(store.exists("key1").await.expect("exists should succeed"));
    }

    #[tokio::test]
    async fn in_memory_cache_with_ttl() {
        let store = InMemoryCacheStore::new(3600);
        store
            .set("key1", &"value1", Some(Duration::from_secs(1)))
            .await
            .expect("set should succeed");
        assert!(store.exists("key1").await.expect("exists should succeed"));
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(!store.exists("key1").await.expect("exists should succeed after TTL"));
    }

    #[tokio::test]
    async fn in_memory_cache_overwrite() {
        let store = InMemoryCacheStore::new(3600);
        store.set("key1", &"value1", None).await.expect("set should succeed");
        store.set("key1", &"value2", None).await.expect("overwrite should succeed");
        let result: Option<String> = store.get("key1").await.expect("get should succeed");
        assert_eq!(result, Some("value2".to_string()));
    }

    #[tokio::test]
    async fn in_memory_cache_serde_struct() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestStruct {
            name: String,
            count: i32,
        }
        let store = InMemoryCacheStore::new(3600);
        let original = TestStruct { name: "test".into(), count: 42 };
        store.set("struct_key", &original, None).await.expect("set should succeed");
        let result: Option<TestStruct> = store.get("struct_key").await.expect("get should succeed");
        assert_eq!(result, Some(original));
    }
}
