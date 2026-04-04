//! agileplus-fixtures
//!
//! Test fixtures and seed data for AgilePlus development and testing.
//!
//! This crate provides:
//! - Dogfood seed data for dashboard development
//! - Feature and work package builders for tests
//! - Common test payloads

pub mod builders;
pub mod dogfood;
pub mod payloads;
pub mod test_fixtures;

// Re-export commonly used functions
pub use dogfood::seed_dogfood_features;
