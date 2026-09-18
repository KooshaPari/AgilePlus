use super::error::CredentialError;
use super::keys;
use sha2::{Digest, Sha256};

const API_KEY_HASH_PREFIX: &str = "sha256:";

/// Port for storing and retrieving credentials.
///
/// All methods are synchronous -- implementations use blocking I/O only.
/// This keeps the trait dyn-compatible without requiring `async_trait`.
///
/// Implementations: `KeychainCredentialStore`, `FileCredentialStore`,
/// `InMemoryCredentialStore` (tests).
pub trait CredentialStore: Send + Sync {
    /// Retrieve a credential value.
    fn get(&self, service: &str, key: &str) -> Result<String, CredentialError>;

    /// Store a credential value.
    fn set(&self, service: &str, key: &str, value: &str) -> Result<(), CredentialError>;

    /// Delete a credential.
    fn delete(&self, service: &str, key: &str) -> Result<(), CredentialError>;

    /// List all stored keys for a service.
    fn list_keys(&self, service: &str) -> Result<Vec<String>, CredentialError>;

    /// Validate whether a raw API key matches any stored API key hash.
    ///
    /// Uses constant-time comparison to prevent timing attacks.
    fn validate_api_key(&self, provided_key: &str) -> Result<bool, CredentialError> {
        let stored = match self.get("agileplus", keys::API_KEYS) {
            Ok(v) => v,
            Err(CredentialError::NotFound(_)) => return Ok(false),
            Err(e) => return Err(e),
        };
        let provided_hash = format_api_key_hash(provided_key);
        if stored
            .split(',')
            .map(str::trim)
            .any(|key| !key.is_empty() && !key.starts_with(API_KEY_HASH_PREFIX))
        {
            return Err(CredentialError::LegacyPlaintextApiKey);
        }
        let valid = stored
            .split(',')
            .map(str::trim)
            .filter(|k| !k.is_empty())
            .any(|stored_key| constant_time_eq(provided_hash.as_bytes(), stored_key.as_bytes()));
        Ok(valid)
    }
}

/// Produce the canonical non-reversible representation of an API key.
pub fn format_api_key_hash(api_key: &str) -> String {
    let digest = Sha256::digest(api_key.as_bytes());
    let mut result = String::with_capacity(API_KEY_HASH_PREFIX.len() + digest.len() * 2);
    result.push_str(API_KEY_HASH_PREFIX);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut result, "{byte:02x}").expect("writing to a String cannot fail");
    }
    result
}

/// Constant-time byte comparison to prevent timing-based key extraction.
pub(crate) fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::InMemoryCredentialStore;

    /// A store whose reads always fail at the backend layer.
    struct FailingStore;

    impl CredentialStore for FailingStore {
        fn get(&self, _service: &str, _key: &str) -> Result<String, CredentialError> {
            Err(CredentialError::BackendError(
                "keychain unavailable".to_string(),
            ))
        }

        fn set(&self, _service: &str, _key: &str, _value: &str) -> Result<(), CredentialError> {
            Err(CredentialError::BackendError(
                "keychain unavailable".to_string(),
            ))
        }

        fn delete(&self, _service: &str, _key: &str) -> Result<(), CredentialError> {
            Err(CredentialError::BackendError(
                "keychain unavailable".to_string(),
            ))
        }

        fn list_keys(&self, _service: &str) -> Result<Vec<String>, CredentialError> {
            Err(CredentialError::BackendError(
                "keychain unavailable".to_string(),
            ))
        }
    }

    fn store_with_api_keys(value: &str) -> InMemoryCredentialStore {
        let store = InMemoryCredentialStore::new();
        store.set("agileplus", keys::API_KEYS, value).unwrap();
        store
    }

    #[test]
    fn format_api_key_hash_is_the_prefixed_sha256_digest() {
        let hashed = format_api_key_hash("secret");
        assert_eq!(
            hashed,
            "sha256:2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b"
        );
        let hex = hashed.strip_prefix(API_KEY_HASH_PREFIX).unwrap();
        assert_eq!(hex.len(), 64);
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
        );
        assert!(!hashed.contains("secret"));
    }

    #[test]
    fn format_api_key_hash_distinguishes_similar_inputs() {
        assert_ne!(format_api_key_hash("key"), format_api_key_hash("key "));
        assert_eq!(format_api_key_hash("key"), format_api_key_hash("key"));
        assert_eq!(format_api_key_hash("").len(), "sha256:".len() + 64);
    }

    #[test]
    fn validate_api_key_prefers_a_hash_but_rejects_legacy_plaintext() {
        let store = store_with_api_keys("legacy-plaintext-key");

        let result = store.validate_api_key("legacy-plaintext-key");

        assert!(matches!(
            result,
            Err(CredentialError::LegacyPlaintextApiKey)
        ));
        assert_eq!(
            CredentialError::LegacyPlaintextApiKey.to_string(),
            "legacy plaintext API key detected; rotate it with AGILEPLUS_API_KEY before starting"
        );
    }

    #[test]
    fn validate_api_key_rejects_a_list_holding_any_plaintext_entry() {
        let store = store_with_api_keys(&format!(
            "{}, rotated-plaintext",
            format_api_key_hash("good-key")
        ));

        assert!(matches!(
            store.validate_api_key("good-key"),
            Err(CredentialError::LegacyPlaintextApiKey)
        ));
    }

    #[test]
    fn validate_api_key_ignores_blank_entries_and_surrounding_whitespace() {
        let store = store_with_api_keys(&format!(
            "  , {},  {} , ",
            format_api_key_hash("key-a"),
            format_api_key_hash("key-b")
        ));

        assert!(store.validate_api_key("key-a").unwrap());
        assert!(store.validate_api_key("key-b").unwrap());
        assert!(!store.validate_api_key("key-c").unwrap());
    }

    #[test]
    fn validate_api_key_treats_an_empty_stored_list_as_no_keys() {
        let store = store_with_api_keys("");

        assert!(!store.validate_api_key("anything").unwrap());
    }

    #[test]
    fn validate_api_key_surfaces_backend_failures_instead_of_denying() {
        let result = FailingStore.validate_api_key("anything");

        assert!(matches!(
            result,
            Err(CredentialError::BackendError(ref message)) if message == "keychain unavailable"
        ));
    }
}
