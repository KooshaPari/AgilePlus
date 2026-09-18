use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD_NO_PAD, URL_SAFE_NO_PAD},
};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use super::error::CredentialError;
use super::store::CredentialStore;

const ENVELOPE_VERSION: u8 = 1;
const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const KEY_LENGTH: usize = 32;

/// Credential store backed by an AES-256-GCM encrypted JSON envelope.
///
/// The envelope is stored at `~/.agileplus/credentials.enc`. Its encryption
/// key is derived from `AGILEPLUS_CREDENTIAL_KEY` with Argon2id. File-backed
/// credential access is unavailable without that key; there is no plaintext
/// compatibility format or fallback.
pub struct FileCredentialStore {
    path: PathBuf,
    passphrase: Zeroizing<String>,
    cache: RwLock<HashMap<String, HashMap<String, String>>>,
    loaded: RwLock<bool>,
}

#[derive(Deserialize, Serialize)]
struct EncryptedEnvelope {
    version: u8,
    salt: String,
    nonce: String,
    ciphertext: String,
}

impl FileCredentialStore {
    /// Construct a store from the process credential key. This is intentionally
    /// fallible so startup fails closed when file storage has no encryption key.
    pub fn new(path: &Path) -> Result<Self, CredentialError> {
        let passphrase = std::env::var("AGILEPLUS_CREDENTIAL_KEY")
            .map_err(|_| CredentialError::MissingEncryptionKey)?;
        Self::with_passphrase(path, passphrase)
    }

    /// Construct a store with an explicit key, primarily for dependency
    /// injection and deterministic tests. Callers must keep it secret.
    pub fn with_passphrase(path: &Path, passphrase: String) -> Result<Self, CredentialError> {
        if passphrase.is_empty() {
            return Err(CredentialError::MissingEncryptionKey);
        }
        Ok(Self {
            path: path.to_owned(),
            passphrase: Zeroizing::new(passphrase),
            cache: RwLock::new(HashMap::new()),
            loaded: RwLock::new(false),
        })
    }

    fn derive_key(&self, salt: &[u8]) -> Result<Zeroizing<[u8; KEY_LENGTH]>, CredentialError> {
        if salt.len() != SALT_LENGTH {
            return Err(CredentialError::Encryption(
                "invalid credential salt length".to_string(),
            ));
        }
        let params = Params::new(19_456, 2, 1, Some(KEY_LENGTH))
            .map_err(|error| CredentialError::Encryption(error.to_string()))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut key = Zeroizing::new([0u8; KEY_LENGTH]);
        argon2
            .hash_password_into(self.passphrase.as_bytes(), salt, key.as_mut())
            .map_err(|error| CredentialError::Encryption(error.to_string()))?;
        Ok(key)
    }

