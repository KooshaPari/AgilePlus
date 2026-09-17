//! CacheStore TTL expiry and LRU capacity eviction contract tests.
//!
//! Uses the shared in-memory store double (see `tests/common/mod.rs`) so that
//! time-based expiry and capacity behaviour are deterministic and do not need a
//! live Dragonfly/Redis server.

mod common;

use agileplus_cache::store::CacheStore;
use common::InMemoryCacheStore;
use std::time::Duration;

async fn sleep_ms(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

// ===========================================================================
// TTL expiry
// ===========================================================================

#[tokio::test]
async fn ttl_none_without_default_never_expires() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &1u32, None).await.unwrap();
    sleep_ms(30).await;

    let value: Option<u32> = store.get("k").await.unwrap();
    assert_eq!(value, Some(1));
}

#[tokio::test]
async fn ttl_entry_is_present_before_deadline() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &"value".to_string(), Some(Duration::from_millis(500)))
        .await
        .unwrap();

    let value: Option<String> = store.get("k").await.unwrap();
    assert_eq!(value.as_deref(), Some("value"));
}

#[tokio::test]
async fn ttl_entry_expires_after_deadline() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &"value".to_string(), Some(Duration::from_millis(20)))
        .await
        .unwrap();

    sleep_ms(60).await;
    let value: Option<String> = store.get("k").await.unwrap();
    assert!(value.is_none(), "entry should have expired");
}

#[tokio::test]
async fn ttl_zero_expires_immediately() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &1u8, Some(Duration::from_millis(0)))
        .await
        .unwrap();

    sleep_ms(5).await;
    let value: Option<u8> = store.get("k").await.unwrap();
    assert!(value.is_none());
}

#[tokio::test]
async fn default_ttl_applies_when_ttl_is_none() {
    let store = InMemoryCacheStore::unbounded().with_default_ttl(Duration::from_millis(20));
    store.set("k", &1u8, None).await.unwrap();

    assert_eq!(store.get::<u8>("k").await.unwrap(), Some(1));
    sleep_ms(60).await;
    assert_eq!(store.get::<u8>("k").await.unwrap(), None);
}

#[tokio::test]
async fn explicit_ttl_overrides_default_ttl() {
    let store = InMemoryCacheStore::unbounded().with_default_ttl(Duration::from_millis(10));
    store
        .set("k", &1u8, Some(Duration::from_millis(500)))
        .await
        .unwrap();

    sleep_ms(40).await;
    assert_eq!(
        store.get::<u8>("k").await.unwrap(),
        Some(1),
        "explicit longer TTL should win over the shorter default"
    );
}

#[tokio::test]
async fn expired_entry_is_removed_on_get() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &1u8, Some(Duration::from_millis(10)))
        .await
        .unwrap();
    assert_eq!(store.live_len(), 1);

    sleep_ms(40).await;
    let _: Option<u8> = store.get("k").await.unwrap();
    assert_eq!(store.live_len(), 0, "expired entry should be purged");
    assert!(!store.raw_contains("k"));
}

#[tokio::test]
async fn expired_entry_reports_not_exists() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &1u8, Some(Duration::from_millis(10)))
        .await
        .unwrap();
    sleep_ms(40).await;

    assert!(!store.exists("k").await.unwrap());
}

#[tokio::test]
async fn exists_removes_expired_entry() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &1u8, Some(Duration::from_millis(10)))
        .await
        .unwrap();
    sleep_ms(40).await;

    assert!(!store.exists("k").await.unwrap());
    assert!(!store.raw_contains("k"));
}

#[tokio::test]
async fn overwrite_resets_ttl_clock() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &1u8, Some(Duration::from_millis(30)))
        .await
        .unwrap();
    sleep_ms(15).await;
    store
        .set("k", &2u8, Some(Duration::from_millis(300)))
        .await
        .unwrap();

    sleep_ms(30).await; // past the original deadline, before the new one
    assert_eq!(store.get::<u8>("k").await.unwrap(), Some(2));
}

#[tokio::test]
async fn get_does_not_extend_ttl() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &1u8, Some(Duration::from_millis(40)))
        .await
        .unwrap();

    assert_eq!(store.get::<u8>("k").await.unwrap(), Some(1));
    sleep_ms(70).await;
    assert_eq!(store.get::<u8>("k").await.unwrap(), None);
}

#[tokio::test]
async fn long_ttl_survives_short_wait() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &1u8, Some(Duration::from_secs(30)))
        .await
        .unwrap();
    sleep_ms(25).await;

    assert_eq!(store.get::<u8>("k").await.unwrap(), Some(1));
}

#[tokio::test]
async fn mixed_ttls_expire_independently() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("short", &"s".to_string(), Some(Duration::from_millis(15)))
        .await
        .unwrap();
    store
        .set("long", &"l".to_string(), Some(Duration::from_millis(500)))
        .await
        .unwrap();

    sleep_ms(40).await;
    assert_eq!(store.get::<String>("short").await.unwrap(), None);
    assert_eq!(
        store.get::<String>("long").await.unwrap().as_deref(),
        Some("l")
    );
}

#[tokio::test]
async fn default_ttl_does_not_affect_explicit_long_ttl() {
    let store = InMemoryCacheStore::unbounded().with_default_ttl(Duration::from_millis(5));
    store.set("a", &1u8, None).await.unwrap();
    store
        .set("b", &2u8, Some(Duration::from_millis(500)))
        .await
        .unwrap();

    sleep_ms(30).await;
    assert_eq!(store.get::<u8>("a").await.unwrap(), None);
    assert_eq!(store.get::<u8>("b").await.unwrap(), Some(2));
}

// ===========================================================================
// LRU capacity eviction
// ===========================================================================

