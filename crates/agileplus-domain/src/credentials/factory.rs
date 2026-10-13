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
    keychain: Box<dyn CredentialStore>,
    file_path: PathBuf,
}

#[cfg(feature = "keychain")]
impl KeychainThenEncryptedFile {
    fn new(file_path: PathBuf) -> Self {
        Self::with_keychain(file_path, Box::new(KeychainCredentialStore::new()))
    }

    /// Test-only constructor that lets a test inject a keychain stand-in
    /// (typically one that always returns [`CredentialError::BackendError`] to
    /// force the file-fallback branch). Production code uses [`Self::new`].
    #[cfg(test)]
    fn with_keychain_for_test(file_path: PathBuf, keychain: Box<dyn CredentialStore>) -> Self {
        Self::with_keychain(file_path, keychain)
    }

    fn with_keychain(file_path: PathBuf, keychain: Box<dyn CredentialStore>) -> Self {
        Self {
            keychain,
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

    /// Mock keychain that always returns [`CredentialError::BackendError`].
    /// Used to force the file-fallback branch in the auto backend. Without a
    /// stand-in like this, mutation testing cannot tell that the fallback
    /// delegation differs from a trivial `Ok(...)` stub, because no test
    /// actually calls the fallback path on a healthy keychain.
    #[cfg(feature = "keychain")]
    struct AlwaysBackendErrorKeychain;

    #[cfg(feature = "keychain")]
    impl CredentialStore for AlwaysBackendErrorKeychain {
        fn get(&self, _service: &str, _key: &str) -> Result<String, CredentialError> {
            Err(CredentialError::BackendError(
                "synthetic keychain failure".to_string(),
            ))
        }

        fn set(&self, _service: &str, _key: &str, _value: &str) -> Result<(), CredentialError> {
            Err(CredentialError::BackendError(
                "synthetic keychain failure".to_string(),
            ))
        }

        fn delete(&self, _service: &str, _key: &str) -> Result<(), CredentialError> {
            Err(CredentialError::BackendError(
                "synthetic keychain failure".to_string(),
            ))
        }

        fn list_keys(&self, _service: &str) -> Result<Vec<String>, CredentialError> {
            Err(CredentialError::BackendError(
                "synthetic keychain failure".to_string(),
            ))
        }
    }

    /// Mock keychain that always returns [`CredentialError::NotFound`]. This is
    /// the non-fallback error category: it must propagate through the auto
    /// backend instead of being absorbed by the file fallback.
    #[cfg(feature = "keychain")]
    struct AlwaysNotFoundKeychain;

    #[cfg(feature = "keychain")]
    impl CredentialStore for AlwaysNotFoundKeychain {
        fn get(&self, _service: &str, _key: &str) -> Result<String, CredentialError> {
            Err(CredentialError::NotFound("missing".to_string()))
        }

        fn set(&self, _service: &str, _key: &str, _value: &str) -> Result<(), CredentialError> {
            Err(CredentialError::NotFound("missing".to_string()))
        }

        fn delete(&self, _service: &str, _key: &str) -> Result<(), CredentialError> {
            Err(CredentialError::NotFound("missing".to_string()))
        }

        fn list_keys(&self, _service: &str) -> Result<Vec<String>, CredentialError> {
            Err(CredentialError::NotFound("missing".to_string()))
        }
    }

    /// Run `body` with [`AGILEPLUS_CREDENTIAL_KEY`] set, restoring the
    /// previous value (or absence) afterwards. The caller must hold
    /// [`env_lock`] for the full duration of `body` because this function
    /// mutates process-global state. `body` runs INSIDE the env-var scope
    /// so that lazy file-fallback opens (which read the env var) see the
    /// test value, not the real one.
    #[cfg(feature = "keychain")]
    fn with_credential_key<T>(key: &str, body: impl FnOnce() -> T) -> T {
        let previous_key = std::env::var_os("AGILEPLUS_CREDENTIAL_KEY");
        unsafe { std::env::set_var("AGILEPLUS_CREDENTIAL_KEY", key) };
        let result = body();
        match previous_key {
            Some(value) => unsafe { std::env::set_var("AGILEPLUS_CREDENTIAL_KEY", value) },
            None => unsafe { std::env::remove_var("AGILEPLUS_CREDENTIAL_KEY") },
        }
        result
    }

    #[cfg(feature = "keychain")]
    #[test]
    fn auto_backend_get_falls_back_to_file_when_keychain_backend_errors() {
        let _guard = env_lock();
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("creds.enc");
        with_credential_key("test-key", || {
            let store = KeychainThenEncryptedFile::with_keychain_for_test(
                file_path.clone(),
                Box::new(AlwaysBackendErrorKeychain),
            );

            // Prime the file fallback path by writing through the adapter. The
            // keychain stand-in errors, so this must land in the encrypted file.
            store
                .set("agileplus", keys::API_KEYS, "fallback-secret")
                .expect("file fallback should absorb the BackendError from the keychain");
            let read = store
                .get("agileplus", keys::API_KEYS)
                .expect("file fallback should return the stored value");
            assert_eq!(read, "fallback-secret");
        });
    }

    #[cfg(feature = "keychain")]
    #[test]
    fn auto_backend_set_falls_back_to_file_when_keychain_backend_errors() {
        let _guard = env_lock();
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("creds.enc");
        with_credential_key("test-key", || {
            let store = KeychainThenEncryptedFile::with_keychain_for_test(
                file_path.clone(),
                Box::new(AlwaysBackendErrorKeychain),
            );

            // A naive mutation replaces the body with `Ok(())` -- that mutation
            // survives only if no test observes the side effect. We do observe
            // the side effect: the encrypted file must exist on disk.
            store
                .set("agileplus", keys::API_KEYS, "value-that-must-persist")
                .unwrap();
            assert!(
                file_path.is_file(),
                "set should have written the encrypted file"
            );
        });
    }

    #[cfg(feature = "keychain")]
    #[test]
    fn auto_backend_delete_falls_back_to_file_when_keychain_backend_errors() {
        let _guard = env_lock();
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("creds.enc");
        with_credential_key("test-key", || {
            let store = KeychainThenEncryptedFile::with_keychain_for_test(
                file_path.clone(),
                Box::new(AlwaysBackendErrorKeychain),
            );

            // First write through the fallback so the encrypted file has a
            // value to delete, then assert the delete is observable by a fresh
            // read returning `NotFound`.
            store
                .set("agileplus", keys::API_KEYS, "to-be-deleted")
                .unwrap();
            store
                .delete("agileplus", keys::API_KEYS)
                .expect("file fallback should absorb the BackendError from the keychain");
            let after = store.get("agileplus", keys::API_KEYS);
            assert!(
                matches!(after, Err(CredentialError::NotFound(_))),
                "delete should remove the entry from the file fallback (got {:?})",
                after
            );
        });
    }

    #[cfg(feature = "keychain")]
    #[test]
    fn auto_backend_list_keys_falls_back_to_file_when_keychain_backend_errors() {
        let _guard = env_lock();
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("creds.enc");
        with_credential_key("test-key", || {
            let store = KeychainThenEncryptedFile::with_keychain_for_test(
                file_path.clone(),
                Box::new(AlwaysBackendErrorKeychain),
            );

            store.set("agileplus", "alpha", "value-alpha").unwrap();
            store.set("agileplus", "beta", "value-beta").unwrap();
            let mut keys = store
                .list_keys("agileplus")
                .expect("file fallback should absorb the BackendError from the keychain");
            keys.sort();
            assert_eq!(keys, vec!["alpha".to_string(), "beta".to_string()]);
        });
    }

    #[cfg(feature = "keychain")]
    #[test]
    fn auto_backend_does_not_fall_back_on_non_backend_errors() {
        let _guard = env_lock();
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("creds.enc");
        with_credential_key("test-key", || {
            let store = KeychainThenEncryptedFile::with_keychain_for_test(
                file_path.clone(),
                Box::new(AlwaysNotFoundKeychain),
            );

            // The keychain stand-in returns NotFound, which is *not* a
            // BackendError. The auto backend must propagate that error instead
            // of silently opening the file fallback.
            let get_err = store.get("agileplus", keys::API_KEYS).unwrap_err();
            assert!(
                matches!(get_err, CredentialError::NotFound(_)),
                "NotFound must propagate, not fall back (got {:?})",
                get_err
            );
            let set_err = store
                .set("agileplus", keys::API_KEYS, "ignored")
                .unwrap_err();
            assert!(
                matches!(set_err, CredentialError::NotFound(_)),
                "NotFound must propagate on set, not fall back (got {:?})",
                set_err
            );
            let delete_err = store.delete("agileplus", keys::API_KEYS).unwrap_err();
            assert!(
                matches!(delete_err, CredentialError::NotFound(_)),
                "NotFound must propagate on delete, not fall back (got {:?})",
                delete_err
            );
            let list_err = store.list_keys("agileplus").unwrap_err();
            assert!(
                matches!(list_err, CredentialError::NotFound(_)),
                "NotFound must propagate on list_keys, not fall back (got {:?})",
                list_err
            );
            // And the encrypted file must remain untouched.
            assert!(!file_path.exists());
        });
    }
}
