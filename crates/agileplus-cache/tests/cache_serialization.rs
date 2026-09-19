//! Payload encoding/decoding contract: how values survive a JSON round-trip
//! through the cache, and which payloads are rejected.
//!
//! Uses the shared in-memory store double (see `tests/common/mod.rs`).

mod common;

use agileplus_cache::store::{CacheError, CacheStore};
use common::{FailsToSerialize, InMemoryCacheStore, Record};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[tokio::test]
async fn hash_map_roundtrips() {
    let store = InMemoryCacheStore::unbounded();
    let mut map: HashMap<String, Vec<u32>> = HashMap::new();
    map.insert("a".to_string(), vec![1, 2, 3]);
    map.insert("b".to_string(), Vec::new());

    store.set("map", &map, None).await.unwrap();
    let stored: HashMap<String, Vec<u32>> = store.get("map").await.unwrap().unwrap();

    assert_eq!(stored, map);
}

#[tokio::test]
async fn nested_structures_roundtrip() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Envelope {
        version: u8,
        payload: Record,
        parent: Option<Record>,
        history: Vec<Record>,
    }

    let store = InMemoryCacheStore::unbounded();
    let envelope = Envelope {
        version: 2,
        payload: Record::sample(1),
        parent: Some(Record::sample(0)),
        history: vec![Record::sample(1), Record::sample(2)],
    };

    store.set("envelope", &envelope, None).await.unwrap();
    let stored: Envelope = store.get("envelope").await.unwrap().unwrap();

    assert_eq!(stored, envelope);
}

#[tokio::test]
async fn tuple_and_char_roundtrip() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("tuple", &(7u8, "seven".to_string(), true), None)
        .await
        .unwrap();
    store.set("char", &'é', None).await.unwrap();

    let tuple: (u8, String, bool) = store.get("tuple").await.unwrap().unwrap();
    assert_eq!(tuple, (7, "seven".to_string(), true));
    assert_eq!(store.get::<char>("char").await.unwrap(), Some('é'));
}

#[tokio::test]
async fn integer_extremes_roundtrip() {
    let store = InMemoryCacheStore::unbounded();
    store.set("i64-min", &i64::MIN, None).await.unwrap();
    store.set("i64-max", &i64::MAX, None).await.unwrap();
    store.set("u64-max", &u64::MAX, None).await.unwrap();
    store.set("i8-min", &i8::MIN, None).await.unwrap();

    assert_eq!(store.get::<i64>("i64-min").await.unwrap(), Some(i64::MIN));
    assert_eq!(store.get::<i64>("i64-max").await.unwrap(), Some(i64::MAX));
    assert_eq!(store.get::<u64>("u64-max").await.unwrap(), Some(u64::MAX));
    assert_eq!(store.get::<i8>("i8-min").await.unwrap(), Some(i8::MIN));
}

#[tokio::test]
async fn json_value_payloads_roundtrip_verbatim() {
    let store = InMemoryCacheStore::unbounded();
    let value = serde_json::json!({
        "nested": {"list": [1, 2.5, null, true], "text": "héllo"},
        "count": 3,
    });

    store.set("value", &value, None).await.unwrap();

    let stored: serde_json::Value = store.get("value").await.unwrap().unwrap();
    assert_eq!(stored, value);
}

#[tokio::test]
async fn empty_collections_roundtrip() {
    let store = InMemoryCacheStore::unbounded();
    let empty_vec: Vec<u32> = Vec::new();
    let empty_map: HashMap<String, u32> = HashMap::new();

    store.set("vec", &empty_vec, None).await.unwrap();
    store.set("map", &empty_map, None).await.unwrap();

    assert_eq!(store.get::<Vec<u32>>("vec").await.unwrap(), Some(empty_vec));
    assert_eq!(
        store.get::<HashMap<String, u32>>("map").await.unwrap(),
        Some(empty_map)
    );
}

