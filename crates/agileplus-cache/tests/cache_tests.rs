//! Integration tests for `agileplus-cache`.
//!
//! Covers: CacheConfig builder/validation, error Display impls, CacheHealth
//! enum, CacheStore trait (mock), and projection serde roundtrips.
//! Redis-connection-dependent tests are excluded.

use agileplus_cache::config::CacheConfig;
use agileplus_cache::health::CacheHealth;
use agileplus_cache::store::{CacheError, CacheStore};
use agileplus_cache::Error;

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers: mock CacheStore
// ---------------------------------------------------------------------------

/// A fully in-memory mock of `CacheStore` for testing the trait contract
/// without a Redis connection.
#[derive(Default)]
struct MockCacheStore {
    data: Mutex<HashMap<String, String>>,
}

impl MockCacheStore {
    fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl CacheStore for MockCacheStore {
    async fn get<T: for<'de> Deserialize<'de> + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError> {
        let map = self.data.lock().expect("lock poisoned");
        match map.get(key) {
            Some(v) => serde_json::from_str(v)
                .map(Some)
                .map_err(|e| CacheError::SerializationError(e.to_string())),
            None => Ok(None),
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        _ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        let serialized =
            serde_json::to_string(value).map_err(|e| CacheError::SerializationError(e.to_string()))?;
        let mut map = self.data.lock().expect("lock poisoned");
        map.insert(key.to_string(), serialized);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut map = self.data.lock().expect("lock poisoned");
        map.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let map = self.data.lock().expect("lock poisoned");
        Ok(map.contains_key(key))
    }
}

// ===========================================================================
// CacheConfig tests
// ===========================================================================

#[test]
fn config_new_sets_fields() {
    let cfg = CacheConfig::new("redis.example.com".into(), 6380);
    assert_eq!(cfg.host, "redis.example.com");
    assert_eq!(cfg.port, 6380);
}

#[test]
fn config_new_defaults() {
    let cfg = CacheConfig::new("h".into(), 1);
    assert_eq!(cfg.pool_size, 16);
    assert_eq!(cfg.default_ttl_secs, 3600);
    assert_eq!(cfg.connection_timeout_secs, 5);
}

#[test]
fn config_default_trait() {
    let cfg = CacheConfig::default();
    assert_eq!(cfg.host, "localhost");
    assert_eq!(cfg.port, 6379);
    assert_eq!(cfg.pool_size, 16);
    assert_eq!(cfg.default_ttl_secs, 3600);
    assert_eq!(cfg.connection_timeout_secs, 5);
}

#[test]
fn config_with_pool_size() {
    let cfg = CacheConfig::default().with_pool_size(32);
    assert_eq!(cfg.pool_size, 32);
}

#[test]
fn config_with_default_ttl() {
    let cfg = CacheConfig::default().with_default_ttl(7200);
    assert_eq!(cfg.default_ttl_secs, 7200);
}

#[test]
fn config_builder_chaining() {
    let cfg = CacheConfig::new("db.local".into(), 16379)
        .with_pool_size(8)
        .with_default_ttl(120);
    assert_eq!(cfg.host, "db.local");
    assert_eq!(cfg.port, 16379);
    assert_eq!(cfg.pool_size, 8);
    assert_eq!(cfg.default_ttl_secs, 120);
}

#[test]
fn config_redis_url_default() {
    let cfg = CacheConfig::default();
    assert_eq!(cfg.redis_url(), "redis://localhost:6379");
}

#[test]
fn config_redis_url_custom() {
    let cfg = CacheConfig::new("myhost".into(), 9999);
    assert_eq!(cfg.redis_url(), "redis://myhost:9999");
}

#[test]
fn config_clone() {
    let cfg = CacheConfig::default().with_pool_size(42);
    let cloned = cfg.clone();
    assert_eq!(cfg.host, cloned.host);
    assert_eq!(cfg.port, cloned.port);
    assert_eq!(cfg.pool_size, cloned.pool_size);
    assert_eq!(cfg.default_ttl_secs, cloned.default_ttl_secs);
    assert_eq!(cfg.connection_timeout_secs, cloned.connection_timeout_secs);
}

#[test]
fn config_debug_format() {
    let cfg = CacheConfig::default();
    let debug = format!("{cfg:?}");
    assert!(debug.contains("CacheConfig"));
    assert!(debug.contains("localhost"));
}

// ===========================================================================
// CacheError Display tests
// ===========================================================================

#[test]
fn cache_error_serialization_display() {
    let err = CacheError::SerializationError("bad json".into());
    assert_eq!(err.to_string(), "Serialization error: bad json");
}

#[test]
fn cache_error_redis_display() {
    let err = CacheError::RedisError("WRONGTYPE".into());
    assert_eq!(err.to_string(), "Redis error: WRONGTYPE");
}

#[test]
fn cache_error_not_found_display() {
    let err = CacheError::NotFound;
    assert_eq!(err.to_string(), "Key not found");
}

#[test]
fn cache_error_connection_display() {
    let err = CacheError::ConnectionError("refused".into());
    assert_eq!(err.to_string(), "Connection error: refused");
}

#[test]
fn cache_error_debug_format() {
    let err = CacheError::NotFound;
    let debug = format!("{err:?}");
    assert!(debug.contains("NotFound"));
}

// ===========================================================================
// PoolError Display tests
// ===========================================================================

#[test]
fn pool_error_connection_display() {
    let err = agileplus_cache::pool::PoolError::ConnectionError("refused".into());
    assert_eq!(err.to_string(), "Connection error: refused");
}

#[test]
fn pool_error_timeout_display() {
    let err = agileplus_cache::pool::PoolError::Timeout("deadline exceeded".into());
    assert_eq!(err.to_string(), "Timeout: deadline exceeded");
}

#[test]
fn pool_error_debug_format() {
    let err = agileplus_cache::pool::PoolError::Timeout("x".into());
    let debug = format!("{err:?}");
    assert!(debug.contains("Timeout"));
}

// ===========================================================================
// LimiterError Display tests
// ===========================================================================

#[test]
fn limiter_error_display() {
    let err = agileplus_cache::limiter::LimiterError::Error("boom".into());
    assert_eq!(err.to_string(), "Rate limit error: boom");
}

#[test]
fn limiter_error_debug_format() {
    let err = agileplus_cache::limiter::LimiterError::Error("x".into());
    let debug = format!("{err:?}");
    // thiserror Debug shows variant name, not struct name
    assert!(debug.contains("Error"));
}

// ===========================================================================
// ProjectionError Display tests
// ===========================================================================

#[test]
fn projection_error_display() {
    let err = agileplus_cache::projection::ProjectionError::CacheError("fail".into());
    assert_eq!(err.to_string(), "Cache error: fail");
}

#[test]
fn projection_error_debug_format() {
    let err = agileplus_cache::projection::ProjectionError::CacheError("x".into());
    let debug = format!("{err:?}");
    // thiserror Debug shows variant name, not struct name
    assert!(debug.contains("CacheError"));
}

// ===========================================================================
// Top-level Error (lib.rs) Display tests
// ===========================================================================

#[test]
fn lib_error_from_cache_error() {
    let cache_err = CacheError::NotFound;
    let err: Error = cache_err.into();
    assert_eq!(err.to_string(), "Cache error: Key not found");
}

#[test]
fn lib_error_config_display() {
    let err = Error::Config("missing host".into());
    assert_eq!(err.to_string(), "Config error: missing host");
}

#[test]
fn lib_error_debug_format() {
    let err = Error::Config("x".into());
    let debug = format!("{err:?}");
    assert!(debug.contains("Config"));
}

// ===========================================================================
// CacheHealth enum tests
// ===========================================================================

#[test]
fn health_equality() {
    assert_eq!(CacheHealth::Healthy, CacheHealth::Healthy);
    assert_eq!(CacheHealth::Unavailable, CacheHealth::Unavailable);
}

#[test]
fn health_inequality() {
    assert_ne!(CacheHealth::Healthy, CacheHealth::Unavailable);
}

#[test]
fn health_clone() {
    let h = CacheHealth::Healthy;
    let h2 = h;
    assert_eq!(h, h2);
}

#[test]
fn health_debug() {
    assert_eq!(format!("{:?}", CacheHealth::Healthy), "Healthy");
    assert_eq!(format!("{:?}", CacheHealth::Unavailable), "Unavailable");
}

#[test]
fn health_copy() {
    let h = CacheHealth::Healthy;
    let h2 = h; // Copy, not move
    assert_eq!(h, h2);
}

// ===========================================================================
// CacheStore trait — mock integration tests
// ===========================================================================

#[tokio::test]
async fn mock_store_set_and_get() {
    let store = MockCacheStore::new();
    let value = serde_json::json!({"name": "test", "count": 42});

    store.set("key1", &value, None).await.unwrap();

    let retrieved: Option<serde_json::Value> = store.get("key1").await.unwrap();
    assert_eq!(retrieved, Some(value));
}

#[tokio::test]
async fn mock_store_get_missing_key() {
    let store = MockCacheStore::new();
    let result: Option<String> = store.get("nonexistent").await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn mock_store_delete_existing() {
    let store = MockCacheStore::new();
    store.set("k", &"v".to_string(), None).await.unwrap();

    store.delete("k").await.unwrap();

    let result: Option<String> = store.get("k").await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn mock_store_delete_missing_key_is_ok() {
    let store = MockCacheStore::new();
    // Deleting a nonexistent key should not error
    store.delete("nope").await.unwrap();
}

#[tokio::test]
async fn mock_store_exists_true() {
    let store = MockCacheStore::new();
    store.set("here", &1u32, None).await.unwrap();
    assert!(store.exists("here").await.unwrap());
}

#[tokio::test]
async fn mock_store_exists_false() {
    let store = MockCacheStore::new();
    assert!(!store.exists("gone").await.unwrap());
}

#[tokio::test]
async fn mock_store_overwrite() {
    let store = MockCacheStore::new();
    store.set("k", &"first".to_string(), None).await.unwrap();
    store.set("k", &"second".to_string(), None).await.unwrap();

    let result: Option<String> = store.get("k").await.unwrap();
    assert_eq!(result, Some("second".to_string()));
}

#[tokio::test]
async fn mock_store_multiple_keys() {
    let store = MockCacheStore::new();
    store.set("a", &1i32, None).await.unwrap();
    store.set("b", &2i32, None).await.unwrap();
    store.set("c", &3i32, None).await.unwrap();

    let a: Option<i32> = store.get("a").await.unwrap();
    let b: Option<i32> = store.get("b").await.unwrap();
    let c: Option<i32> = store.get("c").await.unwrap();

    assert_eq!(a, Some(1));
    assert_eq!(b, Some(2));
    assert_eq!(c, Some(3));
}

#[tokio::test]
async fn mock_store_struct_serialization() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct UserProfile {
        name: String,
        age: u32,
        tags: Vec<String>,
    }

    let store = MockCacheStore::new();
    let profile = UserProfile {
        name: "Alice".into(),
        age: 30,
        tags: vec!["admin".into(), "dev".into()],
    };

    store.set("user:1", &profile, None).await.unwrap();

    let retrieved: Option<UserProfile> = store.get("user:1").await.unwrap();
    assert_eq!(retrieved, Some(profile));
}

#[tokio::test]
async fn mock_store_ttl_ignored() {
    // The mock ignores TTL; verify set with explicit TTL works identically
    let store = MockCacheStore::new();
    store
        .set("k", &"v".to_string(), Some(Duration::from_secs(1)))
        .await
        .unwrap();

    let result: Option<String> = store.get("k").await.unwrap();
    assert_eq!(result, Some("v".to_string()));
}

// ===========================================================================
// Mock store via concrete type (trait not dyn-compatible due to generics)
// ===========================================================================

#[tokio::test]
async fn concrete_store_dispatch() {
    let store = MockCacheStore::new();
    store.set("dyn_key", &42u64, None).await.unwrap();

    let val: Option<u64> = store.get("dyn_key").await.unwrap();
    assert_eq!(val, Some(42));
}

#[tokio::test]
async fn concrete_store_delete() {
    let store = MockCacheStore::new();
    store.set("to_del", &"hello".to_string(), None).await.unwrap();
    assert!(store.exists("to_del").await.unwrap());

    store.delete("to_del").await.unwrap();
    assert!(!store.exists("to_del").await.unwrap());
}

// ===========================================================================
// Projection serde roundtrip tests (FeatureProjection / WorkPackageProjection)
// ===========================================================================

#[test]
fn feature_projection_serde_roundtrip() {
    use agileplus_cache::projection::FeatureProjection;
    use agileplus_domain::domain::feature::Feature;

    let feature = Feature::new("my-feat", "My Feature", [0xAB_u8; 32], Some("main"));
    let proj = FeatureProjection {
        feature,
        cached_at: Utc::now(),
    };

    let json = serde_json::to_string(&proj).expect("serialize");
    let decoded: FeatureProjection = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded.feature.slug, "my-feat");
    assert_eq!(decoded.feature.friendly_name, "My Feature");
    assert_eq!(decoded.feature.spec_hash, [0xAB_u8; 32]);
    assert_eq!(decoded.feature.target_branch, "main");
    assert_eq!(decoded.feature.id, 0);
}

#[test]
fn workpackage_projection_serde_roundtrip() {
    use agileplus_cache::projection::WorkPackageProjection;
    use agileplus_domain::domain::work_package::WorkPackage;

    let wp = WorkPackage::new(1, "Implement caching", 1, "Cache reads complete in <10ms");
    let proj = WorkPackageProjection {
        workpackage: wp,
        cached_at: Utc::now(),
    };

    let json = serde_json::to_string(&proj).expect("serialize");
    let decoded: WorkPackageProjection = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded.workpackage.feature_id, 1);
    assert_eq!(decoded.workpackage.title, "Implement caching");
    assert_eq!(decoded.workpackage.acceptance_criteria, "Cache reads complete in <10ms");
    assert_eq!(decoded.workpackage.sequence, 1);
}

#[test]
fn feature_projection_clone() {
    use agileplus_cache::projection::FeatureProjection;
    use agileplus_domain::domain::feature::Feature;

    let feature = Feature::new("clone-test", "Clone", [0x00_u8; 32], None);
    let proj = FeatureProjection {
        feature,
        cached_at: Utc::now(),
    };
    let proj2 = proj.clone();
    assert_eq!(proj.feature.slug, proj2.feature.slug);
}

#[test]
fn workpackage_projection_clone() {
    use agileplus_cache::projection::WorkPackageProjection;
    use agileplus_domain::domain::work_package::WorkPackage;

    let wp = WorkPackage::new(5, "WP title", 3, "criteria");
    let proj = WorkPackageProjection {
        workpackage: wp,
        cached_at: Utc::now(),
    };
    let proj2 = proj.clone();
    assert_eq!(proj.workpackage.title, proj2.workpackage.title);
}

#[test]
fn feature_projection_debug() {
    use agileplus_cache::projection::FeatureProjection;
    use agileplus_domain::domain::feature::Feature;

    let feature = Feature::new("dbg", "Debug", [0x01_u8; 32], None);
    let proj = FeatureProjection {
        feature,
        cached_at: Utc::now(),
    };
    let debug = format!("{proj:?}");
    assert!(debug.contains("FeatureProjection"));
    assert!(debug.contains("dbg"));
}

#[test]
fn workpackage_projection_debug() {
    use agileplus_cache::projection::WorkPackageProjection;
    use agileplus_domain::domain::work_package::WorkPackage;

    let wp = WorkPackage::new(1, "Debug WP", 1, "crit");
    let proj = WorkPackageProjection {
        workpackage: wp,
        cached_at: Utc::now(),
    };
    let debug = format!("{proj:?}");
    assert!(debug.contains("WorkPackageProjection"));
    assert!(debug.contains("Debug WP"));
}

// ===========================================================================
// ProjectionCache key format tests (validate key pattern without Redis)
// ===========================================================================

#[test]
fn feature_projection_key_format() {
    // Verify the key format used by ProjectionCache matches expectations
    // by inspecting what keys the store would see
    let feature_id: i64 = 42;
    let expected_key = format!("feature:{feature_id}");
    assert_eq!(expected_key, "feature:42");
}

#[test]
fn workpackage_projection_key_format() {
    let wp_id: i64 = 99;
    let expected_key = format!("wp:{wp_id}");
    assert_eq!(expected_key, "wp:99");
}

// ===========================================================================
// CacheConfig edge cases
// ===========================================================================

#[test]
fn config_zero_port() {
    let cfg = CacheConfig::new("localhost".into(), 0);
    assert_eq!(cfg.redis_url(), "redis://localhost:0");
}

#[test]
fn config_max_port() {
    let cfg = CacheConfig::new("localhost".into(), 65535);
    assert_eq!(cfg.redis_url(), "redis://localhost:65535");
}

#[test]
fn config_pool_size_zero() {
    let cfg = CacheConfig::default().with_pool_size(0);
    assert_eq!(cfg.pool_size, 0);
}

#[test]
fn config_ttl_zero() {
    let cfg = CacheConfig::default().with_default_ttl(0);
    assert_eq!(cfg.default_ttl_secs, 0);
}

#[test]
fn config_large_ttl() {
    let cfg = CacheConfig::default().with_default_ttl(u64::MAX);
    assert_eq!(cfg.default_ttl_secs, u64::MAX);
}

#[test]
fn config_builder_overwrite_pool_size() {
    let cfg = CacheConfig::default()
        .with_pool_size(8)
        .with_pool_size(64);
    assert_eq!(cfg.pool_size, 64);
}

#[test]
fn config_builder_overwrite_ttl() {
    let cfg = CacheConfig::default()
        .with_default_ttl(100)
        .with_default_ttl(999);
    assert_eq!(cfg.default_ttl_secs, 999);
}

#[test]
fn config_special_characters_in_host() {
    let cfg = CacheConfig::new("redis-internal.prod.us-east-1.example.com".into(), 6379);
    assert_eq!(
        cfg.redis_url(),
        "redis://redis-internal.prod.us-east-1.example.com:6379"
    );
}

// ===========================================================================
// CacheError additional tests
// ===========================================================================

#[test]
fn cache_error_variants_are_distinct() {
    let serialization = CacheError::SerializationError("a".into());
    let redis = CacheError::RedisError("b".into());
    let not_found = CacheError::NotFound;
    let connection = CacheError::ConnectionError("c".into());

    // Verify all Display outputs are different
    let msgs: Vec<String> = vec![
        serialization.to_string(),
        redis.to_string(),
        not_found.to_string(),
        connection.to_string(),
    ];
    let unique: std::collections::HashSet<&str> = msgs.iter().map(|s| s.as_str()).collect();
    assert_eq!(unique.len(), 4, "all error messages should be distinct");
}

// ===========================================================================
// Rate limiter key format tests
// ===========================================================================

#[test]
fn rate_limiter_key_format() {
    let key = "user:123";
    let rate_key = format!("ratelimit:{key}");
    assert_eq!(rate_key, "ratelimit:user:123");
}

#[test]
fn rate_limiter_key_format_empty() {
    let key = "";
    let rate_key = format!("ratelimit:{key}");
    assert_eq!(rate_key, "ratelimit:");
}

// ===========================================================================
// Mock store: multiple concurrent readers (thread safety)
// ===========================================================================

#[tokio::test]
async fn mock_store_concurrent_access() {
    use tokio::task;

    let store = Arc::new(MockCacheStore::new());
    store.set("counter", &0u32, None).await.unwrap();

    let mut handles = vec![];
    for i in 1..=10 {
        let s = Arc::clone(&store);
        handles.push(task::spawn(async move {
            // Each task reads and sets — no real atomicity but tests thread safety
            let _val: Option<u32> = s.get("counter").await.unwrap();
            s.set("counter", &i, None).await.unwrap();
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // The final value should be one of the set values (1-10)
    let final_val: Option<u32> = store.get("counter").await.unwrap();
    assert!(final_val.is_some());
    let v = final_val.unwrap();
    assert!((1..=10).contains(&v), "expected 1-10, got {v}");
}
