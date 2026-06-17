//! Runtime configuration loaded from environment variables and TOML files.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub core: CoreConfig,
    pub api: ApiConfig,
}

/// Core / infrastructure settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Path to the SQLite database file.
    #[serde(default = "default_database_path")]
    pub database_path: PathBuf,
}

fn default_database_path() -> PathBuf {
    PathBuf::from("agileplus.db")
}

/// HTTP API settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Port the HTTP server listens on.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Comma-separated list of valid API keys (plaintext).
    pub api_keys: Option<String>,
}

fn default_port() -> u16 {
    3030
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            core: CoreConfig {
                database_path: default_database_path(),
            },
            api: ApiConfig {
                port: default_port(),
                api_keys: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.api.port, 3030);
        assert_eq!(cfg.core.database_path, PathBuf::from("agileplus.db"));
        assert!(cfg.api.api_keys.is_none());
    }

    #[test]
    fn app_config_serde_roundtrip() {
        let cfg = AppConfig::default();
        let json = serde_json::to_string(&cfg).expect("domain operation");
        let back: AppConfig = serde_json::from_str(&json).expect("domain operation");
        assert_eq!(back.api.port, cfg.api.port);
        assert_eq!(back.core.database_path, cfg.core.database_path);
    }

    #[test]
    fn load_with_env_overrides_reads_api_port() {
        std::env::set_var("API_PORT", "9090");
        let cfg = AppConfig::load_with_env_overrides().expect("domain operation");
        assert_eq!(cfg.api.port, 9090);
        std::env::remove_var("API_PORT");
    }

    #[test]
    fn load_with_env_overrides_reads_database_path() {
        std::env::set_var("DATABASE_PATH", "/tmp/test.db");
        let cfg = AppConfig::load_with_env_overrides().expect("domain operation");
        assert_eq!(cfg.core.database_path, PathBuf::from("/tmp/test.db"));
        std::env::remove_var("DATABASE_PATH");
    }
}

impl AppConfig {
    /// Load config with environment variable overrides.
    pub fn load_with_env_overrides() -> Result<Self, DomainError> {
        let mut config = Self::default();

        if let Ok(port) = std::env::var("API_PORT").or_else(|_| std::env::var("AGILEPLUS_API_PORT"))
        {
            config.api.port = port
                .parse()
                .map_err(|e| DomainError::Validation(format!("invalid API_PORT '{port}': {e}")))?;
        }

        if let Ok(db) = std::env::var("DATABASE_PATH") {
            config.core.database_path = PathBuf::from(db);
        }

        if let Ok(keys) = std::env::var("AGILEPLUS_API_KEY")
            .or_else(|_| std::env::var("API_KEYS"))
            .or_else(|_| std::env::var("AGILEPLUS_API_KEYS"))
        {
            config.api.api_keys = Some(keys);
        }

        Ok(config)
    }
}
