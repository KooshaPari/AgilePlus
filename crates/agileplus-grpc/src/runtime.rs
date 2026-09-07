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
