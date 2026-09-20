//! Integration contract tests for the AES-256-GCM encrypted file credential
//! store (`agileplus_domain::credentials::file`).
//!
//! This store is the on-disk fallback when no OS keychain is available, so its
//! guarantees are security-critical: the file must never contain plaintext, it
//! must fail closed without `AGILEPLUS_CREDENTIAL_KEY`, a wrong passphrase must
//! be detected (not silently produce garbage), writes must be atomic, and the
//! `0600` mode must be applied before the envelope replaces the old file.
//!
//! Tests that read or mutate `AGILEPLUS_CREDENTIAL_KEY` hold [`env_lock`] for the
//! whole window in which the value matters, because environment variables are
//! process-global and integration tests in one file share a process.
//!
//! Traceability: FR-030, FR-031 / WP15-T088

use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use agileplus_domain::credentials::{CredentialError, CredentialStore, FileCredentialStore};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD_NO_PAD;

// ---------------------------------------------------------------------------
// Shared environment lock
// ---------------------------------------------------------------------------

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Serialize tests that mutate process-wide environment variables.
fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Run `body` with `AGILEPLUS_CREDENTIAL_KEY` set to `value`, restoring the
/// previous value afterwards even if `body` panics.
fn with_credential_key<T>(value: Option<&str>, body: impl FnOnce() -> T) -> T {
    let previous = std::env::var_os("AGILEPLUS_CREDENTIAL_KEY");
    match value {
        Some(key) => unsafe { std::env::set_var("AGILEPLUS_CREDENTIAL_KEY", key) },
        None => unsafe { std::env::remove_var("AGILEPLUS_CREDENTIAL_KEY") },
    }
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
    match previous {
        Some(previous) => unsafe { std::env::set_var("AGILEPLUS_CREDENTIAL_KEY", previous) },
        None => unsafe { std::env::remove_var("AGILEPLUS_CREDENTIAL_KEY") },
    }
    match outcome {
        Ok(value) => value,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

#[test]
fn construction_fails_closed_when_the_process_key_is_absent() {
    let _guard = env_lock();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");

    let missing = with_credential_key(None, || FileCredentialStore::new(&path));
    assert!(matches!(
        missing,
        Err(CredentialError::MissingEncryptionKey)
    ));

    // An empty key is as good as absent: it is rejected without touching disk.
    let empty = with_credential_key(Some(""), || FileCredentialStore::new(&path));
    assert!(matches!(empty, Err(CredentialError::MissingEncryptionKey)));

    // A non-empty process key constructs a usable store.
    with_credential_key(Some("process-key"), || {
        let store = FileCredentialStore::new(&path).unwrap();
        store.set("plane", "api-key", "value").unwrap();
        assert_eq!(store.get("plane", "api-key").unwrap(), "value");
    });
    assert!(path.is_file(), "the store writes the envelope on first set");
}

#[test]
fn the_process_key_and_an_explicit_passphrase_are_interchangeable() {
    let _guard = env_lock();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");

    with_credential_key(Some("shared-secret"), || {
        FileCredentialStore::new(&path)
            .unwrap()
            .set("github", "token", "ghp_example")
            .unwrap();
    });

    let reopened =
        FileCredentialStore::with_passphrase(&path, "shared-secret".to_string()).unwrap();
    assert_eq!(reopened.get("github", "token").unwrap(), "ghp_example");
}

#[test]
fn an_empty_explicit_passphrase_is_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");
    assert!(matches!(
        FileCredentialStore::with_passphrase(&path, String::new()),
        Err(CredentialError::MissingEncryptionKey)
    ));
}

// ---------------------------------------------------------------------------
// Round trips and isolation
// ---------------------------------------------------------------------------

#[test]
fn values_round_trip_per_service_and_are_isolated_between_services() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");
    let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();

    store.set("plane", "api-key", "plane-secret").unwrap();
    store.set("github", "token", "github-secret").unwrap();
    store.set("plane", "other-key", "other-secret").unwrap();

    assert_eq!(store.get("plane", "api-key").unwrap(), "plane-secret");
    assert_eq!(store.get("plane", "other-key").unwrap(), "other-secret");
    assert_eq!(store.get("github", "token").unwrap(), "github-secret");

    // Overwriting a key keeps the rest.
    store.set("plane", "api-key", "rotated").unwrap();
    assert_eq!(store.get("plane", "api-key").unwrap(), "rotated");
    assert_eq!(store.get("plane", "other-key").unwrap(), "other-secret");
    assert_eq!(store.get("github", "token").unwrap(), "github-secret");
}

