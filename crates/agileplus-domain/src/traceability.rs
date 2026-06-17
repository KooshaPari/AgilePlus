//! Traceability value objects — lightweight references to external traced artifacts.
//!
//! `TraceRef` is the shared value object used by the [`TraceabilityPort`](crate::ports::traceability_port::TraceabilityPort)
//! to link AgilePlus domain entities (Epic, Story, WorkPackage) to Tracera (or other external)
//! traceability systems.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A reference to a traced artifact in an external traceability system (e.g. Tracera).
///
/// Each `TraceRef` records *when* the link was created (`linked_at`) so that consumers
/// can reason about trace timing without relying on the entity's `updated_at`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceRef {
    /// UUID of the AgilePlus domain entity this trace link belongs to (e.g. Epic, Story, WorkPackage).
    pub entity_id: Uuid,
    /// Unique identifier of the external traced artifact (e.g. Tracera requirement ID).
    pub trace_id: String,
    /// Category / type of the artifact (e.g. `"requirement"`, `"evidence"`, `"specification"`).
    pub artifact_type: String,
    /// Timestamp when the trace link was established.
    pub linked_at: DateTime<Utc>,
}

impl TraceRef {
    /// Construct a new `TraceRef`, stamping `linked_at` to the current UTC time.
    pub fn new(entity_id: Uuid, trace_id: impl Into<String>, artifact_type: impl Into<String>) -> Self {
        Self {
            entity_id,
            trace_id: trace_id.into(),
            artifact_type: artifact_type.into(),
            linked_at: Utc::now(),
        }
    }
}
