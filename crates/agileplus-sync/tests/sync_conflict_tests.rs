//! Integration tests for SyncConflict, hash_value, and detect_conflict.

use agileplus_sync::conflict::{detect_conflict, hash_value, SyncConflict};
use serde_json::json;

// ---- SyncConflict::new ----

#[test]
fn sync_conflict_new_creates_valid_struct() {
    let local = json!({"id": 1, "name": "local"});
    let remote = json!({"id": 1, "name": "remote"});
    let c = SyncConflict::new("feature", 42, local.clone(), remote.clone());

    assert_eq!(c.entity_type, "feature");
    assert_eq!(c.entity_id, 42);
    assert_eq!(c.local_version, local);
    assert_eq!(c.remote_version, remote);
    assert!(!c.local_hash.is_empty());
    assert!(!c.remote_hash.is_empty());
}

#[test]
fn sync_conflict_accepts_string_types() {
    let c: SyncConflict = SyncConflict::new(String::from("work_package"), 100, json!({}), json!({}));
    assert_eq!(c.entity_type, "work_package");
    assert_eq!(c.entity_id, 100);
}

#[test]
fn sync_conflict_detected_at_is_set() {
    let before = chrono::Utc::now();
    let c = SyncConflict::new("feature", 1, json!({}), json!({}));
    let after = chrono::Utc::now();
    assert!(c.detected_at >= before);
    assert!(c.detected_at <= after);
}

#[test]
fn sync_conflict_with_empty_json_values() {
    let c = SyncConflict::new("feature", 1, json!({}), json!({}));
    assert_eq!(c.local_hash, c.remote_hash);
}

#[test]
fn sync_conflict_with_nested_objects() {
    let local = json!({"nested": {"deep": {"value": 1}}});
    let remote = json!({"nested": {"deep": {"value": 2}}});
    let c = SyncConflict::new("feature", 1, local, remote);
    assert_ne!(c.local_hash, c.remote_hash);
}

#[test]
fn sync_conflict_with_arrays() {
    let c = SyncConflict::new("feature", 1, json!([1, 2, 3]), json!([3, 2, 1]));
    assert_ne!(c.local_hash, c.remote_hash);
}

#[test]
fn sync_conflict_with_unicode_strings() {
    let c = SyncConflict::new("feature", 1, json!({"name": "特征"}), json!({"name": "feature"}));
    assert_ne!(c.local_hash, c.remote_hash);
}

#[test]
fn sync_conflict_with_special_characters() {
    let c = SyncConflict::new("feature", 1, json!({"data": "a\nb\tc"}), json!({"data": "a\nb\td"}));
    assert_ne!(c.local_hash, c.remote_hash);
}

#[test]
fn sync_conflict_with_large_payload() {
    let mut m1 = serde_json::Map::new();
    let mut m2 = serde_json::Map::new();
    for i in 0..50 {
        m1.insert(format!("k{}", i), json!(i));
        m2.insert(format!("k{}", i), json!(i + 1));
    }
    let c = SyncConflict::new("feature", 1, json!(m1), json!(m2));
    assert_ne!(c.local_hash, c.remote_hash);
}

// ---- is_real_conflict ----

#[test]
fn is_real_conflict_true_when_hashes_differ() {
    let c = SyncConflict::new("feature", 1, json!({"v": 1}), json!({"v": 2}));
    assert!(c.is_real_conflict());
}

#[test]
fn is_real_conflict_false_when_hashes_match() {
    let c = SyncConflict::new("feature", 1, json!({"id": 1}), json!({"id": 1}));
    assert!(!c.is_real_conflict());
}

#[test]
fn is_real_conflict_false_for_identical_objects() {
    let v = json!({"id": 42, "name": "test"});
    let c = SyncConflict::new("feature", 42, v.clone(), v);
    assert!(!c.is_real_conflict());
}

#[test]
fn is_real_conflict_true_for_booleans() {
    let c = SyncConflict::new("feature", 1, json!(false), json!(true));
    assert!(c.is_real_conflict());
}

