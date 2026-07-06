// SPDX-License-Identifier: MIT OR Apache-2.0
//! `agileplus-domain` — core domain types, error, and port traits.

pub use error::DomainError;
pub use error::ErrorCode;
pub type DomainResult<T> = std::result::Result<T, DomainError>;

pub mod config;
pub mod credentials;
pub mod builder;
pub mod domain;
pub mod error;
pub mod adapters;
pub mod intent_graph;
pub mod ports;
pub mod traceability;
