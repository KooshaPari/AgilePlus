//! `RedisCacheStore` happy-path behaviour: JSON serialization, TTL plumbing,
//! key semantics, and concurrent use through the pool. Error paths live in
//! `cache_store_redis_errors.rs`.
//!
//! Driven by the in-process RESP double (`tests/common/redis.rs`) so the real
//! adapter is exercised without a live Dragonfly/Redis server.

mod common;

use agileplus_cache::config::CacheConfig;
use agileplus_cache::pool::CachePool;
use agileplus_cache::store::{CacheError, CacheStore, RedisCacheStore};
use common::Record;
use common::redis::MockRedisServer;
use std::sync::Arc;
use std::time::Duration;

async fn store_with(server: &MockRedisServer, default_ttl_secs: u64) -> RedisCacheStore {
    let pool = CachePool::new(&CacheConfig::new(server.host(), server.port()).with_pool_size(4))
        .await
        .expect("pool");
    RedisCacheStore::new(pool, default_ttl_secs)
}

// ===========================================================================
// Serialization round-trips
// ===========================================================================

#[tokio::test]
async fn set_stores_json_under_the_raw_key() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;
    let record = Record::sample(7);

    store.set("record:7", &record, None).await.expect("set");

    let raw = server.stored("record:7").expect("payload stored");
    assert_eq!(raw, serde_json::to_string(&record).expect("json"));
    assert_eq!(server.stored_keys(), vec!["record:7".to_string()]);
}

#[tokio::test]
async fn get_decodes_previously_stored_json() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;

    store.set("k", &Record::sample(3), None).await.expect("set");
    let loaded: Record = store.get("k").await.expect("get").expect("present");

    assert_eq!(loaded, Record::sample(3));
}

#[tokio::test]
async fn get_returns_none_for_absent_key() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;

    let loaded: Option<Record> = store.get("absent").await.expect("get");
    assert!(loaded.is_none());
}

#[tokio::test]
async fn get_decodes_values_written_by_another_client() {
    let server = MockRedisServer::start().await;
    server.seed("external", r#"{"name":"seed"}"#);
    let store = store_with(&server, 3600).await;

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Payload {
        name: String,
    }

    let loaded: Payload = store.get("external").await.expect("get").expect("present");
    assert_eq!(
        loaded,
        Payload {
            name: "seed".to_string()
        }
    );
}

#[tokio::test]
async fn unicode_key_and_value_roundtrip() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;
    let value = "café — 日本語 🚀".to_string();

    store.set("clé:🚀", &value, None).await.expect("set");
    let loaded: String = store.get("clé:🚀").await.expect("get").expect("present");

    assert_eq!(loaded, value);
    assert_eq!(
        server.stored("clé:🚀"),
        Some(serde_json::to_string(&value).unwrap())
    );
}

#[tokio::test]
async fn get_reports_serialization_error_for_non_json_payload() {
    let server = MockRedisServer::start().await;
    server.seed("broken", "not-json");
    let store = store_with(&server, 3600).await;

    let result: Result<Option<Record>, CacheError> = store.get("broken").await;
    assert!(matches!(result, Err(CacheError::SerializationError(_))));
}

#[tokio::test]
async fn get_reports_serialization_error_for_wrong_shape() {
    let server = MockRedisServer::start().await;
    server.seed("wrong", r#"{"unexpected":true}"#);
    let store = store_with(&server, 3600).await;

    let result: Result<Option<Record>, CacheError> = store.get("wrong").await;
    assert!(matches!(result, Err(CacheError::SerializationError(_))));
}

// ===========================================================================
// TTL plumbing
// ===========================================================================

#[tokio::test]
async fn set_uses_configured_default_ttl_when_none_is_given() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 90).await;

    store.set("k", &1u32, None).await.expect("set");

    assert_eq!(server.setex_ttls(), vec![("k".to_string(), 90)]);
}

#[tokio::test]
async fn set_explicit_ttl_overrides_default() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 90).await;

    store
        .set("k", &1u32, Some(Duration::from_secs(15)))
        .await
        .expect("set");

    assert_eq!(server.setex_ttls(), vec![("k".to_string(), 15)]);
}

