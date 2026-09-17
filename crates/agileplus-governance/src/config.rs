//! Configuration for the AgilePlus governance system
//!
//! Configuration can be loaded from:
//! - Environment variables
//! - `governance.toml` file
//! - `governance.json` file
//! - Default values
//!
//! Environment variable prefixes:
//! - `AGILEPLUS_GOVERNANCE_*` for governance config
//! - `AGILEPLUS_LOCAL_*` for local storage config
//! - `AGILEPLUS_SYNC_*` for sync config
//! - `AGILEPLUS_POLICY_*` for policy config
//! - `AGILEPLUS_RATE_LIMIT_*` for rate limit config

use crate::types::AuthMethod;
use serde::{Deserialize, Serialize};

/// Main governance configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Governance settings
    pub governance: GovernanceSettings,
    /// Local storage settings
    pub local: LocalSettings,
    /// Sync settings
    pub sync: SyncSettings,
    /// Policy settings
    pub policy: PolicySettings,
    /// Rate limiting settings
    pub rate_limit: RateLimitSettings,
}

/// Remote governance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceSettings {
    /// Enable governance
    pub enabled: bool,
    /// Base URL for governance API
    pub base_url: String,
    /// Authentication settings
    pub auth: AuthSettings,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Retry attempts
    pub retry_attempts: u32,
}

impl Default for GovernanceSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "http://localhost:8080/api/v1".to_string(),
            auth: AuthSettings::default(),
            timeout_secs: 30,
            retry_attempts: 3,
        }
    }
}

/// Authentication settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    /// Authentication method
    pub method: AuthMethod,
    /// API key (if using api-key auth)
    pub api_key: String,
    /// Bearer token (if using bearer auth)
    pub bearer_token: String,
}

impl Default for AuthSettings {
    fn default() -> Self {
        Self {
            method: AuthMethod::ApiKey,
            api_key: String::new(),
            bearer_token: String::new(),
        }
    }
}

/// Local storage settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSettings {
    /// Enable local governance storage
    pub enabled: bool,
    /// Path to local database
    pub db_path: String,
    /// Retention days for audit logs
    pub retention_days: u32,
}

impl Default for LocalSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            db_path: ".agileplus/governance.db".to_string(),
            retention_days: 90,
        }
    }
}

/// Sync settings for remote governance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    /// Enable sync to remote
    pub enabled: bool,
    /// Sync interval in milliseconds
    pub interval_ms: u64,
    /// Batch size for sync
    pub batch_size: usize,
    /// Sync timeout in seconds
    pub timeout_secs: u64,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 300_000, // 5 minutes
            batch_size: 100,
            timeout_secs: 60,
        }
    }
}

/// Policy enforcement settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySettings {
    /// Enable policy enforcement
    pub enabled: bool,
    /// Default action if no policy matches
    pub default_action: PolicyDefaultAction,
    /// Enforce channel gates
    pub enforce_gates: bool,
    /// Enforce rate limits
    pub enforce_rate_limits: bool,
}

impl Default for PolicySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            default_action: PolicyDefaultAction::Allow,
            enforce_gates: true,
            enforce_rate_limits: true,
        }
    }
}

/// Default action when no policy matches
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyDefaultAction {
    #[default]
    Allow,
    Deny,
}

/// Rate limiting settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitSettings {
    /// Enable rate limiting
    pub enabled: bool,
    /// Max requests per window
    pub max_requests: u64,
    /// Window size in milliseconds
    pub window_ms: u64,
}

impl Default for RateLimitSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            max_requests: 100,
            window_ms: 3_600_000, // 1 hour
        }
    }
}

