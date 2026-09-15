use std::net::SocketAddr;
use std::path::PathBuf;

use agileplus_git::ProjectContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreConfig {
    pub bind: SocketAddr,
    pub project_root: PathBuf,
}

impl CoreConfig {
    pub fn from_values(
        bind: Option<&str>,
        project_root: Option<&str>,
        database: Option<&str>,
    ) -> Result<Self, String> {
        let bind = bind
            .unwrap_or("127.0.0.1:50051")
            .parse::<SocketAddr>()
            .map_err(|error| format!("invalid core bind address: {error}"))?;
        if !bind.ip().is_loopback() {
            return Err("plaintext AgilePlus core must bind to a loopback address".to_owned());
        }

        if database.is_some() {
            return Err(
                "AgilePlus core database path is not supported; project root is required"
                    .to_owned(),
            );
        }
        let project_root = project_root
            .map(str::trim)
            .filter(|root| !root.is_empty())
            .ok_or_else(|| "AgilePlus core project root is required".to_owned())?;

        Ok(Self {
            bind,
            project_root: PathBuf::from(project_root),
        })
    }

    pub fn from_env_values(
        bind: Option<&str>,
        project_root: Option<&str>,
        core_database: Option<&str>,
        legacy_database: Option<&str>,
    ) -> Result<Self, String> {
        Self::from_values(bind, project_root, core_database.or(legacy_database))
    }

    pub fn from_env() -> Result<Self, String> {
        Self::from_env_values(
            std::env::var("AGILEPLUS_GRPC_BIND").ok().as_deref(),
            std::env::var("AGILEPLUS_PROJECT_ROOT").ok().as_deref(),
            std::env::var("AGILEPLUS_CORE_DATABASE_PATH")
                .ok()
                .as_deref(),
            std::env::var("AGILEPLUS_DB_PATH").ok().as_deref(),
        )
    }

    pub fn context(&self) -> Result<ProjectContext, String> {
        ProjectContext::discover(&self.project_root).map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_values_defaults_bind() {
        let config = CoreConfig::from_values(None, Some("/tmp/proj"), None).unwrap();
        assert_eq!(config.bind, "127.0.0.1:50051".parse::<SocketAddr>().unwrap());
        assert_eq!(config.project_root, PathBuf::from("/tmp/proj"));
    }

    #[test]
    fn from_values_custom_bind() {
        let config = CoreConfig::from_values(
            Some("127.0.0.1:9999"),
            Some("/tmp/proj"),
            None,
        )
        .unwrap();
        assert_eq!(config.bind.port(), 9999);
    }

    #[test]
    fn from_values_rejects_non_loopback() {
        let err = CoreConfig::from_values(Some("0.0.0.0:50051"), Some("/tmp"), None)
            .unwrap_err();
        assert!(err.contains("loopback"));
    }

    #[test]
    fn from_values_rejects_database_path() {
        let err = CoreConfig::from_values(Some("127.0.0.1:50051"), Some("/tmp"), Some("db.sqlite"))
            .unwrap_err();
        assert!(err.contains("database path is not supported"));
    }

    #[test]
    fn from_values_rejects_missing_project_root() {
        let err = CoreConfig::from_values(Some("127.0.0.1:50051"), None, None).unwrap_err();
        assert!(err.contains("project root is required"));
    }

    #[test]
    fn from_values_rejects_empty_project_root() {
        let err = CoreConfig::from_values(Some("127.0.0.1:50051"), Some("  "), None).unwrap_err();
        assert!(err.contains("project root is required"));
    }

    #[test]
    fn from_values_rejects_invalid_bind() {
        let err = CoreConfig::from_values(Some("not-an-address"), Some("/tmp"), None).unwrap_err();
        assert!(err.contains("invalid core bind address"));
    }

    #[test]
    fn from_env_values_prefers_core_database() {
        // When both core_database and legacy_database are provided, core wins.
        let err = CoreConfig::from_env_values(
            Some("127.0.0.1:50051"),
            Some("/tmp"),
            Some("core.db"),
            Some("legacy.db"),
        )
        .unwrap_err();
        assert!(err.contains("database path is not supported"));
    }

    #[test]
    fn from_env_values_uses_legacy_database_when_core_absent() {
        let err = CoreConfig::from_env_values(
            Some("127.0.0.1:50051"),
            Some("/tmp"),
            None,
            Some("legacy.db"),
        )
        .unwrap_err();
        assert!(err.contains("database path is not supported"));
    }

    #[test]
    fn core_config_clone_eq() {
        let a = CoreConfig::from_values(None, Some("/a"), None).unwrap();
        let b = a.clone();
        assert_eq!(a, b);
    }
}
