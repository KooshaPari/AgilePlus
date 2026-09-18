use crate::config::{AppConfig, CredentialBackend};

use std::path::PathBuf;

use super::error::CredentialError;
use super::file::FileCredentialStore;
#[cfg(feature = "keychain")]
use super::keychain::KeychainCredentialStore;
use super::store::CredentialStore;

/// Create the appropriate credential store based on the app configuration.
pub fn create_credential_store(
    config: &AppConfig,
) -> Result<Box<dyn CredentialStore>, CredentialError> {
    match config.credentials.backend {
        #[cfg(feature = "keychain")]
        CredentialBackend::Keychain => Ok(Box::new(KeychainCredentialStore::new())),
        CredentialBackend::File => Ok(Box::new(FileCredentialStore::new(
            &config.credentials.file_path,
        )?)),
        CredentialBackend::Auto => {
            #[cfg(feature = "keychain")]
            {
                Ok(Box::new(KeychainThenEncryptedFile::new(
                    config.credentials.file_path.clone(),
                )))
            }
            #[cfg(not(feature = "keychain"))]
            {
                Ok(Box::new(FileCredentialStore::new(
                    &config.credentials.file_path,
                )?))
            }
        }
        #[cfg(not(feature = "keychain"))]
        CredentialBackend::Keychain => Err(CredentialError::BackendError(
            "keychain backend is not compiled into this build".to_string(),
        )),
    }
}

/// Uses the OS keychain whenever it works and lazily opens the encrypted file
/// only when the keychain is unavailable. The file cannot silently downgrade to
/// plaintext because opening it requires `AGILEPLUS_CREDENTIAL_KEY`.
#[cfg(feature = "keychain")]
struct KeychainThenEncryptedFile {
    keychain: KeychainCredentialStore,
    file_path: PathBuf,
}

#[cfg(feature = "keychain")]
impl KeychainThenEncryptedFile {
    fn new(file_path: PathBuf) -> Self {
        Self {
            keychain: KeychainCredentialStore::new(),
            file_path,
        }
    }

    fn with_fallback<T>(
        &self,
        operation: impl FnOnce(&FileCredentialStore) -> Result<T, CredentialError>,
    ) -> Result<T, CredentialError> {
        // Construct on demand so a healthy keychain never requires an
        // encryption key. The file store reloads its encrypted state on each
        // fallback operation, keeping this adapter stateless and fail-closed.
        operation(&FileCredentialStore::new(&self.file_path)?)
    }
}

#[cfg(feature = "keychain")]
impl CredentialStore for KeychainThenEncryptedFile {
    fn get(&self, service: &str, key: &str) -> Result<String, CredentialError> {
        self.keychain
            .get(service, key)
            .or_else(|error| match error {
                CredentialError::BackendError(_) => {
                    self.with_fallback(|file| file.get(service, key))
                }
                other => Err(other),
            })
    }

    fn set(&self, service: &str, key: &str, value: &str) -> Result<(), CredentialError> {
        self.keychain
            .set(service, key, value)
            .or_else(|error| match error {
                CredentialError::BackendError(_) => {
                    self.with_fallback(|file| file.set(service, key, value))
                }
                other => Err(other),
            })
    }

    fn delete(&self, service: &str, key: &str) -> Result<(), CredentialError> {
        self.keychain
            .delete(service, key)
            .or_else(|error| match error {
                CredentialError::BackendError(_) => {
                    self.with_fallback(|file| file.delete(service, key))
                }
                other => Err(other),
            })
    }