#[tokio::test]
async fn non_finite_float_is_stored_as_json_null_and_fails_to_decode() {
    let store = InMemoryCacheStore::unbounded();
    store.set("nan", &f64::NAN, None).await.unwrap();

    assert!(
        store.exists("nan").await.unwrap(),
        "the write itself succeeds; only the value is not representable"
    );
    let result: Result<Option<f64>, CacheError> = store.get("nan").await;
    assert!(
        matches!(result, Err(CacheError::SerializationError(_))),
        "non-finite floats cannot survive a JSON round-trip, got {result:?}"
    );
}

#[tokio::test]
async fn stored_json_missing_a_required_field_fails_to_decode() {
    let store = InMemoryCacheStore::unbounded();
    store
        .set("partial", &serde_json::json!({"id": 5}), None)
        .await
        .unwrap();

    let result: Result<Option<Record>, CacheError> = store.get("partial").await;
    assert!(
        matches!(result, Err(CacheError::SerializationError(_))),
        "a structurally incompatible payload must not decode, got {result:?}"
    );
}

#[tokio::test]
async fn long_and_control_character_keys_roundtrip() {
    let store = InMemoryCacheStore::unbounded();
    let long_key = "k".repeat(4096);
    let odd_key = "line\nbreak\ttab\u{0}nul".to_string();

    store.set(&long_key, &1u8, None).await.unwrap();
    store.set(&odd_key, &2u8, None).await.unwrap();

    assert_eq!(store.get::<u8>(&long_key).await.unwrap(), Some(1));
    assert_eq!(store.get::<u8>(&odd_key).await.unwrap(), Some(2));
    assert!(store.exists(&odd_key).await.unwrap());
}

#[tokio::test]
async fn keys_are_byte_exact_and_case_sensitive() {
    let store = InMemoryCacheStore::unbounded();
    store.set("Key", &1u8, None).await.unwrap();
    store.set("key", &2u8, None).await.unwrap();

    assert_eq!(store.get::<u8>("Key").await.unwrap(), Some(1));
    assert_eq!(store.get::<u8>("key").await.unwrap(), Some(2));
    assert_eq!(store.get::<u8>("KEY").await.unwrap(), None);
    assert_eq!(store.live_len(), 2);
}

// ===========================================================================
// Failures while encoding or decoding a payload
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

#[test]
fn cache_error_serialization_message_is_prefixed() {
    let error = CacheError::SerializationError("bad json".to_string());
    assert!(error.to_string().contains("bad json"));
    assert!(error.to_string().to_lowercase().contains("serialization"));
}

#[tokio::test]
async fn serialization_error_preserves_the_serializer_message() {
    let store = InMemoryCacheStore::unbounded();
    let error = store
        .set("k", &FailsToSerialize, None)
        .await
        .expect_err("must fail");

    assert!(
        error
            .to_string()
            .contains("intentional serialization failure"),
        "the underlying cause must not be swallowed, got {error}"
    );
}

#[tokio::test]
async fn decode_error_reports_the_expected_shape() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &7u32, None).await.unwrap();

    let error = store
        .get::<Record>("k")
        .await
        .expect_err("must fail to decode");

    assert!(matches!(error, CacheError::SerializationError(_)));
    let message = error.to_string();
    assert!(message.starts_with("Serialization error:"), "got {message}");
    assert!(
        message.len() > "Serialization error: ".len(),
        "the serde detail must be included, got {message}"
    );
}

#[tokio::test]
async fn decode_error_is_not_reported_as_a_cache_miss() {
    let store = InMemoryCacheStore::unbounded();
    store.set("k", &"text".to_string(), None).await.unwrap();

    let result: Result<Option<u32>, CacheError> = store.get("k").await;

    assert!(
        result.is_err(),
        "a corrupt payload must not be reported as a miss"
    );
    assert!(store.exists("k").await.unwrap());
}
