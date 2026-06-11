use std::env;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use agileplus_api::AppState;
use agileplus_domain::config::AppConfig;
use agileplus_domain::credentials::create_credential_store;
use agileplus_git::GitVcsAdapter;
use agileplus_sqlite::SqliteStorageAdapter;
use agileplus_telemetry::{TelemetryAdapter, TelemetryConfig};
use anyhow::{anyhow, Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // --dump-openapi: serialize the OpenAPI document to YAML on stdout and exit.
    // When redirected to `openapi.yaml` at the workspace root, this seeds the
    // committed contract used by the (deferred) CI drift check.
    if env::args().any(|a| a == "--dump-openapi") {
        use utoipa::OpenApi;
        let openapi = agileplus_api::openapi::ApiDoc::openapi();
        let yaml =
            serde_yaml::to_string(&openapi).context("failed to serialize OpenAPI to YAML")?;
        print!("{yaml}");
        return Ok(());
    }

    let telemetry_config = TelemetryConfig::load().context("failed to load telemetry config")?;
    let config = load_runtime_config()?;
    ensure_database_parent(&config.core.database_path)?;
    let addr = bind_address(&config)?;

    let storage = Arc::new(SqliteStorageAdapter::new(&config.core.database_path)?);
    let vcs = Arc::new(GitVcsAdapter::from_current_dir()?);
    let telemetry = Arc::new(TelemetryAdapter::new(telemetry_config)?);
    let credentials = create_credential_store(&config);
    let state = AppState::new(storage, vcs, telemetry, Arc::new(config), credentials);

    agileplus_api::router::start_api(addr, state)
        .await
        .map_err(|err| anyhow!(err.to_string()))?;
    Ok(())
}

fn load_runtime_config() -> Result<AppConfig> {
    let mut config = AppConfig::load_with_env_overrides()?;

    if let Ok(database_url) = env::var("DATABASE_URL") {
        config.core.database_path = sqlite_path_from_database_url(&database_url)?;
    }

    Ok(config)
}

fn sqlite_path_from_database_url(database_url: &str) -> Result<PathBuf> {
    let path = database_url
        .strip_prefix("sqlite:")
        .ok_or_else(|| anyhow!("DATABASE_URL must use the sqlite: scheme"))?;

    if path.is_empty() {
        return Err(anyhow!(
            "DATABASE_URL must include a filesystem path after sqlite:"
        ));
    }

    Ok(PathBuf::from(path))
}

fn bind_address(config: &AppConfig) -> Result<SocketAddr> {
    let host = env::var("API_HOST")
        .or_else(|_| env::var("AGILEPLUS_API_HOST"))
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = config.api.port;
    let addr = format!("{host}:{port}");
    addr.to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow!("invalid API bind address"))
}

fn ensure_database_parent(database_path: &Path) -> Result<()> {
    if let Some(parent) = database_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create database directory {parent:?}"))?;
    }
    Ok(())
}
