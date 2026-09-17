// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for serialization / deserialization of structs generated
//! by `config_builder!` when a `serde::Serialize` / `serde::Deserialize` derive
//! is attached by the caller.

use agileplus_config::config_builder;
use serde::{Deserialize, Serialize};

config_builder! {
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct ApiConfig {
        (str)     pub base_url: String = "https://api.example.com".to_string(),
        (val)     pub timeout_ms: u64 = 5000,
        (val)     pub retries: u8 = 3,
        (opt_str) pub api_key: Option<String> = None,
        (val)     pub enabled: bool = true,
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct TopicConfig {
        (str)     pub name: String = "general".to_string(),
        (val)     pub partitions: u16 = 1,
        (val)     pub replication: u16 = 1,
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct FeatureConfig {
        (val)     pub beta: bool = false,
        (val)     pub rollout: f32 = 0.0,
    }
}

// --- JSON serialization ----------------------------------------------------------

#[test]
fn json_default_serializes_all_fields() {
    let c = ApiConfig::default();
    let json = serde_json::to_string(&c).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["base_url"], "https://api.example.com");
    assert_eq!(v["timeout_ms"], 5000);
    assert_eq!(v["retries"], 3);
    assert_eq!(v["enabled"], true);
    assert!(v.get("api_key").unwrap().is_null());
}

#[test]
fn json_serialize_option_some() {
    let c = ApiConfig::default().with_api_key("sk-123");
    let json = serde_json::to_string(&c).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["api_key"], "sk-123");
}

#[test]
fn json_serialize_bool_false() {
    let c = FeatureConfig::default().with_beta(true);
    let json = serde_json::to_string(&c).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["beta"], true);
}

#[test]
fn json_serialize_float() {
    let c = FeatureConfig::default().with_rollout(1.0);
    let json = serde_json::to_string(&c).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["rollout"].as_f64().unwrap(), 1.0);
}

#[test]
fn json_serialize_u8() {
    let c = ApiConfig::default().with_retries(255);
    let json = serde_json::to_string(&c).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["retries"], 255);
}

// --- JSON deserialization --------------------------------------------------------

#[test]
fn json_deserialize_full() {
    let json = r#"{
        "base_url": "http://custom",
        "timeout_ms": 1000,
        "retries": 1,
        "api_key": "abc",
        "enabled": false
    }"#;
    let c: ApiConfig = serde_json::from_str(json).unwrap();
    assert_eq!(c.base_url, "http://custom");
    assert_eq!(c.timeout_ms, 1000);
    assert_eq!(c.retries, 1);
    assert_eq!(c.api_key.as_deref(), Some("abc"));
    assert!(!c.enabled);
}

#[test]
fn json_deserialize_null_api_key() {
    let json =
        r#"{"base_url":"x","timeout_ms":1,"retries":1,"api_key":null,"enabled":true}"#;
    let c: ApiConfig = serde_json::from_str(json).unwrap();
    assert_eq!(c.api_key, None);
}

#[test]
fn json_deserialize_negative_not_allowed_by_type() {
    let json = r#"{"base_url":"x","timeout_ms":-5,"retries":1,"enabled":true}"#;
    // u64 rejects negative, so this must fail.
    assert!(serde_json::from_str::<ApiConfig>(json).is_err());
}

#[test]
fn json_deserialize_type_mismatch() {
    let json =
        r#"{"base_url":"x","timeout_ms":"not-a-number","retries":1,"enabled":true}"#;
    assert!(serde_json::from_str::<ApiConfig>(json).is_err());
}

#[test]
fn json_deserialize_ignores_unknown_fields() {
    let json = r#"{"base_url":"x","timeout_ms":1,"retries":1,"enabled":true,"extra":42}"#;
    let c: ApiConfig = serde_json::from_str(json).unwrap();
    assert_eq!(c.base_url, "x");
}

#[test]
fn json_deserialize_missing_optional_is_none() {
    let json = r#"{"base_url":"x","timeout_ms":1,"retries":1,"enabled":true}"#;
    let c: ApiConfig = serde_json::from_str(json).unwrap();
    assert_eq!(c.api_key, None);
}

#[test]
fn json_deserialize_missing_required_field_fails() {
    // `retries` required -> absent => error.
    let json = r#"{"base_url":"x","timeout_ms":1,"enabled":true}"#;
    assert!(serde_json::from_str::<ApiConfig>(json).is_err());
}

// --- JSON round trips ------------------------------------------------------------

#[test]
fn json_roundtrip_default() {
    let orig = ApiConfig::default();
    let bytes = serde_json::to_vec(&orig).unwrap();
    let back: ApiConfig = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(orig, back);
}

#[test]
fn json_roundtrip_option_set() {
    let orig = ApiConfig::default()
        .with_api_key("key")
        .with_enabled(false);
    let bytes = serde_json::to_vec(&orig).unwrap();
    let back: ApiConfig = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(orig, back);
}

// --- YAML (serde_yaml) -----------------------------------------------------------

#[test]
fn yaml_roundtrip() {
    let orig = TopicConfig::default().with_partitions(12);
    let yaml = serde_yaml::to_string(&orig).unwrap();
    let back: TopicConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(orig, back);
}

#[test]
fn yaml_serialize_contains_keys() {
    let c = TopicConfig::default();
    let yaml = serde_yaml::to_string(&c).unwrap();
    assert!(yaml.contains("name: general"));
    assert!(yaml.contains("partitions: 1"));
}

#[test]
fn yaml_deserialize_values() {
    let yaml = "name: analytics\npartitions: 32\nreplication: 3\n";
    let c: TopicConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(c.name, "analytics");
    assert_eq!(c.partitions, 32);
    assert_eq!(c.replication, 3);
}

#[test]
fn yaml_deserialize_invalid_fails() {
    assert!(serde_yaml::from_str::<TopicConfig>("partitions: notanint").is_err());
}

// --- Cross-format equivalence ----------------------------------------------------

#[test]
fn json_and_yaml_produce_same_values() {
    let c = ApiConfig::default().with_base_url("cross").with_retries(9);
    let json: ApiConfig = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    let yaml: ApiConfig =
        serde_yaml::from_str(&serde_yaml::to_string(&c).unwrap()).unwrap();
    assert_eq!(json, c);
    assert_eq!(yaml, c);
    assert_eq!(json, yaml);
}