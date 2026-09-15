//! Core types for the AgilePlus governance system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an audit event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEventId(pub String);

/// Unique identifier for a policy check
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyCheckId(pub String);

/// Connection status to remote governance
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    /// Connected to remote governance
    Connected,
    /// Disconnected, using local-only mode
    Disconnected,
    /// Error connecting
    Error,
    /// Governance disabled
    #[default]
    Disabled,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Connected => write!(f, "connected"),
            ConnectionStatus::Disconnected => write!(f, "disconnected"),
            ConnectionStatus::Error => write!(f, "error"),
            ConnectionStatus::Disabled => write!(f, "disabled"),
        }
    }
}

/// Governance action categories
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionCategory {
    /// Release management actions
    Release,
    /// Repository operations
    Repository,
    /// Policy changes
    Policy,
    /// Audit operations
    Audit,
    /// Configuration changes
    Config,
    /// General operations
    #[default]
    General,
}

impl std::str::FromStr for ActionCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "release" => Ok(Self::Release),
            "repository" => Ok(Self::Repository),
            "policy" => Ok(Self::Policy),
            "audit" => Ok(Self::Audit),
            "config" => Ok(Self::Config),
            "general" => Ok(Self::General),
            _ => Err(format!("Unknown action category: {}", s)),
        }
    }
}

/// Result of an operation
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationResult {
    /// Operation succeeded
    #[default]
    Success,
    /// Operation failed
    Failure,
    /// Operation partially succeeded
    PartialSuccess,
}

impl std::fmt::Display for OperationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationResult::Success => write!(f, "success"),
            OperationResult::Failure => write!(f, "failure"),
            OperationResult::PartialSuccess => write!(f, "partial_success"),
        }
    }
}

impl std::str::FromStr for OperationResult {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "success" => Ok(OperationResult::Success),
            "failure" => Ok(OperationResult::Failure),
            "partial_success" | "partialsuccess" => Ok(OperationResult::PartialSuccess),
            _ => Err(format!("Unknown result: {}", s)),
        }
    }
}

/// Log level for audit entries
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "error" | "err" => Ok(LogLevel::Error),
            _ => Err(format!("Unknown log level: {}", s)),
        }
    }
}

/// Authentication method for remote governance
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthMethod {
    #[default]
    ApiKey,
    BearerToken,
    None,
}

/// Governance statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GovernanceStats {
    /// Total audit events
    pub total: u64,
    /// Events today
    pub today: u64,
    /// Failed operations
    pub errors: u64,
    /// Events by level
    pub by_level: std::collections::HashMap<String, u64>,
    /// Top actions
    pub top_actions: Vec<TopAction>,
}

/// Top action statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAction {
    pub action: String,
    pub count: u64,
}

/// Governance status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStatus {
    /// Whether governance is initialized
    pub initialized: bool,
    /// Connection status
    pub connection_status: ConnectionStatus,
    /// Remote governance enabled
    pub remote_enabled: bool,
    /// Local governance enabled
    pub local_enabled: bool,
    /// Sync to remote enabled
    pub sync_enabled: bool,
    /// Last sync timestamp
    pub last_sync: Option<DateTime<Utc>>,
    /// Pending operations for sync
    pub pending_operations: u64,
    /// Configuration summary
    pub config: GovernanceStatusConfig,
    /// Statistics
    pub stats: GovernanceStats,
}

impl Default for GovernanceStatus {
    fn default() -> Self {
        Self {
            initialized: false,
            connection_status: ConnectionStatus::Disabled,
            remote_enabled: false,
            local_enabled: false,
            sync_enabled: false,
            last_sync: None,
            pending_operations: 0,
            config: GovernanceStatusConfig::default(),
            stats: GovernanceStats::default(),
        }
    }
}

/// Configuration summary for status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStatusConfig {
    pub governance_url: String,
    pub auth_method: AuthMethod,
    pub policy_enabled: bool,
    pub rate_limit_enabled: bool,
}

impl Default for GovernanceStatusConfig {
    fn default() -> Self {
        Self {
            governance_url: String::new(),
            auth_method: AuthMethod::ApiKey,
            policy_enabled: true,
            rate_limit_enabled: false,
        }
    }
}

impl AuditEventId {
    /// Generate a new audit event ID
    pub fn new() -> Self {
        Self(format!("evt_{}", Uuid::new_v4()))
    }
}

impl Default for AuditEventId {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyCheckId {
    /// Generate a new policy check ID
    pub fn new() -> Self {
        Self(format!("chk_{}", Uuid::new_v4()))
    }
}

impl Default for PolicyCheckId {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_status_display() {
        assert_eq!(ConnectionStatus::Connected.to_string(), "connected");
        assert_eq!(ConnectionStatus::Disconnected.to_string(), "disconnected");
        assert_eq!(ConnectionStatus::Error.to_string(), "error");
        assert_eq!(ConnectionStatus::Disabled.to_string(), "disabled");
    }

    #[test]
    fn connection_status_default() {
        assert_eq!(ConnectionStatus::default(), ConnectionStatus::Disabled);
    }

    #[test]
    fn connection_status_serde_roundtrip() {
        for s in [ConnectionStatus::Connected, ConnectionStatus::Disconnected, ConnectionStatus::Error, ConnectionStatus::Disabled] {
            let j = serde_json::to_string(&s).unwrap();
            let b: ConnectionStatus = serde_json::from_str(&j).unwrap();
            assert_eq!(b, s);
        }
    }