    fn decrypt(
        &self,
        raw: &[u8],
    ) -> Result<HashMap<String, HashMap<String, String>>, CredentialError> {
        let envelope: EncryptedEnvelope = serde_json::from_slice(raw)
            .map_err(|error| CredentialError::Serialization(error.to_string()))?;
        if envelope.version != ENVELOPE_VERSION {
            return Err(CredentialError::Encryption(format!(
                "unsupported credential envelope version {}",
                envelope.version
            )));
        }
        let salt = decode_field("salt", &envelope.salt, SALT_LENGTH)?;
        let nonce = decode_field("nonce", &envelope.nonce, NONCE_LENGTH)?;
        let ciphertext = STANDARD_NO_PAD
            .decode(envelope.ciphertext)
            .map_err(|error| CredentialError::Encryption(error.to_string()))?;
        let key = self.derive_key(&salt)?;
        let cipher = Aes256Gcm::new_from_slice(key.as_ref())
            .map_err(|error| CredentialError::Encryption(error.to_string()))?;
        let nonce = Nonce::try_from(nonce.as_slice())
            .map_err(|_| CredentialError::Encryption("invalid credential nonce".to_string()))?;
        let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref()).map_err(|_| {
            CredentialError::Encryption("credential file authentication failed".to_string())
        })?;
        serde_json::from_slice(&plaintext)
            .map_err(|error| CredentialError::Serialization(error.to_string()))
    }

    fn encrypt(
        &self,
        credentials: &HashMap<String, HashMap<String, String>>,
    ) -> Result<Vec<u8>, CredentialError> {
        let plaintext = serde_json::to_vec(credentials)
            .map_err(|error| CredentialError::Serialization(error.to_string()))?;
        let mut salt = [0u8; SALT_LENGTH];
        let mut nonce = [0u8; NONCE_LENGTH];
        rand::fill(&mut salt);
        rand::fill(&mut nonce);
        let key = self.derive_key(&salt)?;
        let cipher = Aes256Gcm::new_from_slice(key.as_ref())
            .map_err(|error| CredentialError::Encryption(error.to_string()))?;
        let nonce = Nonce::try_from(nonce.as_slice())
            .map_err(|_| CredentialError::Encryption("invalid credential nonce".to_string()))?;
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|_| CredentialError::Encryption("credential encryption failed".to_string()))?;
        let envelope = EncryptedEnvelope {
            version: ENVELOPE_VERSION,
            salt: STANDARD_NO_PAD.encode(salt),
            nonce: STANDARD_NO_PAD.encode(nonce),
            ciphertext: STANDARD_NO_PAD.encode(ciphertext),
        };
        serde_json::to_vec(&envelope)
            .map_err(|error| CredentialError::Serialization(error.to_string()))
    }

    fn ensure_loaded(&self) -> Result<(), CredentialError> {
        if *self.loaded.read().expect("credential loaded lock poisoned") {
            return Ok(());
        }
        let mut loaded = self
            .loaded
            .write()
            .expect("credential loaded lock poisoned");
        if *loaded {
            return Ok(());
        }
        if self.path.exists() {
            let raw = std::fs::read(&self.path)?;
            *self.cache.write().expect("credential cache lock poisoned") = self.decrypt(&raw)?;
        }
        *loaded = true;
        Ok(())
    }

    fn persist_candidate(
        &self,
        credentials: &HashMap<String, HashMap<String, String>>,
    ) -> Result<(), CredentialError> {
        let parent = self.path.parent().ok_or_else(|| {
            CredentialError::Io(std::io::Error::other("credential file path has no parent"))
        })?;
        std::fs::create_dir_all(parent)?;
        let encrypted = self.encrypt(credentials)?;
        let mut suffix = [0u8; 16];
        rand::fill(&mut suffix);
        let temporary = temporary_path(parent, &suffix);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        use std::io::Write as _;
        file.write_all(&encrypted)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))?;
        }
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temporary, &self.path)?;
        #[cfg(unix)]
        std::fs::File::open(parent)?.sync_all()?;
        Ok(())
    }
}

/// Generate a single path component for the atomic-write staging file.
///
/// The random suffix must use the URL-safe alphabet: standard Base64 permits
/// `/`, which would accidentally create a nested path and break atomic writes.
fn temporary_path(parent: &Path, suffix: &[u8; 16]) -> PathBuf {
    parent.join(format!(
        ".credentials-{}.tmp",
        URL_SAFE_NO_PAD.encode(suffix)
    ))
}

fn decode_field(
    field: &str,
    encoded: &str,
    expected_length: usize,
) -> Result<Vec<u8>, CredentialError> {
    let decoded = STANDARD_NO_PAD
        .decode(encoded)
        .map_err(|error| CredentialError::Encryption(format!("invalid {field}: {error}")))?;
    if decoded.len() != expected_length {
        return Err(CredentialError::Encryption(format!(
            "invalid {field} length"
        )));
    }
    Ok(decoded)
}

impl CredentialStore for FileCredentialStore {
    fn get(&self, service: &str, key: &str) -> Result<String, CredentialError> {
        self.ensure_loaded()?;
        self.cache
            .read()
            .expect("credential cache lock poisoned")
            .get(service)
            .and_then(|service| service.get(key))
            .cloned()
            .ok_or_else(|| CredentialError::NotFound(key.to_string()))
    }

