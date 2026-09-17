// SPDX-License-Identifier: MIT OR Apache-2.0
//! Domain error types.

use thiserror::Error;

/// Canonical error code for cross-ecosystem error reporting.
/// Stable, language-agnostic codes used in observability and wire responses.
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ErrorCode {
    /// Entity not found by id, slug, or query.
    NotFound,
    /// Resource or state already exists (conflict, uniqueness violation).
    AlreadyExists,
    /// Invalid input, validation failure, or invariant violation.
    ValidationError,
    /// Not yet implemented.
    NotImplemented,
    /// Internal server error (storage, lock, or system failure).
    InternalError,
}

/// A convenience `Result` alias for domain operations.
pub type DomainResult<T> = Result<T, DomainError>;

/// Top-level domain error.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Feature not in module scope: feature '{feature_slug}' not in module '{module_slug}'")]
    FeatureNotInModuleScope {
        feature_slug: String,
        module_slug: String,
    },

    #[error("Module has dependents: {0}")]
    ModuleHasDependents(String),

    #[error("Cycle not found: {0}")]
    CycleNotFound(String),

    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    #[error("Feature not found: {0}")]
    FeatureNotFound(String),

    #[error("Work package not found: {0}")]
    WorkPackageNotFound(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Not implemented")]
    NotImplemented,

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Invalid transition from {from} to {to}: {reason}")]
    InvalidTransition {
        from: String,
        to: String,
        reason: String,
    },

    #[error("Lock poisoned")]
    LockPoisoned,

    /// The supplied claim is not valid for the requested operation (e.g. the
    /// claim is for a different `kind`, is in the wrong `state`, or is
    /// missing a required reason / agent binding).
    #[error("Invalid claim: {0}")]
    InvalidClaim(String),

    /// A requested state transition would be a no-op (target == current state).
    #[error("No-op transition: already in the requested state")]
    NoOpTransition,

    /// Catch-all for errors that do not map to a more specific variant.
    #[error("{0}")]
    Other(String),

    /// Agent dispatch / execution failure.
    #[error("Agent error: {0}")]
    Agent(String),

    /// Operation timed out after the given number of seconds.
    #[error("Timed out after {0} seconds")]
    Timeout(u64),
}

/// Project the AgilePlus domain error onto the canonical Phenotype wire
/// [`ErrorCode`].
///
/// This is a lossy classification: the structural payload (slugs, transition
/// detail, free-text messages) is dropped. The local [`DomainError`] remains
/// the source of truth for human-facing messaging, while [`ErrorCode`] is the
/// stable, language-agnostic code used for cross-ecosystem reporting.
impl From<DomainError> for ErrorCode {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::CycleNotFound(_)
            | DomainError::ModuleNotFound(_)
            | DomainError::FeatureNotFound(_)
            | DomainError::WorkPackageNotFound(_)
            | DomainError::NotFound(_) => Self::NotFound,

            DomainError::ModuleHasDependents(_) | DomainError::Conflict(_) => Self::AlreadyExists,

            DomainError::Validation(_)
            | DomainError::FeatureNotInModuleScope { .. }
            | DomainError::InvalidTransition { .. }
            | DomainError::InvalidClaim(_) => Self::ValidationError,

            DomainError::NotImplemented => Self::NotImplemented,

            DomainError::NoOpTransition => Self::ValidationError,

            DomainError::Storage(_)
            | DomainError::LockPoisoned
            | DomainError::Other(_)
            | DomainError::Agent(_)
            | DomainError::Timeout(_) => Self::InternalError,
        }
    }
}

#[cfg(test)]
mod code_projection_tests {
    use super::*;

    #[test]
    fn not_found_family_projects_to_not_found() {
        let c: ErrorCode = DomainError::CycleNotFound("c-3".into()).into();
        assert_eq!(c, ErrorCode::NotFound);

        let c: ErrorCode = DomainError::ModuleNotFound("m-1".into()).into();
        assert_eq!(c, ErrorCode::NotFound);

        let c: ErrorCode = DomainError::FeatureNotFound("f-9".into()).into();
        assert_eq!(c, ErrorCode::NotFound);

        let c: ErrorCode = DomainError::WorkPackageNotFound("wp-7".into()).into();
        assert_eq!(c, ErrorCode::NotFound);

        let c: ErrorCode = DomainError::NotFound("x".into()).into();
        assert_eq!(c, ErrorCode::NotFound);
    }