impl GovernanceConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            governance: GovernanceSettings {
                enabled: std::env::var("AGILEPLUS_GOVERNANCE_ENABLED")
                    .map(|v| v == "true")
                    .unwrap_or_default(),
                base_url: std::env::var("AGILEPLUS_GOVERNANCE_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:8080/api/v1".to_string()),
                auth: AuthSettings {
                    method: std::env::var("AGILEPLUS_GOVERNANCE_AUTH_METHOD")
                        .map(|v| match v.as_str() {
                            "bearer" => AuthMethod::BearerToken,
                            "none" => AuthMethod::None,
                            _ => AuthMethod::ApiKey,
                        })
                        .unwrap_or(AuthMethod::ApiKey),
                    api_key: std::env::var("AGILEPLUS_GOVERNANCE_API_KEY").unwrap_or_default(),
                    bearer_token: std::env::var("AGILEPLUS_GOVERNANCE_BEARER_TOKEN")
                        .unwrap_or_default(),
                },
                timeout_secs: std::env::var("AGILEPLUS_GOVERNANCE_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                retry_attempts: std::env::var("AGILEPLUS_GOVERNANCE_RETRY")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()
                    .unwrap_or(3),
            },
            local: LocalSettings {
                enabled: std::env::var("AGILEPLUS_LOCAL_ENABLED")
                    .map(|v| v == "true")
                    .unwrap_or(true),
                db_path: std::env::var("AGILEPLUS_LOCAL_DB_PATH")
                    .unwrap_or_else(|_| ".agileplus/governance.db".to_string()),
                retention_days: std::env::var("AGILEPLUS_LOCAL_RETENTION_DAYS")
                    .unwrap_or_else(|_| "90".to_string())
                    .parse()
                    .unwrap_or(90),
            },
            sync: SyncSettings {
                enabled: std::env::var("AGILEPLUS_SYNC_ENABLED")
                    .map(|v| v == "true")
                    .unwrap_or(true),
                interval_ms: std::env::var("AGILEPLUS_SYNC_INTERVAL")
                    .unwrap_or_else(|_| "300000".to_string())
                    .parse()
                    .unwrap_or(300_000),
                batch_size: std::env::var("AGILEPLUS_SYNC_BATCH_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                timeout_secs: std::env::var("AGILEPLUS_SYNC_TIMEOUT")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
            },
            policy: PolicySettings {
                enabled: std::env::var("AGILEPLUS_POLICY_ENABLED")
                    .map(|v| v == "true")
                    .unwrap_or(true),
                default_action: std::env::var("AGILEPLUS_POLICY_DEFAULT")
                    .map(|v| {
                        if v == "deny" {
                            PolicyDefaultAction::Deny
                        } else {
                            PolicyDefaultAction::Allow
                        }
                    })
                    .unwrap_or(PolicyDefaultAction::Allow),
                enforce_gates: std::env::var("AGILEPLUS_POLICY_ENFORCE_GATES")
                    .map(|v| v == "true")
                    .unwrap_or(true),
                enforce_rate_limits: std::env::var("AGILEPLUS_POLICY_ENFORCE_RATE_LIMITS")
                    .map(|v| v == "true")
                    .unwrap_or(true),
            },
            rate_limit: RateLimitSettings {
                enabled: std::env::var("AGILEPLUS_RATE_LIMIT_ENABLED")
                    .map(|v| v == "true")
                    .unwrap_or(false),
                max_requests: std::env::var("AGILEPLUS_RATE_LIMIT_MAX")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                window_ms: std::env::var("AGILEPLUS_RATE_LIMIT_WINDOW")
                    .unwrap_or_else(|_| "3600000".to_string())
                    .parse()
                    .unwrap_or(3_600_000),
            },
        }
    }

    /// Load configuration from a file
    pub fn from_file(path: &std::path::Path) -> crate::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "json" => Ok(serde_json::from_str(&contents)?),
            "toml" => Ok(toml::from_str(&contents)?),
            _ => Ok(serde_json::from_str(&contents)?),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.governance.enabled {
            if self.governance.base_url.is_empty() {
                errors.push("Governance base URL is required when enabled".to_string());
            }

            if self.governance.auth.method == AuthMethod::ApiKey
                && self.governance.auth.api_key.is_empty()
            {
                errors.push("API key is required for api-key authentication".to_string());
            }
        }

        if self.local.enabled && self.local.db_path.is_empty() {
            errors
                .push("Local database path is required when local storage is enabled".to_string());
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GovernanceConfig::default();
        assert!(!config.governance.enabled);
        assert!(config.local.enabled);
        assert!(config.policy.enabled);
    }

    #[test]
    fn test_env_config() {
        std::env::set_var("AGILEPLUS_GOVERNANCE_ENABLED", "true");
        std::env::set_var("AGILEPLUS_GOVERNANCE_BASE_URL", "http://governance:8080");

        let config = GovernanceConfig::from_env();
        assert!(config.governance.enabled);
        assert_eq!(config.governance.base_url, "http://governance:8080");

        std::env::remove_var("AGILEPLUS_GOVERNANCE_ENABLED");
        std::env::remove_var("AGILEPLUS_GOVERNANCE_BASE_URL");
    }

    #[test]
    fn test_config_validation_clean() {
        assert!(GovernanceConfig::default().validate().is_empty());
    }

    #[test]
    fn test_config_validation_governance_enabled_no_url() {
        let mut config = GovernanceConfig::default();
        config.governance.enabled = true;
        config.governance.base_url = "".into();
        assert!(config.validate().iter().any(|e| e.contains("base URL")));
    }

    #[test]
    fn test_config_validation_governance_enabled_no_api_key() {
        let mut config = GovernanceConfig::default();
        config.governance.enabled = true;
        config.governance.auth.api_key = "".into();
        assert!(config.validate().iter().any(|e| e.contains("API key")));
    }

    #[test]
    fn test_config_from_file_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, r#"{"governance":{"enabled":true,"base_url":"http://test:8080","auth":{"method":"api-key","api_key":"k","bearer_token":""},"timeout_secs":60,"retry_attempts":5},"local":{"enabled":false,"db_path":"/tmp/db","retention_days":30},"sync":{"enabled":false,"interval_ms":1000,"batch_size":10,"timeout_secs":10},"policy":{"enabled":true,"default_action":"allow","enforce_gates":true,"enforce_rate_limits":true},"rate_limit":{"enabled":true,"max_requests":50,"window_ms":60000}}"#).unwrap();
        let config = GovernanceConfig::from_file(&path).unwrap();
        assert!(config.governance.enabled);
        assert_eq!(config.governance.base_url, "http://test:8080");
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let config = GovernanceConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let back: GovernanceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.governance.enabled, config.governance.enabled);
    }

    #[test]
    fn test_default_settings_values() {
        let config = GovernanceConfig::default();
        assert!(!config.governance.enabled);
        assert!(config.local.enabled);
        assert!(config.sync.enabled);
        assert!(config.policy.enabled);
        assert!(!config.rate_limit.enabled);
    }

}
#[cfg(test)]
mod extended_tests {
    use super::*;

