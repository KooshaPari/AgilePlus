// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for the `config_builder!` macro.

use agileplus_config::config_builder;

// ---------------------------------------------------------------------------
// Struct definitions (one per concern, exercising every field kind)
// ---------------------------------------------------------------------------

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct ServerConfig {
        (str)     pub host: String = "127.0.0.1".to_string(),
        (val)     pub port: u16 = 8080,
        (val)     pub workers: usize = 4,
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct CacheConfig {
        (str)     pub host: String = "localhost".to_string(),
        (val)     pub port: u16 = 6379,
        (val)     pub pool_size: u32 = 16,
        (val)     pub default_ttl_secs: u64 = 3600,
        (val)     pub connection_timeout_secs: u64 = 5,
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct NatsConfig {
        (str)     pub url: String = "nats://localhost:4222".to_string(),
        (opt_str) pub auth_token: Option<String> = None,
        (str)     pub subject_prefix: String = "agileplus".to_string(),
        (val)     pub max_payload: usize = 1_048_576,
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct FeatureFlags {
        (val)     pub enabled: bool = true,
        (val)     pub max_retries: u32 = 3,
        (opt_str) pub rollout_group: Option<String> = None,
    }
}

// Struct using a non-primitive (enum) with `val` kind.
config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct DbConfig {
        (str)     pub url: String = "postgres://localhost/mydb".to_string(),
        (val)     pub pool_max: u32 = 10,
        (val)     pub ssl_mode: SslMode = SslMode::Prefer,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

// Struct with doc attributes to verify they survive macro expansion.
config_builder! {
    /// This doc comment should appear on the generated struct.
    #[derive(Clone, Debug, PartialEq)]
    pub struct DocumentedConfig {
        /// Doc on field.
        (str) pub name: String = "default".to_string(),
        (val) pub value: u32 = 0,
    }
}

// Struct with no fields (edge case).
config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct EmptyConfig {}
}

// Struct with a single field.
config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct SingleFieldConfig {
        (str) pub label: String = "hello".to_string(),
    }
}

// Struct with multiple `opt_str` fields.
config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct MultiOptional {
        (opt_str) pub primary: Option<String> = None,
        (opt_str) pub secondary: Option<String> = None,
        (opt_str) pub tertiary: Option<String> = None,
    }
}

// ---------------------------------------------------------------------------
// Tests: Default values
// ---------------------------------------------------------------------------

#[test]
fn default_server_config() {
    let c = ServerConfig::default();
    assert_eq!(c.host, "127.0.0.1");
    assert_eq!(c.port, 8080);
    assert_eq!(c.workers, 4);
}

#[test]
fn default_cache_config() {
    let c = CacheConfig::default();
    assert_eq!(c.host, "localhost");
    assert_eq!(c.port, 6379);
    assert_eq!(c.pool_size, 16);
    assert_eq!(c.default_ttl_secs, 3600);
    assert_eq!(c.connection_timeout_secs, 5);
}

#[test]
fn default_nats_config() {
    let c = NatsConfig::default();
    assert_eq!(c.url, "nats://localhost:4222");
    assert!(c.auth_token.is_none());
    assert_eq!(c.subject_prefix, "agileplus");
    assert_eq!(c.max_payload, 1_048_576);
}

#[test]
fn default_feature_flags() {
    let c = FeatureFlags::default();
    assert!(c.enabled);
    assert_eq!(c.max_retries, 3);
    assert!(c.rollout_group.is_none());
}

#[test]
fn default_db_config_uses_enum_default() {
    let c = DbConfig::default();
    assert_eq!(c.ssl_mode, SslMode::Prefer);
}

#[test]
fn default_empty_config() {
    let c = EmptyConfig::default();
    // Just verify it compiles and can be created.
    let _ = c;
}

#[test]
fn default_single_field() {
    let c = SingleFieldConfig::default();
    assert_eq!(c.label, "hello");
}

#[test]
fn default_multi_optional_all_none() {
    let c = MultiOptional::default();
    assert!(c.primary.is_none());
    assert!(c.secondary.is_none());
    assert!(c.tertiary.is_none());
}

// ---------------------------------------------------------------------------
// Tests: `str` field kind — accepts &str, String, and other Into<String>
// ---------------------------------------------------------------------------

#[test]
fn str_field_accepts_str_slice() {
    let c = ServerConfig::default().with_host("remote-host");
    assert_eq!(c.host, "remote-host");
}

#[test]
fn str_field_accepts_owned_string() {
    let c = ServerConfig::default().with_host(String::from("owned-host"));
    assert_eq!(c.host, "owned-host");
}

#[test]
fn str_field_accepts_string_clone() {
    let original = "cloned-host".to_string();
    let c = ServerConfig::default().with_host(original.clone());
    assert_eq!(c.host, "cloned-host");
}

#[test]
fn str_field_does_not_mutate_other_fields() {
    let c = ServerConfig::default().with_host("new");
    assert_eq!(c.port, 8080);
    assert_eq!(c.workers, 4);
}

