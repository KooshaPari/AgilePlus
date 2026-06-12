//! Application-layer error type.
//!
//! Never leaks storage implementation details (no sqlx types, no raw DB errors).

use std::error::Error;

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
