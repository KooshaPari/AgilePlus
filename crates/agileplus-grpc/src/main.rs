use std::sync::Arc;

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::agent::{AgentConfig, AgentResult, AgentStatus, AgentTask};
use agileplus_domain::ports::observability::{LogEntry, ObservabilityPort, SpanContext};
use agileplus_domain::ports::review::{CiStatus, PrInfo, ReviewComment, ReviewStatus};
use agileplus_domain::ports::{AgentPort, ReviewPort};
use agileplus_git::GitVcsAdapter;
use agileplus_grpc::event_bus::EventBus;
use agileplus_grpc::proxy::ProxyRouter;
use agileplus_grpc::runtime::CoreConfig;
use agileplus_sqlite::SqliteStorageAdapter;

#[derive(Debug, Default)]
struct UnavailableAgent;

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

#[derive(Debug, Default)]
struct UnavailableReview;

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

#[derive(Debug, Default)]
struct LogOnlyObservability;

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
    fn set_span_error(&self, _: &SpanContext, error: &str) {
        tracing::error!(%error);
    }
    fn record_counter(&self, _: &str, _: u64, _: &[(&str, &str)]) {}
    fn record_histogram(&self, _: &str, _: f64, _: &[(&str, &str)]) {}
    fn record_gauge(&self, _: &str, _: f64, _: &[(&str, &str)]) {}
    fn log(&self, entry: &LogEntry) {
        tracing::info!(message = %entry.message);
    }
    fn log_info(&self, message: &str) {
        tracing::info!(%message);
    }
    fn log_warn(&self, message: &str) {
        tracing::warn!(%message);
    }
    fn log_error(&self, message: &str) {
        tracing::error!(%message);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let config = CoreConfig::from_env()?;
    if let Some(parent) = config.database.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let storage = Arc::new(SqliteStorageAdapter::new(&config.database)?);
    let vcs = Arc::new(GitVcsAdapter::from_current_dir()?);
    let proxy = Arc::new(ProxyRouter::new(None, None).await);
    tracing::info!(bind = %config.bind, database = %config.database.display(), "starting AgilePlus core");

    agileplus_grpc::server::start_server(
        config.bind,
        storage,
        Arc::new(EventBus::new(256)),
        proxy,
    )
    .await
}
