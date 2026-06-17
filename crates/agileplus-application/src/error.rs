//! Application-layer error type.
//!
//! Never leaks storage implementation details (no sqlx types, no raw DB errors).

use std::error::Error;

use phenotype_error_core::ErrorCode;
use thiserror::Error;

use agileplus_domain::error::DomainError;

#[derive(Debug, Error)]
pub enum AppError {
    /// Domain invariant violation — validation, invalid transition, etc.
    #[error(transparent)]
    Domain(#[from] DomainError),

    /// Entity looked up by id / slug does not exist.
    #[error("not found: {0}")]
    NotFound(String),

    /// Persistence failure, wrapped without leaking implementation types.
    #[error("storage error")]
    Storage(#[source] Box<dyn Error + Send + Sync>),
}

/// Project the application error onto the canonical Phenotype wire
/// [`ErrorCode`].
///
/// Lossy by design: `AppError`'s boxed source / messages remain the source of
/// truth for human-facing reporting, while [`ErrorCode`] is the stable,
/// language-agnostic code for observability and wire responses.
impl From<AppError> for ErrorCode {
    fn from(err: AppError) -> Self {
        match err {
            // `agileplus-domain` is zero-dependency, so it projects onto its own
            // local `ErrorCode` mirror; remap that onto the canonical wire enum.
            AppError::Domain(d) => match agileplus_domain::error::ErrorCode::from(d) {
                agileplus_domain::error::ErrorCode::InternalError => Self::InternalError,
                agileplus_domain::error::ErrorCode::NotFound => Self::NotFound,
                agileplus_domain::error::ErrorCode::AlreadyExists => Self::AlreadyExists,
                agileplus_domain::error::ErrorCode::NotImplemented => Self::NotImplemented,
                agileplus_domain::error::ErrorCode::ValidationError => Self::ValidationError,
            },
            AppError::NotFound(_) => Self::NotFound,
            AppError::Storage(_) => Self::InternalError,
        }
    }
}

#[cfg(test)]
mod code_projection_tests {
    use super::*;

    #[test]
    fn not_found_projects_to_not_found() {
        let c: ErrorCode = AppError::NotFound("user 1".into()).into();
        assert_eq!(c, ErrorCode::NotFound);
    }

    #[test]
    fn domain_validation_chains_through() {
        let app = AppError::Domain(DomainError::Validation("name required".into()));
        let c: ErrorCode = app.into();
        assert_eq!(c, ErrorCode::ValidationError);
    }

    #[test]
    fn storage_projects_to_internal_error() {
        let src: Box<dyn Error + Send + Sync> = "db down".to_string().into();
        let c: ErrorCode = AppError::Storage(src).into();
        assert_eq!(c, ErrorCode::InternalError);
    }
}