#[tokio::test]
async fn set_truncates_sub_second_ttl_to_whole_seconds() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 90).await;

    store
        .set("short", &1u32, Some(Duration::from_millis(1500)))
        .await
        .expect("set");
    store
        .set("shorter", &1u32, Some(Duration::from_millis(500)))
        .await
        .expect("set");

    assert_eq!(
        server.setex_ttls(),
        vec![("short".to_string(), 1), ("shorter".to_string(), 0)]
    );
}

#[tokio::test]
async fn set_sends_setex_with_key_and_value() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 60).await;

    store.set("k", &"v".to_string(), None).await.expect("set");

    let command = server.first_command("SETEX").expect("SETEX issued");
    assert_eq!(command.args.len(), 3, "SETEX key ttl value");
    assert_eq!(command.args[0], "k");
    assert_eq!(command.args[1], "60");
    assert_eq!(command.args[2], "\"v\"");
}

// ===========================================================================
// delete / exists
// ===========================================================================

#[tokio::test]
async fn delete_removes_stored_value() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;
    store.set("k", &1u32, None).await.expect("set");

    store.delete("k").await.expect("delete");

    assert_eq!(server.stored("k"), None);
    let loaded: Option<u32> = store.get("k").await.expect("get");
    assert!(loaded.is_none());
    assert!(server.count_commands("DEL") >= 1);
}

#[tokio::test]
async fn delete_of_absent_key_succeeds() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;

    store
        .delete("never-written")
        .await
        .expect("delete is idempotent");
}

#[tokio::test]
async fn exists_reflects_server_state() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;

    assert!(!store.exists("k").await.expect("exists"));
    store.set("k", &1u32, None).await.expect("set");
    assert!(store.exists("k").await.expect("exists"));
}

#[tokio::test]
async fn exists_does_not_read_the_value() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;
    store.set("k", &1u32, None).await.expect("set");

    assert!(store.exists("k").await.expect("exists"));
    assert_eq!(server.count_commands("GET"), 0, "EXISTS must not fetch");
}

#[tokio::test]
async fn exists_is_false_after_delete() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;
    store.set("k", &1u32, None).await.expect("set");
    store.delete("k").await.expect("delete");

    assert!(!store.exists("k").await.expect("exists"));
}

#[tokio::test]
async fn overwrite_replaces_stored_payload() {
    let server = MockRedisServer::start().await;
    let store = store_with(&server, 3600).await;

    store.set("k", &1u32, None).await.expect("set");
    store.set("k", &2u32, None).await.expect("set");

    assert_eq!(store.get::<u32>("k").await.unwrap(), Some(2));
}

// ===========================================================================
// Concurrent use
// ===========================================================================

#[tokio::test]
async fn concurrent_writes_through_one_pool_are_all_readable() {
    let server = MockRedisServer::start().await;
    let store = Arc::new(store_with(&server, 3600).await);

    let mut handles = Vec::new();
    for task in 0..16u32 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            for iteration in 0..5u32 {
                let key = format!("t{task}:{iteration}");
                let value = Record::sample(i64::from(task * 100 + iteration));
                store.set(&key, &value, None).await.expect("set");
                let loaded: Record = store.get(&key).await.expect("get").expect("present");
                assert_eq!(loaded, value);
            }
        }));
    }
    for handle in handles {
        handle.await.expect("task joins");
    }

    assert_eq!(server.stored_keys().len(), 80);
    assert_eq!(
        store.get::<Record>("t15:4").await.unwrap(),
        Some(Record::sample(1504))
    );
}

#[tokio::test]
async fn concurrent_readers_see_a_stable_value() {
    let server = MockRedisServer::start().await;
    let store = Arc::new(store_with(&server, 3600).await);
    store
        .set("stable", &"unchanged".to_string(), None)
        .await
        .expect("set");

    let mut handles = Vec::new();
    for _ in 0..16 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            let loaded: String = store.get("stable").await.expect("get").expect("present");
            assert_eq!(loaded, "unchanged");
        }));
    }
    for handle in handles {
        handle.await.expect("task joins");
    }
}
