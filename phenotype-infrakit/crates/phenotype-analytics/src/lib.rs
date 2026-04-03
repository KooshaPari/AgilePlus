//! Analytics module for phenotype-infrakit
//!
//! Provides event tracking and analytics infrastructure.

pub mod error;
pub mod event;
pub mod track;

#[cfg(feature = "http-client")]
pub mod http_client;

pub use error::{AnalyticsError, Result};
pub use event::Event;
pub use track::Tracker;
