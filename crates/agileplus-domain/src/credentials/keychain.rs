#[cfg(feature = "keychain")]
use super::error::CredentialError;
#[cfg(feature = "keychain")]
use super::store::CredentialStore;

/// Credential store backed by the OS keychain (macOS Keychain / Linux secret-service).
///
/// Requires the `keyring` crate feature to be enabled.
#[cfg(feature = "keychain")]
pub struct KeychainCredentialStore {
    service_prefix: String,
}

#[cfg(feature = "keychain")]
impl KeychainCredentialStore {
    pub fn new() -> Self {
        Self {
            service_prefix: "agileplus".to_string(),
        }
    }

    fn entry_service(&self, service: &str) -> String {
        format!("{}-{}", self.service_prefix, service)
    }
}

#[cfg(feature = "keychain")]
impl Default for KeychainCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "keychain")]
impl CredentialStore for KeychainCredentialStore {
    fn get(&self, service: &str, key: &str) -> Result<String, CredentialError> {
        let entry = keyring::Entry::new(&self.entry_service(service), key)
            .map_err(|e| CredentialError::BackendError(e.to_string()))?;
        entry.get_password().map_err(|e| match e {
            keyring::Error::NoEntry => CredentialError::NotFound(key.to_string()),
            other => CredentialError::BackendError(other.to_string()),
        })
    }

    fn set(&self, service: &str, key: &str, value: &str) -> Result<(), CredentialError> {
        let entry = keyring::Entry::new(&self.entry_service(service), key)
            .map_err(|e| CredentialError::BackendError(e.to_string()))?;
        entry
            .set_password(value)
            .map_err(|e| CredentialError::BackendError(e.to_string()))
    }

    fn delete(&self, service: &str, key: &str) -> Result<(), CredentialError> {
        let entry = keyring::Entry::new(&self.entry_service(service), key)
            .map_err(|e| CredentialError::BackendError(e.to_string()))?;
        entry.delete_credential().map_err(|e| match e {
            keyring::Error::NoEntry => CredentialError::NotFound(key.to_string()),
            other => CredentialError::BackendError(other.to_string()),
        })
    }

    fn list_keys(&self, _service: &str) -> Result<Vec<String>, CredentialError> {
        Ok(Vec::new())
    }
}

// These tests deliberately never call `get`/`set`/`delete`: those reach the
// real OS keychain of whatever machine runs the suite. Only the pure
// name-mapping behavior and the non-enumerable `list_keys` contract are
// asserted here.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_service_prefixes_the_requested_service() {
        let store = KeychainCredentialStore::new();

        assert_eq!(store.entry_service("github"), "agileplus-github");
        assert_eq!(store.entry_service("plane"), "agileplus-plane");
        assert_eq!(store.entry_service(""), "agileplus-");
    }

    #[test]
    fn distinct_services_never_collide() {
        let store = KeychainCredentialStore::new();

        assert_ne!(store.entry_service("a"), store.entry_service("b"));
        assert_ne!(store.entry_service("a-b"), store.entry_service("ab"));
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(
            KeychainCredentialStore::default().entry_service("github"),
            KeychainCredentialStore::new().entry_service("github")
        );
    }

    #[test]
    fn list_keys_does_not_enumerate_the_os_keychain() {
        let store = KeychainCredentialStore::new();

        // The OS keychain has no enumeration API, so the port reports an empty
        // list rather than erroring.
        assert!(store.list_keys("github").unwrap().is_empty());
    }
}
