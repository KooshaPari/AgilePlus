//! CacheStore behaviour under concurrent access, including eviction pressure.
//!
//! Uses the shared in-memory store double (see `tests/common/mod.rs`).

mod common;

use agileplus_cache::store::CacheStore;
use common::InMemoryCacheStore;
use std::sync::Arc;
use std::time::Duration;

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

// ===========================================================================
// Concurrency under eviction pressure
// ===========================================================================

#[tokio::test]
async fn concurrent_eviction_keeps_capacity_and_value_integrity() {
    const CAPACITY: usize = 8;
    let store = Arc::new(InMemoryCacheStore::with_capacity(CAPACITY));

    let mut handles = Vec::new();
    for task in 0..24u32 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            for iteration in 0..5u32 {
                store
                    .set(
                        &format!("t{task}:{iteration}"),
                        &(task * 10 + iteration),
                        None,
                    )
                    .await
                    .unwrap();
            }
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }

    assert!(
        store.live_len() <= CAPACITY,
        "capacity must hold under concurrent writes, saw {}",
        store.live_len()
    );

    let mut survivors = 0;
    for task in 0..24u32 {
        for iteration in 0..5u32 {
            let key = format!("t{task}:{iteration}");
            if store.exists(&key).await.unwrap() {
                let value: Option<u32> = store.get(&key).await.unwrap();
                assert_eq!(
                    value,
                    Some(task * 10 + iteration),
                    "a surviving key must hold its own value"
                );
                survivors += 1;
            }
        }
    }
    assert!(survivors > 0, "some entries must survive eviction");
}

#[tokio::test]
async fn concurrent_set_and_delete_keep_exists_consistent_with_get() {
    let store = Arc::new(InMemoryCacheStore::with_capacity(4));

    let mut handles = Vec::new();
    for index in 1..=20u32 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            if index % 2 == 0 {
                store.set("shared", &index, None).await.unwrap();
            } else {
                store.delete("shared").await.unwrap();
            }
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }

    let exists = store.exists("shared").await.unwrap();
    let value: Option<u32> = store.get("shared").await.unwrap();
    assert_eq!(
        exists,
        value.is_some(),
        "exists and get must agree after concurrent churn"
    );
}