    #[test]
    fn governance_settings_defaults() {
        let s = GovernanceSettings::default();
        assert!(!s.enabled);
        assert_eq!(s.base_url, "http://localhost:8080/api/v1");
        assert_eq!(s.timeout_secs, 30);
        assert_eq!(s.retry_attempts, 3);
    }

    #[test]
    fn local_settings_defaults() {
        let s = LocalSettings::default();
        assert!(s.enabled);
        assert_eq!(s.db_path, ".agileplus/governance.db");
        assert_eq!(s.retention_days, 90);
    }

    #[test]
    fn sync_settings_defaults() {
        let s = SyncSettings::default();
        assert!(s.enabled);
        assert_eq!(s.interval_ms, 300_000);
        assert_eq!(s.batch_size, 100);
        assert_eq!(s.timeout_secs, 60);
    }

    #[test]
    fn policy_settings_defaults() {
        let s = PolicySettings::default();
        assert!(s.enabled);
        assert_eq!(s.default_action, PolicyDefaultAction::Allow);
        assert!(s.enforce_gates);
        assert!(s.enforce_rate_limits);
    }

    #[test]
    fn rate_limit_settings_defaults() {
        let s = RateLimitSettings::default();
        assert!(!s.enabled);
        assert_eq!(s.max_requests, 100);
        assert_eq!(s.window_ms, 3_600_000);
    }