    #[test]
    fn conflicts_project_to_already_exists() {
        let c: ErrorCode = DomainError::ModuleHasDependents("m-1".into()).into();
        assert_eq!(c, ErrorCode::AlreadyExists);

        let c: ErrorCode = DomainError::Conflict("dup".into()).into();
        assert_eq!(c, ErrorCode::AlreadyExists);
    }

    #[test]
    fn validation_shaped_errors_project_to_validation_error() {
        let c: ErrorCode = DomainError::Validation("name required".into()).into();
        assert_eq!(c, ErrorCode::ValidationError);

        let c: ErrorCode = DomainError::FeatureNotInModuleScope {
            feature_slug: "f-1".into(),
            module_slug: "m-1".into(),
        }
        .into();
        assert_eq!(c, ErrorCode::ValidationError);

        let c: ErrorCode = DomainError::InvalidTransition {
            from: "draft".into(),
            to: "done".into(),
            reason: "missing review".into(),
        }
        .into();
        assert_eq!(c, ErrorCode::ValidationError);

        let c: ErrorCode = DomainError::InvalidClaim("bad claim".into()).into();
        assert_eq!(c, ErrorCode::ValidationError);
    }

    #[test]
    fn storage_and_lock_project_to_internal_error() {
        let c: ErrorCode = DomainError::Storage("db down".into()).into();
        assert_eq!(c, ErrorCode::InternalError);

        let c: ErrorCode = DomainError::LockPoisoned.into();
        assert_eq!(c, ErrorCode::InternalError);
    }

    #[test]
    fn not_implemented_projects_to_not_implemented() {
        let c: ErrorCode = DomainError::NotImplemented.into();
        assert_eq!(c, ErrorCode::NotImplemented);
    }

    #[test]
    fn domain_error_remains_source_of_truth_for_messaging() {
        let e = DomainError::InvalidTransition {
            from: "draft".into(),
            to: "done".into(),
            reason: "missing review".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("draft") && msg.contains("done") && msg.contains("missing review"));
    }

    // --- Display tests for each DomainError variant ---

    #[test]
    fn display_feature_not_in_module_scope() {
        let e = DomainError::FeatureNotInModuleScope {
            feature_slug: "auth".into(),
            module_slug: "core".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("auth"));
        assert!(msg.contains("core"));
        assert!(msg.contains("not in module"));
    }

    #[test]
    fn display_module_has_dependents() {
        let e = DomainError::ModuleHasDependents("m-1".into());
        assert_eq!(e.to_string(), "Module has dependents: m-1");
    }

    #[test]
    fn display_cycle_not_found() {
        let e = DomainError::CycleNotFound("c-1".into());
        assert_eq!(e.to_string(), "Cycle not found: c-1");
    }

    #[test]
    fn display_module_not_found() {
        let e = DomainError::ModuleNotFound("m-1".into());
        assert_eq!(e.to_string(), "Module not found: m-1");
    }

    #[test]
    fn display_feature_not_found() {
        let e = DomainError::FeatureNotFound("f-1".into());
        assert_eq!(e.to_string(), "Feature not found: f-1");
    }

    #[test]
    fn display_work_package_not_found() {
        let e = DomainError::WorkPackageNotFound("wp-1".into());
        assert_eq!(e.to_string(), "Work package not found: wp-1");
    }

    #[test]
    fn display_not_found() {
        let e = DomainError::NotFound("thing".into());
        assert_eq!(e.to_string(), "Not found: thing");
    }

    #[test]
    fn display_not_implemented() {
        let e = DomainError::NotImplemented;
        assert_eq!(e.to_string(), "Not implemented");
    }

    #[test]
    fn display_storage() {
        let e = DomainError::Storage("disk full".into());
        assert_eq!(e.to_string(), "Storage error: disk full");
    }

    #[test]
    fn display_validation() {
        let e = DomainError::Validation("bad name".into());
        assert_eq!(e.to_string(), "Validation error: bad name");
    }

    #[test]
    fn display_conflict() {
        let e = DomainError::Conflict("slug taken".into());
        assert_eq!(e.to_string(), "Conflict: slug taken");
    }

