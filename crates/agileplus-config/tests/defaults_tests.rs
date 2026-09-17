// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for default values of generated structs.

use agileplus_config::config_builder;

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

#[derive(Clone, Debug, PartialEq)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct DbConfig {
        (str)     pub url: String = "postgres://localhost/mydb".to_string(),
        (val)     pub pool_max: u32 = 10,
        (val)     pub ssl_mode: SslMode = SslMode::Prefer,
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct DocumentedConfig {
        /// Doc on field.
        (str) pub name: String = "default".to_string(),
        (val) pub value: u32 = 0,
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct EmptyConfig {}
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct SingleFieldConfig {
        (str) pub label: String = "hello".to_string(),
    }
}

config_builder! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct MultiOptional {
        (opt_str) pub primary: Option<String> = None,
        (opt_str) pub secondary: Option<String> = None,
        (opt_str) pub tertiary: Option<String> = None,
    }
}

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