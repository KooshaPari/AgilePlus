//! Traceability value objects — lightweight references to external traced artifacts.
//!
//! `TraceRef` is the shared value object used by the [`TraceabilityPort`](crate::ports::traceability_port::TraceabilityPort)
//! to link AgilePlus domain entities (Epic, Story, WorkPackage) to Tracera (or other external)
//! traceability systems.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A reference to a traced artifact in an external traceability system (e.g. Tracera).
///
/// Each `TraceRef` records *when* the link was created (`linked_at`) so that consumers
/// can reason about trace timing without relying on the entity's `updated_at`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceRef {
    /// Unique identifier of the external traced artifact (e.g. Tracera requirement ID).
    pub trace_id: String,
    /// Category / type of the artifact (e.g. `"requirement"`, `"evidence"`, `"specification"`).
    pub artifact_type: String,
    /// Timestamp when the trace link was established.
    pub linked_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_ref_serde_roundtrip() {
        let r = TraceRef {
            trace_id: "TRAC-001".to_string(),
            artifact_type: "requirement".to_string(),
            linked_at: Utc::now(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: TraceRef = serde_json::from_str(&json).unwrap();
        assert_eq!(back.trace_id, "TRAC-001");
        assert_eq!(back.artifact_type, "requirement");
    }

    #[test]
    fn trace_ref_clone_and_debug() {
        let r = TraceRef {
            trace_id: "id".to_string(),
            artifact_type: "evidence".to_string(),
            linked_at: DateTime::from_timestamp(1_000_000, 0).unwrap(),
        };
        let r2 = r.clone();
        assert_eq!(r, r2);
        let dbg = format!("{:?}", r);
        assert!(dbg.contains("TraceRef"));
    }

    #[test]
    fn trace_ref_partial_eq() {
        let ts = DateTime::from_timestamp(1_000_000, 0).unwrap();
        let a = TraceRef { trace_id: "x".to_string(), artifact_type: "y".to_string(), linked_at: ts };
        let b = TraceRef { trace_id: "x".to_string(), artifact_type: "y".to_string(), linked_at: ts };
        assert_eq!(a, b);
    }
}