#[test]
fn str_nats_url_accepts_str() {
    let c = NatsConfig::default().with_url("nats://prod:4222");
    assert_eq!(c.url, "nats://prod:4222");
}

#[test]
fn str_nats_subject_prefix_accepts_string() {
    let c = NatsConfig::default().with_subject_prefix(String::from("myapp"));
    assert_eq!(c.subject_prefix, "myapp");
}

// ---------------------------------------------------------------------------
// Tests: `opt_str` field kind — wraps in Some(val.into())
// ---------------------------------------------------------------------------

#[test]
fn opt_str_field_wraps_string_in_some() {
    let c = NatsConfig::default().with_auth_token("secret-tok");
    assert_eq!(c.auth_token, Some("secret-tok".to_string()));
}

#[test]
fn opt_str_field_accepts_owned_string() {
    let c = NatsConfig::default().with_auth_token(String::from("owned-tok"));
    assert_eq!(c.auth_token, Some("owned-tok".to_string()));
}

#[test]
fn opt_str_feature_flag_rollout() {
    let c = FeatureFlags::default().with_rollout_group("beta-users");
    assert_eq!(c.rollout_group, Some("beta-users".to_string()));
}

#[test]
fn opt_str_multi_optional_set_one() {
    let c = MultiOptional::default().with_primary("first");
    assert_eq!(c.primary, Some("first".to_string()));
    assert!(c.secondary.is_none());
    assert!(c.tertiary.is_none());
}

#[test]
fn opt_str_multi_optional_set_all() {
    let c = MultiOptional::default()
        .with_primary("p")
        .with_secondary("s")
        .with_tertiary("t");
    assert_eq!(c.primary, Some("p".to_string()));
    assert_eq!(c.secondary, Some("s".to_string()));
    assert_eq!(c.tertiary, Some("t".to_string()));
}

#[test]
fn opt_str_overwrites_previous_value() {
    let c = NatsConfig::default()
        .with_auth_token("first")
        .with_auth_token("second");
    assert_eq!(c.auth_token, Some("second".to_string()));
}

// ---------------------------------------------------------------------------
// Tests: `val` field kind — direct assignment
// ---------------------------------------------------------------------------

#[test]
fn val_field_sets_u16() {
    let c = ServerConfig::default().with_port(9090);
    assert_eq!(c.port, 9090);
}

#[test]
fn val_field_sets_usize() {
    let c = ServerConfig::default().with_workers(8);
    assert_eq!(c.workers, 8);
}

#[test]
fn val_field_sets_u32() {
    let c = CacheConfig::default().with_pool_size(64);
    assert_eq!(c.pool_size, 64);
}

#[test]
fn val_field_sets_u64() {
    let c = CacheConfig::default().with_default_ttl_secs(7200);
    assert_eq!(c.default_ttl_secs, 7200);
}

#[test]
fn val_field_sets_bool() {
    let c = FeatureFlags::default().with_enabled(false);
    assert!(!c.enabled);
}

#[test]
fn val_field_sets_enum() {
    let c = DbConfig::default().with_ssl_mode(SslMode::Require);
    assert_eq!(c.ssl_mode, SslMode::Require);
}

#[test]
fn val_field_sets_zero() {
    let c = SingleFieldConfig::default().with_label("");
    assert_eq!(c.label, "");
}

#[test]
fn val_feature_flags_max_retries() {
    let c = FeatureFlags::default().with_max_retries(0);
    assert_eq!(c.max_retries, 0);
}

// ---------------------------------------------------------------------------
// Tests: Method chaining
// ---------------------------------------------------------------------------

#[test]
fn chaining_server() {
    let c = ServerConfig::default()
        .with_host("prod.example.com")
        .with_port(443)
        .with_workers(16);
    assert_eq!(c.host, "prod.example.com");
    assert_eq!(c.port, 443);
    assert_eq!(c.workers, 16);
}

#[test]
fn chaining_cache() {
    let c = CacheConfig::default()
        .with_host("redis-cluster.internal")
        .with_port(6380)
        .with_pool_size(32)
        .with_default_ttl_secs(1800)
        .with_connection_timeout_secs(10);
    assert_eq!(c.host, "redis-cluster.internal");
    assert_eq!(c.port, 6380);
    assert_eq!(c.pool_size, 32);
    assert_eq!(c.default_ttl_secs, 1800);
    assert_eq!(c.connection_timeout_secs, 10);
}

#[test]
fn chaining_nats_mixed_kinds() {
    let c = NatsConfig::default()
        .with_url("nats://prod:4222")
        .with_auth_token("tok-abc")
        .with_subject_prefix("production")
        .with_max_payload(2_097_152);
    assert_eq!(c.url, "nats://prod:4222");
    assert_eq!(c.auth_token, Some("tok-abc".to_string()));
    assert_eq!(c.subject_prefix, "production");
    assert_eq!(c.max_payload, 2_097_152);
}

