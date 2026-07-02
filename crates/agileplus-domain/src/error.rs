// SPDX-License-Identifier: MIT OR Apache-2.0
//! Domain error types.

use thiserror::Error;

/// Canonical error code for cross-ecosystem observability and wire responses.
/// Maps AgilePlus domain and application errors to stable, language-agnostic codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ErrorCode {
    /// Request succeeded; used as default OK status.
    Ok = 0,
    /// Request cancelled by client or service.
    Cancelled = 1,
    /// Unknown error (unmapped or internal).
    Unknown = 2,
    /// Client provided invalid argument(s).
    InvalidArgument = 3,
    /// Deadline exceeded (timeout).
    DeadlineExceeded = 4,
    /// Resource not found.
    NotFound = 5,
    /// Resource already exists.
    AlreadyExists = 6,
    /// Caller lacks permission.
    PermissionDenied = 7,
    /// Resource exhausted (quota, limits).
    ResourceExhausted = 8,
    /// Precondition failed.
    FailedPrecondition = 9,
    /// Request aborted.
    Aborted = 10,
    /// Operation out of order or invalid state.
    OutOfRange = 11,
    /// Operation not implemented.
    NotImplemented = 12,
    /// Internal server error.
    InternalError = 13,
    /// Service unavailable.
    Unavailable = 14,
    /// Data loss or corruption.
    DataLoss = 15,
    /// Authentication failed.
    Unauthenticated = 16,
    /// Validation failed (domain invariant, schema, business rules).
    ValidationError = 100,
}

/// Wire envelope for errors — used in event bus, API responses, and cross-repo communication.
/// Provides a stable, language-agnostic payload for error reporting and observability.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorEnvelope {
    /// Machine-readable error code.
    pub code: ErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Whether this error is transient and safe to retry.
    #[serde(default)]
    pub retryable: bool,
}

impl ErrorEnvelope {
    /// Create a new error envelope.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: false,
        }
    }

    /// Mark this error as retryable.
    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }
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

            DomainError::Storage(_) | DomainError::LockPoisoned => Self::InternalError,
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
}