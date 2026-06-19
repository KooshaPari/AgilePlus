// SPDX-License-Identifier: MIT OR Apache-2.0
//! Domain error types.

use phenotype_error_core::PhenotypeErrorKind as ErrorKind;
use thiserror::Error;

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
            // (InvalidClaim is claim-bound: a precondition failure, same shape
            // as a bad argument — so it projects to ValidationError too.)
            DomainError::Validation(_)
            | DomainError::FeatureNotInModuleScope { .. }
            | DomainError::InvalidTransition { .. }
            | DomainError::InvalidClaim(_) => Self::ValidationError,

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
    fn invalid_claim_projects_to_validation_error() {
        let c: ErrorCode = DomainError::InvalidClaim("not active".into()).into();
        assert_eq!(c, ErrorCode::ValidationError);
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

/// Lift the AgilePlus domain error into the canonical Phenotype error kind.
///
/// This is a *lossy* projection: domain-specific structural data (e.g.
/// `InvalidTransition { from, to, reason }`) is flattened into the kind's
/// display string. The local [`DomainError`] remains the source of truth
/// for cross-crate messaging; [`PhenotypeErrorKind`] is for cross-ecosystem
/// reporting (logs, wire codes, observability).
impl From<DomainError> for ErrorKind {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::FeatureNotInModuleScope {
                feature_slug,
                module_slug,
            } => Self::Domain(format!(
                "feature '{feature_slug}' not in module '{module_slug}'"
            )),
            DomainError::ModuleHasDependents(s) => Self::Conflict(s),
            DomainError::CycleNotFound(s) => Self::NotFound(format!("cycle {s}")),
            DomainError::ModuleNotFound(s) => Self::NotFound(format!("module {s}")),
            DomainError::FeatureNotFound(s) => Self::NotFound(format!("feature {s}")),
            DomainError::WorkPackageNotFound(s) => Self::NotFound(format!("work package {s}")),
            DomainError::NotFound(s) => Self::NotFound(s),
            DomainError::NotImplemented => Self::Domain("not implemented".into()),
            DomainError::Storage(s) => Self::Storage(s),
            DomainError::Validation(s) => Self::Validation(s),
            DomainError::Conflict(s) => Self::Conflict(s),
            DomainError::InvalidTransition { from, to, reason } => {
                Self::Domain(format!("invalid transition from {from} to {to}: {reason}"))
            }
            DomainError::LockPoisoned => Self::Domain("lock poisoned".into()),
        }
    }
}

#[cfg(test)]
mod kind_lift_tests {
    use super::*;

    #[test]
    fn module_not_found_lifts_to_not_found() {
        let k: ErrorKind = DomainError::ModuleNotFound("m-1".into()).into();
        assert!(matches!(k, ErrorKind::NotFound(s) if s == "module m-1"));
    }

    #[test]
    fn work_package_not_found_lifts_to_not_found() {
        let k: ErrorKind = DomainError::WorkPackageNotFound("wp-7".into()).into();
        assert!(matches!(k, ErrorKind::NotFound(s) if s == "work package wp-7"));
    }

    #[test]
    fn cycle_not_found_lifts_to_not_found() {
        let k: ErrorKind = DomainError::CycleNotFound("c-3".into()).into();
        assert!(matches!(k, ErrorKind::NotFound(s) if s == "cycle c-3"));
    }

    #[test]
    fn module_has_dependents_lifts_to_conflict() {
        let k: ErrorKind = DomainError::ModuleHasDependents("m-1".into()).into();
        assert!(matches!(k, ErrorKind::Conflict(s) if s == "m-1"));
    }

    #[test]
    fn invalid_transition_lifts_to_domain_preserving_payload() {
        let k: ErrorKind = DomainError::InvalidTransition {
            from: "draft".into(),
            to: "done".into(),
            reason: "missing review".into(),
        }
        .into();
        let s = k.to_string();
        assert!(s.contains("draft"));
        assert!(s.contains("done"));
        assert!(s.contains("missing review"));
        assert!(matches!(k, ErrorKind::Domain(_)));
    }

    #[test]
    fn validation_lifts_to_validation() {
        let k: ErrorKind = DomainError::Validation("name required".into()).into();
        assert!(matches!(k, ErrorKind::Validation(s) if s == "name required"));
    }

    #[test]
    fn storage_lifts_to_storage() {
        let k: ErrorKind = DomainError::Storage("db down".into()).into();
        assert!(matches!(k, ErrorKind::Storage(s) if s == "db down"));
    }

    #[test]
    fn feature_not_in_module_scope_lifts_to_domain() {
        let k: ErrorKind = DomainError::FeatureNotInModuleScope {
            feature_slug: "f-1".into(),
            module_slug: "m-1".into(),
        }
        .into();
        assert!(matches!(k, ErrorKind::Domain(_)));
        assert!(k.to_string().contains("f-1"));
        assert!(k.to_string().contains("m-1"));
    }

