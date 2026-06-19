// SPDX-License-Identifier: MIT OR Apache-2.0
//! Credential store port and helpers.

use std::sync::Arc;

use crate::config::AppConfig;

/// Well-known key names stored in the credential store.
pub mod keys {
    pub const API_KEY: &str = "api_key";
    /// Alias used by the key lifecycle helper.
    pub const API_KEYS: &str = "api_keys";
}

/// Trait for validating and storing API keys.  Constant-time comparison is
/// expected by implementations to prevent timing attacks.
pub trait CredentialStore: Send + Sync {
    /// Returns `Ok(true)` if the plaintext `key` is valid.
    fn validate_api_key(&self, key: &str)
        -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;

    /// Retrieve a named credential value (namespace + key).
    fn get(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    /// Store a named credential value.
    fn set(
        &self,
        namespace: &str,
        key: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// In-memory credential store backed by a comma-separated list of allowed keys.
pub struct InMemoryCredentialStore {
    keys: Vec<String>,
}

impl InMemoryCredentialStore {
    pub fn new(keys: Vec<String>) -> Self {
        Self { keys }
    }
}

impl CredentialStore for InMemoryCredentialStore {
    fn validate_api_key(
        &self,
        key: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Constant-time comparison to resist timing attacks.
        use std::hint::black_box;
        let target = key.as_bytes();
        let valid = self.keys.iter().any(|k| {
            let kb = k.as_bytes();
            if kb.len() != target.len() {
                black_box(false)
            } else {
                let matches = kb
                    .iter()
                    .zip(target.iter())
                    .fold(0u8, |acc, (a, b)| acc | (a ^ b));
                black_box(matches == 0)
            }
        });
        Ok(valid)
    }

    fn get(
        &self,
        _namespace: &str,
        _key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // In-memory store doesn't persist; return empty to signal no existing key.
        Ok(String::new())
    }

    fn set(
        &self,
        _namespace: &str,
        _key: &str,
        _value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In-memory store is read-only post-construction; silently accept writes.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiConfig, AppConfig, CoreConfig};

    fn make_config(keys: Option<&str>) -> AppConfig {
        AppConfig {
            core: CoreConfig {
                database_path: "test.db".into(),
            },
            api: ApiConfig {
                port: 3030,
                api_keys: keys.map(String::from),
            },
        }
    }

    #[test]
    fn in_memory_store_validates_correct_key() {
        let store = InMemoryCredentialStore::new(vec!["key1".to_string(), "key2".to_string()]);
        assert!(store.validate_api_key("key1").unwrap());
        assert!(store.validate_api_key("key2").unwrap());
    }

    #[test]
    fn in_memory_store_rejects_wrong_key() {
        let store = InMemoryCredentialStore::new(vec!["valid-key".to_string()]);
        assert!(!store.validate_api_key("invalid-key").unwrap());
        assert!(!store.validate_api_key("").unwrap());
    }

    #[test]
    fn in_memory_store_get_returns_empty() {
        let store = InMemoryCredentialStore::new(vec![]);
        assert_eq!(store.get("ns", "k").unwrap(), "");
    }

    #[test]
    fn create_credential_store_parses_comma_separated_keys() {
        let cfg = make_config(Some("key-a, key-b , key-c"));
        let store = create_credential_store(&cfg);
        assert!(store.validate_api_key("key-a").unwrap());
        assert!(store.validate_api_key("key-b").unwrap());
        assert!(store.validate_api_key("key-c").unwrap());
        assert!(!store.validate_api_key("key-d").unwrap());
    }

    #[test]
    fn create_credential_store_empty_config_rejects_all() {
        let cfg = make_config(None);
        let store = create_credential_store(&cfg);
        assert!(!store.validate_api_key("any").unwrap());
    }

    #[test]
    fn keys_constants_are_correct() {
        assert_eq!(keys::API_KEY, "api_key");
        assert_eq!(keys::API_KEYS, "api_keys");
    }
}

/// Create the appropriate `CredentialStore` from runtime config.
pub fn create_credential_store(config: &AppConfig) -> Arc<dyn CredentialStore> {
    let raw = config.api.api_keys.clone().unwrap_or_default();
    let keys: Vec<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    Arc::new(InMemoryCredentialStore::new(keys))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiConfig, AppConfig, CoreConfig};

    fn make_config(keys: Option<&str>) -> AppConfig {
        AppConfig {
            core: CoreConfig {
                database_path: "test.db".into(),
            },
            api: ApiConfig {
                port: 3030,
                api_keys: keys.map(String::from),
            },
        }
    }

    #[test]
    fn in_memory_store_validates_correct_key() {
        let store = InMemoryCredentialStore::new(vec!["key1".to_string(), "key2".to_string()]);
        assert!(store.validate_api_key("key1").expect("domain operation"));
        assert!(store.validate_api_key("key2").expect("domain operation"));
    }

    #[test]
    fn in_memory_store_rejects_wrong_key() {
        let store = InMemoryCredentialStore::new(vec!["valid-key".to_string()]);
        assert!(!store
            .validate_api_key("invalid-key")
            .expect("domain operation"));
        assert!(!store.validate_api_key("").expect("domain operation"));
    }

    #[test]
    fn in_memory_store_get_returns_empty() {
        let store = InMemoryCredentialStore::new(vec![]);
        assert_eq!(store.get("ns", "k").expect("domain operation"), "");
    }

    #[test]
    fn create_credential_store_parses_comma_separated_keys() {
        let cfg = make_config(Some("key-a, key-b , key-c"));
        let store = create_credential_store(&cfg);
        assert!(store.validate_api_key("key-a").expect("domain operation"));
        assert!(store.validate_api_key("key-b").expect("domain operation"));
        assert!(store.validate_api_key("key-c").expect("domain operation"));
        assert!(!store.validate_api_key("key-d").expect("domain operation"));
    }

    #[test]
    fn create_credential_store_empty_config_rejects_all() {
        let cfg = make_config(None);
        let store = create_credential_store(&cfg);
        assert!(!store.validate_api_key("any").expect("domain operation"));
    }

    #[test]
    fn keys_constants_are_correct() {
        assert_eq!(keys::API_KEY, "api_key");
        assert_eq!(keys::API_KEYS, "api_keys");
    }
}