#[test]
fn is_real_conflict_true_for_null_vs_value() {
    let c = SyncConflict::new("feature", 1, json!(null), json!("hello"));
    assert!(c.is_real_conflict());
}

#[test]
fn is_real_conflict_true_for_numbers() {
    let c = SyncConflict::new("feature", 1, json!(0), json!(1));
    assert!(c.is_real_conflict());
}

// ---- hash_value ----

#[test]
fn hash_value_returns_64_char_hex() {
    let h = hash_value(&json!({"test": true}));
    assert_eq!(h.len(), 64);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hash_value_is_deterministic() {
    let v = json!({"nested": {"deep": {"value": [1, 2, 3]}}});
    assert_eq!(hash_value(&v), hash_value(&v));
}

#[test]
fn hash_value_same_for_equivalent_structures() {
    let v1 = json!({"a": 1, "b": 2, "c": 3});
    let v2 = json!({"c": 3, "b": 2, "a": 1});
    assert_eq!(hash_value(&v1), hash_value(&v2));
}

#[test]
fn hash_value_different_for_different_content() {
    assert_ne!(hash_value(&json!({"key": "v1"})), hash_value(&json!({"key": "v2"})));
}

#[test]
fn hash_value_for_null() {
    assert_eq!(hash_value(&json!(null)).len(), 64);
}

#[test]
fn hash_value_for_boolean_true() {
    assert_eq!(hash_value(&json!(true)).len(), 64);
}

#[test]
fn hash_value_for_boolean_false() {
    assert_eq!(hash_value(&json!(false)).len(), 64);
}

#[test]
fn hash_value_for_number_zero() {
    assert_eq!(hash_value(&json!(0)).len(), 64);
}

#[test]
fn hash_value_for_negative_number() {
    assert_eq!(hash_value(&json!(-42.5)).len(), 64);
}

#[test]
fn hash_value_for_scientific_notation() {
    assert_eq!(hash_value(&json!(1.23e10)).len(), 64);
}

#[test]
fn hash_value_for_string_with_escapes() {
    let v1 = json!(r"line1\nline2");
    let v2 = json!(r"line1\nline2");
    assert_eq!(hash_value(&v1), hash_value(&v2));
}

#[test]
fn hash_value_large_object() {
    let mut m = serde_json::Map::new();
    for i in 0..100 {
        m.insert(format!("key_{}", i), json!(i));
    }
    assert_eq!(hash_value(&json!(m)).len(), 64);
}

// ---- detect_conflict ----

#[test]
fn detect_conflict_returns_none_when_both_identical() {
    let stored = hash_value(&json!({"id": 1}));
    let result = detect_conflict("feature", 1, json!({"id": 1}), json!({"id": 1}), &stored);
    assert!(result.is_none());
}

#[test]
fn detect_conflict_returns_none_when_only_local_changed() {
    let stored = hash_value(&json!({"id": 1, "name": "old"}));
    let result = detect_conflict("feature", 1, json!({"id": 1, "name": "new"}), json!({"id": 1, "name": "old"}), &stored);
    assert!(result.is_none());
}

#[test]
fn detect_conflict_returns_none_when_only_remote_changed() {
    let stored = hash_value(&json!({"id": 1, "name": "old"}));
    let result = detect_conflict("feature", 1, json!({"id": 1, "name": "old"}), json!({"id": 1, "name": "new"}), &stored);
    assert!(result.is_none());
}

#[test]
fn detect_conflict_returns_none_when_neither_changed() {
    let stored = hash_value(&json!({"id": 1}));
    let result = detect_conflict("feature", 1, json!({"id": 1}), json!({"id": 1}), &stored);
    assert!(result.is_none());
}

#[test]
fn detect_conflict_detects_real_conflict() {
    let stored = hash_value(&json!({"id": 1, "name": "original"}));
    let result = detect_conflict("feature", 1, json!({"id": 1, "name": "new"}), json!({"id": 1, "name": "also new"}), &stored);
    assert!(result.is_some());
    let c = result.unwrap();
    assert_eq!(c.entity_type, "feature");
    assert_eq!(c.entity_id, 1);
    assert!(c.is_real_conflict());
}

#[test]
fn detect_conflict_requires_both_changed() {
    let stored = hash_value(&json!({"id": 1}));
    let result = detect_conflict("feature", 1, json!({"id": 1, "name": "new"}), json!({"id": 1}), &stored);
    assert!(result.is_none());
}

#[test]
fn detect_conflict_requires_hashes_different() {
    let stored = hash_value(&json!({"id": 1}));
    let result = detect_conflict("feature", 1, json!({"id": 1, "name": "x"}), json!({"id": 1, "name": "x"}), &stored);
    assert!(result.is_none());
}

#[test]
fn detect_conflict_with_empty_stored_hash() {
    let result = detect_conflict("feature", 1, json!({"id": 1}), json!({"id": 1, "new": true}), "");
    assert!(result.is_some());
}

#[test]
fn detect_conflict_with_zero_entity_id() {
    let stored = hash_value(&json!({"id": 1}));
    let result = detect_conflict("feature", 0, json!({"id": 1}), json!({"id": 1}), &stored);
    assert!(result.is_none());
}

#[test]
fn detect_conflict_unicode_entity_type() {
    let stored = hash_value(&json!({"id": 1}));
    let result = detect_conflict("特征", 1, json!({"id": 1, "name": "new"}), json!({"id": 1, "name": "different"}), &stored);
    assert!(result.is_some());
}

#[test]
fn detect_conflict_large_payload() {
    let mut ml = serde_json::Map::new();
    let mut mr = serde_json::Map::new();
    for i in 0..50 {
        ml.insert(format!("k{}", i), json!(i));
        mr.insert(format!("k{}", i), json!(i + 1));
    }
    // Stored hash represents a different baseline, so both local and remote changed.
    let stored = hash_value(&json!({"baseline": 0}));
    let result = detect_conflict("feature", 1, json!(ml), json!(mr), &stored);
    assert!(result.is_some());
}

// ---- Serialization ----

#[test]
fn sync_conflict_serializes_to_json() {
    let c = SyncConflict::new("feature", 1, json!({"id": 1}), json!({"id": 2}));
    let j = serde_json::to_string(&c).unwrap();
    assert!(j.contains("entity_type"));
    assert!(j.contains("local_hash"));
    assert!(j.contains("remote_hash"));
}

#[test]
fn sync_conflict_deserializes_from_json() {
    let c = SyncConflict::new("feature", 42, json!({"a": 1}), json!({"b": 2}));
    let j = serde_json::to_string(&c).unwrap();
    let restored: SyncConflict = serde_json::from_str(&j).unwrap();
    assert_eq!(c.entity_type, restored.entity_type);
    assert_eq!(c.entity_id, restored.entity_id);
    assert_eq!(c.local_hash, restored.local_hash);
    assert_eq!(c.remote_hash, restored.remote_hash);
}

#[test]
fn sync_conflict_roundtrip_preserves_all_data() {
    let local = json!({"id": 1, "name": "test", "nested": {"a": true}});
    let remote = json!({"id": 1, "name": "updated", "nested": {"b": false}});
    let original = SyncConflict::new("work_package", 99, local, remote);
    let j = serde_json::to_string(&original).unwrap();
    let restored: SyncConflict = serde_json::from_str(&j).unwrap();

    assert_eq!(original.entity_type, restored.entity_type);
    assert_eq!(original.entity_id, restored.entity_id);
    assert_eq!(original.local_version, restored.local_version);
    assert_eq!(original.remote_version, restored.remote_version);
    assert_eq!(original.local_hash, restored.local_hash);
    assert_eq!(original.remote_hash, restored.remote_hash);
}