    #[test]
    fn lock_poisoned_lifts_to_domain() {
        let k: ErrorKind = DomainError::LockPoisoned.into();
        assert!(matches!(k, ErrorKind::Domain(_)));
    }

    #[test]
    fn not_implemented_lifts_to_domain() {
        let k: ErrorKind = DomainError::NotImplemented.into();
        assert!(matches!(k, ErrorKind::Domain(_)));
    }
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

/// Lift the AgilePlus domain error into the canonical Phenotype error kind.
///
/// This is a *lossy* projection: domain-specific structural data (e.g.
/// `InvalidTransition { from, to, reason }`) is flattened into the kind's
/// display string. The local [`DomainError`] remains the source of truth
/// for cross-crate messaging; [`PhenotypeErrorKind`] is for cross-ecosystem
/// reporting (logs, wire codes, observability).
impl From<DomainError> for ErrorKind {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::FeatureNotInModuleScope {
                feature_slug,
                module_slug,
            } => Self::Domain(format!(
                "feature '{feature_slug}' not in module '{module_slug}'"
            )),
            DomainError::ModuleHasDependents(s) => Self::Conflict(s),
            DomainError::CycleNotFound(s) => Self::NotFound(format!("cycle {s}")),
            DomainError::ModuleNotFound(s) => Self::NotFound(format!("module {s}")),
            DomainError::FeatureNotFound(s) => Self::NotFound(format!("feature {s}")),
            DomainError::WorkPackageNotFound(s) => Self::NotFound(format!("work package {s}")),
            DomainError::NotFound(s) => Self::NotFound(s),
            DomainError::NotImplemented => Self::Domain("not implemented".into()),
            DomainError::Storage(s) => Self::Storage(s),
            DomainError::Validation(s) => Self::Validation(s),
            DomainError::Conflict(s) => Self::Conflict(s),
            DomainError::InvalidTransition { from, to, reason } => {
                Self::Domain(format!("invalid transition from {from} to {to}: {reason}"))
            }
            DomainError::LockPoisoned => Self::Domain("lock poisoned".into()),
        }
    }
}

#[cfg(test)]
mod kind_lift_tests {
    use super::*;

    #[test]
    fn module_not_found_lifts_to_not_found() {
        let k: ErrorKind = DomainError::ModuleNotFound("m-1".into()).into();
        assert!(matches!(k, ErrorKind::NotFound(s) if s == "module m-1"));
    }

    #[test]
    fn work_package_not_found_lifts_to_not_found() {
        let k: ErrorKind = DomainError::WorkPackageNotFound("wp-7".into()).into();
        assert!(matches!(k, ErrorKind::NotFound(s) if s == "work package wp-7"));
    }

    #[test]
    fn cycle_not_found_lifts_to_not_found() {
        let k: ErrorKind = DomainError::CycleNotFound("c-3".into()).into();
        assert!(matches!(k, ErrorKind::NotFound(s) if s == "cycle c-3"));
    }

    #[test]
    fn module_has_dependents_lifts_to_conflict() {
        let k: ErrorKind = DomainError::ModuleHasDependents("m-1".into()).into();
        assert!(matches!(k, ErrorKind::Conflict(s) if s == "m-1"));
    }

    #[test]
    fn invalid_transition_lifts_to_domain_preserving_payload() {
        let k: ErrorKind = DomainError::InvalidTransition {
            from: "draft".into(),
            to: "done".into(),
            reason: "missing review".into(),
        }
        .into();
        let s = k.to_string();
        assert!(s.contains("draft"));
        assert!(s.contains("done"));
        assert!(s.contains("missing review"));
        assert!(matches!(k, ErrorKind::Domain(_)));
    }

    #[test]
    fn validation_lifts_to_validation() {
        let k: ErrorKind = DomainError::Validation("name required".into()).into();
        assert!(matches!(k, ErrorKind::Validation(s) if s == "name required"));
    }

    #[test]
    fn storage_lifts_to_storage() {
        let k: ErrorKind = DomainError::Storage("db down".into()).into();
        assert!(matches!(k, ErrorKind::Storage(s) if s == "db down"));
    }

    #[test]
    fn feature_not_in_module_scope_lifts_to_domain() {
        let k: ErrorKind = DomainError::FeatureNotInModuleScope {
            feature_slug: "f-1".into(),
            module_slug: "m-1".into(),
        }
        .into();
        assert!(matches!(k, ErrorKind::Domain(_)));
        assert!(k.to_string().contains("f-1"));
        assert!(k.to_string().contains("m-1"));
    }

    #[test]
    fn lock_poisoned_lifts_to_domain() {
        let k: ErrorKind = DomainError::LockPoisoned.into();
        assert!(matches!(k, ErrorKind::Domain(_)));
    }

    #[test]
    fn not_implemented_lifts_to_domain() {
        let k: ErrorKind = DomainError::NotImplemented.into();
        assert!(matches!(k, ErrorKind::Domain(_)));
    }
}