#[test]
fn chaining_preserves_unset_defaults() {
    let c = FeatureFlags::default().with_enabled(false);
    assert!(!c.enabled);
    assert_eq!(c.max_retries, 3); // untouched
    assert!(c.rollout_group.is_none()); // untouched
}

// ---------------------------------------------------------------------------
// Tests: Struct derived traits (Clone, Debug, PartialEq)
// ---------------------------------------------------------------------------

#[test]
fn clone_produces_equal_copy() {
    let a = ServerConfig::default().with_host("clone-test");
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn debug_format_contains_struct_name() {
    let c = ServerConfig::default();
    let dbg = format!("{:?}", c);
    assert!(dbg.contains("ServerConfig"));
    assert!(dbg.contains("127.0.0.1"));
}

#[test]
fn partial_eq_equal() {
    let a = CacheConfig::default();
    let b = CacheConfig::default();
    assert_eq!(a, b);
}

#[test]
fn partial_eq_not_equal() {
    let a = CacheConfig::default();
    let b = CacheConfig::default().with_port(9999);
    assert_ne!(a, b);
}

// ---------------------------------------------------------------------------
// Tests: Empty and single-field edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_config_clone_and_eq() {
    let a = EmptyConfig::default();
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn single_field_builder() {
    let c = SingleFieldConfig::default().with_label("custom");
    assert_eq!(c.label, "custom");
}

// ---------------------------------------------------------------------------
// Tests: Serde serialization / deserialization
// ---------------------------------------------------------------------------

config_builder! {
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct SerdeConfig {
        (str)     pub name: String = "default-name".to_string(),
        (opt_str) pub tag: Option<String> = None,
        (val)     pub count: u32 = 42,
        (val)     pub enabled: bool = true,
    }
}

#[test]
fn serde_default_serializes_to_json() {
    let c = SerdeConfig::default();
    let json = serde_json::to_string(&c).unwrap();
    assert!(json.contains("\"name\":\"default-name\""));
    assert!(json.contains("\"tag\":null"));
    assert!(json.contains("\"count\":42"));
    assert!(json.contains("\"enabled\":true"));
}

#[test]
fn serde_custom_values_serialize() {
    let c = SerdeConfig::default()
        .with_name("custom")
        .with_tag("v2")
        .with_count(100)
        .with_enabled(false);
    let json = serde_json::to_string(&c).unwrap();
    assert!(json.contains("\"name\":\"custom\""));
    assert!(json.contains("\"tag\":\"v2\""));
    assert!(json.contains("\"count\":100"));
    assert!(json.contains("\"enabled\":false"));
}

#[test]
fn serde_roundtrip_json() {
    let original = SerdeConfig::default()
        .with_name("roundtrip")
        .with_tag("test")
        .with_count(7)
        .with_enabled(false);
    let json = serde_json::to_string(&original).unwrap();
    let restored: SerdeConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn serde_deserialize_from_json() {
    let json = r#"{"name":"from-json","tag":" deser ","count":99,"enabled":false}"#;
    let c: SerdeConfig = serde_json::from_str(json).unwrap();
    assert_eq!(c.name, "from-json");
    assert_eq!(c.tag, Some(" deser ".to_string()));
    assert_eq!(c.count, 99);
    assert!(!c.enabled);
}

#[test]
fn serde_roundtrip_yaml() {
    let original = SerdeConfig::default()
        .with_name("yaml-test")
        .with_count(55);
    let yaml = serde_yaml::to_string(&original).unwrap();
    let restored: SerdeConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn serde_deserialize_from_yaml() {
    let yaml = r#"
name: from-yaml
tag: null
count: 123
enabled: true
"#;
    let c: SerdeConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(c.name, "from-yaml");
    assert!(c.tag.is_none());
    assert_eq!(c.count, 123);
    assert!(c.enabled);
}

#[test]
fn serde_json_pretty_format() {
    let c = SerdeConfig::default().with_name("pretty");
    let json = serde_json::to_string_pretty(&c).unwrap();
    assert!(json.contains('\n'));
    assert!(json.contains("\"name\": \"pretty\""));
}

// ---------------------------------------------------------------------------
// Tests: Serde with opt_str — Some vs None roundtrips
// ---------------------------------------------------------------------------

config_builder! {
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct OptStrSerde {
        (opt_str) pub token: Option<String> = None,
        (val)     pub id: u64 = 0,
    }
}

#[test]
fn serde_opt_str_none_roundtrips() {
    let original = OptStrSerde::default();
    let json = serde_json::to_string(&original).unwrap();
    assert!(json.contains("\"token\":null"));
    let restored: OptStrSerde = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn serde_opt_str_some_roundtrips() {
    let original = OptStrSerde::default().with_token("abc").with_id(42);
    let json = serde_json::to_string(&original).unwrap();
    assert!(json.contains("\"token\":\"abc\""));
    let restored: OptStrSerde = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}
