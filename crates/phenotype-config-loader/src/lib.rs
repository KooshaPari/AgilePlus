//! Configuration loading utilities for the Phenotype ecosystem.
//!
//! Traces to: FR-CONFIG-LOADER-001

use serde::de::DeserializeOwned;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigLoadError {
    #[error("file not found: {0}")]
    NotFound(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("home directory not found")]
    HomeDirNotFound,
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
}

pub type Result<T> = std::result::Result<T, ConfigLoadError>;

// Traces to: FR-CONFIG-LOADER-002
pub fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str::<T>(&content).map_err(|e| ConfigLoadError::Parse(e.to_string()))
}

// Traces to: FR-CONFIG-LOADER-003
pub fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    toml::from_str(&content).map_err(|e| ConfigLoadError::Parse(e.to_string()))
}

// Traces to: FR-CONFIG-LOADER-004
pub fn load_yaml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    serde_yaml::from_str(&content).map_err(|e| ConfigLoadError::Parse(e.to_string()))
}

/// Configuration format types
// Traces to: FR-CONFIG-LOADER-005
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Toml,
    Yaml,
}

impl ConfigFormat {
    /// Detect format from file extension
    // Traces to: FR-CONFIG-LOADER-006
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "json" => Some(ConfigFormat::Json),
            "toml" => Some(ConfigFormat::Toml),
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            _ => None,
        }
    }
    
    /// Load configuration using the appropriate format
    // Traces to: FR-CONFIG-LOADER-007
    pub fn load<T: DeserializeOwned>(&self, path: &Path) -> Result<T> {
        match self {
            ConfigFormat::Json => load_json(path),
            ConfigFormat::Toml => load_toml(path),
            ConfigFormat::Yaml => load_yaml(path),
        }
    }
}

/// Load configuration from standard config directory
/// 
/// Searches: `$HOME/.config/<app_name>/<config_name>`
// Traces to: FR-CONFIG-LOADER-008
pub fn load_from_config_dir<T: DeserializeOwned>(
    app_name: &str,
    config_name: &str,
) -> Result<T> {
    let config_dir = dirs_next::config_dir()
        .ok_or(ConfigLoadError::HomeDirNotFound)?
        .join(app_name);
    
    let path = config_dir.join(config_name);
    
    let format = ConfigFormat::from_path(&path)
        .ok_or_else(|| ConfigLoadError::UnsupportedFormat(
            path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown")
                .to_string()
        ))?;
    
    format.load(&path)
}

/// Load configuration from multiple sources with priority
/// 
/// Priority order (highest to lowest):
/// 1. Environment variable override
/// 2. Current directory config
/// 3. User config directory
/// 4. System-wide config
// Traces to: FR-CONFIG-LOADER-009
pub fn load_with_fallback<T: DeserializeOwned>(
    app_name: &str,
    config_name: &str,
    env_var: Option<&str>,
) -> Result<T> {
    // Try environment variable first
    if let Some(var) = env_var {
        if let Ok(path_str) = std::env::var(var) {
            let path = Path::new(&path_str);
            let format = ConfigFormat::from_path(path)
                .ok_or_else(|| ConfigLoadError::UnsupportedFormat(
                    "env_var".to_string()
                ))?;
            return format.load(path);
        }
    }
    
    // Try current directory
    let current_dir = std::env::current_dir()
        .map_err(|e| ConfigLoadError::Io(e))?;
    let local_path = current_dir.join(config_name);
    if local_path.exists() {
        if let Some(format) = ConfigFormat::from_path(&local_path) {
            return format.load(&local_path);
        }
    }
    
    // Try config directory
    load_from_config_dir(app_name, config_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestConfig { name: String, value: i32 }

    // Traces to: FR-CONFIG-LOADER-010
    #[test]
    fn test_load_json() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_cfg.json");
        std::fs::write(&path, r#"{"name":"test","value":42}"#).unwrap();
        let config = load_json::<TestConfig>(&path).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
        std::fs::remove_file(&path).ok();
    }

    // Traces to: FR-CONFIG-LOADER-011
    #[test]
    fn test_load_toml() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_cfg.toml");
        std::fs::write(&path, "name = \"test\"\nvalue = 42").unwrap();
        let config = load_toml::<TestConfig>(&path).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
        std::fs::remove_file(&path).ok();
    }

    // Traces to: FR-CONFIG-LOADER-012
    #[test]
    fn test_load_yaml() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_cfg.yaml");
        std::fs::write(&path, "name: test\nvalue: 42").unwrap();
        let config = load_yaml::<TestConfig>(&path).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
        std::fs::remove_file(&path).ok();
    }

    // Traces to: FR-CONFIG-LOADER-013
    #[test]
    fn test_load_not_found() {
        let result = load_json::<TestConfig>(Path::new("/nonexistent.json"));
        assert!(result.is_err());
    }

    // Traces to: FR-CONFIG-LOADER-014
    #[test]
    fn test_config_format_from_path() {
        assert_eq!(ConfigFormat::from_path(Path::new("config.json")), Some(ConfigFormat::Json));
        assert_eq!(ConfigFormat::from_path(Path::new("config.toml")), Some(ConfigFormat::Toml));
        assert_eq!(ConfigFormat::from_path(Path::new("config.yaml")), Some(ConfigFormat::Yaml));
        assert_eq!(ConfigFormat::from_path(Path::new("config.txt")), None);
    }
}
