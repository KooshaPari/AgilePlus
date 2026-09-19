//! CacheStore error-path contract for injected backend failures: nothing is
//! partially applied, every operation reports its own failure, and the store
//! recovers once the backend does.
//!
//! Payload encode/decode failures live in `cache_serialization.rs`.
//!
//! Uses the shared in-memory store double (see `tests/common/mod.rs`).

mod common;

use agileplus_cache::store::{CacheError, CacheStore};
use common::{FailureMode, InMemoryCacheStore};

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
fn cache_error_redis_message_is_prefixed() {
    let error = CacheError::RedisError("conn reset".to_string());
    assert!(error.to_string().contains("conn reset"));
    assert!(error.to_string().to_lowercase().contains("redis"));
}

#[test]
fn cache_error_not_found_display_is_stable() {
    assert_eq!(CacheError::NotFound.to_string(), "Key not found");
}

// ===========================================================================
// Injected failures must not partially mutate the store
// ===========================================================================

#[tokio::test]
async fn injected_delete_failure_leaves_the_entry_present() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &1u32, None).await.unwrap();
    store.fail(FailureMode {
        delete: Some("delete exploded".to_string()),
        ..Default::default()
    });

    assert!(store.delete("k").await.is_err());

    assert!(store.exists("k").await.unwrap());
    assert_eq!(store.get::<u32>("k").await.unwrap(), Some(1));
    assert_eq!(store.live_len(), 1);
}

#[tokio::test]
async fn injected_exists_failure_leaves_the_entry_untouched() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &1u32, None).await.unwrap();
    store.fail(FailureMode {
        exists: Some("exists exploded".to_string()),
        ..Default::default()
    });

    assert!(store.exists("k").await.is_err());

    store.clear_failures();
    assert!(store.exists("k").await.unwrap());
    assert!(store.raw_contains("k"), "the entry must still exist");
}

#[tokio::test]
async fn injected_get_failure_does_not_consume_the_entry() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &"payload".to_string(), None).await.unwrap();
    store.fail(FailureMode {
        get: Some("get exploded".to_string()),
        ..Default::default()
    });

    assert!(store.get::<String>("k").await.is_err());

    store.clear_failures();
    assert_eq!(
        store.get::<String>("k").await.unwrap().as_deref(),
        Some("payload")
    );
}

#[tokio::test]
async fn injected_set_failure_does_not_evict_existing_entries() {
    let store = InMemoryCacheStore::with_capacity(1);
    store.set("kept", &1u8, None).await.unwrap();
    store.fail(FailureMode {
        set: Some("set exploded".to_string()),
        ..Default::default()
    });

    assert!(store.set("rejected", &2u8, None).await.is_err());

    assert_eq!(
        store.get::<u8>("kept").await.unwrap(),
        Some(1),
        "a failed write must not make room for itself"
    );
    assert!(!store.exists("rejected").await.unwrap());
    assert_eq!(store.live_len(), 1);
}

// ===========================================================================
// Failure routing across operations
// ===========================================================================

#[tokio::test]
async fn each_operation_reports_its_own_injected_message() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &1u32, None).await.unwrap();
    store.fail(FailureMode {
        get: Some("get-failed".to_string()),
        set: Some("set-failed".to_string()),
        delete: Some("delete-failed".to_string()),
        exists: Some("exists-failed".to_string()),
    });

    let get_error = store.get::<u32>("k").await.expect_err("get fails");
    let set_error = store.set("k", &2u32, None).await.expect_err("set fails");
    let delete_error = store.delete("k").await.expect_err("delete fails");
    let exists_error = store.exists("k").await.expect_err("exists fails");

    assert!(get_error.to_string().contains("get-failed"));
    assert!(set_error.to_string().contains("set-failed"));
    assert!(delete_error.to_string().contains("delete-failed"));
    assert!(exists_error.to_string().contains("exists-failed"));
}

#[tokio::test]
async fn clearing_failures_restores_every_operation() {
    let store = InMemoryCacheStore::unbounded();
    store.fail(FailureMode {
        get: Some("down".to_string()),
        set: Some("down".to_string()),
        delete: Some("down".to_string()),
        exists: Some("down".to_string()),
    });

    assert!(store.get::<u32>("k").await.is_err());
    assert!(store.set("k", &1u32, None).await.is_err());
    assert!(store.delete("k").await.is_err());
    assert!(store.exists("k").await.is_err());

    store.clear_failures();

    store.set("k", &1u32, None).await.unwrap();
    assert_eq!(store.get::<u32>("k").await.unwrap(), Some(1));
    assert!(store.exists("k").await.unwrap());
    store.delete("k").await.unwrap();
    assert!(!store.exists("k").await.unwrap());
}

#[tokio::test]
async fn default_failure_mode_blocks_nothing() {
    let store = InMemoryCacheStore::unbounded();
    store.fail(FailureMode::default());

    store.set("k", &1u32, None).await.unwrap();
    assert_eq!(store.get::<u32>("k").await.unwrap(), Some(1));
    assert!(store.exists("k").await.unwrap());
    store.delete("k").await.unwrap();
}

#[tokio::test]
async fn replacing_a_failure_mode_replaces_the_previous_one() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &1u32, None).await.unwrap();
    store.fail(FailureMode {
        get: Some("first".to_string()),
        ..Default::default()
    });
    assert!(store.get::<u32>("k").await.is_err());

    store.fail(FailureMode {
        set: Some("second".to_string()),
        ..Default::default()
    });

    assert!(
        store.get::<u32>("k").await.is_ok(),
        "the previous get failure must be gone"
    );
    assert!(store.set("k", &2u32, None).await.is_err());
}
