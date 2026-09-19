//! CacheStore basic operation contract: typed round-trips and key semantics.
//!
//! Uses the shared in-memory store double (see `tests/common/mod.rs`).

mod common;

use agileplus_cache::store::CacheStore;
use common::{InMemoryCacheStore, Record};
use std::collections::BTreeMap;

// ===========================================================================
// Basic operations and typed round-trips
// ===========================================================================

#[tokio::test]
async fn set_then_get_roundtrips_u64() {
    let store = InMemoryCacheStore::unbounded();
    store.set("answer", &42u64, None).await.unwrap();
    assert_eq!(store.get::<u64>("answer").await.unwrap(), Some(42));
}

#[tokio::test]
async fn get_missing_key_returns_none() {
    let store = InMemoryCacheStore::unbounded();
    assert_eq!(store.get::<u64>("absent").await.unwrap(), None);
}

#[tokio::test]
async fn overwrite_replaces_previous_value() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &"first".to_string(), None).await.unwrap();
    store.set("k", &"second".to_string(), None).await.unwrap();

    assert_eq!(
        store.get::<String>("k").await.unwrap().as_deref(),
        Some("second")
    );
}

#[tokio::test]
async fn delete_removes_key() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &1u8, None).await.unwrap();
    assert!(store.exists("k").await.unwrap());

    store.delete("k").await.unwrap();
    assert!(!store.exists("k").await.unwrap());
    assert_eq!(store.get::<u8>("k").await.unwrap(), None);
}

#[tokio::test]
async fn delete_missing_key_is_ok() {
    let store = InMemoryCacheStore::unbounded();
    store.delete("never-set").await.unwrap();
}

#[tokio::test]
async fn exists_true_after_set() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &1u8, None).await.unwrap();
    assert!(store.exists("k").await.unwrap());
}

#[tokio::test]
async fn exists_false_when_never_set() {
    let store = InMemoryCacheStore::unbounded();
    assert!(!store.exists("k").await.unwrap());
}

#[tokio::test]
async fn exists_false_after_delete() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &1u8, None).await.unwrap();
    store.delete("k").await.unwrap();
    assert!(!store.exists("k").await.unwrap());
}

#[tokio::test]
async fn multiple_keys_are_independent() {
    let store = InMemoryCacheStore::unbounded();
    store.set("a", &1u32, None).await.unwrap();
    store.set("b", &2u32, None).await.unwrap();
    store.set("c", &3u32, None).await.unwrap();

    assert_eq!(store.get::<u32>("a").await.unwrap(), Some(1));
    assert_eq!(store.get::<u32>("b").await.unwrap(), Some(2));
    assert_eq!(store.get::<u32>("c").await.unwrap(), Some(3));
}

#[tokio::test]
async fn empty_string_value_roundtrips() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &String::new(), None).await.unwrap();
    assert_eq!(store.get::<String>("k").await.unwrap().as_deref(), Some(""));
}

#[tokio::test]
async fn empty_key_is_supported() {
    let store = InMemoryCacheStore::unbounded();
    store.set("", &7u8, None).await.unwrap();
    assert_eq!(store.get::<u8>("").await.unwrap(), Some(7));
    assert!(store.exists("").await.unwrap());
}

#[tokio::test]
async fn struct_roundtrips() {
    let store = InMemoryCacheStore::unbounded();
    let record = Record::sample(1);
    store.set("record:1", &record, None).await.unwrap();

    let stored: Record = store.get("record:1").await.unwrap().unwrap();
    assert_eq!(stored, record);
}

#[tokio::test]
async fn vec_roundtrips() {
    let store = InMemoryCacheStore::unbounded();
    let values = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    store.set("list", &values, None).await.unwrap();

    let stored: Vec<String> = store.get("list").await.unwrap().unwrap();
    assert_eq!(stored, values);
}

#[tokio::test]
async fn map_roundtrips() {
    let store = InMemoryCacheStore::unbounded();
    let mut map = BTreeMap::new();
    map.insert("one".to_string(), 1u32);
    map.insert("two".to_string(), 2u32);
    store.set("map", &map, None).await.unwrap();

    let stored: BTreeMap<String, u32> = store.get("map").await.unwrap().unwrap();
    assert_eq!(stored, map);
}

#[tokio::test]
async fn option_none_roundtrips() {
    let store = InMemoryCacheStore::unbounded();
    store.set("optional", &None::<u32>, None).await.unwrap();
    let stored: Option<Option<u32>> = store.get("optional").await.unwrap();
    assert_eq!(stored, Some(None));
}

#[tokio::test]
async fn bool_and_float_roundtrip() {
    let store = InMemoryCacheStore::unbounded();
    store.set("flag", &true, None).await.unwrap();
    store.set("ratio", &0.5f64, None).await.unwrap();

    assert_eq!(store.get::<bool>("flag").await.unwrap(), Some(true));
    assert_eq!(store.get::<f64>("ratio").await.unwrap(), Some(0.5));
}

#[tokio::test]
async fn unicode_value_roundtrips() {
    let store = InMemoryCacheStore::unbounded();
    let text = "café — 日本語 🚀".to_string();
    store.set("unicode", &text, None).await.unwrap();

    assert_eq!(store.get::<String>("unicode").await.unwrap(), Some(text));
}

#[tokio::test]
async fn large_value_roundtrips() {
    let store = InMemoryCacheStore::unbounded();
    let blob = "x".repeat(100_000);
    store.set("blob", &blob, None).await.unwrap();

    let stored: String = store.get("blob").await.unwrap().unwrap();
    assert_eq!(stored.len(), 100_000);
}

#[tokio::test]
async fn many_keys_all_readable() {
    let store = InMemoryCacheStore::unbounded();
    for i in 0..500u32 {
        store.set(&format!("key:{i}"), &i, None).await.unwrap();
    }
    for i in (0..500u32).step_by(97) {
        assert_eq!(
            store.get::<u32>(&format!("key:{i}")).await.unwrap(),
            Some(i)
        );
    }
}