    fn set(&self, service: &str, key: &str, value: &str) -> Result<(), CredentialError> {
        self.ensure_loaded()?;
        let mut candidate = self
            .cache
            .read()
            .expect("credential cache lock poisoned")
            .clone();
        candidate
            .entry(service.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
        self.persist_candidate(&candidate)?;
        *self.cache.write().expect("credential cache lock poisoned") = candidate;
        Ok(())
    }

    fn delete(&self, service: &str, key: &str) -> Result<(), CredentialError> {
        self.ensure_loaded()?;
        let mut candidate = self
            .cache
            .read()
            .expect("credential cache lock poisoned")
            .clone();
        let removed = candidate
            .get_mut(service)
            .and_then(|service| service.remove(key));
        if removed.is_none() {
            return Err(CredentialError::NotFound(key.to_string()));
        }
        self.persist_candidate(&candidate)?;
        *self.cache.write().expect("credential cache lock poisoned") = candidate;
        Ok(())
    }

    fn list_keys(&self, service: &str) -> Result<Vec<String>, CredentialError> {
        self.ensure_loaded()?;
        Ok(self
            .cache
            .read()
            .expect("credential cache lock poisoned")
            .get(service)
            .map(|service| service.keys().cloned().collect())
            .unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypted_file_does_not_contain_credential_plaintext() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        let store =
            FileCredentialStore::with_passphrase(&path, "test encryption key".to_string()).unwrap();
        store.set("plane", "api-key", "super-secret-value").unwrap();
        let raw = std::fs::read_to_string(path).unwrap();
        assert!(!raw.contains("super-secret-value"));
        assert_eq!(store.get("plane", "api-key").unwrap(), "super-secret-value");
    }

    #[test]
    fn encrypted_file_reopens_with_same_key_and_rejects_wrong_key() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        FileCredentialStore::with_passphrase(&path, "correct key".to_string())
            .unwrap()
            .set("plane", "api-key", "secret")
            .unwrap();
        let reopened =
            FileCredentialStore::with_passphrase(&path, "correct key".to_string()).unwrap();
        assert_eq!(reopened.get("plane", "api-key").unwrap(), "secret");
        let wrong_key =
            FileCredentialStore::with_passphrase(&path, "wrong key".to_string()).unwrap();
        assert!(matches!(
            wrong_key.get("plane", "api-key"),
            Err(CredentialError::Encryption(_))
        ));
    }

    #[test]
    fn missing_file_key_fails_closed() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        assert!(matches!(
            FileCredentialStore::with_passphrase(&path, String::new()),
            Err(CredentialError::MissingEncryptionKey)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn atomic_persist_leaves_no_temp_file_and_restricts_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        let store =
            FileCredentialStore::with_passphrase(&path, "test encryption key".to_string()).unwrap();
        store.set("plane", "api-key", "secret").unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let temp_count = std::fs::read_dir(directory.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".credentials-")
            })
            .count();
        assert_eq!(temp_count, 0);
    }

