//! Application-layer error type.
//!
//! Never leaks storage implementation details (no sqlx types, no raw DB errors).

use std::error::Error;

use phenotype_error_core::PhenotypeErrorKind as ErrorKind;
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

/// Lift the application error into the canonical Phenotype error kind.
///
/// `AppError::Storage` carries a boxed source; we render that into the
/// `ErrorKind::Storage` payload so observability consumers see the cause
/// without needing to downcast the original.
impl From<AppError> for ErrorKind {
    fn from(err: AppError) -> Self {
        match err {
            AppError::Domain(d) => d.into(),
            AppError::NotFound(s) => Self::NotFound(s),
            AppError::Storage(src) => Self::Storage(src.to_string()),
        }
    }
}

#[cfg(test)]
mod kind_lift_tests {
    use super::*;

    #[test]
    fn not_found_lifts_to_not_found() {
        let k: ErrorKind = AppError::NotFound("user 1".into()).into();
        assert!(matches!(k, ErrorKind::NotFound(s) if s == "user 1"));
    }

    #[test]
    fn domain_validation_chains_through() {
        let app = AppError::Domain(DomainError::Validation("name required".into()));
        let k: ErrorKind = app.into();
        assert!(matches!(k, ErrorKind::Validation(s) if s == "name required"));
    }

    #[test]
    fn storage_lifts_to_storage_with_source_string() {
        let src: Box<dyn Error + Send + Sync> = "db down".to_string().into();
        let k: ErrorKind = AppError::Storage(src).into();
        assert!(matches!(k, ErrorKind::Storage(s) if s == "db down"));
    }
}
