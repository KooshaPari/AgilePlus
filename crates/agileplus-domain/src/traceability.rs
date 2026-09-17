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

#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn ts(secs: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(secs, 0).unwrap()
    }

    #[test]
    fn trace_ref_field_access() {
        let r = TraceRef {
            trace_id: "TRAC-9".to_string(),
            artifact_type: "specification".to_string(),
            linked_at: ts(1234),
        };
        assert_eq!(r.trace_id, "TRAC-9");
        assert_eq!(r.artifact_type, "specification");
        assert_eq!(r.linked_at, ts(1234));
    }

    #[test]
    fn trace_ref_inequality_on_each_field() {
        let base = TraceRef {
            trace_id: "a".to_string(),
            artifact_type: "b".to_string(),
            linked_at: ts(1),
        };
        let mut other = base.clone();
        other.trace_id = "z".to_string();
        assert_ne!(base, other);

        let mut other = base.clone();
        other.artifact_type = "z".to_string();
        assert_ne!(base, other);

        let mut other = base.clone();
        other.linked_at = ts(2);
        assert_ne!(base, other);
    }

    #[test]
    fn trace_ref_debug_contains_fields() {
        let r = TraceRef {
            trace_id: "TRAC-XYZ".to_string(),
            artifact_type: "requirement".to_string(),
            linked_at: ts(0),
        };
        let dbg = format!("{r:?}");
        assert!(dbg.contains("TRAC-XYZ"));
        assert!(dbg.contains("requirement"));
    }

    #[test]
    fn trace_ref_empty_strings_are_allowed() {
        let r = TraceRef {
            trace_id: String::new(),
            artifact_type: String::new(),
            linked_at: ts(0),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: TraceRef = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn trace_ref_json_shape() {
        let r = TraceRef {
            trace_id: "T1".to_string(),
            artifact_type: "evidence".to_string(),
            linked_at: ts(0),
        };
        let v: serde_json::Value = serde_json::to_value(&r).unwrap();
        assert_eq!(v["trace_id"], "T1");
        assert_eq!(v["artifact_type"], "evidence");
        assert!(v["linked_at"].is_string());
    }

    #[test]
    fn trace_ref_vec_serde_roundtrip() {
        let refs = vec![
            TraceRef {
                trace_id: "T1".to_string(),
                artifact_type: "requirement".to_string(),
                linked_at: ts(1),
            },
            TraceRef {
                trace_id: "T2".to_string(),
                artifact_type: "evidence".to_string(),
                linked_at: ts(2),
            },
        ];
        let json = serde_json::to_string(&refs).unwrap();
        let back: Vec<TraceRef> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, refs);
    }
}
