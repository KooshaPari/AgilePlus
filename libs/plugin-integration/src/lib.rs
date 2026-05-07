//! Integration layer bridging plugin-registry with external agileplus-plugin-core.
//!
//! This crate provides the bridge between our internal plugin-registry and
//! the external agileplus-plugin-core repository.
//!
//! ## Architecture
//!
//! ```text
//! +---------------------------+     +---------------------------+
//! |   External Repos          |     |   Internal Registry       |
//! |   (agileplus-plugin-core, | --> |   (plugin-registry)       |
//! |    -git, -sqlite)         |     |                           |
//! +---------------------------+     +---------------------------+
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use plugin_integration::UnifiedPluginRegistry;
//! use plugin_registry::plugin_trait::PluginConfig;
//!
//! #[tokio::main]
//! async fn main() {
//!     let registry = UnifiedPluginRegistry::new();
//!     // Load and manage plugins
//! }
//! ```

mod unified_registry;

pub use unified_registry::UnifiedPluginRegistry;

// Re-export types from plugin-registry for convenience
pub use plugin_registry::{
    error::PluginError,
    plugin_trait::{Plugin, PluginConfig, PluginMetadata},
    registry::PluginRegistry,
};