    #[test]
    fn atomic_temp_path_never_contains_a_separator_from_random_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let path = temporary_path(directory.path(), &[0xff; 16]);
        assert_eq!(path.parent(), Some(directory.path()));
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(name.starts_with(".credentials-"));
        assert!(name.ends_with(".tmp"));
        assert!(!name.contains(['/', '\\']));
    }

    #[test]
    fn failed_set_keeps_previous_cached_value() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        let store = FileCredentialStore::with_passphrase(&path, "test key".to_string()).unwrap();
        store.set("plane", "api-key", "old").unwrap();
        std::fs::remove_dir_all(directory.path()).unwrap();
        std::fs::write(directory.path(), "not a directory").unwrap();

        assert!(store.set("plane", "api-key", "new").is_err());
        assert_eq!(store.get("plane", "api-key").unwrap(), "old");
    }

    #[test]
    fn failed_delete_keeps_previous_cached_value() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        let store = FileCredentialStore::with_passphrase(&path, "test key".to_string()).unwrap();
        store.set("plane", "api-key", "old").unwrap();
        std::fs::remove_dir_all(directory.path()).unwrap();
        std::fs::write(directory.path(), "not a directory").unwrap();

        assert!(store.delete("plane", "api-key").is_err());
        assert_eq!(store.get("plane", "api-key").unwrap(), "old");
    }

    /// Rebuild a structurally valid envelope, then hand back the JSON so a test
    /// can tamper with exactly one field.
    fn tampered_envelope(
        store: &FileCredentialStore,
        mutate: impl FnOnce(&mut serde_json::Value),
    ) -> Vec<u8> {
        let raw = store.encrypt(&HashMap::new()).unwrap();
        let mut envelope: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        mutate(&mut envelope);
        serde_json::to_vec(&envelope).unwrap()
    }

    /// One named mutation applied to an otherwise valid envelope.
    type EnvelopeTamper = (&'static str, Box<dyn FnOnce(&mut serde_json::Value)>);

    #[test]
    fn new_reads_the_process_encryption_key() {
        let _guard = crate::test_support::env_lock();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        let previous = std::env::var_os("AGILEPLUS_CREDENTIAL_KEY");

        unsafe { std::env::remove_var("AGILEPLUS_CREDENTIAL_KEY") };
        let missing = FileCredentialStore::new(&path);
        assert!(matches!(
            missing,
            Err(CredentialError::MissingEncryptionKey)
        ));

        unsafe { std::env::set_var("AGILEPLUS_CREDENTIAL_KEY", "process-key") };
        let from_env = FileCredentialStore::new(&path).unwrap();
        from_env.set("plane", "api-key", "value").unwrap();
        assert_eq!(from_env.get("plane", "api-key").unwrap(), "value");
        // The file is unreadable without the same process key.
        let reopened =
            FileCredentialStore::with_passphrase(&path, "other-key".to_string()).unwrap();
        assert!(matches!(
            reopened.get("plane", "api-key"),
            Err(CredentialError::Encryption(_))
        ));

        match previous {
            Some(value) => unsafe { std::env::set_var("AGILEPLUS_CREDENTIAL_KEY", value) },
            None => unsafe { std::env::remove_var("AGILEPLUS_CREDENTIAL_KEY") },
        }
    }

    #[test]
    fn set_creates_missing_parent_directories() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory
            .path()
            .join("nested")
            .join("deeper")
            .join("credentials.enc");
        let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();

        store.set("plane", "api-key", "value").unwrap();

        assert!(path.is_file());
        assert_eq!(store.get("plane", "api-key").unwrap(), "value");
    }

    #[test]
    fn get_and_delete_report_not_found_for_unknown_keys() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();

        // Unknown service: nothing was ever persisted, so no file is created.
        assert!(matches!(
            store.get("plane", "api-key"),
            Err(CredentialError::NotFound(ref key)) if key == "api-key"
        ));
        assert!(!path.exists());

        store.set("plane", "api-key", "value").unwrap();
        assert!(matches!(
            store.get("plane", "other-key"),
            Err(CredentialError::NotFound(ref key)) if key == "other-key"
        ));
        store.delete("plane", "api-key").unwrap();
        assert!(matches!(
            store.delete("plane", "api-key"),
            Err(CredentialError::NotFound(ref key)) if key == "api-key"
        ));
    }

    #[test]
    fn list_keys_is_empty_for_unknown_services_and_lists_stored_keys() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();

        assert!(store.list_keys("plane").unwrap().is_empty());
        store.set("plane", "b-key", "1").unwrap();
        store.set("plane", "a-key", "2").unwrap();
        store.set("github", "token", "3").unwrap();
        let mut keys = store.list_keys("plane").unwrap();
        keys.sort();
        assert_eq!(keys, vec!["a-key".to_string(), "b-key".to_string()]);
        assert_eq!(store.list_keys("github").unwrap().len(), 1);
    }

    #[test]
    fn derive_key_rejects_a_salt_of_the_wrong_length() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();

        for salt_length in [0usize, 8, 15, 17, 32] {
            let salt = vec![1u8; salt_length];
            assert!(
                matches!(
                    store.derive_key(&salt),
                    Err(CredentialError::Encryption(ref message))
                        if message == "invalid credential salt length"
                ),
                "salt length {salt_length} should be rejected"
            );
        }
        assert!(store.derive_key(&[1u8; SALT_LENGTH]).is_ok());
    }

    #[test]
    fn reading_rejects_an_unsupported_envelope_version() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();
        std::fs::write(
            &path,
            tampered_envelope(&store, |envelope| {
                envelope["version"] = serde_json::json!(2);
            }),
        )
        .unwrap();

        let reopened = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();
        assert!(matches!(
            reopened.get("plane", "api-key"),
            Err(CredentialError::Encryption(ref message))
                if message == "unsupported credential envelope version 2"
        ));
    }

    #[test]
    fn reading_rejects_malformed_envelope_fields() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();

        let cases: Vec<EnvelopeTamper> = vec![
            (
                "non-base64 salt",
                Box::new(|envelope: &mut serde_json::Value| {
                    envelope["salt"] = serde_json::json!("not base64!!");
                }),
            ),
            (
                "short salt",
                Box::new(|envelope: &mut serde_json::Value| {
                    envelope["salt"] = serde_json::json!(STANDARD_NO_PAD.encode([0u8; 8]));
                }),
            ),
            (
                "non-base64 nonce",
                Box::new(|envelope: &mut serde_json::Value| {
                    envelope["nonce"] = serde_json::json!("!");
                }),
            ),
            (
                "short nonce",
                Box::new(|envelope: &mut serde_json::Value| {
                    envelope["nonce"] = serde_json::json!(STANDARD_NO_PAD.encode([0u8; 4]));
                }),
            ),
            (
                "non-base64 ciphertext",
                Box::new(|envelope: &mut serde_json::Value| {
                    envelope["ciphertext"] = serde_json::json!("!!!");
                }),
            ),
            (
                "tampered ciphertext",
                Box::new(|envelope: &mut serde_json::Value| {
                    envelope["ciphertext"] = serde_json::json!(STANDARD_NO_PAD.encode([7u8; 32]));
                }),
            ),
        ];

        for (name, mutate) in cases {
            std::fs::write(&path, tampered_envelope(&store, mutate)).unwrap();
            let reopened = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();
            let result = reopened.get("plane", "api-key");
            assert!(
                matches!(result, Err(CredentialError::Encryption(_))),
                "{name} should fail closed, got {result:?}"
            );
        }
    }

    #[test]
    fn reading_rejects_a_file_that_is_not_an_envelope() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        std::fs::write(&path, b"definitely not json").unwrap();
        let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();

        assert!(matches!(
            store.get("plane", "api-key"),
            Err(CredentialError::Serialization(_))
        ));
    }

    #[test]
    fn persist_rejects_a_path_without_a_parent_directory() {
        let store =
            FileCredentialStore::with_passphrase(Path::new("/"), "key".to_string()).unwrap();

        let error = store.persist_candidate(&HashMap::new()).unwrap_err();
        assert!(matches!(
            error,
            CredentialError::Io(ref io) if io.to_string() == "credential file path has no parent"
        ));
    }

    #[test]
    fn encrypting_the_same_state_twice_produces_different_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();
        let mut state: HashMap<String, HashMap<String, String>> = HashMap::new();
        state.insert("plane".to_string(), HashMap::new());

        let first = store.encrypt(&state).unwrap();
        let second = store.encrypt(&state).unwrap();

        assert_ne!(first, second, "a fresh salt/nonce must be used per write");
        assert_eq!(store.decrypt(&first).unwrap(), state);
        assert_eq!(store.decrypt(&second).unwrap(), state);
    }
}
