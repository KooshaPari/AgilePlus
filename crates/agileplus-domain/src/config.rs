//! Runtime configuration loaded from environment variables and TOML files.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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

impl AppConfig {
    /// Load config with environment variable overrides.
    pub fn load_with_env_overrides() -> anyhow::Result<Self> {
        let mut config = Self::default();

        if let Ok(port) = std::env::var("API_PORT").or_else(|_| std::env::var("AGILEPLUS_API_PORT"))
        {
            config.api.port = port.parse()?;
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
