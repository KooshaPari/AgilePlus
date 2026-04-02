//! Loaders for different configuration sources

pub mod error;

pub use error::{LoaderError, Result};
use async_trait::async_trait;
use phenotype_config_core::ConfigValue;
use std::collections::HashMap;
use std::path::PathBuf;

#[async_trait]
pub trait ConfigLoader: Send + Sync {
    async fn load(&self) -> Result<HashMap<String, ConfigValue>>;
}

pub struct FileSystemLoader {
    path: PathBuf,
}

impl FileSystemLoader {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[async_trait]
impl ConfigLoader for FileSystemLoader {
    async fn load(&self) -> Result<HashMap<String, ConfigValue>> {
        let content = tokio::fs::read_to_string(&self.path).await?;
        let values: HashMap<String, serde_json::Value> = if self.path.extension().and_then(|e| e.to_str()) == Some("toml") {
            toml::from_str(&content).map_err(|e| LoaderError::Parse(e.to_string()))?
        } else {
            serde_json::from_str(&content).map_err(|e| LoaderError::Parse(e.to_string()))?
        };

        let mut config = HashMap::new();
        for (key, val) in values {
            config.insert(key, ConfigValue::from_json(val));
        }
        Ok(config)
    }
}

pub struct EnvLoader {
    prefix: String,
}

impl EnvLoader {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self { prefix: prefix.into() }
    }
}

#[async_trait]
impl ConfigLoader for EnvLoader {
    async fn load(&self) -> Result<HashMap<String, ConfigValue>> {
        let mut config = HashMap::new();
        for (key, val) in std::env::vars() {
            if key.starts_with(&self.prefix) {
                let clean_key = key[self.prefix.len()..].to_string().to_lowercase();
                config.insert(clean_key, ConfigValue::String(val));
            }
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_file_system_loader_toml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "key = 'value'").unwrap();
        let path = file.path().to_path_buf();
        // Force toml extension for test
        let toml_path = path.with_extension("toml");
        std::fs::rename(&path, &toml_path).unwrap();

        let loader = FileSystemLoader::new(&toml_path);
        let config = loader.load().await.unwrap();
        assert_eq!(config.get("key").unwrap().as_str(), Some("value"));
        
        std::fs::remove_file(toml_path).unwrap();
    }

    #[tokio::test]
    async fn test_env_loader() {
        std::env::set_var("PHENO_TEST_KEY", "env_value");
        let loader = EnvLoader::new("PHENO_TEST_");
        let config = loader.load().await.unwrap();
        assert_eq!(config.get("key").unwrap().as_str(), Some("env_value"));
    }
}
