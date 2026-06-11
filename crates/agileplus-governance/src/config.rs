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
use agileplus_config::config_builder;
use serde::{Deserialize, Serialize};

/// Main governance configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

config_builder! {
    /// Remote governance settings
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GovernanceSettings {
        /// Enable governance
        (val)  pub enabled: bool = false,
        /// Base URL for governance API
        (str)  pub base_url: String = "http://localhost:8080/api/v1".to_string(),
        /// Authentication settings
        (val)  pub auth: AuthSettings = AuthSettings::default(),
        /// Request timeout in seconds
        (val)  pub timeout_secs: u64 = 30,
        /// Retry attempts
        (val)  pub retry_attempts: u32 = 3,
    }
}

config_builder! {
    /// Authentication settings
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AuthSettings {
        /// Authentication method
        (val) pub method: AuthMethod = AuthMethod::ApiKey,
        /// API key (if using api-key auth)
        (str) pub api_key: String = String::new(),
        /// Bearer token (if using bearer auth)
        (str) pub bearer_token: String = String::new(),
    }
}

config_builder! {
    /// Local storage settings
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LocalSettings {
        /// Enable local governance storage
        (val) pub enabled: bool = true,
        /// Path to local database
        (str) pub db_path: String = ".agileplus/governance.db".to_string(),
        /// Retention days for audit logs
        (val) pub retention_days: u32 = 90,
    }
}

config_builder! {
    /// Sync settings for remote governance
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SyncSettings {
        /// Enable sync to remote
        (val) pub enabled: bool = true,
        /// Sync interval in milliseconds
        (val) pub interval_ms: u64 = 300_000,
        /// Batch size for sync
        (val) pub batch_size: usize = 100,
        /// Sync timeout in seconds
        (val) pub timeout_secs: u64 = 60,
    }
}

/// Default action when no policy matches
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyDefaultAction {
    #[default]
    Allow,
    Deny,
}

config_builder! {
    /// Policy enforcement settings
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PolicySettings {
        /// Enable policy enforcement
        (val) pub enabled: bool = true,
        /// Default action if no policy matches
        (val) pub default_action: PolicyDefaultAction = PolicyDefaultAction::Allow,
        /// Enforce channel gates
        (val) pub enforce_gates: bool = true,
        /// Enforce rate limits
        (val) pub enforce_rate_limits: bool = true,
    }
}

config_builder! {
    /// Rate limiting settings
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RateLimitSettings {
        /// Enable rate limiting
        (val) pub enabled: bool = false,
        /// Max requests per window
        (val) pub max_requests: u64 = 100,
        /// Window size in milliseconds
        (val) pub window_ms: u64 = 3_600_000,
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
}
