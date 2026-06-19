//! No-op adapter for [`TraceabilityPort`].
//!
//! Returns `Ok(())` / `Ok(vec![])` for every call. Useful when no external traceability
//! system is connected, in integration tests, or as a default/fallback implementation.

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::ports::traceability_port::TraceabilityPort;
use crate::traceability::TraceRef;

/// A [`TraceabilityPort`] that accepts every link and always returns an empty trace list.
pub struct NoopTraceAdapter;

#[async_trait]
impl TraceabilityPort for NoopTraceAdapter {
    async fn link_trace(&self, _entity_id: Uuid, _trace_ref: TraceRef) -> Result<(), DomainError> {
        Ok(())
    }

    async fn get_traces(&self, _entity_id: Uuid) -> Result<Vec<TraceRef>, DomainError> {
        Ok(vec![])
    }
}