    #[test]
    fn validate_empty_when_disabled() {
        let config = GovernanceConfig::default();
        let errors = config.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_missing_url_when_enabled() {
        let mut config = GovernanceConfig::default();
        config.governance.enabled = true;
        config.governance.base_url = String::new();
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.contains("base URL")));
    }

    #[test]
    fn validate_missing_api_key() {
        let mut config = GovernanceConfig::default();
        config.governance.enabled = true;
        config.governance.auth.method = AuthMethod::ApiKey;
        config.governance.auth.api_key = String::new();
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.contains("API key")));
    }

    #[test]
    fn validate_missing_db_path() {
        let mut config = GovernanceConfig::default();
        config.local.enabled = true;
        config.local.db_path = String::new();
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.contains("database path")));
    }

    #[test]
    fn from_file_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, r#"{"governance":{"enabled":true,"base_url":"http://x","auth":{"method":"api-key","api_key":"k","bearer_token":""},"timeout_secs":10,"retry_attempts":1},"local":{"enabled":false,"db_path":"x","retention_days":1},"sync":{"enabled":false,"interval_ms":1,"batch_size":1,"timeout_secs":1},"policy":{"enabled":true,"default_action":"allow","enforce_gates":true,"enforce_rate_limits":true},"rate_limit":{"enabled":false,"max_requests":1,"window_ms":1}}"#).unwrap();
        let config = GovernanceConfig::from_file(&path).unwrap();
        assert!(config.governance.enabled);
        assert_eq!(config.governance.timeout_secs, 10);
    }

    #[test]
    fn from_file_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, r#"
[governance]
enabled = true
base_url = "http://y"
timeout_secs = 5
retry_attempts = 2

[governance.auth]
method = "api-key"
api_key = "key"
bearer_token = ""

[local]
enabled = true
db_path = "/tmp/test.db"
retention_days = 30

[sync]
enabled = false
interval_ms = 1000
batch_size = 50
timeout_secs = 10

[policy]
enabled = true
default_action = "deny"
enforce_gates = false
enforce_rate_limits = true