#[tokio::test]
async fn capacity_one_evicts_previous_insert() {
    let store = InMemoryCacheStore::with_capacity(1);
    store.set("a", &1u8, None).await.unwrap();
    store.set("b", &2u8, None).await.unwrap();

    assert_eq!(store.get::<u8>("a").await.unwrap(), None);
    assert_eq!(store.get::<u8>("b").await.unwrap(), Some(2));
}

#[tokio::test]
async fn capacity_two_keeps_both_entries() {
    let store = InMemoryCacheStore::with_capacity(2);
    store.set("a", &1u8, None).await.unwrap();
    store.set("b", &2u8, None).await.unwrap();

    assert_eq!(store.get::<u8>("a").await.unwrap(), Some(1));
    assert_eq!(store.get::<u8>("b").await.unwrap(), Some(2));
}

#[tokio::test]
async fn eviction_removes_least_recently_used_entry() {
    let store = InMemoryCacheStore::with_capacity(2);
    store.set("a", &1u8, None).await.unwrap();
    store.set("b", &2u8, None).await.unwrap();

    // Touch "a" so it becomes the most recently used.
    assert_eq!(store.get::<u8>("a").await.unwrap(), Some(1));

    store.set("c", &3u8, None).await.unwrap();

    assert_eq!(
        store.get::<u8>("a").await.unwrap(),
        Some(1),
        "touched key survives"
    );
    assert_eq!(
        store.get::<u8>("b").await.unwrap(),
        None,
        "stale key evicted"
    );
    assert_eq!(store.get::<u8>("c").await.unwrap(), Some(3));
}

#[tokio::test]
async fn overwrite_counts_as_recent_access() {
    let store = InMemoryCacheStore::with_capacity(2);
    store.set("a", &1u8, None).await.unwrap();
    store.set("b", &2u8, None).await.unwrap();

    store.set("a", &10u8, None).await.unwrap(); // refresh "a"
    store.set("c", &3u8, None).await.unwrap();

    assert_eq!(store.get::<u8>("a").await.unwrap(), Some(10));
    assert_eq!(store.get::<u8>("b").await.unwrap(), None);
    assert_eq!(store.get::<u8>("c").await.unwrap(), Some(3));
}

#[tokio::test]
async fn live_len_never_exceeds_capacity() {
    let store = InMemoryCacheStore::with_capacity(3);
    for i in 0..25u32 {
        store.set(&format!("k{i}"), &i, None).await.unwrap();
    }
    assert_eq!(store.live_len(), 3);
}

#[tokio::test]
async fn zero_capacity_evicts_immediately() {
    let store = InMemoryCacheStore::with_capacity(0);
    store.set("k", &1u8, None).await.unwrap();

    assert_eq!(store.live_len(), 0);
    assert_eq!(store.get::<u8>("k").await.unwrap(), None);
}

#[tokio::test]
async fn large_capacity_performs_no_eviction() {
    let store = InMemoryCacheStore::with_capacity(100);
    for i in 0..50u32 {
        store.set(&format!("k{i}"), &i, None).await.unwrap();
    }
    assert_eq!(store.live_len(), 50);
}

#[tokio::test]
async fn eviction_keeps_most_recent_keys() {
    let store = InMemoryCacheStore::with_capacity(2);
    for (key, value) in [("k1", 1u8), ("k2", 2), ("k3", 3), ("k4", 4)] {
        store.set(key, &value, None).await.unwrap();
    }

    assert_eq!(store.get::<u8>("k1").await.unwrap(), None);
    assert_eq!(store.get::<u8>("k2").await.unwrap(), None);
    assert_eq!(store.get::<u8>("k3").await.unwrap(), Some(3));
    assert_eq!(store.get::<u8>("k4").await.unwrap(), Some(4));
}

#[tokio::test]
async fn evicted_key_is_missing_and_not_exists() {
    let store = InMemoryCacheStore::with_capacity(1);
    store.set("gone", &1u8, None).await.unwrap();
    store.set("kept", &2u8, None).await.unwrap();

    assert_eq!(store.get::<u8>("gone").await.unwrap(), None);
    assert!(!store.exists("gone").await.unwrap());
    assert!(!store.raw_contains("gone"));
}

#[tokio::test]
async fn delete_frees_capacity_for_new_entries() {
    let store = InMemoryCacheStore::with_capacity(2);
    store.set("a", &1u8, None).await.unwrap();
    store.set("b", &2u8, None).await.unwrap();
    store.delete("a").await.unwrap();
    store.set("c", &3u8, None).await.unwrap();

    assert_eq!(store.live_len(), 2);
    assert_eq!(store.get::<u8>("b").await.unwrap(), Some(2));
    assert_eq!(store.get::<u8>("c").await.unwrap(), Some(3));
}

#[tokio::test]
async fn eviction_does_not_remove_a_just_inserted_entry() {
    let store = InMemoryCacheStore::with_capacity(1);
    store.set("a", &1u8, None).await.unwrap();
    store.set("b", &2u8, None).await.unwrap();

    assert_eq!(
        store.get::<u8>("b").await.unwrap(),
        Some(2),
        "the most recently inserted entry must survive eviction"
    );
}

#[tokio::test]
async fn expired_entries_do_not_count_toward_capacity() {
    let store = InMemoryCacheStore::with_capacity(2);
    store
        .set("expiring", &1u8, Some(Duration::from_millis(10)))
        .await
        .unwrap();
    sleep_ms(40).await;

    store.set("a", &1u8, None).await.unwrap();
    store.set("b", &2u8, None).await.unwrap();
    store.set("c", &3u8, None).await.unwrap();

    assert_eq!(store.live_len(), 2);
    assert!(!store.raw_contains("expiring"));
}
