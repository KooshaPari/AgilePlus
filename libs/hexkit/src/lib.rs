//! Hexkit - Hexagonal Architecture Helpers for AgilePlus
//!
//! This library provides utilities to reduce boilerplate when working with
//! hexagonal architecture patterns in AgilePlus projects.

pub mod builders;
pub mod defaults;
pub mod error;
pub mod helpers;
pub mod registry;

pub use crate::error::{HexkitError, HexkitResult};
pub use builders::{ContextBuilder, PortBuilder};
pub use defaults::{DefaultPort, NoOpPort};
pub use helpers::{retry_on_failure, with_timeout};
pub use registry::{PortKey, PortRegistry};
