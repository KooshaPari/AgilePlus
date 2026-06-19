//! Cross-dialect development library for AgilePlus
//!
//! Provides utilities for converting between different configuration formats:
//! - JSON (`.json`)
//! - TOML (`.toml`)
//! - YAML (`.yaml`, `.yml`)
//!
//! Useful for AgilePlus spec files that may be stored in different formats.

pub mod converter;
pub mod dialect;
pub mod error;
pub mod registry;

pub use converter::DialectConverter;
pub use dialect::{Dialect, DialectType};
pub use error::{XddError, XddResult};
pub use registry::DialectRegistry;
