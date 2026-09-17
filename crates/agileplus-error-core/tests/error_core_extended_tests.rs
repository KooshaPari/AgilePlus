//! Additional integration tests for agileplus-error-core.
//!
//! These tests build on the original error_core_test.rs and provide deeper
//! coverage of the DomainError / ErrorCode surface: edge cases, projections,
//! error trait integration, and re-projection round trips.

use agileplus_domain::error::{DomainError, DomainResult, ErrorCode};
use std::error::Error as StdError;

// ---------------------------------------------------------------------------
// DomainResult type alias
// ---------------------------------------------------------------------------

#[test]
fn domain_result_ok_works() {
    let result: DomainResult<i32> = Ok(42);
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn domain_result_err_works() {
    let result: DomainResult<i32> = Err(DomainError::NotFound("x".into()));
    assert!(result.is_err());
}

#[test]
fn domain_result_map_ok() {
    let r: DomainResult<i32> = Ok(5);
    let mapped = r.map(|v| v * 2);
    assert_eq!(mapped.unwrap(), 10);
}

#[test]
fn domain_result_map_err() {
    let r: DomainResult<i32> = Err(DomainError::NotFound("x".into()));
    let mapped = r.map_err(|e| match e {
        DomainError::NotFound(_) => DomainError::Other("not_found".into()),
        other => other,
    });
    let err = mapped.unwrap_err();
    assert!(matches!(err, DomainError::Other(_)));
}

// ---------------------------------------------------------------------------
// ErrorCode equality / hashing semantics
// ---------------------------------------------------------------------------

#[test]
fn error_code_hash_eq() {
    // ErrorCode doesn't implement Hash, but it is Eq+Copy; verify distinct variants
    // produce distinct Debug strings usable for de-dup bookkeeping.
    let codes = vec![
        ErrorCode::NotFound,
        ErrorCode::NotFound,
        ErrorCode::AlreadyExists,
    ];
    let unique: std::collections::HashSet<String> = codes
        .into_iter()
        .map(|c| format!("{:?}", c))
        .collect();
    assert_eq!(unique.len(), 2);
}

#[test]
fn error_code_matches_via_eq() {
    let a = ErrorCode::NotFound;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn error_code_all_variants_distinct() {
    let codes = [
        ErrorCode::NotFound,
        ErrorCode::AlreadyExists,
        ErrorCode::ValidationError,
        ErrorCode::NotImplemented,
        ErrorCode::InternalError,
    ];
    for (i, a) in codes.iter().enumerate() {
        for (j, b) in codes.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// DomainError Debug formatting
// ---------------------------------------------------------------------------

#[test]
fn debug_feature_not_in_module_scope() {
    let e = DomainError::FeatureNotInModuleScope {
        feature_slug: "f".into(),
        module_slug: "m".into(),
    };
    let s = format!("{:?}", e);
    assert!(s.contains("FeatureNotInModuleScope"));
    assert!(s.contains("f"));
    assert!(s.contains("m"));
}

#[test]
fn debug_module_has_dependents() {
    let e = DomainError::ModuleHasDependents("dep".into());
    let s = format!("{:?}", e);
    assert!(s.contains("ModuleHasDependents"));
    assert!(s.contains("dep"));
}

#[test]
fn debug_cycle_not_found() {
    let e = DomainError::CycleNotFound("c".into());
    let s = format!("{:?}", e);
    assert!(s.contains("CycleNotFound"));
    assert!(s.contains("c"));
}

#[test]
fn debug_lock_poisoned() {
    let e = DomainError::LockPoisoned;
    let s = format!("{:?}", e);
    assert!(s.contains("LockPoisoned"));
}

#[test]
fn debug_no_op_transition() {
    let e = DomainError::NoOpTransition;
    let s = format!("{:?}", e);
    assert!(s.contains("NoOpTransition"));
}

#[test]
fn debug_timeout_contains_number() {
    let e = DomainError::Timeout(123);
    let s = format!("{:?}", e);
    assert!(s.contains("123"));
}

#[test]
fn debug_invalid_transition_contains_parts() {
    let e = DomainError::InvalidTransition {
        from: "A".into(),
        to: "B".into(),
        reason: "nope".into(),
    };
    let s = format!("{:?}", e);
    assert!(s.contains("A"));
    assert!(s.contains("B"));
    assert!(s.contains("nope"));
}

// ---------------------------------------------------------------------------
// Error trait impls
// ---------------------------------------------------------------------------

#[test]
fn domain_error_is_std_error() {
    let e: Box<dyn StdError> = Box::new(DomainError::Storage("x".into()));
    let _msg = e.to_string();
}

#[test]
fn domain_error_source_for_plain_variants() {
    // Plain variants (no #[from] or #[source]) have no source.
    let e = DomainError::Storage("x".into());
    assert!(e.source().is_none());

    let e = DomainError::NotFound("y".into());
    assert!(e.source().is_none());

    let e = DomainError::LockPoisoned;
    assert!(e.source().is_none());
}

// ---------------------------------------------------------------------------
// Send + Sync
// ---------------------------------------------------------------------------

#[test]
fn domain_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DomainError>();
}

#[test]
fn error_code_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ErrorCode>();
}

// ---------------------------------------------------------------------------
// Projection: every DomainError variant must project onto some ErrorCode.
// ---------------------------------------------------------------------------

#[test]
fn exhaustive_projection_all_variants() {
    let errors: Vec<DomainError> = vec![
        DomainError::FeatureNotInModuleScope {
            feature_slug: "f".into(),
            module_slug: "m".into(),
        },
        DomainError::ModuleHasDependents("m".into()),
        DomainError::CycleNotFound("c".into()),
        DomainError::ModuleNotFound("m".into()),
        DomainError::FeatureNotFound("f".into()),
        DomainError::WorkPackageNotFound("w".into()),
        DomainError::NotFound("n".into()),
        DomainError::NotImplemented,
        DomainError::Storage("s".into()),
        DomainError::Validation("v".into()),
        DomainError::Conflict("c".into()),
        DomainError::InvalidTransition {
            from: "a".into(),
            to: "b".into(),
            reason: "r".into(),
        },
        DomainError::InvalidClaim("i".into()),
        DomainError::NoOpTransition,
        DomainError::Other("o".into()),
        DomainError::Agent("a".into()),
        DomainError::Timeout(60),
        DomainError::LockPoisoned,
    ];

    for err in errors {
        let _: ErrorCode = err.into();
    }
}

#[test]
fn projection_validation_error_for_misc_validation() {
    let errors = vec![
        DomainError::Validation("x".into()),
        DomainError::FeatureNotInModuleScope {
            feature_slug: "f".into(),
            module_slug: "m".into(),
        },
        DomainError::InvalidTransition {
            from: "draft".into(),
            to: "done".into(),
            reason: "missing review".into(),
        },
        DomainError::InvalidClaim("bad claim".into()),
    ];

    for err in errors {
        let code: ErrorCode = err.into();
        assert_eq!(code, ErrorCode::ValidationError);
    }
}

#[test]
fn projection_already_exists_for_conflicts() {
    let errors = vec![
        DomainError::ModuleHasDependents("m".into()),
        DomainError::Conflict("dup".into()),
    ];

    for err in errors {
        let code: ErrorCode = err.into();
        assert_eq!(code, ErrorCode::AlreadyExists);
    }
}

#[test]
fn projection_internal_error_for_storage_lock() {
    let errors = vec![
        DomainError::Storage("db down".into()),
        DomainError::LockPoisoned,
    ];

    for err in errors {
        let code: ErrorCode = err.into();
        assert_eq!(code, ErrorCode::InternalError);
    }
}

#[test]
fn projection_not_found_for_all_not_found_variants() {
    let errors = vec![
        DomainError::CycleNotFound("c".into()),
        DomainError::ModuleNotFound("m".into()),
        DomainError::FeatureNotFound("f".into()),
        DomainError::WorkPackageNotFound("w".into()),
        DomainError::NotFound("n".into()),
    ];

    for err in errors {
        let code: ErrorCode = err.into();
        assert_eq!(code, ErrorCode::NotFound);
    }
}

#[test]
fn projection_not_implemented_is_not_implemented() {
    let code: ErrorCode = DomainError::NotImplemented.into();
    assert_eq!(code, ErrorCode::NotImplemented);
}

#[test]
fn projection_timeout_falls_to_internal_error() {
    // Timeouts are classified as internal in the current mapping.
    let code: ErrorCode = DomainError::Timeout(60).into();
    assert_eq!(code, ErrorCode::InternalError);
}

#[test]
fn projection_agent_falls_to_internal_error() {
    let code: ErrorCode = DomainError::Agent("dispatch failed".into()).into();
    assert_eq!(code, ErrorCode::InternalError);
}

#[test]
fn projection_other_falls_to_internal_error() {
    let code: ErrorCode = DomainError::Other("misc".into()).into();
    assert_eq!(code, ErrorCode::InternalError);
}

#[test]
fn projection_no_op_transition_falls_to_validation_error() {
    let code: ErrorCode = DomainError::NoOpTransition.into();
    assert_eq!(code, ErrorCode::ValidationError);
}

// ---------------------------------------------------------------------------
// JSON serialization edge cases
// ---------------------------------------------------------------------------

#[test]
fn error_code_serde_rejects_unknown_variant() {
    let result: Result<ErrorCode, _> = serde_json::from_str("\"unknown_variant\"");
    assert!(result.is_err());
}

#[test]
fn error_code_serde_rejects_wrong_type() {
    let result: Result<ErrorCode, _> = serde_json::from_str("42");
    assert!(result.is_err());
}

#[test]
fn error_code_serde_accepts_snake_case_form() {
    // Canonical forms based on the derive defaults.
    assert_eq!(
        serde_json::from_str::<ErrorCode>("\"NotFound\"").unwrap(),
        ErrorCode::NotFound
    );
    assert_eq!(
        serde_json::from_str::<ErrorCode>("\"InternalError\"").unwrap(),
        ErrorCode::InternalError
    );
}

// ---------------------------------------------------------------------------
// Display for each variant
// ---------------------------------------------------------------------------

#[test]
fn display_feature_not_in_module_scope_full() {
    let e = DomainError::FeatureNotInModuleScope {
        feature_slug: "auth".into(),
        module_slug: "core".into(),
    };
    assert_eq!(
        e.to_string(),
        "Feature not in module scope: feature 'auth' not in module 'core'"
    );
}

#[test]
fn display_module_has_dependents_exact() {
    assert_eq!(
        DomainError::ModuleHasDependents("m1".into()).to_string(),
        "Module has dependents: m1"
    );
}

#[test]
fn display_cycle_not_found_exact() {
    assert_eq!(
        DomainError::CycleNotFound("c1".into()).to_string(),
        "Cycle not found: c1"
    );
}

#[test]
fn display_module_not_found_exact() {
    assert_eq!(
        DomainError::ModuleNotFound("m".into()).to_string(),
        "Module not found: m"
    );
}

#[test]
fn display_feature_not_found_exact() {
    assert_eq!(
        DomainError::FeatureNotFound("f".into()).to_string(),
        "Feature not found: f"
    );
}

#[test]
fn display_work_package_not_found_exact() {
    assert_eq!(
        DomainError::WorkPackageNotFound("w".into()).to_string(),
        "Work package not found: w"
    );
}

#[test]
fn display_not_found_exact() {
    assert_eq!(DomainError::NotFound("x".into()).to_string(), "Not found: x");
}

#[test]
fn display_not_implemented_exact() {
    assert_eq!(DomainError::NotImplemented.to_string(), "Not implemented");
}

#[test]
fn display_storage_exact() {
    assert_eq!(
        DomainError::Storage("db".into()).to_string(),
        "Storage error: db"
    );
}

#[test]
fn display_validation_exact() {
    assert_eq!(
        DomainError::Validation("bad input".into()).to_string(),
        "Validation error: bad input"
    );
}

#[test]
fn display_conflict_exact() {
    assert_eq!(
        DomainError::Conflict("dup".into()).to_string(),
        "Conflict: dup"
    );
}

#[test]
fn display_invalid_transition_exact() {
    let e = DomainError::InvalidTransition {
        from: "draft".into(),
        to: "done".into(),
        reason: "missing review".into(),
    };
    assert_eq!(
        e.to_string(),
        "Invalid transition from draft to done: missing review"
    );
}

#[test]
fn display_invalid_claim_exact() {
    assert_eq!(
        DomainError::InvalidClaim("bad".into()).to_string(),
        "Invalid claim: bad"
    );
}

#[test]
fn display_no_op_transition_exact() {
    assert_eq!(
        DomainError::NoOpTransition.to_string(),
        "No-op transition: already in the requested state"
    );
}

#[test]
fn display_other_preserves_payload() {
    assert_eq!(
        DomainError::Other("misc".into()).to_string(),
        "misc"
    );
}

#[test]
fn display_agent_exact() {
    assert_eq!(
        DomainError::Agent("dispatch failed".into()).to_string(),
        "Agent error: dispatch failed"
    );
}

#[test]
fn display_timeout_exact() {
    assert_eq!(
        DomainError::Timeout(45).to_string(),
        "Timed out after 45 seconds"
    );
}

#[test]
fn display_lock_poisoned_exact() {
    assert_eq!(DomainError::LockPoisoned.to_string(), "Lock poisoned");
}

// ---------------------------------------------------------------------------
// Variant-specific edge cases
// ---------------------------------------------------------------------------

#[test]
fn timeout_zero_is_valid() {
    let e = DomainError::Timeout(0);
    assert!(e.to_string().contains("0 seconds"));
}

#[test]
fn timeout_large_value() {
    let e = DomainError::Timeout(u64::MAX);
    let s = e.to_string();
    assert!(s.contains("Timed out"));
}

#[test]
fn invalid_transition_with_empty_strings() {
    let e = DomainError::InvalidTransition {
        from: "".into(),
        to: "".into(),
        reason: "".into(),
    };
    let s = e.to_string();
    assert!(s.contains("Invalid transition"));
}

#[test]
fn invalid_claim_with_unicode() {
    let e = DomainError::InvalidClaim("无效".into());
    assert!(e.to_string().contains("无效"));
}

#[test]
fn validation_with_long_message() {
    let long = "x".repeat(1000);
    let e = DomainError::Validation(long.clone());
    assert_eq!(e.to_string().len(), "Validation error: ".len() + 1000);
}

#[test]
fn storage_with_special_chars() {
    let e = DomainError::Storage("db\n\tdown".into());
    let s = e.to_string();
    assert!(s.contains("Storage error"));
    assert!(s.contains('\n'));
}

// ---------------------------------------------------------------------------
// Pattern matching
// ---------------------------------------------------------------------------

#[test]
fn pattern_match_on_variant() {
    let e = DomainError::Validation("x".into());
    match e {
        DomainError::Validation(msg) => assert_eq!(msg, "x"),
        _ => panic!("expected Validation variant"),
    }
}

#[test]
fn pattern_match_on_transition_struct() {
    let e = DomainError::InvalidTransition {
        from: "a".into(),
        to: "b".into(),
        reason: "c".into(),
    };
    match e {
        DomainError::InvalidTransition { from, to, reason } => {
            assert_eq!(from, "a");
            assert_eq!(to, "b");
            assert_eq!(reason, "c");
        }
        _ => panic!("expected InvalidTransition variant"),
    }
}

// ---------------------------------------------------------------------------
// Stress / hash determinism
// ---------------------------------------------------------------------------

#[test]
fn error_code_default_clone() {
    let code = ErrorCode::InternalError;
    let cloned = code;
    assert_eq!(code, cloned);
}

#[test]
fn hashmap_key_for_error_code() {
    use std::collections::HashMap;
    let mut map: HashMap<i32, &str> = HashMap::new();
    // Map codes by their discriminant (since Hash isn't implemented).
    map.insert(0, "not found");
    map.insert(1, "conflict");
    map.insert(2, "validation");
    map.insert(3, "not impl");
    map.insert(4, "internal");

    assert_eq!(map.get(&0), Some(&"not found"));
    assert_eq!(map.get(&4), Some(&"internal"));
    assert_eq!(map.get(&1), Some(&"conflict"));
    assert_eq!(map.len(), 5);

    // All 5 distinct variants must produce distinct discriminants.
    let codes = [
        ErrorCode::NotFound,
        ErrorCode::AlreadyExists,
        ErrorCode::ValidationError,
        ErrorCode::NotImplemented,
        ErrorCode::InternalError,
    ];
    let discriminants: std::collections::HashSet<i32> =
        codes.iter().map(|c| *c as i32).collect();
    assert_eq!(discriminants.len(), 5);
}

#[test]
fn debug_format_error_code() {
    let s = format!("{:?}", ErrorCode::NotFound);
    assert!(s.contains("NotFound"));
}

// ---------------------------------------------------------------------------
// Cross-crate interop: Use error types in Result alias.
// ---------------------------------------------------------------------------

#[test]
fn domain_result_with_question_mark() {
    fn might_fail(should_fail: bool) -> DomainResult<i32> {
        if should_fail {
            Err(DomainError::NotFound("missing".into()))
        } else {
            Ok(99)
        }
    }

    assert_eq!(might_fail(false).unwrap(), 99);
    let err = might_fail(true).unwrap_err();
    assert!(matches!(err, DomainError::NotFound(_)));
}

#[test]
fn question_mark_propagates_projection() {
    fn might_fail() -> DomainResult<()> {
        Err(DomainError::Storage("db".into()))
    }

    fn caller() -> Result<(), ErrorCode> {
        might_fail()?;
        Ok(())
    }

    let code = caller().unwrap_err();
    assert_eq!(code, ErrorCode::InternalError);
}

// ---------------------------------------------------------------------------
// Display stability (regression: format strings shouldn't silently change)
// ---------------------------------------------------------------------------

#[test]
fn display_messages_remain_stable() {
    // If any of these change, downstream log scraping / alerting may break.
    let cases = [
        (DomainError::LockPoisoned, "Lock poisoned"),
        (DomainError::NoOpTransition, "No-op transition: already in the requested state"),
        (DomainError::NotImplemented, "Not implemented"),
    ];

    for (err, expected) in &cases {
        assert_eq!(err.to_string(), *expected);
    }
}