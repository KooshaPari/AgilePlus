//! Integration tests for agileplus-error-core.
//!
//! Since the error-core crate is a thin shell, these tests exercise the
//! DomainError and ErrorCode types from agileplus-domain to ensure the
//! error contract is well-defined and stable.

use agileplus_domain::error::{DomainError, ErrorCode};

// ---------------------------------------------------------------------------
// ErrorCode enum variants
// ---------------------------------------------------------------------------

#[test]
fn error_code_debug_format() {
    let codes = [
        ErrorCode::NotFound,
        ErrorCode::AlreadyExists,
        ErrorCode::ValidationError,
        ErrorCode::NotImplemented,
        ErrorCode::InternalError,
    ];
    for code in &codes {
        let debug = format!("{:?}", code);
        assert!(!debug.is_empty());
    }
}

#[test]
fn error_code_clone() {
    let code = ErrorCode::NotFound;
    let cloned = code;
    assert_eq!(code, cloned);
}

#[test]
fn error_code_copy() {
    let code = ErrorCode::ValidationError;
    let copied = code;
    assert_eq!(code, copied);
}

#[test]
fn error_code_equality() {
    assert_eq!(ErrorCode::NotFound, ErrorCode::NotFound);
    assert_ne!(ErrorCode::NotFound, ErrorCode::AlreadyExists);
    assert_ne!(ErrorCode::ValidationError, ErrorCode::InternalError);
}

#[test]
fn error_code_all_variants_serialize_deserialize() {
    let variants = [
        ErrorCode::NotFound,
        ErrorCode::AlreadyExists,
        ErrorCode::ValidationError,
        ErrorCode::NotImplemented,
        ErrorCode::InternalError,
    ];
    for variant in &variants {
        let json = serde_json::to_string(variant).unwrap();
        let restored: ErrorCode = serde_json::from_str(&json).unwrap();
        assert_eq!(*variant, restored);
    }
}

// ---------------------------------------------------------------------------
// DomainError Display trait
// ---------------------------------------------------------------------------

#[test]
fn domain_error_display_all_variants() {
    let cases: Vec<(DomainError, &str)> = vec![
        (
            DomainError::FeatureNotInModuleScope {
                feature_slug: "f1".into(),
                module_slug: "m1".into(),
            },
            "f1",
        ),
        (DomainError::ModuleHasDependents("dep".into()), "dep"),
        (DomainError::CycleNotFound("c1".into()), "c1"),
        (DomainError::ModuleNotFound("m1".into()), "m1"),
        (DomainError::FeatureNotFound("f1".into()), "f1"),
        (DomainError::WorkPackageNotFound("wp1".into()), "wp1"),
        (DomainError::NotFound("x".into()), "x"),
        (DomainError::Storage("db down".into()), "db down"),
        (DomainError::Validation("bad input".into()), "bad input"),
        (DomainError::Conflict("dup".into()), "dup"),
        (
            DomainError::InvalidTransition {
                from: "a".into(),
                to: "b".into(),
                reason: "nope".into(),
            },
            "nope",
        ),
        (DomainError::InvalidClaim("bad claim".into()), "bad claim"),
        (DomainError::Other("misc".into()), "misc"),
        (DomainError::Agent("dispatch failed".into()), "dispatch failed"),
        (DomainError::Timeout(30), "30"),
    ];

    for (error, expected_substring) in &cases {
        let display = error.to_string();
        assert!(
            display.contains(expected_substring),
            "Expected '{}' in display of {:?}: {}",
            expected_substring,
            error,
            display
        );
    }
}

#[test]
fn domain_error_not_implemented_display() {
    let e = DomainError::NotImplemented;
    assert_eq!(e.to_string(), "Not implemented");
}

#[test]
fn domain_error_lock_poisoned_display() {
    let e = DomainError::LockPoisoned;
    assert_eq!(e.to_string(), "Lock poisoned");
}

#[test]
fn domain_error_no_op_transition_display() {
    let e = DomainError::NoOpTransition;
    assert_eq!(e.to_string(), "No-op transition: already in the requested state");
}

// ---------------------------------------------------------------------------
// ErrorCode projection: NotFound family
// ---------------------------------------------------------------------------

#[test]
fn error_code_projection_not_found_family() {
    let cases: Vec<DomainError> = vec![
        DomainError::CycleNotFound("c".into()),
        DomainError::ModuleNotFound("m".into()),
        DomainError::FeatureNotFound("f".into()),
        DomainError::WorkPackageNotFound("wp".into()),
        DomainError::NotFound("any".into()),
    ];
    for err in cases {
        let variant_desc = format!("{:?}", err);
        let code: ErrorCode = err.into();
        assert_eq!(code, ErrorCode::NotFound, "Expected NotFound for {variant_desc}");
    }
}

