//! `agileplus-domain` — domain types for AgilePlus event sourcing.

pub mod domain;

/// Re-export all domain types at the crate root for ergonomic access.
pub use domain::event::Event;
pub use domain::snapshot::Snapshot;
