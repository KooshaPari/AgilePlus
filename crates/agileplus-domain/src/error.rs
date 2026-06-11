//! Domain error types.

use phenotype_error_core::PhenotypeErrorKind as ErrorKind;
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