// ---------------------------------------------------------------------------
// ErrorCode projection: AlreadyExists family
// ---------------------------------------------------------------------------

#[test]
fn error_code_projection_already_exists_family() {
    let cases: Vec<DomainError> = vec![
        DomainError::ModuleHasDependents("m".into()),
        DomainError::Conflict("dup".into()),
    ];
    for err in cases {
        let variant_desc = format!("{:?}", err);
        let code: ErrorCode = err.into();
        assert_eq!(code, ErrorCode::AlreadyExists, "Expected AlreadyExists for {variant_desc}");
    }
}

// ---------------------------------------------------------------------------
// ErrorCode projection: ValidationError family
// ---------------------------------------------------------------------------

#[test]
fn error_code_projection_validation_family() {
    let cases: Vec<DomainError> = vec![
        DomainError::Validation("bad".into()),
        DomainError::FeatureNotInModuleScope {
            feature_slug: "f".into(),
            module_slug: "m".into(),
        },
        DomainError::InvalidTransition {
            from: "a".into(),
            to: "b".into(),
            reason: "no".into(),
        },
        DomainError::InvalidClaim("claim".into()),
        DomainError::NoOpTransition,
    ];
    for err in cases {
        let variant_desc = format!("{:?}", err);
        let code: ErrorCode = err.into();
        assert_eq!(code, ErrorCode::ValidationError, "Expected ValidationError for {variant_desc}");
    }
}

// ---------------------------------------------------------------------------
// ErrorCode projection: InternalError family
// ---------------------------------------------------------------------------

#[test]
fn error_code_projection_internal_error_family() {
    let cases: Vec<DomainError> = vec![
        DomainError::Storage("db".into()),
        DomainError::LockPoisoned,
        DomainError::Other("misc".into()),
        DomainError::Agent("dispatch".into()),
        DomainError::Timeout(60),
    ];
    for err in cases {
        let variant_desc = format!("{:?}", err);
        let code: ErrorCode = err.into();
        assert_eq!(code, ErrorCode::InternalError, "Expected InternalError for {variant_desc}");
    }
}

// ---------------------------------------------------------------------------
// ErrorCode projection: NotImplemented
// ---------------------------------------------------------------------------

#[test]
fn error_code_projection_not_implemented() {
    let code: ErrorCode = DomainError::NotImplemented.into();
    assert_eq!(code, ErrorCode::NotImplemented);
}

// ---------------------------------------------------------------------------
// DomainError is Send + Sync (compile-time)
// ---------------------------------------------------------------------------

fn _assert_send_sync<T: Send + Sync>() {}
#[test]
fn domain_error_is_send_sync() {
    _assert_send_sync::<DomainError>();
    _assert_send_sync::<ErrorCode>();
}

// ---------------------------------------------------------------------------
// DomainError from_string via Other variant
// ---------------------------------------------------------------------------

#[test]
fn domain_error_other_preserves_message() {
    let e = DomainError::Other("something happened".into());
    assert_eq!(e.to_string(), "something happened");
}

#[test]
fn domain_error_timeout_contains_seconds() {
    let e = DomainError::Timeout(42);
    assert_eq!(e.to_string(), "Timed out after 42 seconds");
}

// ---------------------------------------------------------------------------
// ErrorCode serde edge cases
// ---------------------------------------------------------------------------

