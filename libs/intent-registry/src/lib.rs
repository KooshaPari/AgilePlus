//! Intent-driven service registry for AgilePlus
//!
//! This module provides an intent-based service discovery system where services
//! are registered by capability intent rather than by name, enabling dynamic
//! and flexible service composition.

pub mod error;
pub mod intent;
pub mod registry;
pub mod resolver;
pub mod service;

pub use error::IntentRegistryError;
pub use intent::{Capability, ServiceIntent};
pub use registry::IntentRegistry;
pub use resolver::IntentResolver;
pub use service::{RegisteredService, ServiceMetadata};
