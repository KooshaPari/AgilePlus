//! Traceability integration — port for linking AgilePlus domain entities to Tracera.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A reference to a traced artifact in Tracera (or another external traceability system).
/// Allows domain entities (Epic, Story, Task) to carry links back to requirement/evidence artifacts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceRef {
    /// Unique identifier of the external traced artifact (e.g. Tracera requirement ID, UUID).
    pub trace_id: String,
    /// Type of artifact (e.g. "requirement", "evidence", "specification").
    pub artifact_type: String,
    /// The local entity (Epic, Story, Task) being traced.
    pub entity_id: Uuid,
}

/// Hexagonal port: traceability service integration.
/// Enables AgilePlus domain entities to link with and query external traceability artifacts.
#[async_trait]
pub trait TraceabilityPort: Send + Sync {
    /// Create or update a link from a domain entity to a traced artifact.
    /// Returns `Ok(())` on success, `Err(msg)` on failure.
    async fn link_trace(&self, entity_id: Uuid, trace_ref: TraceRef) -> Result<(), String>;

    /// Retrieve all trace links for a given domain entity.
    /// Returns empty vec if no traces exist, not an error.
    async fn get_traces(&self, entity_id: Uuid) -> Result<Vec<TraceRef>, String>;
}

/// A no-op implementation of the traceability port.
/// Used when no external traceability system is connected, or for testing.
pub struct NoopTraceAdapter;

#[async_trait]
impl TraceabilityPort for NoopTraceAdapter {
    async fn link_trace(&self, _entity_id: Uuid, _trace_ref: TraceRef) -> Result<(), String> {
        Ok(())
    }

    async fn get_traces(&self, _entity_id: Uuid) -> Result<Vec<TraceRef>, String> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_ref_serialize_roundtrip() {
        let entity_id = Uuid::new_v4();
        let trace_ref = TraceRef {
            trace_id: "FR-001".to_string(),
            artifact_type: "requirement".to_string(),
            entity_id,
        };

        let json = serde_json::to_string(&trace_ref).expect("serialize");
        let deserialized: TraceRef = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(trace_ref, deserialized);
    }

    #[tokio::test]
    async fn test_noop_link_trace_ok() {
        let adapter = NoopTraceAdapter;
        let entity_id = Uuid::new_v4();
        let trace_ref = TraceRef {
            trace_id: "FR-002".to_string(),
            artifact_type: "requirement".to_string(),
            entity_id,
        };

        let result = adapter.link_trace(entity_id, trace_ref).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_noop_get_traces_empty() {
        let adapter = NoopTraceAdapter;
        let entity_id = Uuid::new_v4();

        let result = adapter.get_traces(entity_id).await;
        assert!(result.is_ok());
        assert_eq!(result.expect("domain operation"), vec![]);
    }

    #[test]
    fn test_trace_ref_clone() {
        let entity_id = Uuid::new_v4();
        let trace_ref = TraceRef {
            trace_id: "FR-003".to_string(),
            artifact_type: "evidence".to_string(),
            entity_id,
        };

        let cloned = trace_ref.clone();
        assert_eq!(trace_ref, cloned);
    }

    #[test]
    fn test_trace_ref_eq() {
        let entity_id = Uuid::new_v4();
        let trace_ref_1 = TraceRef {
            trace_id: "FR-004".to_string(),
            artifact_type: "specification".to_string(),
            entity_id,
        };
        let trace_ref_2 = TraceRef {
            trace_id: "FR-004".to_string(),
            artifact_type: "specification".to_string(),
            entity_id,
        };

        assert_eq!(trace_ref_1, trace_ref_2);
    }
}
