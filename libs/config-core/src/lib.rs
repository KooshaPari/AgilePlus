//! Configuration management for AgilePlus
//!
//! Provides trait-based configuration loading and saving with multiple format support.

pub mod error;

pub use error::{ConfigError, Result};

use serde::{Serialize, de::DeserializeOwned};

/// Configuration trait for loading and saving application settings.
pub trait Config: Sized {
    /// The configuration type.
    type Config: Serialize + DeserializeOwned + Default;

    /// Load configuration from a string.
    fn from_str(contents: &str) -> Result<Self>;

    /// Load configuration from a file.
    fn from_file(path: &str) -> Result<Self>;

    /// Save configuration to a string.
    fn to_string(&self) -> Result<String>;

    /// Save configuration to a file.
    fn to_file(&self, path: &str) -> Result<()>;

    /// Get the underlying configuration.
    fn config(&self) -> &Self::Config;

    /// Update configuration with a closure.
    fn update<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self::Config);
}

/// JSON configuration implementation.
#[derive(Debug, Clone)]
pub struct JsonConfig<T: Serialize + DeserializeOwned + Default> {
    config: T,
}

impl<T: Serialize + DeserializeOwned + Default> JsonConfig<T> {
    /// Create a new JSON configuration with default values.
    pub fn new() -> Self {
        Self {
            config: T::default(),
        }
    }

    /// Create from an existing config value.
    pub fn from_config(config: T) -> Self {
        Self { config }
    }
}

impl<T: Serialize + DeserializeOwned + Default> Default for JsonConfig<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Serialize + DeserializeOwned + Default> Config for JsonConfig<T> {
    type Config = T;

    fn from_str(contents: &str) -> Result<Self> {
        serde_json::from_str::<T>(contents)
            .map(JsonConfig::from_config)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    fn from_file(path: &str) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::LoadFailed(format!("{}: {}", path, e)))?;
        Self::from_str(&contents)
    }

    fn to_string(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| ConfigError::SaveFailed(e.to_string()))
    }

    fn to_file(&self, path: &str) -> Result<()> {
        let contents = self.to_string()?;
        std::fs::write(path, contents)
            .map_err(|e| ConfigError::SaveFailed(format!("{}: {}", path, e)))
    }

    fn config(&self) -> &T {
        &self.config
    }

    fn update<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut T),
    {
        f(&mut self.config);
        Ok(())
    }
}

/// TOML configuration implementation.
#[derive(Debug, Clone)]
pub struct TomlConfig<T: Serialize + DeserializeOwned + Default> {
    config: T,
}

impl<T: Serialize + DeserializeOwned + Default> TomlConfig<T> {
    /// Create a new TOML configuration with default values.
    pub fn new() -> Self {
        Self {
            config: T::default(),
        }
    }

    /// Create from an existing config value.
    pub fn from_config(config: T) -> Self {
        Self { config }
    }
}

impl<T: Serialize + DeserializeOwned + Default> Default for TomlConfig<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Serialize + DeserializeOwned + Default> Config for TomlConfig<T> {
    type Config = T;

    fn from_str(contents: &str) -> Result<Self> {
        toml::from_str::<T>(contents)
            .map(TomlConfig::from_config)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    fn from_file(path: &str) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::LoadFailed(format!("{}: {}", path, e)))?;
        Self::from_str(&contents)
    }

    fn to_string(&self) -> Result<String> {
        toml::to_string_pretty(&self.config).map_err(|e| ConfigError::SaveFailed(e.to_string()))
    }

    fn to_file(&self, path: &str) -> Result<()> {
        let contents = self.to_string()?;
        std::fs::write(path, contents)
            .map_err(|e| ConfigError::SaveFailed(format!("{}: {}", path, e)))
    }

    fn config(&self) -> &T {
        &self.config
    }

    fn update<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut T),
    {
        f(&mut self.config);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestConfig {
        name: String,
        port: u16,
        #[serde(default)]
        enabled: bool,
    }

    #[test]
    fn test_json_config_from_str() {
        let json = r#"{"name":"test","port":8080,"enabled":true}"#;
        let config: JsonConfig<TestConfig> = JsonConfig::from_str(json).unwrap();
        assert_eq!(config.config.name, "test");
        assert_eq!(config.config.port, 8080);
        assert!(config.config.enabled);
    }

    #[test]
    fn test_json_config_to_string() {
        let config = JsonConfig::from_config(TestConfig {
            name: "app".to_string(),
            port: 3000,
            enabled: false,
        });
        let json = config.to_string().unwrap();
        assert!(json.contains("\"app\""));
        assert!(json.contains("3000"));
    }

    #[test]
    fn test_toml_config_from_str() {
        let toml_str = r#"
name = "service"
port = 9000
enabled = true
"#;
        let config: TomlConfig<TestConfig> = TomlConfig::from_str(toml_str).unwrap();
        assert_eq!(config.config.name, "service");
        assert_eq!(config.config.port, 9000);
        assert!(config.config.enabled);
    }

    #[test]
    fn test_config_update() {
        let mut config: JsonConfig<TestConfig> = JsonConfig::new();
        config
            .update(|c: &mut TestConfig| {
                c.name = "updated".to_string();
                c.port = 9999;
            })
            .unwrap();
        assert_eq!(config.config.name, "updated");
        assert_eq!(config.config.port, 9999);
    }
}
