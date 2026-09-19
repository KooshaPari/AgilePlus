//! `RedisCacheStore` error paths: server-side command failures, payloads that
//! cannot be encoded, and an unreachable backend.
//!
//! The store is exercised through the in-process RESP double
//! (`tests/common/redis.rs`), which can be told to fail a specific command.

mod common;

use agileplus_cache::config::CacheConfig;
use agileplus_cache::pool::CachePool;
use agileplus_cache::store::{CacheError, CacheStore, RedisCacheStore};
use common::FailsToSerialize;
use common::redis::MockRedisServer;

async fn store_with(server: &MockRedisServer, default_ttl_secs: u64) -> RedisCacheStore {
    let pool = CachePool::new(&CacheConfig::new(server.host(), server.port()).with_pool_size(2))
        .await
        .expect("pool");
    RedisCacheStore::new(pool, default_ttl_secs)
}

// ===========================================================================
// Error propagation
// ===========================================================================

#[tokio::test]
async fn get_surfaces_server_error_as_redis_error() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;
    server.fail_command("GET", "READONLY replica");

    let error = store.get::<u32>("k").await.expect_err("must fail");
    assert!(matches!(error, CacheError::RedisError(_)), "got {error:?}");
    assert!(error.to_string().contains("READONLY replica"));
}

#[tokio::test]
async fn set_surfaces_server_error_as_redis_error() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;
    server.fail_command("SETEX", "OOM command not allowed");

    let error = store.set("k", &1u32, None).await.expect_err("must fail");
    assert!(matches!(error, CacheError::RedisError(_)), "got {error:?}");
    assert!(error.to_string().contains("OOM command not allowed"));
    assert_eq!(server.stored("k"), None);
}

#[tokio::test]
async fn delete_surfaces_server_error_as_redis_error() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;
    server.fail_command("DEL", "NOPROTO");

    let error = store.delete("k").await.expect_err("must fail");
    assert!(matches!(error, CacheError::RedisError(_)), "got {error:?}");
}

#[tokio::test]
async fn exists_surfaces_server_error_as_redis_error() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;
    server.fail_command("EXISTS", "busy");

    let error = store.exists("k").await.expect_err("must fail");
    assert!(matches!(error, CacheError::RedisError(_)), "got {error:?}");
}

#[tokio::test]
async fn unserializable_value_fails_without_writing() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;

    let error = store
        .set("k", &FailsToSerialize, None)
        .await
        .expect_err("must fail");
    assert!(
        matches!(error, CacheError::SerializationError(_)),
        "got {error:?}"
    );
    assert!(
        error
            .to_string()
            .contains("intentional serialization failure")
    );

    assert_eq!(server.count_commands("SETEX"), 0, "nothing must be written");
    assert!(server.stored_keys().is_empty());
}

#[tokio::test]
async fn failed_set_leaves_previous_value_intact() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;
    store.set("k", &1u32, None).await.expect("set");
    server.fail_command("SETEX", "rejected");

    let _ = store.set("k", &2u32, None).await;

    assert_eq!(store.get::<u32>("k").await.unwrap(), Some(1));
}

// ===========================================================================
// Connection failures
// ===========================================================================

/// A pool that parses but can never connect: port 0 is not a valid target, and
/// bb8 builds lazily, so the store is constructible and only fails on use.
async fn unreachable_store() -> RedisCacheStore {
    let mut config = CacheConfig::new("127.0.0.1".to_string(), 0);
    config.connection_timeout_secs = 1;
    let pool = CachePool::new(&config).await.expect("lazy build");
    RedisCacheStore::new(pool, 3600)
}

#[tokio::test]
async fn get_reports_connection_error_when_the_backend_is_unreachable() {
    let store = unreachable_store().await;

    let error = store.get::<u32>("k").await.expect_err("must fail");
    assert!(
        matches!(error, CacheError::ConnectionError(_)),
        "got {error:?}"
    );
    assert!(!error.to_string().is_empty());
}

#[tokio::test]
async fn set_reports_connection_error_when_the_backend_is_unreachable() {
    let store = unreachable_store().await;

    let error = store.set("k", &1u32, None).await.expect_err("must fail");
    assert!(
        matches!(error, CacheError::ConnectionError(_)),
        "got {error:?}"
    );
}

#[tokio::test]
async fn delete_reports_connection_error_when_the_backend_is_unreachable() {
    let store = unreachable_store().await;

    let error = store.delete("k").await.expect_err("must fail");
    assert!(
        matches!(error, CacheError::ConnectionError(_)),
        "got {error:?}"
    );
}

#[tokio::test]
async fn exists_reports_connection_error_when_the_backend_is_unreachable() {
    let store = unreachable_store().await;

    let error = store.exists("k").await.expect_err("must fail");
    assert!(
        matches!(error, CacheError::ConnectionError(_)),
        "got {error:?}"
    );
}
