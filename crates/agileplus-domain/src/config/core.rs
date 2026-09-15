use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoreConfig {
    #[serde(default = "default_db_path")]
    pub database_path: PathBuf,
    #[serde(default = "default_specs_dir")]
    pub specs_dir: String,
    #[serde(default = "default_target_branch")]
    pub default_target_branch: String,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            database_path: default_db_path(),
            specs_dir: default_specs_dir(),
            default_target_branch: default_target_branch(),
        }
    }
}

fn default_db_path() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".agileplus")
        .join("agileplus.db")
}

fn default_specs_dir() -> String {
    "agileplus".to_string()
}

fn default_target_branch() -> String {
    "main".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_core_config() {
        let c = CoreConfig::default();
        assert_eq!(c.specs_dir, "agileplus");
        assert_eq!(c.default_target_branch, "main");
        assert!(c.database_path.to_string_lossy().contains("agileplus.db"));
    }

    #[test]
    fn serde_roundtrip() {
        let c = CoreConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: CoreConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.specs_dir, c.specs_dir);
        assert_eq!(back.default_target_branch, c.default_target_branch);
    }
}