    #[test]
    fn action_category_from_str() {
        assert_eq!("release".parse::<ActionCategory>().unwrap(), ActionCategory::Release);
        assert_eq!("REPOSITORY".parse::<ActionCategory>().unwrap(), ActionCategory::Repository);
        assert_eq!("Policy".parse::<ActionCategory>().unwrap(), ActionCategory::Policy);
        assert_eq!("audit".parse::<ActionCategory>().unwrap(), ActionCategory::Audit);
        assert_eq!("config".parse::<ActionCategory>().unwrap(), ActionCategory::Config);
        assert_eq!("general".parse::<ActionCategory>().unwrap(), ActionCategory::General);
        assert!("invalid".parse::<ActionCategory>().is_err());
    }

    #[test]
    fn action_category_default() {
        assert_eq!(ActionCategory::default(), ActionCategory::General);
    }

    #[test]
    fn action_category_serde_roundtrip() {
        for c in [ActionCategory::Release, ActionCategory::Repository, ActionCategory::Policy, ActionCategory::Audit, ActionCategory::Config, ActionCategory::General] {
            let j = serde_json::to_string(&c).unwrap();
            let b: ActionCategory = serde_json::from_str(&j).unwrap();
            assert_eq!(b, c);
        }
    }

    #[test]
    fn operation_result_display() {
        assert_eq!(OperationResult::Success.to_string(), "success");
        assert_eq!(OperationResult::Failure.to_string(), "failure");
        assert_eq!(OperationResult::PartialSuccess.to_string(), "partial_success");
    }

    #[test]
    fn operation_result_from_str() {
        assert_eq!("success".parse::<OperationResult>().unwrap(), OperationResult::Success);
        assert_eq!("failure".parse::<OperationResult>().unwrap(), OperationResult::Failure);
        assert_eq!("partial_success".parse::<OperationResult>().unwrap(), OperationResult::PartialSuccess);
        assert_eq!("partialsuccess".parse::<OperationResult>().unwrap(), OperationResult::PartialSuccess);
        assert!("unknown".parse::<OperationResult>().is_err());
    }

    #[test]
    fn operation_result_default() {
        assert_eq!(OperationResult::default(), OperationResult::Success);
    }

    #[test]
    fn log_level_display() {
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Warn.to_string(), "warn");
        assert_eq!(LogLevel::Error.to_string(), "error");
    }

    #[test]
    fn log_level_from_str() {
        assert_eq!("debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("info".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("warn".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("warning".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("error".parse::<LogLevel>().unwrap(), LogLevel::Error);
        assert_eq!("err".parse::<LogLevel>().unwrap(), LogLevel::Error);
        assert!("trace".parse::<LogLevel>().is_err());
    }

    #[test]
    fn log_level_default() {
        assert_eq!(LogLevel::default(), LogLevel::Info);
    }

    #[test]
    fn auth_method_default() {
        assert_eq!(AuthMethod::default(), AuthMethod::ApiKey);
    }

    #[test]
    fn auth_method_serde_roundtrip() {
        for m in [AuthMethod::ApiKey, AuthMethod::BearerToken, AuthMethod::None] {
            let j = serde_json::to_string(&m).unwrap();
            let b: AuthMethod = serde_json::from_str(&j).unwrap();
            assert_eq!(b, m);
        }
    }

    #[test]
    fn audit_event_id_prefix() {
        let id = AuditEventId::new();
        assert!(id.0.starts_with("evt_"));
    }

    #[test]
    fn audit_event_id_clone_eq() {
        let id = AuditEventId("evt_test123".to_string());
        assert_eq!(id, id.clone());
    }

    #[test]
    fn policy_check_id_prefix() {
        let id = PolicyCheckId::new();
        assert!(id.0.starts_with("chk_"));
    }

    #[test]
    fn policy_check_id_clone_eq() {
        let id = PolicyCheckId("chk_test456".to_string());
        assert_eq!(id, id.clone());
    }

    #[test]
    fn governance_stats_default() {
        let s = GovernanceStats::default();
        assert_eq!(s.total, 0);
        assert_eq!(s.errors, 0);
        assert!(s.by_level.is_empty());
        assert!(s.top_actions.is_empty());
    }

    #[test]
    fn governance_status_default() {
        let s = GovernanceStatus::default();
        assert!(!s.initialized);
        assert_eq!(s.connection_status, ConnectionStatus::Disabled);
        assert!(!s.remote_enabled);
        assert!(!s.local_enabled);
        assert!(!s.sync_enabled);
        assert!(s.last_sync.is_none());
        assert_eq!(s.pending_operations, 0);
    }

    #[test]
    fn governance_status_config_default() {
        let c = GovernanceStatusConfig::default();
        assert!(c.governance_url.is_empty());
        assert_eq!(c.auth_method, AuthMethod::ApiKey);
        assert!(c.policy_enabled);
        assert!(!c.rate_limit_enabled);
    }

    #[test]
    fn top_action_serde_roundtrip() {
        let a = TopAction { action: "deploy".into(), count: 42 };
        let j = serde_json::to_string(&a).unwrap();
        let b: TopAction = serde_json::from_str(&j).unwrap();
        assert_eq!(b.action, "deploy");
        assert_eq!(b.count, 42);
    }
}
