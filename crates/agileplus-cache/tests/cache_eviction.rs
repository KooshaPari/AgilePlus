//! CacheStore LRU capacity eviction contract: which entries survive when the
//! store is full, and how eviction interacts with expiry.
//!
//! Uses the shared in-memory store double (see `tests/common/mod.rs`).

mod common;

use agileplus_cache::store::CacheStore;
use common::InMemoryCacheStore;
use std::time::Duration;

async fn sleep_ms(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

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

// ===========================================================================
// TTL boundaries
// ===========================================================================

#[tokio::test]
async fn capacity_exactly_filled_does_not_evict() {
    let store = InMemoryCacheStore::with_capacity(3);
    for (key, value) in [("a", 1u8), ("b", 2), ("c", 3)] {
        store.set(key, &value, None).await.unwrap();
    }

    assert_eq!(store.live_len(), 3);
    for (key, value) in [("a", 1u8), ("b", 2), ("c", 3)] {
        assert_eq!(store.get::<u8>(key).await.unwrap(), Some(value));
    }
}

#[tokio::test]
async fn delete_then_reinsert_makes_the_key_most_recent() {
    let store = InMemoryCacheStore::with_capacity(2);
    store.set("a", &1u8, None).await.unwrap();
    store.set("b", &2u8, None).await.unwrap();
    store.delete("a").await.unwrap();

    store.set("a", &10u8, None).await.unwrap();
    store.set("c", &3u8, None).await.unwrap();

    assert_eq!(store.get::<u8>("a").await.unwrap(), Some(10));
    assert_eq!(
        store.get::<u8>("b").await.unwrap(),
        None,
        "'b' is now the least recently used"
    );
    assert_eq!(store.get::<u8>("c").await.unwrap(), Some(3));
    assert_eq!(store.live_len(), 2);
}

#[tokio::test]
async fn reading_a_key_makes_it_survive_the_next_insert() {
    let store = InMemoryCacheStore::with_capacity(2);
    store.set("a", &1u8, None).await.unwrap();
    store.set("b", &2u8, None).await.unwrap();
    store.set("c", &3u8, None).await.unwrap();
    // "a" was evicted first; refresh "c" so "b" becomes the victim.
    assert_eq!(store.get::<u8>("c").await.unwrap(), Some(3));
    store.set("d", &4u8, None).await.unwrap();

    assert_eq!(store.get::<u8>("c").await.unwrap(), Some(3));
    assert_eq!(store.get::<u8>("d").await.unwrap(), Some(4));
    assert_eq!(store.live_len(), 2);
}

#[tokio::test]
async fn capacity_invariant_holds_across_mixed_ttl_and_inserts() {
    let store = InMemoryCacheStore::with_capacity(3);
    for index in 0..12u32 {
        let ttl = if index % 3 == 0 {
            Some(Duration::from_millis(5))
        } else {
            None
        };
        store.set(&format!("k{index}"), &index, ttl).await.unwrap();
        assert!(
            store.live_len() <= 3,
            "capacity breached after k{index}: {} live",
            store.live_len()
        );
    }

    sleep_ms(20).await;
    assert!(store.live_len() <= 3);
    assert_eq!(
        store.get::<u8>("k11").await.unwrap(),
        Some(11),
        "the most recent insert must survive"
    );
}
