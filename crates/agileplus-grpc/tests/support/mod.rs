//! Shared fixtures for the gRPC RPC-handler integration tests.
//!
//! Each harness owns an isolated in-memory SQLite store plus stub downstream
//! ports, so the handler bodies in `server::mod` can be driven over real
//! persistence without opening a socket or touching the filesystem.
//!
//! Traceability: WP14-T079, T080b

#![allow(dead_code)] // every test binary uses a different subset of these helpers

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::governance::{
    Evidence, EvidenceType, GovernanceContract, GovernanceRule,
};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{DependencyType, WorkPackage, WpDependency, WpState};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::agent::{AgentConfig, AgentResult, AgentStatus, AgentTask};
use agileplus_domain::ports::observability::{LogEntry, ObservabilityPort, SpanContext};
use agileplus_domain::ports::review::{CiStatus, PrInfo, ReviewComment, ReviewStatus};
use agileplus_domain::ports::{AgentPort, ReviewPort, StoragePort};
use agileplus_git::GitVcsAdapter;
use agileplus_grpc::event_bus::EventBus;
use agileplus_grpc::proxy::ProxyRouter;
use agileplus_grpc::server::AgilePlusCoreServer;
use agileplus_proto::agileplus::v1::{CommandRequest, ProjectScope};
use agileplus_sqlite::SqliteStorageAdapter;
use chrono::{DateTime, Utc};

/// Canonical repository root the test server is bound to.
///
/// Scope validation is a string comparison, so this path never has to exist.
pub const REPO_ROOT: &str = "/repo/agileplus-core";

/// Agent port with no downstream service: every call reports `NotImplemented`.
#[derive(Debug, Default)]
pub struct UnavailableAgent;

impl AgentPort for UnavailableAgent {
    async fn dispatch(&self, _: AgentTask, _: &AgentConfig) -> Result<AgentResult, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn dispatch_async(&self, _: AgentTask, _: &AgentConfig) -> Result<String, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn query_status(&self, _: &str) -> Result<AgentStatus, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn cancel(&self, _: &str) -> Result<(), DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn send_instruction(&self, _: &str, _: &str) -> Result<(), DomainError> {
        Err(DomainError::NotImplemented)
    }
}

/// Review port with no downstream service.
#[derive(Debug, Default)]
pub struct UnavailableReview;

impl ReviewPort for UnavailableReview {
    async fn get_review_status(&self, _: &str) -> Result<ReviewStatus, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn get_review_comments(&self, _: &str) -> Result<Vec<ReviewComment>, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn get_actionable_comments(&self, _: &str) -> Result<Vec<ReviewComment>, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn get_ci_status(&self, _: &str) -> Result<CiStatus, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn get_pr_info(&self, _: &str) -> Result<PrInfo, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn await_review(&self, _: &str, _: u64) -> Result<ReviewStatus, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn await_ci(&self, _: &str, _: u64) -> Result<CiStatus, DomainError> {
        Err(DomainError::NotImplemented)
    }
}

/// Telemetry port that discards everything (no exporter in tests).
#[derive(Debug, Default)]
pub struct LogOnlyObservability;

impl ObservabilityPort for LogOnlyObservability {
    fn start_span(&self, _: &str, _: Option<&SpanContext>) -> SpanContext {
        SpanContext {
            trace_id: String::new(),
            span_id: String::new(),
            parent_span_id: None,
        }
    }

    fn end_span(&self, _: &SpanContext) {}

    fn add_span_event(&self, _: &SpanContext, _: &str, _: &[(&str, &str)]) {}

    fn set_span_error(&self, _: &SpanContext, _: &str) {}

    fn record_counter(&self, _: &str, _: u64, _: &[(&str, &str)]) {}

    fn record_histogram(&self, _: &str, _: f64, _: &[(&str, &str)]) {}

    fn record_gauge(&self, _: &str, _: f64, _: &[(&str, &str)]) {}

    fn log(&self, _: &LogEntry) {}

    fn log_info(&self, _: &str) {}

