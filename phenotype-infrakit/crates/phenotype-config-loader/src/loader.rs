//! Configuration loader

use crate::error::{ConfigError, ConfigResult};

pub struct ConfigLoader {
    base_path: std::path::PathBuf,
}

impl ConfigLoader {
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
    
    pub fn load(&self, name: &str) -> ConfigResult<serde_json::Value> {
        let path = self.base_path.join(format!("{}.json", name));
        let content = std::fs::read_to_string(&path)
            .map_err(|e| ConfigError::LoadFailed(e.to_string()))?;
        serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}
