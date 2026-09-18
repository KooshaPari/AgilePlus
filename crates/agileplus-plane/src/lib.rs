//! AgilePlus Plane.so sync adapter.
//!
//! Bidirectional sync between AgilePlus entities and Plane.so issues.
//! Supports webhook ingestion, outbound push, state mapping, label sync,
//! content-hash conflict detection, and a bounded retry queue.
//!
//! Traceability: FR-051 / WP08

pub mod client;
pub mod content_hash;
pub mod daemon;
pub mod inbound;
pub mod labels;
pub mod outbound;
pub mod runtime;
pub mod state_mapper;
pub mod sync;
pub mod sync_queue;
pub mod webhook;

#[cfg(test)]
mod extended_tests;

#[cfg(test)]
pub mod mock_storage;

pub use client::PlaneClient;
pub use client::{
    PlaneCreateCycleRequest, PlaneCreateModuleRequest, PlaneCycleResponse, PlaneModuleResponse,
};
pub use content_hash::{ConflictStatus, compute_content_hash, detect_conflict};
pub use inbound::{InboundOutcome, InboundSync, LocalEntityStore};
pub use labels::{LabelSync, PlaneLabel};
pub use outbound::{
    OutboundSync, push_cycle, push_cycle_delete, push_feature_cycle_assignment,
    push_feature_module_assignment, push_module, push_module_delete,
};
pub use runtime::*;
pub use state_mapper::{PlaneStateMapper, PlaneStateMapperConfig};
pub use sync::{PlaneSyncAdapter, SyncState};
pub use sync_queue::{MAX_RETRIES, SyncOpKind, SyncQueue, SyncQueueItem, SyncQueueStore, SyncTask};
pub use webhook::{
    PlaneInboundEvent, PlaneWebhookAction, PlaneWebhookCycle, PlaneWebhookModule,
    PlaneWebhookPayload, handle_plane_webhook, parse_webhook, verify_hmac_signature,
    verify_webhook_signature,
};

/// Test-only helpers shared by the crate's unit tests.
#[cfg(test)]
pub(crate) mod test_env {
    use std::sync::{Mutex, MutexGuard};

    /// Serializes every unit test that mutates process-global `PLANE_*`
    /// environment variables.
    ///
    /// `set_var`/`remove_var` are process-global, and cargo runs the tests of a
    /// single binary on parallel threads, so tests that read `PLANE_*` (e.g.
    /// `runtime::plane_client_from_env`) will observe whatever a concurrent test
    /// just wrote unless they all take this lock.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    /// Acquire the environment lock.
    ///
    /// Poisoning is ignored on purpose: one failing test must not turn every
    /// other environment test into a spurious failure.
    pub(crate) fn lock_env() -> MutexGuard<'static, ()> {
        ENV_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