#[test]
fn a_surviving_file_reopens_with_the_same_key_and_rejects_a_wrong_one() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");
    FileCredentialStore::with_passphrase(&path, "correct key".to_string())
        .unwrap()
        .set("plane", "api-key", "secret")
        .unwrap();

    let reopened = FileCredentialStore::with_passphrase(&path, "correct key".to_string()).unwrap();
    assert_eq!(reopened.get("plane", "api-key").unwrap(), "secret");

    let wrong = FileCredentialStore::with_passphrase(&path, "wrong key".to_string()).unwrap();
    assert!(matches!(
        wrong.get("plane", "api-key"),
        Err(CredentialError::Encryption(ref message))
            if message == "credential file authentication failed"
    ));
    // A wrong key cannot read, so it cannot safely mutate either: every write
    // first loads the existing state, which fails closed.
    assert!(matches!(
        wrong.set("plane", "api-key", "rewritten"),
        Err(CredentialError::Encryption(_))
    ));
    assert!(matches!(
        wrong.delete("plane", "api-key"),
        Err(CredentialError::Encryption(_))
    ));
    assert!(matches!(
        wrong.list_keys("plane"),
        Err(CredentialError::Encryption(_))
    ));

    // The original data is untouched and still readable with the right key.
    let still_correct =
        FileCredentialStore::with_passphrase(&path, "correct key".to_string()).unwrap();
    assert_eq!(still_correct.get("plane", "api-key").unwrap(), "secret");
}

#[test]
fn stored_bytes_never_contain_the_plaintext() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");
    let store =
        FileCredentialStore::with_passphrase(&path, "test encryption key".to_string()).unwrap();

    store
        .set("plane", "api-key", "super-secret-value-that-must-not-leak")
        .unwrap();

    let raw = std::fs::read(&path).unwrap();
    let as_text = String::from_utf8_lossy(&raw);
    assert!(!as_text.contains("super-secret-value-that-must-not-leak"));
    // The envelope is JSON with the four expected fields.
    let envelope: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(envelope["version"], serde_json::json!(1));
    assert!(envelope.get("salt").is_some());
    assert!(envelope.get("nonce").is_some());
    assert!(envelope.get("ciphertext").is_some());

    // Salt is 16 bytes and nonce 12 bytes, base64 standard-without-padding.
    let salt = STANDARD_NO_PAD
        .decode(envelope["salt"].as_str().unwrap())
        .unwrap();
    let nonce = STANDARD_NO_PAD
        .decode(envelope["nonce"].as_str().unwrap())
        .unwrap();
    assert_eq!(salt.len(), 16);
    assert_eq!(nonce.len(), 12);

    // Re-encrypting even identical state must use a fresh salt/nonce, so the
    // ciphertext changes between writes.
    let first = std::fs::read(&path).unwrap();
    store
        .set("plane", "api-key", "super-secret-value-that-must-not-leak")
        .unwrap();
    let second = std::fs::read(&path).unwrap();
    assert_ne!(first, second);
}

#[test]
fn nested_parent_directories_are_created_on_first_write() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory
        .path()
        .join("nested")
        .join("deeper")
        .join("credentials.enc");
    let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();

    store.set("plane", "api-key", "value").unwrap();

    assert!(path.is_file());
    assert!(path.parent().unwrap().is_dir());
    assert_eq!(store.get("plane", "api-key").unwrap(), "value");
}

// ---------------------------------------------------------------------------
// Missing-key semantics
// ---------------------------------------------------------------------------

