//! `agileplus-domain` — core domain types, error, and port traits.

pub use error::DomainError;
pub type DomainResult<T> = std::result::Result<T, DomainError>;

pub mod config;
pub mod credentials;
pub mod domain;
pub mod error;
pub mod ports;
