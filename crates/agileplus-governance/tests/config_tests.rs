//! Integration tests for GovernanceConfig: serde, file loading, validation, defaults.
//! Complements the inline unit tests in src/config.rs.

use agileplus_governance::*;
use agileplus_governance::config::PolicyDefaultAction;
use tempfile::tempdir;

// ── Default values ───────────────────────────────────────────────────

#[test]
fn default_governance_settings() {
    let config = GovernanceConfig::default();
    assert!(!config.governance.enabled);
    assert_eq!(config.governance.base_url, "http://localhost:8080/api/v1");
    assert_eq!(config.governance.timeout_secs, 30);
    assert_eq!(config.governance.retry_attempts, 3);
}

#[test]
fn default_local_settings() {
    let config = GovernanceConfig::default();
    assert!(config.local.enabled);
    assert_eq!(config.local.db_path, ".agileplus/governance.db");
    assert_eq!(config.local.retention_days, 90);
}

#[test]
fn default_sync_settings() {
    let config = GovernanceConfig::default();
    assert!(config.sync.enabled);
    assert_eq!(config.sync.interval_ms, 300_000);
    assert_eq!(config.sync.batch_size, 100);
    assert_eq!(config.sync.timeout_secs, 60);
}

#[test]
fn default_policy_settings() {
    let config = GovernanceConfig::default();
    assert!(config.policy.enabled);
    assert_eq!(config.policy.default_action, PolicyDefaultAction::Allow);
    assert!(config.policy.enforce_gates);
    assert!(config.policy.enforce_rate_limits);
}

#[test]
fn default_rate_limit_settings() {
    let config = GovernanceConfig::default();
    assert!(!config.rate_limit.enabled);
    assert_eq!(config.rate_limit.max_requests, 100);
    assert_eq!(config.rate_limit.window_ms, 3_600_000);
}

#[test]
fn default_auth_settings() {
    let config = GovernanceConfig::default();
    assert_eq!(config.governance.auth.method, AuthMethod::ApiKey);
    assert!(config.governance.auth.api_key.is_empty());
    assert!(config.governance.auth.bearer_token.is_empty());
}

// ── Validation ───────────────────────────────────────────────────────

#[test]
fn validation_clean_default() {
    let config = GovernanceConfig::default();
    assert!(config.validate().is_empty());
}

#[test]
fn validation_governance_enabled_no_url() {
    let mut config = GovernanceConfig::default();
    config.governance.enabled = true;
    config.governance.base_url = String::new();
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("base URL")));
}

#[test]
fn validation_governance_enabled_no_api_key() {
    let mut config = GovernanceConfig::default();
    config.governance.enabled = true;
    config.governance.auth.api_key = String::new();
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("API key")));
}

#[test]
fn validation_local_enabled_no_db_path() {
    let mut config = GovernanceConfig::default();
    config.local.enabled = true;
    config.local.db_path = String::new();
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("database path")));
}

#[test]
fn validation_governance_disabled_no_errors() {
    let mut config = GovernanceConfig::default();
    config.governance.enabled = false;
    config.governance.base_url = String::new();
    config.governance.auth.api_key = String::new();
    let errors = config.validate();
    assert!(errors.is_empty());
}

#[test]
fn validation_bearer_auth_no_api_key_needed() {
    let mut config = GovernanceConfig::default();
    config.governance.enabled = true;
    config.governance.auth.method = AuthMethod::BearerToken;
    config.governance.auth.api_key = String::new();
    let errors = config.validate();
    // Bearer auth should not require API key
    assert!(!errors.iter().any(|e| e.contains("API key")));
}

// ── File loading ─────────────────────────────────────────────────────

#[test]
fn from_file_json() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.json");
    let json = serde_json::json!({
        "governance": {
            "enabled": true,
            "base_url": "http://test:8080",
            "auth": {"method": "api-key", "api_key": "key123", "bearer_token": ""},
            "timeout_secs": 10,
            "retry_attempts": 2
        },
        "local": {
            "enabled": false,
            "db_path": "/tmp/test.db",
            "retention_days": 30
        },
        "sync": {
            "enabled": false,
            "interval_ms": 60000,
            "batch_size": 10,
            "timeout_secs": 30
        },
        "policy": {
            "enabled": false,
            "default_action": "deny",
            "enforce_gates": false,
            "enforce_rate_limits": false
        },
        "rate_limit": {
            "enabled": true,
            "max_requests": 50,
            "window_ms": 60000
        }
    });
    std::fs::write(&path, json.to_string()).unwrap();
    let config = GovernanceConfig::from_file(&path).unwrap();
    assert!(config.governance.enabled);
    assert_eq!(config.governance.base_url, "http://test:8080");
    assert_eq!(config.governance.timeout_secs, 10);
    assert!(!config.local.enabled);
    assert_eq!(config.local.db_path, "/tmp/test.db");
    assert!(!config.policy.enabled);
    assert_eq!(config.policy.default_action, PolicyDefaultAction::Deny);
    assert!(config.rate_limit.enabled);
    assert_eq!(config.rate_limit.max_requests, 50);
}

#[test]
fn from_file_toml() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let toml_content = r#"
[governance]
enabled = true
base_url = "http://toml-test:8080"
timeout_secs = 15
retry_attempts = 1

[governance.auth]
method = "bearer-token"
api_key = ""
bearer_token = "tok_xxx"

[local]
enabled = true
db_path = "/data/governance.db"
retention_days = 30

[sync]
enabled = false
interval_ms = 60000
batch_size = 50
timeout_secs = 30

[policy]
enabled = true
default_action = "allow"
enforce_gates = true
enforce_rate_limits = true

[rate_limit]
enabled = true
max_requests = 200
window_ms = 120000
"#;
    std::fs::write(&path, toml_content).unwrap();
    let config = GovernanceConfig::from_file(&path).unwrap();
    assert!(config.governance.enabled);
    assert_eq!(config.governance.base_url, "http://toml-test:8080");
    assert_eq!(config.governance.auth.method, AuthMethod::BearerToken);
    assert_eq!(config.governance.auth.bearer_token, "tok_xxx");
    assert!(config.rate_limit.enabled);
    assert_eq!(config.rate_limit.max_requests, 200);
}

// ── Serde roundtrip ──────────────────────────────────────────────────

#[test]
fn config_serde_roundtrip() {
    let config = GovernanceConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let back: GovernanceConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.governance.enabled, config.governance.enabled);
    assert_eq!(back.local.db_path, config.local.db_path);
    assert_eq!(back.sync.batch_size, config.sync.batch_size);
    assert_eq!(back.policy.default_action, config.policy.default_action);
    assert_eq!(back.rate_limit.max_requests, config.rate_limit.max_requests);
}

// ── PolicyDefaultAction ──────────────────────────────────────────────

#[test]
fn policy_default_action_serde() {
    assert_eq!(
        serde_json::to_string(&PolicyDefaultAction::Allow).unwrap(),
        "\"allow\""
    );
    assert_eq!(
        serde_json::to_string(&PolicyDefaultAction::Deny).unwrap(),
        "\"deny\""
    );
}

#[test]
fn policy_default_action_deserialize() {
    let allow: PolicyDefaultAction = serde_json::from_str("\"allow\"").unwrap();
    assert_eq!(allow, PolicyDefaultAction::Allow);
    let deny: PolicyDefaultAction = serde_json::from_str("\"deny\"").unwrap();
    assert_eq!(deny, PolicyDefaultAction::Deny);
}

#[test]
fn policy_default_action_default_is_allow() {
    assert_eq!(PolicyDefaultAction::default(), PolicyDefaultAction::Allow);
}