    fn log_warn(&self, _: &str) {}

    fn log_error(&self, _: &str) {}
}

/// The service under test, wired with the fixture ports above.
pub type TestServer = AgilePlusCoreServer<
    SqliteStorageAdapter,
    GitVcsAdapter,
    UnavailableAgent,
    UnavailableReview,
    LogOnlyObservability,
>;

/// Fixture holding the server, its storage, and the event bus it publishes on.
pub struct Harness {
    pub server: TestServer,
    pub storage: Arc<SqliteStorageAdapter>,
    pub bus: Arc<EventBus>,
}

impl Harness {
    /// Build an isolated server backed by an in-memory store.
    pub async fn new() -> Self {
        Self::with_downstream(None, None).await
    }

    /// Build a server whose proxy router targets the given downstream
    /// addresses (`None` = stub mode for that service).
    pub async fn with_downstream(agents: Option<String>, integrations: Option<String>) -> Self {
        let storage = Arc::new(
            SqliteStorageAdapter::in_memory().expect("in-memory storage adapter should open"),
        );
        let bus = Arc::new(EventBus::new(64));
        let proxy = Arc::new(ProxyRouter::new(agents, integrations).await);

        let server = AgilePlusCoreServer::new(
            PathBuf::from(REPO_ROOT),
            Arc::clone(&storage),
            Arc::new(GitVcsAdapter::new(PathBuf::from(REPO_ROOT))),
            Arc::new(UnavailableAgent),
            Arc::new(UnavailableReview),
            Arc::new(LogOnlyObservability),
            Arc::clone(&bus),
            proxy,
        );

        Self {
            server,
            storage,
            bus,
        }
    }

    /// Persist a feature in `state` and return it with its assigned id.
    pub async fn seed_feature(&self, slug: &str, state: FeatureState) -> Feature {
        let mut feature = Feature::new(slug, &format!("Feature {slug}"), [0x5a; 32], Some("main"));
        feature.state = FeatureState::Created;
        let id = self
            .storage
            .create_feature(&feature)
            .await
            .expect("feature should be persisted");
        feature.id = id;

        if state != FeatureState::Created {
            self.storage
                .update_feature_state(id, state)
                .await
                .expect("feature state should be persisted");
            feature.state = state;
        }
        feature
    }

    /// Persist a work package for `feature_id`, accepting the state directly
    /// (bypassing the transition table so any state can be reached).
    pub async fn seed_wp(&self, feature_id: i64, sequence: i32, state: WpState) -> WorkPackage {
        let mut wp = WorkPackage::new(
            feature_id,
            &format!("WP {sequence}"),
            sequence,
            "acceptance criteria met",
        );
        wp.state = state;
        let id = self
            .storage
            .create_work_package(&wp)
            .await
            .expect("work package should be persisted");
        wp.id = id;
        wp
    }

    /// Persist an explicit dependency edge `wp_id -> depends_on`.
    pub async fn seed_dependency(&self, wp_id: i64, depends_on: i64, kind: DependencyType) {
        self.storage
            .add_wp_dependency(&WpDependency {
                wp_id,
                depends_on,
                dep_type: kind,
            })
            .await
            .expect("dependency should be persisted");
    }

    /// Persist evidence linked to `wp_id` and return its id.
    pub async fn seed_evidence(&self, wp_id: i64, fr_id: &str, kind: EvidenceType) -> i64 {
        self.storage
            .create_evidence(&Evidence {
                id: 0,
                wp_id,
                fr_id: fr_id.to_string(),
                evidence_type: kind,
                artifact_path: format!("artifacts/{fr_id}.json"),
                metadata: None,
                created_at: Utc::now(),
            })
            .await
            .expect("evidence should be persisted")
    }

    /// Persist a governance contract with the given rules and return its id.
    pub async fn seed_contract(
        &self,
        feature_id: i64,
        version: i32,
        rules: Vec<GovernanceRule>,
    ) -> i64 {
        self.storage
            .create_governance_contract(&GovernanceContract {
                id: 0,
                feature_id,
                version,
                rules,
                bound_at: Utc::now(),
            })
            .await
            .expect("governance contract should be persisted")
    }

