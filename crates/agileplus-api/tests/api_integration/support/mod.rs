#![allow(dead_code)]

pub(crate) mod observability;
mod storage;
mod storage_port_impl;
pub(crate) mod vcs;

use std::sync::Arc;

use agileplus_api::{AppState, create_router};
use agileplus_domain::config::AppConfig;
use agileplus_domain::credentials::{CredentialStore, InMemoryCredentialStore, keys as cred_keys};
use axum_test::TestServer;
use tokio::sync::broadcast;

use self::observability::MockObs;
pub(crate) use self::storage::MockStorage;
pub(crate) use self::vcs::MockVcs;

pub(crate) const TEST_API_KEY: &str = "test-api-key-12345";

/// SSE broadcast capacity used when a test does not care about it.
pub(crate) const EVENT_CHANNEL_CAPACITY: usize = 256;

/// Credential store seeded with [`TEST_API_KEY`].
pub(crate) fn test_credentials() -> Arc<dyn CredentialStore> {
    let creds_inner = InMemoryCredentialStore::new();
    creds_inner
        .set(
            "agileplus",
            cred_keys::API_KEYS,
            &agileplus_domain::credentials::format_api_key_hash(TEST_API_KEY),
        )
        .expect("setting test API key should succeed");
    Arc::new(creds_inner)
}

/// Build the application state together with a handle to the SSE broadcast
/// sender, so tests can publish domain events and observe the streaming
/// endpoint forward them.
pub(crate) fn setup_state_with_storage(
    storage: MockStorage,
    event_capacity: usize,
) -> (
    AppState<MockStorage, MockVcs, MockObs>,
    broadcast::Sender<serde_json::Value>,
) {
    setup_state_with_ports(storage, MockVcs::new(), event_capacity)
}

/// Build the application state with an explicitly configured [`MockVcs`], so a
/// test can inject port failures or canned results and still keep the handle
/// needed to inspect the recorded calls.
pub(crate) fn setup_state_with_ports(
    storage: MockStorage,
    vcs: MockVcs,
    event_capacity: usize,
) -> (
    AppState<MockStorage, MockVcs, MockObs>,
    broadcast::Sender<serde_json::Value>,
) {
    let (event_tx, _) = broadcast::channel(event_capacity);
    let state = AppState::with_event_tx(
        Arc::new(storage),
        Arc::new(vcs),
        Arc::new(MockObs),
        Arc::new(AppConfig::default()),
        test_credentials(),
        event_tx.clone(),
    );
    (state, event_tx)
}

pub(crate) async fn setup_test_server() -> TestServer {
    setup_test_server_with_storage(MockStorage::with_test_data()).await
}

pub(crate) async fn setup_test_server_with_storage(storage: MockStorage) -> TestServer {
    setup_test_server_with_ports(storage, MockVcs::new()).await
}

/// A server whose `VcsPort` is the caller's mock, so the test can both drive
/// failures and read back the calls the handlers made.
pub(crate) async fn setup_test_server_with_ports(storage: MockStorage, vcs: MockVcs) -> TestServer {
    let (state, _event_tx) = setup_state_with_ports(storage, vcs, EVENT_CHANNEL_CAPACITY);
    TestServer::new(create_router(state))
}
