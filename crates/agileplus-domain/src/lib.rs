//! AgilePlus domain layer — entities, FSM, governance, audit.
//! Implements core business logic with zero I/O dependencies.
//!
//! Traceability: FR-DOMAIN-* / WP01-T002

pub use error::DomainError;
pub type DomainResult<T> = std::result::Result<T, DomainError>;

pub mod config;
pub mod credentials;
pub mod domain;
pub mod error;
pub mod ports;
