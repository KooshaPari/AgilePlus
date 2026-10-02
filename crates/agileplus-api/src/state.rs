// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared application state threaded through every axum handler.
//! Traceability: WP11-T069

use agileplus_application::use_cases::{
    advance_feature::AdvanceFeature, create_epic::CreateEpic, create_feature::CreateFeature,
    create_story::CreateStory, transition_story::TransitionStory,
};
use agileplus_domain::config::AppConfig;
use agileplus_domain::credentials::CredentialStore;
use agileplus_domain::ports::vcs::VcsPort;
use agileplus_domain::ports::{ObservabilityPort, StoragePort, execution::AtomicAcceptancePort};
use std::sync::Arc;
use tokio::sync::broadcast;

const EVENT_CHANNEL_CAPACITY: usize = 256;

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
    pub event_tx: broadcast::Sender<serde_json::Value>,
    /// Embedders without an atomic adapter remain fail-closed.
    pub atomic_acceptance: Option<Arc<dyn AtomicAcceptancePort>>,
    pub create_feature_uc: Arc<CreateFeature>,
    pub advance_feature_uc: Arc<AdvanceFeature>,
    pub create_story_uc: Arc<CreateStory>,
    pub transition_story_uc: Arc<TransitionStory>,
    pub create_epic_uc: Arc<CreateEpic>,
}

impl<S, V, O> Clone for AppState<S, V, O>
where
    S: StoragePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            vcs: self.vcs.clone(),
            telemetry: self.telemetry.clone(),
            config: self.config.clone(),
            credentials: self.credentials.clone(),
            event_tx: self.event_tx.clone(),
            atomic_acceptance: self.atomic_acceptance.clone(),
            create_feature_uc: self.create_feature_uc.clone(),
            advance_feature_uc: self.advance_feature_uc.clone(),
            create_story_uc: self.create_story_uc.clone(),
            transition_story_uc: self.transition_story_uc.clone(),
            create_epic_uc: self.create_epic_uc.clone(),
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
    ) -> Self {
        let (event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self::with_event_tx(storage, vcs, telemetry, config, credentials, event_tx)
    }

    pub fn with_event_tx(
        storage: Arc<S>,
        vcs: Arc<V>,
        telemetry: Arc<O>,
        config: Arc<AppConfig>,
        credentials: Arc<dyn CredentialStore>,
        event_tx: broadcast::Sender<serde_json::Value>,
    ) -> Self {
        let publisher: Arc<dyn agileplus_domain::ports::events::DomainEventPublisher> =
            Arc::new(NoOpPublisher);
        let create_feature_uc = Arc::new(CreateFeature::new(storage.clone(), publisher.clone()));
        let advance_feature_uc = Arc::new(AdvanceFeature::new(storage.clone(), publisher.clone()));
        let create_story_uc = Arc::new(CreateStory::new(storage.clone(), publisher.clone()));
        let transition_story_uc =
            Arc::new(TransitionStory::new(storage.clone(), publisher.clone()));
        let create_epic_uc = Arc::new(CreateEpic::new(storage.clone(), publisher));
        Self {
            storage,
            vcs,
            telemetry,
            config,
            credentials,
            event_tx,
            atomic_acceptance: None,
            create_feature_uc,
            advance_feature_uc,
            create_story_uc,
            transition_story_uc,
            create_epic_uc,
        }
    }
}

impl<S, V, O> AppState<S, V, O>
where
    S: StoragePort + AtomicAcceptancePort + Send + Sync + 'static,
    V: VcsPort + Send + Sync + 'static,
    O: ObservabilityPort + Send + Sync + 'static,
{
    /// Use the same concrete backend as all other application reads.
    pub fn with_atomic_acceptance(mut self) -> Self {
        self.atomic_acceptance = Some(self.storage.clone());
        self
    }
}

struct NoOpPublisher;
impl agileplus_domain::ports::events::DomainEventPublisher for NoOpPublisher {
    fn publish(
        &self,
        _event: agileplus_domain::ports::events::DomainEvent,
    ) -> Result<(), agileplus_domain::error::DomainError> {
        Ok(())
    }
}
