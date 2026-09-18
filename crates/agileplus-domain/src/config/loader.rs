use std::path::PathBuf;
use std::{env, fs};

use super::AppConfig;

/// Errors that can occur when loading or saving configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("invalid config value: {0}")]
    Validation(String),
    #[error("environment variable parse error: {0}")]
    EnvParse(String),
}

impl AppConfig {
    /// Path to the user-level config file: `~/.agileplus/config.toml`.
    pub fn config_path() -> PathBuf {
        dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".agileplus")
            .join("config.toml")
    }

    /// Load config from disk, falling back to defaults if no file exists.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: AppConfig = toml::from_str(&content)?;
            config.validate()?;
            Ok(config)
        } else {
            Ok(AppConfig::default())
        }
    }

    /// Load config with environment variable overrides applied after file parsing.
    ///
    /// Supported env vars: `API_PORT`, `AGILEPLUS_HTTP_PORT`,
    /// `AGILEPLUS_API_PORT`, `AGILEPLUS_GRPC_PORT`,
    /// `AGILEPLUS_TELEMETRY_LOG_LEVEL`, `AGILEPLUS_CORE_DB_PATH`,
    /// `AGILEPLUS_CORE_SPECS_DIR`.
    pub fn load_with_env_overrides() -> Result<Self, ConfigError> {
        let mut config = Self::load()?;

        if let Ok(port) = env::var("API_PORT") {
            config.api.port = port
                .parse()
                .map_err(|_| ConfigError::EnvParse(format!("API_PORT={port}")))?;
        }
        if let Ok(port) = env::var("AGILEPLUS_HTTP_PORT") {
            config.api.port = port
                .parse()
                .map_err(|_| ConfigError::EnvParse(format!("AGILEPLUS_HTTP_PORT={port}")))?;
        }
        if let Ok(port) = env::var("AGILEPLUS_API_PORT") {
            config.api.port = port
                .parse()
                .map_err(|_| ConfigError::EnvParse(format!("AGILEPLUS_API_PORT={port}")))?;
        }
        if let Ok(port) = env::var("AGILEPLUS_GRPC_PORT") {
            config.api.grpc_port = port
                .parse()
                .map_err(|_| ConfigError::EnvParse(format!("AGILEPLUS_GRPC_PORT={port}")))?;
        }
        if let Ok(level) = env::var("AGILEPLUS_TELEMETRY_LOG_LEVEL") {
            config.telemetry.log_level = level;
        }
        if let Ok(db) = env::var("AGILEPLUS_CORE_DB_PATH") {
            config.core.database_path = PathBuf::from(db);
        }
        if let Ok(dir) = env::var("AGILEPLUS_CORE_SPECS_DIR") {
            config.core.specs_dir = dir;
        }

        config.validate()?;
        Ok(config)
    }

    /// Write the current config (or a fresh default) to disk if the file does not yet exist.
    ///
    /// Returns the path that was written.
    pub fn init_default() -> Result<PathBuf, ConfigError> {
        let path = Self::config_path();
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = toml::to_string_pretty(&AppConfig::default())?;
            fs::write(&path, content)?;
        }
        Ok(path)
    }

    /// Validate that the loaded configuration is internally consistent.
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.telemetry.log_level.to_lowercase().as_str()) {
            return Err(ConfigError::Validation(format!(
                "invalid log_level '{}'; must be one of: {}",
                self.telemetry.log_level,
                valid_log_levels.join(", ")
            )));
        }
        if self.api.port == 0 {
            return Err(ConfigError::Validation("api.port must be > 0".to_string()));
        }
        if self.api.grpc_port == 0 {
            return Err(ConfigError::Validation(
                "api.grpc_port must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::test_support::env_lock;

    /// Env vars `load_with_env_overrides` understands, cleared for the duration
    /// of a test so the ambient developer environment cannot leak in.
    const OVERRIDE_VARS: [&str; 7] = [
        "API_PORT",
        "AGILEPLUS_HTTP_PORT",
        "AGILEPLUS_API_PORT",
        "AGILEPLUS_GRPC_PORT",
        "AGILEPLUS_TELEMETRY_LOG_LEVEL",
        "AGILEPLUS_CORE_DB_PATH",
        "AGILEPLUS_CORE_SPECS_DIR",
    ];

    struct EnvSandbox {
        _guard: std::sync::MutexGuard<'static, ()>,
        previous_home: Option<std::ffi::OsString>,
        previous_vars: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }

    impl EnvSandbox {
        /// Take the process-wide env lock, point `HOME` at `home`, and clear all
        /// override variables. Everything is restored on drop.
        fn new(home: &Path) -> Self {
            let guard = env_lock();
            let previous_home = std::env::var_os("HOME");
            let previous_vars = OVERRIDE_VARS
                .iter()
                .map(|name| (*name, std::env::var_os(name)))
                .collect();
            for name in OVERRIDE_VARS {
                unsafe { std::env::remove_var(name) };
            }
            unsafe { std::env::set_var("HOME", home) };
            Self {
                _guard: guard,
                previous_home,
                previous_vars,
            }
        }

        fn set(&self, name: &str, value: &str) {
            assert!(OVERRIDE_VARS.contains(&name), "unexpected var {name}");
            unsafe { std::env::set_var(name, value) };
        }
    }

    impl Drop for EnvSandbox {
        fn drop(&mut self) {
            for (name, value) in self.previous_vars.drain(..) {
                match value {
                    Some(value) => unsafe { std::env::set_var(name, value) },
                    None => unsafe { std::env::remove_var(name) },
                }
            }
            match self.previous_home.take() {
                Some(value) => unsafe { std::env::set_var("HOME", value) },
                None => unsafe { std::env::remove_var("HOME") },
            }
        }
    }

    fn home_with_config(contents: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir_all(home.join(".agileplus")).unwrap();
        std::fs::write(home.join(".agileplus/config.toml"), contents).unwrap();
        (dir, home)
    }

    #[test]
    fn config_path_is_under_the_home_directory() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let _sandbox = EnvSandbox::new(&home);
        assert_eq!(
            AppConfig::config_path(),
            home.join(".agileplus").join("config.toml")
        );
    }

    #[test]
    fn load_uses_defaults_when_no_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let _sandbox = EnvSandbox::new(&home);

        let config = AppConfig::load().unwrap();
        assert_eq!(config.api.port, AppConfig::default().api.port);
        assert_eq!(config.agents.max_subagents, 3);
        assert!(!home.join(".agileplus/config.toml").exists());
    }

    #[test]
    fn load_reads_and_validates_an_existing_file() {
        let (_dir, home) = home_with_config("[api]\nport = 4242\n");
        let _sandbox = EnvSandbox::new(&home);

        let config = AppConfig::load().unwrap();
        assert_eq!(config.api.port, 4242);
    }

    #[test]
    fn load_rejects_a_file_that_fails_validation() {
        let (_dir, home) = home_with_config("[api]\nport = 0\n");
        let _sandbox = EnvSandbox::new(&home);

        let error = AppConfig::load().unwrap_err();
        assert!(matches!(
            error,
            ConfigError::Validation(ref message) if message == "api.port must be > 0"
        ));
    }

    #[test]
    fn init_default_writes_a_loadable_file_once() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let _sandbox = EnvSandbox::new(&home);

        let written = AppConfig::init_default().unwrap();
        assert_eq!(written, home.join(".agileplus").join("config.toml"));
        assert!(written.is_file());

        // A second call is a no-op, so caller edits survive.
        let edited = AppConfig {
            api: crate::config::ApiConfig {
                port: 7777,
                ..AppConfig::default().api
            },
            ..AppConfig::default()
        };
        std::fs::write(&written, toml::to_string_pretty(&edited).unwrap()).unwrap();
        let second = AppConfig::init_default().unwrap();
        assert_eq!(second, written);
        assert_eq!(AppConfig::load().unwrap().api.port, 7777);
    }

    #[test]
    fn load_with_env_overrides_applies_every_supported_variable() {
        let (_dir, home) = home_with_config("[api]\nport = 1111\ngrpc_port = 2222\n");
        let sandbox = EnvSandbox::new(&home);
        sandbox.set("AGILEPLUS_API_PORT", "3333");
        sandbox.set("AGILEPLUS_GRPC_PORT", "4444");
        sandbox.set("AGILEPLUS_TELEMETRY_LOG_LEVEL", "debug");
        sandbox.set("AGILEPLUS_CORE_DB_PATH", "/tmp/env-override.db");
        sandbox.set("AGILEPLUS_CORE_SPECS_DIR", "specs-from-env");

        let config = AppConfig::load_with_env_overrides().unwrap();
        assert_eq!(config.api.port, 3333);
        assert_eq!(config.api.grpc_port, 4444);
        assert_eq!(config.telemetry.log_level, "debug");
        assert_eq!(
            config.core.database_path,
            PathBuf::from("/tmp/env-override.db")
        );
        assert_eq!(config.core.specs_dir, "specs-from-env");
    }

    #[test]
    fn later_port_overrides_win_over_earlier_ones() {
        let (_dir, home) = home_with_config("");
        let sandbox = EnvSandbox::new(&home);
        sandbox.set("API_PORT", "5001");
        sandbox.set("AGILEPLUS_HTTP_PORT", "5002");
        sandbox.set("AGILEPLUS_API_PORT", "5003");

        let config = AppConfig::load_with_env_overrides().unwrap();
        assert_eq!(config.api.port, 5003);
    }

    #[test]
    fn unparseable_env_port_is_reported_with_the_variable_name() {
        let (_dir, home) = home_with_config("");
        let sandbox = EnvSandbox::new(&home);
        sandbox.set("AGILEPLUS_HTTP_PORT", "not-a-port");

        let error = AppConfig::load_with_env_overrides().unwrap_err();
        assert!(matches!(
            error,
            ConfigError::EnvParse(ref message)
                if message == "AGILEPLUS_HTTP_PORT=not-a-port"
        ));
    }

    #[test]
    fn overrides_are_validated_after_being_applied() {
        let (_dir, home) = home_with_config("");
        let sandbox = EnvSandbox::new(&home);
        sandbox.set("AGILEPLUS_TELEMETRY_LOG_LEVEL", "verbose");

        let error = AppConfig::load_with_env_overrides().unwrap_err();
        assert!(matches!(
            error,
            ConfigError::Validation(ref message) if message.contains("invalid log_level 'verbose'")
        ));
    }

    #[test]
    fn zero_grpc_port_is_rejected() {
        let config = AppConfig {
            api: crate::config::ApiConfig {
                grpc_port: 0,
                ..AppConfig::default().api
            },
            ..AppConfig::default()
        };
        assert!(matches!(
            config.validate().unwrap_err(),
            ConfigError::Validation(ref message) if message == "api.grpc_port must be > 0"
        ));
    }
}
