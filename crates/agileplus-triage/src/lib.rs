//! agileplus-triage — rule-based triage engine for synced items.
//!
//! # Modules
//! - `engine`: hexagonal classify port — pure `classify(item, rules) -> TriageOutcome`
//! - `classifier`: free-text keyword classifier (legacy / internal use)
//! - `backlog`: in-memory backlog store
//! - `adapter`: high-level orchestration combining classifier + store
//! - `router`: governance file (CLAUDE.md / AGENTS.md) generator
//!
//! Traceability: FR-AGP-017

pub mod adapter;
pub mod backlog;
pub mod classifier;
pub mod engine;
pub mod router;

// Re-export the main engine surface so consumers can do:
//   use agileplus_triage::{SyncedItem, TriageRules, TriageOutcome, classify};
pub use engine::{classify, SyncedItem, TriageOutcome, TriageRule, TriageRules};

// Re-export BacklogStoreOps trait (used by adapter consumers)
pub use adapter::BacklogStoreOps;
