//! CacheStore operation contract: typed round-trips, key semantics, and
//! concurrent access.
//!
//! Uses the shared in-memory store double (see `tests/common/mod.rs`).

mod common;

use agileplus_cache::store::CacheStore;
use common::{InMemoryCacheStore, Record};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

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

// ===========================================================================
// Concurrency
// ===========================================================================

#[tokio::test]
async fn concurrent_writes_to_distinct_keys_are_all_visible() {
    let store = Arc::new(InMemoryCacheStore::unbounded());
    let mut handles = Vec::new();
    for i in 0..32u32 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            store.set(&format!("k{i}"), &i, None).await.unwrap();
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }

    for i in 0..32u32 {
        assert_eq!(store.get::<u32>(&format!("k{i}")).await.unwrap(), Some(i));
    }
}

#[tokio::test]
async fn concurrent_writes_to_same_key_do_not_panic() {
    let store = Arc::new(InMemoryCacheStore::unbounded());
    store.set("shared", &0u32, None).await.unwrap();

    let mut handles = Vec::new();
    for i in 1..=20u32 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            store.set("shared", &i, None).await.unwrap();
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }

    let value: Option<u32> = store.get("shared").await.unwrap();
    assert!(value.is_some(), "key must remain present");
    let value = value.unwrap();
    assert!((1..=20).contains(&value), "value must be one of the writes");
}

#[tokio::test]
async fn concurrent_reads_after_write_are_consistent() {
    let store = Arc::new(InMemoryCacheStore::unbounded());
    store.set("k", &"stable".to_string(), None).await.unwrap();

    let mut handles = Vec::new();
    for _ in 0..32 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            let value: Option<String> = store.get("k").await.unwrap();
            assert_eq!(value.as_deref(), Some("stable"));
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn concurrent_mixed_operations_do_not_deadlock() {
    let store = Arc::new(InMemoryCacheStore::unbounded());
    let mut handles = Vec::new();

    for i in 0..24u32 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            let key = format!("mixed:{i}");
            store.set(&key, &i, None).await.unwrap();
            let _ = store.get::<u32>(&key).await.unwrap();
            let _ = store.exists(&key).await.unwrap();
            if i % 2 == 0 {
                store.delete(&key).await.unwrap();
            }
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }

    // Odd keys were never deleted.
    for i in (1..24u32).step_by(2) {
        assert!(store.exists(&format!("mixed:{i}")).await.unwrap());
    }
}

#[tokio::test]
async fn concurrent_deletes_are_safe() {
    let store = Arc::new(InMemoryCacheStore::unbounded());
    for i in 0..16u32 {
        store.set(&format!("d{i}"), &i, None).await.unwrap();
    }

    let mut handles = Vec::new();
    for i in 0..16u32 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            store.delete(&format!("d{i}")).await.unwrap();
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }

    for i in 0..16u32 {
        assert!(!store.exists(&format!("d{i}")).await.unwrap());
    }
}

#[tokio::test]
async fn concurrent_exists_checks_are_consistent() {
    let store = Arc::new(InMemoryCacheStore::unbounded());
    for i in 0..20u32 {
        store.set(&format!("e{i}"), &i, None).await.unwrap();
    }

    let mut handles = Vec::new();
    for _ in 0..16 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            for i in 0..20u32 {
                assert!(store.exists(&format!("e{i}")).await.unwrap());
            }
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn concurrent_writes_with_ttl_remain_readable() {
    let store = Arc::new(InMemoryCacheStore::unbounded());
    let mut handles = Vec::new();
    for i in 0..16u32 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            store
                .set(&format!("ttl{i}"), &i, Some(Duration::from_secs(30)))
                .await
                .unwrap();
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }

    for i in 0..16u32 {
        assert_eq!(store.get::<u32>(&format!("ttl{i}")).await.unwrap(), Some(i));
    }
}

#[tokio::test]
async fn concurrent_stress_many_tasks() {
    let store = Arc::new(InMemoryCacheStore::unbounded());
    let mut handles = Vec::new();
    for task in 0..64u32 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            for iteration in 0..10u32 {
                let key = format!("t{task}:{iteration}");
                store
                    .set(&key, &(task * 100 + iteration), None)
                    .await
                    .unwrap();
                let _ = store.get::<u32>(&key).await.unwrap();
            }
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(store.get::<u32>("t63:9").await.unwrap(), Some(6309));
    assert_eq!(store.live_len(), 640);
}