    #[test]
    fn display_invalid_transition() {
        let e = DomainError::InvalidTransition {
            from: "A".into(),
            to: "B".into(),
            reason: "blocked".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("A") && msg.contains("B") && msg.contains("blocked"));
    }

    #[test]
    fn display_lock_poisoned() {
        let e = DomainError::LockPoisoned;
        assert_eq!(e.to_string(), "Lock poisoned");
    }

    #[test]
    fn display_invalid_claim() {
        let e = DomainError::InvalidClaim("expired".into());
        assert_eq!(e.to_string(), "Invalid claim: expired");
    }

    #[test]
    fn display_no_op_transition() {
        let e = DomainError::NoOpTransition;
        assert!(e.to_string().contains("already in the requested state"));
    }

    #[test]
    fn display_other() {
        let e = DomainError::Other("something weird".into());
        assert_eq!(e.to_string(), "something weird");
    }

    #[test]
    fn display_agent() {
        let e = DomainError::Agent("dispatch failed".into());
        assert_eq!(e.to_string(), "Agent error: dispatch failed");
    }

    #[test]
    fn display_timeout() {
        let e = DomainError::Timeout(30);
        assert_eq!(e.to_string(), "Timed out after 30 seconds");
    }

    // --- Remaining ErrorCode projections ---

    #[test]
    fn noop_transition_projects_to_validation_error() {
        let c: ErrorCode = DomainError::NoOpTransition.into();
        assert_eq!(c, ErrorCode::ValidationError);
    }

    #[test]
    fn other_projects_to_internal_error() {
        let c: ErrorCode = DomainError::Other("misc".into()).into();
        assert_eq!(c, ErrorCode::InternalError);
    }

    #[test]
    fn agent_projects_to_internal_error() {
        let c: ErrorCode = DomainError::Agent("err".into()).into();
        assert_eq!(c, ErrorCode::InternalError);
    }

    #[test]
    fn timeout_projects_to_internal_error() {
        let c: ErrorCode = DomainError::Timeout(5).into();
        assert_eq!(c, ErrorCode::InternalError);
    }

    // --- ErrorCode serde ---

    #[test]
    fn error_code_serde_roundtrip() {
        let codes = [
            ErrorCode::NotFound,
            ErrorCode::AlreadyExists,
            ErrorCode::ValidationError,
            ErrorCode::NotImplemented,
            ErrorCode::InternalError,
        ];
        for code in codes {
            let json = serde_json::to_string(&code).unwrap();
            let back: ErrorCode = serde_json::from_str(&json).unwrap();
            assert_eq!(back, code);
        }
    }

    // --- DomainResult alias ---

    #[test]
    fn domain_result_ok_variant() {
        let r: DomainResult<i32> = Ok(42);
        assert_eq!(r.unwrap(), 42);
    }

    #[test]
    fn domain_result_err_variant() {
        let r: DomainResult<i32> = Err(DomainError::NotImplemented);
        assert!(r.is_err());
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn every_variant() -> Vec<DomainError> {
        vec![
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
            DomainError::LockPoisoned,
            DomainError::InvalidClaim("i".into()),
            DomainError::NoOpTransition,
            DomainError::Other("o".into()),
            DomainError::Agent("a".into()),
            DomainError::Timeout(9),
        ]
    }

    #[test]
    fn all_variants_display_nonempty_and_unique() {
        let mut seen = std::collections::HashSet::new();
        for e in every_variant() {
            let msg = e.to_string();
            assert!(!msg.is_empty(), "empty display for {e:?}");
            seen.insert(msg);
        }
        // Every variant produced a message (18 variants, all distinct here).
        assert_eq!(seen.len(), 18);
    }

    #[test]
    fn domain_error_implements_std_error() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<DomainError>();
        let boxed: Box<dyn std::error::Error> = Box::new(DomainError::NotImplemented);
        assert_eq!(boxed.to_string(), "Not implemented");
    }

