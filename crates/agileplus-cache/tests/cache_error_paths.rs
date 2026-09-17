//! CacheStore error-path contract: injected backend failures and
//! serialization failures.
//!
//! Uses the shared in-memory store double (see `tests/common/mod.rs`).

mod common;

use agileplus_cache::store::{CacheError, CacheStore};
use common::{FailsToSerialize, FailureMode, InMemoryCacheStore, Record};

fn assert_send_sync<T: Send + Sync>() {}
fn assert_std_error<T: std::error::Error + Send + Sync>() {}

// ===========================================================================
// Injected backend failures
// ===========================================================================

#[tokio::test]
async fn injected_get_failure_returns_redis_error() {
    let store = InMemoryCacheStore::unbounded();
    store.fail(FailureMode {
        get: Some("get exploded".to_string()),
        ..Default::default()
    });

    let result: Result<Option<u32>, CacheError> = store.get("k").await;
    assert!(matches!(result, Err(CacheError::RedisError(_))));
}

#[tokio::test]
async fn injected_set_failure_returns_redis_error() {
    let store = InMemoryCacheStore::unbounded();
    store.fail(FailureMode {
        set: Some("set exploded".to_string()),
        ..Default::default()
    });

    let result = store.set("k", &1u32, None).await;
    assert!(matches!(result, Err(CacheError::RedisError(_))));
}

#[tokio::test]
async fn injected_delete_failure_returns_redis_error() {
    let store = InMemoryCacheStore::unbounded();
    store.fail(FailureMode {
        delete: Some("delete exploded".to_string()),
        ..Default::default()
    });

    let result = store.delete("k").await;
    assert!(matches!(result, Err(CacheError::RedisError(_))));
}

#[tokio::test]
async fn injected_exists_failure_returns_redis_error() {
    let store = InMemoryCacheStore::unbounded();
    store.fail(FailureMode {
        exists: Some("exists exploded".to_string()),
        ..Default::default()
    });

    let result = store.exists("k").await;
    assert!(matches!(result, Err(CacheError::RedisError(_))));
}

#[tokio::test]
async fn injected_failure_message_is_preserved() {
    let store = InMemoryCacheStore::unbounded();
    store.fail(FailureMode {
        get: Some("boom-42".to_string()),
        ..Default::default()
    });

    let error = store.get::<u32>("k").await.unwrap_err();
    assert!(error.to_string().contains("boom-42"), "got: {error}");
}

#[tokio::test]
async fn clearing_failures_restores_operations() {
    let store = InMemoryCacheStore::unbounded();
    store.fail(FailureMode {
        get: Some("temporary".to_string()),
        set: Some("temporary".to_string()),
        ..Default::default()
    });
    assert!(store.set("k", &1u32, None).await.is_err());

    store.clear_failures();
    store.set("k", &1u32, None).await.unwrap();
    assert_eq!(store.get::<u32>("k").await.unwrap(), Some(1));
}

#[tokio::test]
async fn failure_in_one_operation_does_not_affect_others() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &1u32, None).await.unwrap();
    store.fail(FailureMode {
        get: Some("only get fails".to_string()),
        ..Default::default()
    });

    assert!(store.get::<u32>("k").await.is_err());
    assert!(store.exists("k").await.unwrap());
    assert!(store.delete("k").await.is_ok());
}

// ===========================================================================
// Serialization failures
// ===========================================================================

#[tokio::test]
async fn set_unserializable_value_returns_serialization_error() {
    let store = InMemoryCacheStore::unbounded();
    let result = store.set("k", &FailsToSerialize, None).await;
    assert!(
        matches!(result, Err(CacheError::SerializationError(_))),
        "expected SerializationError, got: {result:?}"
    );
}

#[tokio::test]
async fn set_unserializable_does_not_mutate_store() {
    let store = InMemoryCacheStore::unbounded();
    let _ = store.set("k", &FailsToSerialize, None).await;

    assert!(!store.exists("k").await.unwrap());
    assert_eq!(store.live_len(), 0);
}

#[tokio::test]
async fn failed_set_does_not_overwrite_existing_value() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &1u32, None).await.unwrap();

    let _ = store.set("k", &FailsToSerialize, None).await;

    assert_eq!(store.get::<u32>("k").await.unwrap(), Some(1));
}

#[tokio::test]
async fn get_type_mismatch_returns_serialization_error() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &"text".to_string(), None).await.unwrap();

    let result: Result<Option<u32>, CacheError> = store.get("k").await;
    assert!(matches!(result, Err(CacheError::SerializationError(_))));
}

#[tokio::test]
async fn get_struct_from_scalar_returns_serialization_error() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &7u32, None).await.unwrap();

    let result: Result<Option<Record>, CacheError> = store.get("k").await;
    assert!(matches!(result, Err(CacheError::SerializationError(_))));
}

#[tokio::test]
async fn get_scalar_from_struct_returns_serialization_error() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &Record::sample(3), None).await.unwrap();

    let result: Result<Option<u32>, CacheError> = store.get("k").await;
    assert!(matches!(result, Err(CacheError::SerializationError(_))));
}

// ===========================================================================
// Error type contracts
// ===========================================================================

#[test]
fn cache_error_is_send_sync() {
    assert_send_sync::<CacheError>();
}

#[test]
fn cache_error_implements_std_error() {
    assert_std_error::<CacheError>();
}

#[test]
fn cache_error_serialization_message_is_prefixed() {
    let error = CacheError::SerializationError("bad json".to_string());
    assert!(error.to_string().contains("bad json"));
    assert!(error.to_string().to_lowercase().contains("serialization"));
}

#[test]
fn cache_error_redis_message_is_prefixed() {
    let error = CacheError::RedisError("conn reset".to_string());
    assert!(error.to_string().contains("conn reset"));
    assert!(error.to_string().to_lowercase().contains("redis"));
}

#[test]
fn cache_error_not_found_display_is_stable() {
    assert_eq!(CacheError::NotFound.to_string(), "Key not found");
}
