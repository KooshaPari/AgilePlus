//! agileplus-triage — rule-based classifier, backlog store, and sync adapter.

pub mod adapter;
pub mod backlog;
pub mod classifier;
pub mod router;

pub use adapter::{
    BacklogStoreOps, ClassifyOptions, ClassifyOutcome, TriageAdapter, TriageOp, TriageSource,
};
pub use backlog::{BacklogItem, BacklogPriority, BacklogStatus, BacklogStore, Intent};
pub use agileplus_domain::domain::backlog::BacklogSort;
pub use classifier::{TriageClassifier, TriageResult};