    #[test]
    fn error_code_projection_covers_every_variant() {
        let mappings: Vec<(DomainError, ErrorCode)> = vec![
            (DomainError::CycleNotFound("c".into()), ErrorCode::NotFound),
            (DomainError::ModuleNotFound("m".into()), ErrorCode::NotFound),
            (DomainError::FeatureNotFound("f".into()), ErrorCode::NotFound),
            (
                DomainError::WorkPackageNotFound("w".into()),
                ErrorCode::NotFound,
            ),
            (DomainError::NotFound("n".into()), ErrorCode::NotFound),
            (
                DomainError::ModuleHasDependents("m".into()),
                ErrorCode::AlreadyExists,
            ),
            (DomainError::Conflict("c".into()), ErrorCode::AlreadyExists),
            (
                DomainError::Validation("v".into()),
                ErrorCode::ValidationError,
            ),
            (
                DomainError::FeatureNotInModuleScope {
                    feature_slug: "f".into(),
                    module_slug: "m".into(),
                },
                ErrorCode::ValidationError,
            ),
            (
                DomainError::InvalidTransition {
                    from: "a".into(),
                    to: "b".into(),
                    reason: "r".into(),
                },
                ErrorCode::ValidationError,
            ),
            (
                DomainError::InvalidClaim("i".into()),
                ErrorCode::ValidationError,
            ),
            (DomainError::NotImplemented, ErrorCode::NotImplemented),
            (DomainError::NoOpTransition, ErrorCode::ValidationError),
            (DomainError::Storage("s".into()), ErrorCode::InternalError),
            (DomainError::LockPoisoned, ErrorCode::InternalError),
            (DomainError::Other("o".into()), ErrorCode::InternalError),
            (DomainError::Agent("a".into()), ErrorCode::InternalError),
            (DomainError::Timeout(1), ErrorCode::InternalError),
        ];
        assert_eq!(mappings.len(), 18);
        for (err, expected) in mappings {
            let got: ErrorCode = err.into();
            assert_eq!(got, expected);
        }
    }

    #[test]
    fn error_code_serde_wire_strings() {
        assert_eq!(
            serde_json::to_string(&ErrorCode::NotFound).unwrap(),
            "\"NotFound\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::AlreadyExists).unwrap(),
            "\"AlreadyExists\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::ValidationError).unwrap(),
            "\"ValidationError\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::NotImplemented).unwrap(),
            "\"NotImplemented\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::InternalError).unwrap(),
            "\"InternalError\""
        );
    }

    #[test]
    fn error_code_is_copy_and_clone() {
        let a = ErrorCode::NotFound;
        let b = a; // Copy
        #[allow(clippy::clone_on_copy)]
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_ne!(ErrorCode::NotFound, ErrorCode::InternalError);
    }

    #[test]
    fn error_code_debug() {
        assert_eq!(format!("{:?}", ErrorCode::NotFound), "NotFound");
        assert_eq!(format!("{:?}", ErrorCode::InternalError), "InternalError");
    }

    #[test]
    fn timeout_display_includes_seconds() {
        assert_eq!(DomainError::Timeout(0).to_string(), "Timed out after 0 seconds");
        assert_eq!(
            DomainError::Timeout(u64::MAX).to_string(),
            format!("Timed out after {} seconds", u64::MAX)
        );
    }

    #[test]
    fn feature_not_in_module_scope_display_exact() {
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
    fn invalid_transition_display_exact() {
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
    fn no_op_transition_display_exact() {
        assert_eq!(
            DomainError::NoOpTransition.to_string(),
            "No-op transition: already in the requested state"
        );
    }

    #[test]
    fn domain_result_alias_works_with_question_mark() {
        fn inner() -> DomainResult<i32> {
            Err(DomainError::NotFound("x".into()))
        }
        fn outer() -> DomainResult<i32> {
            Ok(inner()? + 1)
        }
        assert!(outer().is_err());
    }

    #[test]
    fn domain_result_alias_ok_path() {
        let r: DomainResult<String> = Ok("ok".to_string());
        assert_eq!(r.unwrap(), "ok");
    }

    #[test]
    fn debug_impls_present_for_errors() {
        for e in every_variant() {
            let dbg = format!("{e:?}");
            assert!(!dbg.is_empty());
        }
    }

    #[test]
    fn error_source_is_none_for_unit_variants() {
        use std::error::Error;
        assert!(DomainError::LockPoisoned.source().is_none());
        assert!(DomainError::NotImplemented.source().is_none());
    }
}