#[test]
fn error_code_serde_invalid_variant_fails() {
    let result = serde_json::from_str::<ErrorCode>(r#""BogusVariant""#);
    assert!(result.is_err());
}

#[test]
fn error_code_serde_empty_string_fails() {
    let result = serde_json::from_str::<ErrorCode>(r#""""#);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// DomainError: feature-not-in-module-scope structured fields
// ---------------------------------------------------------------------------

#[test]
fn feature_not_in_module_scope_preserves_both_slugs() {
    let e = DomainError::FeatureNotInModuleScope {
        feature_slug: "login-feature".into(),
        module_slug: "auth-module".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("login-feature"), "Missing feature_slug in: {msg}");
    assert!(msg.contains("auth-module"), "Missing module_slug in: {msg}");
    assert!(msg.contains("not in module"), "Missing 'not in module' in: {msg}");
}

// ---------------------------------------------------------------------------
// DomainError: InvalidTransition structured fields
// ---------------------------------------------------------------------------

#[test]
fn invalid_transition_preserves_from_to_reason() {
    let e = DomainError::InvalidTransition {
        from: "draft".into(),
        to: "shipped".into(),
        reason: "skipped validation".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("draft"), "Missing from in: {msg}");
    assert!(msg.contains("shipped"), "Missing to in: {msg}");
    assert!(msg.contains("skipped validation"), "Missing reason in: {msg}");
}

// ---------------------------------------------------------------------------
// DomainError: exhaustive match coverage (compile-time)
// ---------------------------------------------------------------------------

#[test]
fn domain_error_exhaustive_display_coverage() {
    // Ensure every variant has a Display implementation by constructing and displaying each
    let variants: Vec<DomainError> = vec![
        DomainError::FeatureNotInModuleScope { feature_slug: "f".into(), module_slug: "m".into() },
        DomainError::ModuleHasDependents("d".into()),
        DomainError::CycleNotFound("c".into()),
        DomainError::ModuleNotFound("m".into()),
        DomainError::FeatureNotFound("f".into()),
        DomainError::WorkPackageNotFound("w".into()),
        DomainError::NotFound("n".into()),
        DomainError::NotImplemented,
        DomainError::Storage("s".into()),
        DomainError::Validation("v".into()),
        DomainError::Conflict("c".into()),
        DomainError::InvalidTransition { from: "a".into(), to: "b".into(), reason: "r".into() },
        DomainError::LockPoisoned,
        DomainError::InvalidClaim("c".into()),
        DomainError::NoOpTransition,
        DomainError::Other("o".into()),
        DomainError::Agent("a".into()),
        DomainError::Timeout(10),
    ];

    for variant in &variants {
        let display = variant.to_string();
        assert!(!display.is_empty(), "Empty Display for {:?}", variant);
    }

    assert_eq!(variants.len(), 18, "Expected 18 DomainError variants");
}

// ---------------------------------------------------------------------------
// ErrorCode: exhaustive match from DomainError
// ---------------------------------------------------------------------------

#[test]
fn error_code_exhaustive_projection_coverage() {
    // Verify that every DomainError variant produces exactly one ErrorCode
    let mapping: Vec<(DomainError, ErrorCode)> = vec![
        (DomainError::CycleNotFound("c".into()), ErrorCode::NotFound),
        (DomainError::ModuleNotFound("m".into()), ErrorCode::NotFound),
        (DomainError::FeatureNotFound("f".into()), ErrorCode::NotFound),
        (DomainError::WorkPackageNotFound("w".into()), ErrorCode::NotFound),
        (DomainError::NotFound("n".into()), ErrorCode::NotFound),
        (DomainError::ModuleHasDependents("d".into()), ErrorCode::AlreadyExists),
        (DomainError::Conflict("c".into()), ErrorCode::AlreadyExists),
        (DomainError::Validation("v".into()), ErrorCode::ValidationError),
        (DomainError::FeatureNotInModuleScope { feature_slug: "f".into(), module_slug: "m".into() }, ErrorCode::ValidationError),
        (DomainError::InvalidTransition { from: "a".into(), to: "b".into(), reason: "r".into() }, ErrorCode::ValidationError),
        (DomainError::InvalidClaim("c".into()), ErrorCode::ValidationError),
        (DomainError::NoOpTransition, ErrorCode::ValidationError),
        (DomainError::NotImplemented, ErrorCode::NotImplemented),
        (DomainError::Storage("s".into()), ErrorCode::InternalError),
        (DomainError::LockPoisoned, ErrorCode::InternalError),
        (DomainError::Other("o".into()), ErrorCode::InternalError),
        (DomainError::Agent("a".into()), ErrorCode::InternalError),
        (DomainError::Timeout(10), ErrorCode::InternalError),
    ];

    for (err, expected_code) in mapping {
        let code: ErrorCode = err.into();
        assert_eq!(code, expected_code, "Projection mismatch for variant");
    }
}

// ---------------------------------------------------------------------------
// ErrorCode serde: roundtrip all variants
// ---------------------------------------------------------------------------

#[test]
fn error_code_json_roundtrip_all_variants() {
    let all = [
        ErrorCode::NotFound,
        ErrorCode::AlreadyExists,
        ErrorCode::ValidationError,
        ErrorCode::NotImplemented,
        ErrorCode::InternalError,
    ];
    for code in &all {
        let json = serde_json::to_string(code).unwrap();
        let back: ErrorCode = serde_json::from_str(&json).unwrap();
        assert_eq!(*code, back, "Roundtrip failed for {:?}", code);
    }
}
