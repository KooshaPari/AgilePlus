//! AgilePlus Triage & Backlog adapter.
//!
//! Provides rule-based intent classification, backlog item management,
//! and CLAUDE.md/AGENTS.md prompt router generation.
//!
//! Traceability: FR-040, FR-041, FR-042, FR-046, FR-047, FR-048, FR-049, FR-050 / WP17

pub mod adapter;
pub mod backlog;
pub mod classifier;
pub mod router;

pub use adapter::{
    BacklogStoreOps, ClassifyOptions, ClassifyOutcome, TriageAdapter, TriageOp, TriageSource,
};
pub use agileplus_domain::domain::backlog::{BacklogItem, BacklogPriority, BacklogStatus, Intent};
pub use classifier::{TriageClassifier, TriageResult};
pub use router::RouterGenerator;