#[test]
fn get_and_delete_report_not_found_without_creating_a_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");
    let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();

    assert!(matches!(
        store.get("plane", "api-key"),
        Err(CredentialError::NotFound(ref key)) if key == "api-key"
    ));
    assert!(matches!(
        store.delete("plane", "api-key"),
        Err(CredentialError::NotFound(ref key)) if key == "api-key"
    ));
    assert!(
        !path.exists(),
        "reads and failed deletes must not create the file"
    );

    store.set("plane", "api-key", "value").unwrap();
    assert!(matches!(
        store.get("plane", "other-key"),
        Err(CredentialError::NotFound(ref key)) if key == "other-key"
    ));
    assert!(matches!(
        store.delete("plane", "another-key"),
        Err(CredentialError::NotFound(_))
    ));
    store.delete("plane", "api-key").unwrap();
    assert!(matches!(
        store.delete("plane", "api-key"),
        Err(CredentialError::NotFound(_))
    ));
}

#[test]
fn list_keys_is_scoped_to_the_service_and_empty_for_unknown_ones() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");
    let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();

    assert!(store.list_keys("plane").unwrap().is_empty());
    store.set("plane", "b-key", "1").unwrap();
    store.set("plane", "a-key", "2").unwrap();
    store.set("github", "token", "3").unwrap();

    let mut plane_keys = store.list_keys("plane").unwrap();
    plane_keys.sort();
    assert_eq!(plane_keys, vec!["a-key".to_string(), "b-key".to_string()]);
    assert_eq!(
        store.list_keys("github").unwrap(),
        vec!["token".to_string()]
    );
    assert!(store.list_keys("unknown-service").unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Failure atomicity
// ---------------------------------------------------------------------------

#[test]
fn a_failed_write_keeps_the_previous_cached_value() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");
    let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();
    store.set("plane", "api-key", "old").unwrap();

    // Replace the directory with a plain file so the atomic write cannot stage.
    std::fs::remove_dir_all(directory.path()).unwrap();
    std::fs::write(directory.path(), "not a directory").unwrap();

    assert!(store.set("plane", "api-key", "new").is_err());
    assert_eq!(
        store.get("plane", "api-key").unwrap(),
        "old",
        "the in-memory cache must not diverge from the last successful write"
    );
}

#[test]
fn a_failed_delete_keeps_the_previous_cached_value() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");
    let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();
    store.set("plane", "api-key", "old").unwrap();

    std::fs::remove_dir_all(directory.path()).unwrap();
    std::fs::write(directory.path(), "not a directory").unwrap();

    assert!(store.delete("plane", "api-key").is_err());
    assert_eq!(store.get("plane", "api-key").unwrap(), "old");
}

#[test]
fn a_path_that_cannot_be_read_as_a_file_fails_with_an_io_error() {
    // `/` has no parent directory, so an atomic staged write is impossible. The
    // store reports this as an I/O failure rather than panicking.
    let store = FileCredentialStore::with_passphrase(Path::new("/"), "key".to_string()).unwrap();
    let error = store.set("plane", "api-key", "value").unwrap_err();
    assert!(matches!(error, CredentialError::Io(_)), "{error:?}");
}

#[cfg(unix)]
#[test]
fn a_successful_write_leaves_no_staging_file_and_uses_0600() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");
    let store =
        FileCredentialStore::with_passphrase(&path, "test encryption key".to_string()).unwrap();

    // Multiple writes exercise the staging suffix path repeatedly.
    for value in ["one", "two", "three"] {
        store.set("plane", "api-key", value).unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let leftovers: Vec<_> = std::fs::read_dir(directory.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(".credentials-"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "staging files left behind: {leftovers:?}"
    );
}

// ---------------------------------------------------------------------------
// Malformed envelopes
// ---------------------------------------------------------------------------

/// Read the on-disk envelope, apply `mutate`, and write it back.
fn rewrite_envelope(path: &Path, mutate: impl FnOnce(&mut serde_json::Value)) {
    let raw = std::fs::read(path).unwrap();
    let mut envelope: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    mutate(&mut envelope);
    std::fs::write(path, serde_json::to_vec(&envelope).unwrap()).unwrap();
}

#[test]
fn an_unsupported_envelope_version_fails_closed() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");
    FileCredentialStore::with_passphrase(&path, "key".to_string())
        .unwrap()
        .set("plane", "api-key", "value")
        .unwrap();

    rewrite_envelope(&path, |envelope| {
        envelope["version"] = serde_json::json!(2);
    });

    let reopened = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();
    assert!(matches!(
        reopened.get("plane", "api-key"),
        Err(CredentialError::Encryption(ref message))
            if message == "unsupported credential envelope version 2"
    ));
}

