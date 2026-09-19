//! Contract tests for the credential error surface and credential key names.
//!
//! `credentials::error` and `credentials::keys` had no test coverage at all.
//! `CredentialError` is the error every `CredentialStore` implementation
//! returns (OS keychain, encrypted file, in-memory), so its `Display` text is
//! what an operator sees in the CLI/API when a token is missing, the keychain
//! is unavailable, or a legacy plaintext key is still in place. The `#[from]`
//! conversion is what lets `?` bridge `std::io::Error` from the file store into
//! the credential layer without losing the underlying `io::ErrorKind`.
//!
//! The key constants are the keychain entry names. Two constants colliding
//! would make one secret silently readable as another, so the names are
//! asserted distinct rather than merely non-empty.
//!
//! These tests never read or write `HOME` / `AGILEPLUS_*`, so they do not need
//! the shared test-support env lock.
//!
//! Traceability: FR-030, FR-031 / WP15-T088

use std::error::Error;
use std::io::ErrorKind;

use agileplus_domain::credentials::{
    API_KEYS, CODERABBIT_KEY, CredentialError, GITHUB_TOKEN, PLANESO_KEY,
};

#[test]
fn credential_error_display_messages_are_stable() {
    // Operators copy these into bug reports and rotate-token guidance, so the
    // wording is part of the observable contract.
    let cases: [(CredentialError, &str); 5] = [
        (
            CredentialError::NotFound("agileplus/github-token".to_string()),
            "credential not found: agileplus/github-token",
        ),
        (
            CredentialError::BackendError("keyring locked".to_string()),
            "keychain backend error: keyring locked",
        ),
        (
            CredentialError::Serialization("missing field `nonce`".to_string()),
            "serialization error: missing field `nonce`",
        ),
        (
            CredentialError::Encryption("tag mismatch".to_string()),
            "encryption error: tag mismatch",
        ),
        (
            CredentialError::MissingEncryptionKey,
            "credential encryption key is required for file-backed credentials",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }

    // The legacy-plaintext warning must name the exact remediation env var.
    let legacy = CredentialError::LegacyPlaintextApiKey;
    assert_eq!(
        legacy.to_string(),
        "legacy plaintext API key detected; rotate it with AGILEPLUS_API_KEY before starting"
    );
    assert!(
        legacy.to_string().contains("AGILEPLUS_API_KEY"),
        "the remediation hint must stay in the message"
    );
}

#[test]
fn credential_error_io_conversion_preserves_the_source_error() {
    let io_error = std::io::Error::new(ErrorKind::PermissionDenied, "denied by sandbox");
    let error: CredentialError = io_error.into();

    assert!(
        matches!(error, CredentialError::Io(_)),
        "an io::Error must convert into the Io variant, got {error:?}"
    );
    assert_eq!(error.to_string(), "IO error: denied by sandbox");

    // The wrapped io::Error is reachable through the std::error::Error chain,
    // and keeps its kind so callers can tell "missing" from "unreadable".
    let source = error.source().expect("Io variant must expose its source");
    let recovered = source
        .downcast_ref::<std::io::Error>()
        .expect("source must be the original io::Error");
    assert_eq!(recovered.kind(), ErrorKind::PermissionDenied);
    assert_eq!(recovered.to_string(), "denied by sandbox");

    // Variants that carry only a message must not claim a source.
    assert!(
        CredentialError::NotFound("k".to_string())
            .source()
            .is_none()
    );
    assert!(
        CredentialError::Encryption("x".to_string())
            .source()
            .is_none()
    );
    assert!(CredentialError::MissingEncryptionKey.source().is_none());
    assert!(CredentialError::LegacyPlaintextApiKey.source().is_none());
}

#[test]
fn credential_error_bridges_with_the_question_mark_operator() {
    fn load(store: impl Fn() -> Result<String, CredentialError>) -> Result<String, Box<dyn Error>> {
        Ok(store()?)
    }

    let missing = load(|| Err(CredentialError::NotFound("agileplus/api-keys".to_string())));
    let error = missing.unwrap_err();
    assert_eq!(
        error.to_string(),
        "credential not found: agileplus/api-keys"
    );

    // An io::Error from the file store also bridges in one `?`.
    let unreadable = load(|| Err(std::io::Error::new(ErrorKind::NotFound, "no such file").into()));
    let error = unreadable.unwrap_err();
    assert_eq!(error.to_string(), "IO error: no such file");
}

#[test]
fn credential_error_variants_stay_distinguishable() {
    // Callers branch on the variant (e.g. the factory retries the file store
    // only on BackendError), so no two variants may collapse together.
    let errors = [
        CredentialError::NotFound("a".to_string()),
        CredentialError::BackendError("a".to_string()),
        CredentialError::Io(std::io::Error::other("a")),
        CredentialError::Serialization("a".to_string()),
        CredentialError::Encryption("a".to_string()),
        CredentialError::MissingEncryptionKey,
        CredentialError::LegacyPlaintextApiKey,
    ];

    let debug_forms: std::collections::HashSet<String> =
        errors.iter().map(|e| format!("{e:?}")).collect();
    assert_eq!(
        debug_forms.len(),
        errors.len(),
        "every CredentialError variant must be distinct: {debug_forms:?}"
    );

    for error in &errors {
        assert!(
            !error.to_string().is_empty(),
            "{error:?} must render a message"
        );
    }

    assert!(matches!(
        errors[1],
        CredentialError::BackendError(ref message) if message == "a"
    ));
    assert!(matches!(errors[5], CredentialError::MissingEncryptionKey));
    assert!(matches!(errors[6], CredentialError::LegacyPlaintextApiKey));
}

#[test]
fn credential_key_names_are_non_empty_and_mutually_distinct() {
    let keys = [GITHUB_TOKEN, CODERABBIT_KEY, PLANESO_KEY, API_KEYS];

    let unique: std::collections::HashSet<&str> = keys.into_iter().collect();
    assert_eq!(
        unique.len(),
        keys.len(),
        "credential key names must not collide: {keys:?}"
    );

    for key in keys {
        assert!(!key.is_empty(), "a credential key name must not be empty");
        assert_eq!(
            key.trim(),
            key,
            "credential key {key:?} has stray whitespace"
        );
        assert!(
            key.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "credential key {key:?} must be a lowercase, hyphenated keychain name"
        );
    }
}
