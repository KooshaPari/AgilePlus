//! No‑op adapter for [`TraceabilityPort`].
//!
//! Returns `Ok(())` / `Ok(vec![])` for every call. Useful when no external traceability
//! system is connected, in integration tests, or as a default/fallback implementation.

use async_trait::async_trait;

use crate::error::DomainError;
use crate::ports::traceability_port::TraceabilityPort;
use crate::traceability::TraceRef;

/// A [`TraceabilityPort`] that accepts every link and always returns an empty trace list.
///
/// # Examples
///
/// ```rust,ignore
/// // Doctest ignored: the example body is pre-migration (uses TraceRef.entity_id,
/// // which no longer exists). See the unit tests in tests/traceability_test.rs
/// // (currently `#[ignore]`d as a whole file) for the current API contract.
/// use agileplus_domain::adapters::noop_trace_adapter::NoopTraceAdapter;
/// use agileplus_domain::ports::traceability_port::TraceabilityPort;
/// use agileplus_domain::traceability::TraceRef;
/// use chrono::Utc;
///
/// # tokio_test::block_on(async {
/// let adapter = NoopTraceAdapter;
/// let entity_id = format!("trace-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
/// let trace_ref = TraceRef {
///     trace_id: "FR-001".into(),
///     artifact_type: "requirement".into(),
///     linked_at: Utc::now(),
/// };
///
/// let link = adapter.link_trace(entity_id, trace_ref).await;
/// assert!(link.is_ok());
///
/// let traces = adapter.get_traces(entity_id).await;
/// assert!(traces.is_ok());
/// assert!(traces.unwrap().is_empty());
/// # })
/// ```
pub struct NoopTraceAdapter;

#[async_trait]
impl TraceabilityPort for NoopTraceAdapter {
    async fn link_trace(
        &self,
        _entity_id: String,
        _trace_ref: TraceRef,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn get_traces(&self, _entity_id: String) -> Result<Vec<TraceRef>, DomainError> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[tokio::test]
    async fn link_trace_returns_ok_for_any_input() {
        let adapter = NoopTraceAdapter;
        let r = adapter
            .link_trace(
                "entity-1".to_string(),
                TraceRef {
                    trace_id: "T".to_string(),
                    artifact_type: "requirement".to_string(),
                    linked_at: chrono::Utc::now(),
                },
            )
            .await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn link_trace_is_idempotent() {
        let adapter = NoopTraceAdapter;
        let mk = || TraceRef {
            trace_id: "T".to_string(),
            artifact_type: "requirement".to_string(),
            linked_at: chrono::Utc::now(),
        };
        assert!(adapter.link_trace("e".to_string(), mk()).await.is_ok());
        assert!(adapter.link_trace("e".to_string(), mk()).await.is_ok());
    }

    #[tokio::test]
    async fn get_traces_returns_empty_vec() {
        let adapter = NoopTraceAdapter;
        let traces = adapter.get_traces("anything".to_string()).await.unwrap();
        assert!(traces.is_empty());
    }

    #[tokio::test]
    async fn get_traces_ignores_empty_entity_id() {
        let adapter = NoopTraceAdapter;
        assert!(adapter.get_traces(String::new()).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn works_as_trait_object() {
        let adapter: Box<dyn TraceabilityPort> = Box::new(NoopTraceAdapter);
        assert!(adapter.get_traces("x".to_string()).await.unwrap().is_empty());
        let link = adapter
            .link_trace(
                "x".to_string(),
                TraceRef {
                    trace_id: "id".to_string(),
                    artifact_type: "spec".to_string(),
                    linked_at: chrono::Utc::now(),
                },
            )
            .await;
        assert!(link.is_ok());
    }

    #[tokio::test]
    async fn concurrent_links_are_all_ok() {
        let adapter = std::sync::Arc::new(NoopTraceAdapter);
        let mut handles = Vec::new();
        for i in 0..8 {
            let a = adapter.clone();
            handles.push(tokio::spawn(async move {
                a.link_trace(
                    format!("e-{i}"),
                    TraceRef {
                        trace_id: format!("T-{i}"),
                        artifact_type: "requirement".to_string(),
                        linked_at: chrono::Utc::now(),
                    },
                )
                .await
            }));
        }
        for h in handles {
            assert!(h.await.unwrap().is_ok());
        }
    }
}
