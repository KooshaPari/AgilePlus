//! Domain error types.

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