    /// Append a correctly hash-chained audit entry for `feature_id`.
    ///
    /// Timestamps are pinned to whole seconds so the RFC-3339 round trip
    /// through SQLite reproduces the same hash input.
    pub async fn seed_audit_entry(&self, feature_id: i64, actor: &str) -> AuditEntry {
        let mut entry = self.build_audit_entry(feature_id, actor).await;
        entry.hash = hash_entry(&entry);
        let id = self
            .storage
            .append_audit_entry(&entry)
            .await
            .expect("audit entry should be persisted");
        entry.id = id;
        entry
    }

    /// Append an audit entry whose stored hash does not match its contents,
    /// simulating a tampered row that a hash-chain verification must catch.
    pub async fn seed_tampered_audit_entry(&self, feature_id: i64, actor: &str) -> AuditEntry {
        let mut entry = self.build_audit_entry(feature_id, actor).await;
        let mut forged = entry.clone();
        forged.actor = "forged-actor".to_string();
        entry.hash = hash_entry(&forged);
        let id = self
            .storage
            .append_audit_entry(&entry)
            .await
            .expect("tampered audit entry should be persisted");
        entry.id = id;
        entry
    }

    async fn build_audit_entry(&self, feature_id: i64, actor: &str) -> AuditEntry {
        let prev_hash = self
            .storage
            .get_latest_audit_entry(feature_id)
            .await
            .expect("latest audit entry lookup should succeed")
            .map(|entry| entry.hash)
            .unwrap_or([0u8; 32]);

        AuditEntry {
            id: 0,
            feature_id,
            wp_id: None,
            timestamp: self.next_audit_timestamp().await,
            actor: actor.to_string(),
            transition: format!("{actor}->recorded"),
            evidence_refs: Vec::new(),
            prev_hash,
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        }
    }

    async fn next_audit_timestamp(&self) -> DateTime<Utc> {
        let existing = self
            .storage
            .list_all_features()
            .await
            .map(|features| features.len())
            .unwrap_or(0);
        DateTime::from_timestamp(1_700_000_000 + existing as i64, 0)
            .expect("fixed audit timestamp should be valid")
    }

    /// Read a feature back out of storage to observe persisted state.
    pub async fn persisted_feature(&self, slug: &str) -> Feature {
        self.storage
            .get_feature_by_slug(slug)
            .await
            .expect("feature lookup should succeed")
            .unwrap_or_else(|| panic!("feature '{slug}' should be persisted"))
    }

    /// Force a feature into `state` without going through the RPC surface.
    pub async fn set_feature_state(&self, feature_id: i64, state: FeatureState) {
        self.storage
            .update_feature_state(feature_id, state)
            .await
            .expect("feature state update should succeed");
    }
}

/// A `ProjectScope` matching the harness repository.
pub fn scope() -> Option<ProjectScope> {
    Some(ProjectScope {
        canonical_repo_root: REPO_ROOT.to_string(),
    })
}

/// A `ProjectScope` that names a different repository.
pub fn foreign_scope() -> Option<ProjectScope> {
    Some(ProjectScope {
        canonical_repo_root: "/repo/some-other-project".to_string(),
    })
}

/// A `CommandRequest` for the `DispatchCommand` RPC.
pub fn command_request(command: &str, feature_slug: &str, args: &[(&str, &str)]) -> CommandRequest {
    CommandRequest {
        command: command.to_string(),
        feature_slug: feature_slug.to_string(),
        args: args
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<String, String>>(),
    }
}

/// A governance rule that requires `evidence` on `transition` ("" = all).
pub fn rule(transition: &str, evidence: &[&str]) -> GovernanceRule {
    GovernanceRule {
        transition: transition.to_string(),
        required_evidence: evidence.iter().map(|e| e.to_string()).collect(),
        policy_refs: Vec::new(),
    }
}