[rate_limit]
enabled = true
max_requests = 200
window_ms = 60000
"#).unwrap();
        let config = GovernanceConfig::from_file(&path).unwrap();
        assert!(config.governance.enabled);
        assert_eq!(config.governance.timeout_secs, 5);
        assert_eq!(config.local.retention_days, 30);
        assert_eq!(config.policy.default_action, PolicyDefaultAction::Deny);
    }

    #[test]
    fn governance_config_serde_roundtrip() {
        let config = GovernanceConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deser: GovernanceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.governance.enabled, deser.governance.enabled);
        assert_eq!(config.local.db_path, deser.local.db_path);
    }

    #[test]
    fn auth_settings_api_key_default() {
        let s = AuthSettings::default();
        assert_eq!(s.method, AuthMethod::ApiKey);
        assert!(s.api_key.is_empty());
    }

    #[test]
    fn from_file_unknown_extension_falls_back_to_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.xyz");
        std::fs::write(
            &path,
            r#"{"governance":{"enabled":false,"base_url":"x","auth":{"method":"api-key","api_key":"","bearer_token":""},"timeout_secs":1,"retry_attempts":1},"local":{"enabled":false,"db_path":"","retention_days":1},"sync":{"enabled":false,"interval_ms":1,"batch_size":1,"timeout_secs":1},"policy":{"enabled":false,"default_action":"allow","enforce_gates":false,"enforce_rate_limits":false},"rate_limit":{"enabled":false,"max_requests":1,"window_ms":1}}"#,
        )
        .unwrap();
        let config = GovernanceConfig::from_file(&path).unwrap();
        assert!(!config.governance.enabled);
    }

    #[test]
    fn validate_local_db_path_empty() {
        let mut config = GovernanceConfig::default();
        config.local.enabled = true;
        config.local.db_path = String::new();
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.contains("database path")));
    }

    #[test]
    fn validate_governance_disabled_no_errors() {
        let mut config = GovernanceConfig::default();
        config.governance.enabled = false;
        config.governance.base_url = String::new();
        config.governance.auth.api_key = String::new();
        let errors = config.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_all_clean_with_valid_config() {
        let config = GovernanceConfig {
            governance: GovernanceSettings {
                enabled: true,
                base_url: "http://ok:8080".into(),
                auth: AuthSettings {
                    method: AuthMethod::ApiKey,
                    api_key: "key123".into(),
                    bearer_token: String::new(),
                },
                timeout_secs: 10,
                retry_attempts: 3,
            },
            local: LocalSettings {
                enabled: true,
                db_path: "/tmp/db".into(),
                retention_days: 30,
            },
            ..Default::default()
        };
        let errors = config.validate();
        assert!(errors.is_empty());
    }

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
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn governance_settings_defaults() {
        let s = GovernanceSettings::default();
        assert!(!s.enabled);
        assert_eq!(s.base_url, "http://localhost:8080/api/v1");
        assert_eq!(s.timeout_secs, 30);
        assert_eq!(s.retry_attempts, 3);
    }

    #[test]
    fn auth_settings_defaults() {
        let s = AuthSettings::default();
        assert_eq!(s.method, AuthMethod::ApiKey);
        assert!(s.api_key.is_empty());
        assert!(s.bearer_token.is_empty());
    }

    #[test]
    fn local_settings_defaults() {
        let s = LocalSettings::default();
        assert!(s.enabled);
        assert_eq!(s.db_path, ".agileplus/governance.db");
        assert_eq!(s.retention_days, 90);
    }

    #[test]
    fn sync_settings_defaults() {
        let s = SyncSettings::default();
        assert!(s.enabled);
        assert_eq!(s.interval_ms, 300_000);
        assert_eq!(s.batch_size, 100);
        assert_eq!(s.timeout_secs, 60);
    }

    #[test]
    fn policy_settings_defaults() {
        let s = PolicySettings::default();
        assert!(s.enabled);
        assert_eq!(s.default_action, PolicyDefaultAction::Allow);
        assert!(s.enforce_gates);
        assert!(s.enforce_rate_limits);
    }

    #[test]
    fn rate_limit_settings_defaults() {
        let s = RateLimitSettings::default();
        assert!(!s.enabled);
        assert_eq!(s.max_requests, 100);
        assert_eq!(s.window_ms, 3_600_000);
    }

    #[test]
    fn policy_default_action_default_is_allow() {
        assert_eq!(PolicyDefaultAction::default(), PolicyDefaultAction::Allow);
    }

    #[test]
    fn governance_config_default_matches_section_defaults() {
        let config = GovernanceConfig::default();
        assert_eq!(config.governance.base_url, GovernanceSettings::default().base_url);
        assert_eq!(config.local.db_path, LocalSettings::default().db_path);
        assert_eq!(config.sync.batch_size, SyncSettings::default().batch_size);
        assert_eq!(config.rate_limit.max_requests, RateLimitSettings::default().max_requests);
    }

    #[test]
    fn governance_config_serde_roundtrip_all_sections() {
        let mut config = GovernanceConfig::default();
        config.governance.enabled = true;
        config.governance.base_url = "http://x:1".into();
        config.policy.default_action = PolicyDefaultAction::Deny;
        config.rate_limit.max_requests = 12;
        let json = serde_json::to_string(&config).unwrap();
        let back: GovernanceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.governance.base_url, "http://x:1");
        assert_eq!(back.policy.default_action, PolicyDefaultAction::Deny);
        assert_eq!(back.rate_limit.max_requests, 12);
    }

    #[test]
    fn governance_settings_serde_roundtrip() {
        let s = GovernanceSettings::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: GovernanceSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.timeout_secs, s.timeout_secs);
        assert_eq!(back.auth.method, AuthMethod::ApiKey);
    }

    #[test]
    fn auth_settings_serde_roundtrip() {
        let s = AuthSettings {
            method: AuthMethod::BearerToken,
            api_key: "k".into(),
            bearer_token: "t".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: AuthSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.method, AuthMethod::BearerToken);
        assert_eq!(back.bearer_token, "t");
    }

    #[test]
    fn local_settings_serde_roundtrip() {
        let s = LocalSettings { enabled: false, db_path: "/db".into(), retention_days: 5 };
        let json = serde_json::to_string(&s).unwrap();
        let back: LocalSettings = serde_json::from_str(&json).unwrap();
        assert!(!back.enabled);
        assert_eq!(back.retention_days, 5);
    }

    #[test]
    fn sync_settings_serde_roundtrip() {
        let s = SyncSettings { enabled: false, interval_ms: 1, batch_size: 2, timeout_secs: 3 };
        let json = serde_json::to_string(&s).unwrap();
        let back: SyncSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.batch_size, 2);
        assert!(!back.enabled);
    }

    #[test]
    fn policy_settings_serde_roundtrip() {
        let s = PolicySettings {
            enabled: false,
            default_action: PolicyDefaultAction::Deny,
            enforce_gates: false,
            enforce_rate_limits: false,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: PolicySettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.default_action, PolicyDefaultAction::Deny);
        assert!(!back.enforce_gates);
    }

    #[test]
    fn rate_limit_settings_serde_roundtrip() {
        let s = RateLimitSettings { enabled: true, max_requests: 9, window_ms: 10 };
        let json = serde_json::to_string(&s).unwrap();
        let back: RateLimitSettings = serde_json::from_str(&json).unwrap();
        assert!(back.enabled);
        assert_eq!(back.window_ms, 10);
    }

    #[test]
    fn policy_default_action_serde_strings() {
        assert_eq!(serde_json::to_string(&PolicyDefaultAction::Allow).unwrap(), "\"allow\"");
        assert_eq!(serde_json::to_string(&PolicyDefaultAction::Deny).unwrap(), "\"deny\"");
    }

    #[test]
    fn validate_clean_default() {
        assert!(GovernanceConfig::default().validate().is_empty());
    }

    #[test]
    fn validate_governance_enabled_requires_url() {
        let mut config = GovernanceConfig::default();
        config.governance.enabled = true;
        config.governance.auth.api_key = "k".into();
        config.governance.base_url = String::new();
        assert!(config.validate().iter().any(|e| e.contains("base URL")));
    }

    #[test]
    fn validate_governance_enabled_requires_api_key() {
        let mut config = GovernanceConfig::default();
        config.governance.enabled = true;
        config.governance.auth.method = AuthMethod::ApiKey;
        config.governance.auth.api_key = String::new();
        assert!(config.validate().iter().any(|e| e.contains("API key")));
    }

    #[test]
    fn validate_bearer_method_skips_api_key_check() {
        let mut config = GovernanceConfig::default();
        config.governance.enabled = true;
        config.governance.base_url = "http://x".into();
        config.governance.auth.method = AuthMethod::BearerToken;
        config.governance.auth.api_key = String::new();
        assert!(config.validate().is_empty());
    }

    #[test]
    fn validate_none_method_skips_api_key_check() {
        let mut config = GovernanceConfig::default();
        config.governance.enabled = true;
        config.governance.base_url = "http://x".into();
        config.governance.auth.method = AuthMethod::None;
        assert!(config.validate().is_empty());
    }

    #[test]
    fn validate_local_enabled_requires_db_path() {
        let mut config = GovernanceConfig::default();
        config.local.enabled = true;
        config.local.db_path = String::new();
        assert!(config.validate().iter().any(|e| e.contains("database path")));
    }

    #[test]
    fn validate_returns_multiple_errors() {
        let mut config = GovernanceConfig::default();
        config.governance.enabled = true;
        config.governance.base_url = String::new();
        config.governance.auth.api_key = String::new();
        config.local.db_path = String::new();
        assert!(config.validate().len() >= 3);
    }

    #[test]
    fn from_file_toml_parses_all_sections() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.toml");
        std::fs::write(
            &path,
            r#"
[governance]
enabled = true
base_url = "http://z"
timeout_secs = 7
retry_attempts = 2
[governance.auth]
method = "bearer-token"
api_key = ""
bearer_token = "tok"
[local]
enabled = true
db_path = "/tmp/z.db"
retention_days = 11
[sync]
enabled = false
interval_ms = 5
batch_size = 6
timeout_secs = 7
[policy]
enabled = true
default_action = "allow"
enforce_gates = true
enforce_rate_limits = false
[rate_limit]
enabled = true
max_requests = 8
window_ms = 9
"#,
        )
        .unwrap();
        let config = GovernanceConfig::from_file(&path).unwrap();
        assert_eq!(config.governance.timeout_secs, 7);
        assert_eq!(config.local.retention_days, 11);
        assert_eq!(config.rate_limit.max_requests, 8);
        assert_eq!(config.governance.auth.method, AuthMethod::BearerToken);
    }

    #[test]
    fn from_file_invalid_json_is_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "{not json").unwrap();
        assert!(GovernanceConfig::from_file(&path).is_err());
    }

    #[test]
    fn from_file_missing_path_is_error() {
        let path = std::path::Path::new("/nonexistent/governance-config.json");
        assert!(GovernanceConfig::from_file(path).is_err());
    }

    #[test]
    fn from_env_returns_positive_numeric_defaults() {
        let config = GovernanceConfig::from_env();
        assert!(config.governance.timeout_secs > 0);
        assert!(config.governance.retry_attempts > 0);
        assert!(config.local.retention_days > 0);
        assert!(config.rate_limit.window_ms > 0);
    }
}
