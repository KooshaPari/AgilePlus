//! Integration tests for traceability port and adapters.

use agileplus_domain::traceability::{NoopTraceAdapter, TraceRef, TraceabilityPort};
use uuid::Uuid;

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
    assert_eq!(result.unwrap(), vec![]);
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
