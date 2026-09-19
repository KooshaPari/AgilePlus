//! CacheStore TTL expiry contract: deadlines, default vs explicit TTL, and
//! boundary cases at zero and very long lifetimes.
//!
//! Uses the shared in-memory store double (see `tests/common/mod.rs`) so expiry
//! behaviour is deterministic and needs no live Dragonfly/Redis server.
//! Sleeps are deliberately generous multiples of the TTL under test.

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
// TTL boundaries
// ===========================================================================

#[tokio::test]
async fn sub_millisecond_ttl_expires_well_within_the_sleep_margin() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &1u8, Some(Duration::from_micros(1)))
        .await
        .unwrap();

    // 20ms is 20_000x the TTL, so this does not depend on tight timing.
    sleep_ms(20).await;

    assert_eq!(store.get::<u8>("k").await.unwrap(), None);
    assert!(!store.raw_contains("k"), "expired entry must be purged");
}

#[tokio::test]
async fn year_long_ttl_entry_stays_live() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &1u8, Some(Duration::from_secs(365 * 24 * 3600)))
        .await
        .unwrap();

    sleep_ms(20).await;

    assert_eq!(store.get::<u8>("k").await.unwrap(), Some(1));
    assert!(store.exists("k").await.unwrap());
}

#[tokio::test]
async fn zero_ttl_entry_is_absent_for_reads_without_a_sleep() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &1u8, Some(Duration::ZERO)).await.unwrap();

    assert!(
        !store.exists("k").await.unwrap(),
        "a zero TTL is already past its deadline"
    );
    assert!(!store.raw_contains("k"), "the dead entry must be purged");
    assert_eq!(store.live_len(), 0);
}

#[tokio::test]
async fn overwrite_without_ttl_clears_a_previous_expiry() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &1u8, Some(Duration::from_millis(20)))
        .await
        .unwrap();
    sleep_ms(40).await;
    assert_eq!(
        store.get::<u8>("k").await.unwrap(),
        None,
        "first entry expired"
    );

    store.set("k", &2u8, None).await.unwrap();
    sleep_ms(40).await;

    assert_eq!(
        store.get::<u8>("k").await.unwrap(),
        Some(2),
        "the replacement must not inherit the old deadline"
    );
}

#[tokio::test]
async fn overwrite_without_ttl_falls_back_to_the_default_ttl() {
    let store = InMemoryCacheStore::unbounded().with_default_ttl(Duration::from_millis(20));
    store
        .set("k", &1u8, Some(Duration::from_secs(30)))
        .await
        .unwrap();
    assert_eq!(store.get::<u8>("k").await.unwrap(), Some(1));

    store.set("k", &2u8, None).await.unwrap();
    sleep_ms(60).await;

    assert_eq!(
        store.get::<u8>("k").await.unwrap(),
        None,
        "the default TTL must apply to the replacement"
    );
}

#[tokio::test]
async fn refilling_an_expired_key_yields_a_live_entry() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("k", &1u8, Some(Duration::from_millis(10)))
        .await
        .unwrap();
    sleep_ms(40).await;

    store.set("k", &2u8, None).await.unwrap();

    assert_eq!(store.get::<u8>("k").await.unwrap(), Some(2));
    assert_eq!(store.live_len(), 1);
}

#[tokio::test]
async fn expiry_does_not_disturb_a_live_sibling() {
    let store = InMemoryCacheStore::with_capacity(2);
    store
        .set("expiring", &1u8, Some(Duration::from_millis(10)))
        .await
        .unwrap();
    store.set("stable", &2u8, None).await.unwrap();
    sleep_ms(40).await;

    assert_eq!(store.get::<u8>("expiring").await.unwrap(), None);
    assert_eq!(store.get::<u8>("stable").await.unwrap(), Some(2));
    assert_eq!(store.live_len(), 1);
}

#[tokio::test]
async fn live_len_ignores_expired_entries_without_a_read() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("short", &1u8, Some(Duration::from_millis(10)))
        .await
        .unwrap();
    store
        .set("long", &2u8, Some(Duration::from_secs(30)))
        .await
        .unwrap();
    sleep_ms(40).await;

    assert_eq!(store.live_len(), 1, "expired entries are not live");
}

// ===========================================================================
// Eviction policy boundaries
// ===========================================================================
