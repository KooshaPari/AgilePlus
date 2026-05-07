//! Plugin Registry for AgilePlus
//!
//! Provides dynamic plugin loading and management capabilities.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    PluginRegistry                            │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
//! │  │  plugin-core│  │  plugin-git │  │plugin-sqlite│         │
//! │  │  (dynamic) │  │  (dynamic)  │  │  (dynamic)  │         │
//! │  └─────────────┘  └─────────────┘  └─────────────┘         │
//! │                                                              │
//! │  discover() → load() → initialize() → shutdown()            │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use plugin_registry::{PluginRegistry, Plugin};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let registry = PluginRegistry::new();
//!     let plugins = registry.list_plugins().await;
//!     println!("Loaded {} plugins", plugins.len());
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod plugin_trait;
pub mod registry;

pub use error::{PluginError, Result};
pub use plugin_trait::{Plugin, PluginConfig, PluginMetadata};
pub use registry::PluginRegistry;