    fn list_keys(&self, service: &str) -> Result<Vec<String>, CredentialError> {
        self.keychain
            .list_keys(service)
            .or_else(|error| match error {
                CredentialError::BackendError(_) => {
                    self.with_fallback(|file| file.list_keys(service))
                }
                other => Err(other),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::keys;
    use crate::test_support::env_lock;
    use std::path::Path;

    /// Point `HOME` at `home` for the duration of a closure, restoring the
    /// previous value afterwards. The caller must hold [`env_lock`].
    fn with_home<T>(home: &Path, operation: impl FnOnce() -> T) -> T {
        let previous_home = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", home) };
        let result = operation();
        match previous_home {
            Some(value) => unsafe { std::env::set_var("HOME", value) },
            None => unsafe { std::env::remove_var("HOME") },
        }
        result
    }

    #[test]
    fn configured_file_backend_uses_exact_path_and_survives_reload() {
        let _guard = env_lock();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("configured.enc");
        let home = dir.path().join("home");
        std::fs::create_dir_all(home.join(".agileplus")).unwrap();
        std::fs::write(
            home.join(".agileplus/config.toml"),
            format!(
                "[credentials]\nbackend = \"file\"\nfile_path = \"{}\"\n",
                path.display()
            ),
        )
        .unwrap();
        let previous_key = std::env::var_os("AGILEPLUS_CREDENTIAL_KEY");
        let loaded = with_home(&home, || {
            unsafe { std::env::set_var("AGILEPLUS_CREDENTIAL_KEY", "test-key") };
            let config = AppConfig::load().unwrap();
            let store = create_credential_store(&config).unwrap();
            store
                .set("agileplus", keys::API_KEYS, "sha256:test")
                .unwrap();
            (
                store.get("agileplus", keys::API_KEYS).unwrap(),
                config.credentials.file_path.clone(),
            )
        });
        match previous_key {
            Some(value) => unsafe { std::env::set_var("AGILEPLUS_CREDENTIAL_KEY", value) },
            None => unsafe { std::env::remove_var("AGILEPLUS_CREDENTIAL_KEY") },
        }
        assert_eq!(loaded.0, "sha256:test");
        assert_eq!(loaded.1, path);
        assert!(path.is_file());
    }

    #[test]
    fn malformed_app_config_fails_closed() {
        let _guard = env_lock();
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir_all(home.join(".agileplus")).unwrap();
        std::fs::write(
            home.join(".agileplus/config.toml"),
            "[credentials\nbackend = \"file\"",
        )
        .unwrap();
        let error = with_home(&home, AppConfig::load).unwrap_err();
        assert!(matches!(error, crate::config::ConfigError::TomlParse(_)));
    }

    #[test]
    fn file_backend_without_encryption_key_fails_closed() {
        let _guard = env_lock();
        let dir = tempfile::tempdir().unwrap();
        let config = AppConfig {
            credentials: crate::config::CredentialConfig {
                backend: CredentialBackend::File,
                file_path: dir.path().join("creds.enc"),
            },
            ..AppConfig::default()
        };
        let previous_key = std::env::var_os("AGILEPLUS_CREDENTIAL_KEY");
        unsafe { std::env::remove_var("AGILEPLUS_CREDENTIAL_KEY") };
        let result = create_credential_store(&config);
        match previous_key {
            Some(value) => unsafe { std::env::set_var("AGILEPLUS_CREDENTIAL_KEY", value) },
            None => unsafe { std::env::remove_var("AGILEPLUS_CREDENTIAL_KEY") },
        }
        assert!(matches!(
            result.err(),
            Some(CredentialError::MissingEncryptionKey)
        ));
    }

    #[cfg(feature = "keychain")]
    #[test]
    fn auto_backend_prefers_keychain_without_touching_the_file() {
        let _guard = env_lock();
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("creds.enc");
        let config = AppConfig {
            credentials: crate::config::CredentialConfig {
                backend: CredentialBackend::Auto,
                file_path: file_path.clone(),
            },
            ..AppConfig::default()
        };
        let previous_key = std::env::var_os("AGILEPLUS_CREDENTIAL_KEY");
        unsafe { std::env::remove_var("AGILEPLUS_CREDENTIAL_KEY") };
        let result = create_credential_store(&config);
        match previous_key {
            Some(value) => unsafe { std::env::set_var("AGILEPLUS_CREDENTIAL_KEY", value) },
            None => unsafe { std::env::remove_var("AGILEPLUS_CREDENTIAL_KEY") },
        }
        // Construction must not need the encryption key: the encrypted file is
        // only opened lazily when the keychain is unavailable.
        assert!(result.is_ok());
        assert!(!file_path.exists());
    }
}
