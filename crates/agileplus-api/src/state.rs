//! Shared application state threaded through every axum handler.
//!
//! Traceability: WP11-T069

use std::sync::Arc;

use agileplus_cache::store::CacheStore;
use agileplus_domain::config::AppConfig;
use agileplus_domain::credentials::CredentialStore;
use agileplus_domain::ports::{
    observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort,
};
use agileplus_events::store::EventStore;
use agileplus_graph::store::GraphStore;
use agileplus_sync::store::SyncMappingStore;
use tokio::sync::broadcast;

/// Broadcast channel capacity for SSE event streaming.
const EVENT_CHANNEL_CAPACITY: usize = 256;

/// Shared state injected into every axum handler via `State<AppState<…>>`.
pub struct AppState<S, V, O>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    pub storage: Arc<S>,
    pub vcs: Arc<V>,
    pub telemetry: Arc<O>,
    pub config: Arc<AppConfig>,
    pub credentials: Arc<dyn CredentialStore>,
    pub event_store: Arc<dyn EventStore>,
    pub cache_store: Arc<dyn CacheStore>,
    pub sync_orchestrator: Arc<dyn SyncMappingStore>,
    pub graph_store: Arc<dyn GraphStore>,
    pub event_tx: broadcast::Sender<serde_json::Value>,
}

impl<S, V, O> Clone for AppState<S, V, O>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            vcs: Arc::clone(&self.vcs),
            telemetry: Arc::clone(&self.telemetry),
            config: Arc::clone(&self.config),
            credentials: Arc::clone(&self.credentials),
            event_store: Arc::clone(&self.event_store),
            cache_store: Arc::clone(&self.cache_store),
            sync_orchestrator: Arc::clone(&self.sync_orchestrator),
            graph_store: Arc::clone(&self.graph_store),
            event_tx: self.event_tx.clone(),
        }
    }
}

impl<S, V, O> AppState<S, V, O>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    pub fn new(
        storage: Arc<S>,
        vcs: Arc<V>,
        telemetry: Arc<O>,
        config: Arc<AppConfig>,
        credentials: Arc<dyn CredentialStore>,
        event_store: Arc<dyn EventStore>,
        cache_store: Arc<dyn CacheStore>,
        sync_orchestrator: Arc<dyn SyncMappingStore>,
        graph_store: Arc<dyn GraphStore>,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self {
            storage,
            vcs,
            telemetry,
            config,
            credentials,
            event_store,
            cache_store,
            sync_orchestrator,
            graph_store,
            event_tx,
        }
    }

    pub fn with_event_tx(
        storage: Arc<S>,
        vcs: Arc<V>,
        telemetry: Arc<O>,
        config: Arc<AppConfig>,
        credentials: Arc<dyn CredentialStore>,
        event_store: Arc<dyn EventStore>,
        cache_store: Arc<dyn CacheStore>,
        sync_orchestrator: Arc<dyn SyncMappingStore>,
        graph_store: Arc<dyn GraphStore>,
        event_tx: broadcast::Sender<serde_json::Value>,
    ) -> Self {
        Self {
            storage,
            vcs,
            telemetry,
            config,
            credentials,
            event_store,
            cache_store,
            sync_orchestrator,
            graph_store,
            event_tx,
        }
    }
}