#[test]
fn malformed_salt_nonce_and_ciphertext_fields_are_all_rejected() {
    let tamperers: Vec<(&str, bool, Box<dyn FnOnce(&mut serde_json::Value)>)> = vec![
        (
            "non-base64 salt",
            true,
            Box::new(|e: &mut serde_json::Value| {
                e["salt"] = serde_json::json!("not base64!!");
            }),
        ),
        (
            "short salt",
            true,
            Box::new(|e: &mut serde_json::Value| {
                e["salt"] = serde_json::json!(STANDARD_NO_PAD.encode([0u8; 8]));
            }),
        ),
        (
            "long salt",
            true,
            Box::new(|e: &mut serde_json::Value| {
                e["salt"] = serde_json::json!(STANDARD_NO_PAD.encode([0u8; 32]));
            }),
        ),
        // A missing required field fails at serde decode time, before crypto.
        (
            "missing salt",
            false,
            Box::new(|e: &mut serde_json::Value| {
                e.as_object_mut().unwrap().remove("salt");
            }),
        ),
        (
            "non-base64 nonce",
            true,
            Box::new(|e: &mut serde_json::Value| {
                e["nonce"] = serde_json::json!("!");
            }),
        ),
        (
            "short nonce",
            true,
            Box::new(|e: &mut serde_json::Value| {
                e["nonce"] = serde_json::json!(STANDARD_NO_PAD.encode([0u8; 4]));
            }),
        ),
        (
            "non-base64 ciphertext",
            true,
            Box::new(|e: &mut serde_json::Value| {
                e["ciphertext"] = serde_json::json!("!!!");
            }),
        ),
        (
            "random ciphertext",
            true,
            Box::new(|e: &mut serde_json::Value| {
                e["ciphertext"] = serde_json::json!(STANDARD_NO_PAD.encode([7u8; 32]));
            }),
        ),
    ];

    for (name, expect_encryption, mutate) in tamperers {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.enc");
        FileCredentialStore::with_passphrase(&path, "key".to_string())
            .unwrap()
            .set("plane", "api-key", "value")
            .unwrap();
        rewrite_envelope(&path, mutate);

        let reopened = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();
        let result = reopened.get("plane", "api-key");
        if expect_encryption {
            assert!(
                matches!(result, Err(CredentialError::Encryption(_))),
                "{name} should be an encryption failure, got {result:?}"
            );
        } else {
            assert!(
                matches!(result, Err(CredentialError::Serialization(_))),
                "{name} should be a serialization failure, got {result:?}"
            );
        }
    }
}

#[test]
fn a_file_that_is_not_an_envelope_is_a_serialization_error() {
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
fn valid_json_that_is_not_a_decryptable_payload_is_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credentials.enc");
    // A structurally valid envelope encrypted under a different passphrase.
    FileCredentialStore::with_passphrase(&path, "other-key".to_string())
        .unwrap()
        .set("plane", "api-key", "value")
        .unwrap();

    let store = FileCredentialStore::with_passphrase(&path, "key".to_string()).unwrap();
    assert!(matches!(
        store.get("plane", "api-key"),
        Err(CredentialError::Encryption(_))
    ));
}

#[test]
fn error_display_text_for_credential_failures_is_stable() {
    // Operators paste these strings into rotation runbooks.
    assert_eq!(
        CredentialError::NotFound("agileplus/github-token".to_string()).to_string(),
        "credential not found: agileplus/github-token"
    );
    assert_eq!(
        CredentialError::Encryption("tag mismatch".to_string()).to_string(),
        "encryption error: tag mismatch"
    );
    assert_eq!(
        CredentialError::MissingEncryptionKey.to_string(),
        "credential encryption key is required for file-backed credentials"
    );
}
