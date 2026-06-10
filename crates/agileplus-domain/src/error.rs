//! Domain error types.

use phenotype_error_core::ErrorCode;
use thiserror::Error;

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
}

/// Project the AgilePlus domain error onto the canonical Phenotype wire
/// [`ErrorCode`].
///
/// This is a *lossy* classification: the structural payload (slugs, transition
/// detail, free-text messages) is dropped — the local [`DomainError`] remains
/// the source of truth for human-facing messaging, while [`ErrorCode`] is the
/// stable, language-agnostic code used for cross-ecosystem reporting (logs,
/// wire responses, observability, Rust↔TS parity).
impl From<DomainError> for ErrorCode {
    fn from(err: DomainError) -> Self {
        match err {
            // "not found" family
            DomainError::CycleNotFound(_)
            | DomainError::ModuleNotFound(_)
            | DomainError::FeatureNotFound(_)
            | DomainError::WorkPackageNotFound(_)
            | DomainError::NotFound(_) => Self::NotFound,

            // conflicts / already-exists
            DomainError::ModuleHasDependents(_) | DomainError::Conflict(_) => Self::AlreadyExists,

            // validation / bad input (scope + state-machine violations are
            // invalid-argument-shaped from the caller's perspective)
            DomainError::Validation(_)
            | DomainError::FeatureNotInModuleScope { .. }
            | DomainError::InvalidTransition { .. } => Self::ValidationError,

            DomainError::NotImplemented => Self::NotImplemented,

            // infrastructure / internal faults
            DomainError::Storage(_) | DomainError::LockPoisoned => Self::InternalError,
        }
    }
}

#[cfg(test)]
mod code_projection_tests {
    use super::*;

    #[test]
    fn module_not_found_projects_to_not_found() {
        let c: ErrorCode = DomainError::ModuleNotFound("m-1".into()).into();
        assert_eq!(c, ErrorCode::NotFound);
    }

    #[test]
    fn work_package_not_found_projects_to_not_found() {
        let c: ErrorCode = DomainError::WorkPackageNotFound("wp-7".into()).into();
        assert_eq!(c, ErrorCode::NotFound);
    }

    #[test]
    fn cycle_and_feature_not_found_project_to_not_found() {
        let c: ErrorCode = DomainError::CycleNotFound("c-3".into()).into();
        assert_eq!(c, ErrorCode::NotFound);
        let c: ErrorCode = DomainError::FeatureNotFound("f-9".into()).into();
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
        // The structural payload is preserved on DomainError itself even though
        // the ErrorCode projection is lossy.
        let e = DomainError::InvalidTransition {
            from: "draft".into(),
            to: "done".into(),
            reason: "missing review".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("draft") && msg.contains("done") && msg.contains("missing review"));
    }
}
