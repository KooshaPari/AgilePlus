//! Disposable local browser witness: real Axum, canonical acceptance, file SQLite.
//! Only the example uses in-memory fixture credentials; production auth is unchanged.
use agileplus_api::{AppState, create_router};
use agileplus_domain::{
    config::AppConfig,
    credentials::{CredentialStore, InMemoryCredentialStore},
};
use agileplus_git::GitVcsAdapter;
use agileplus_sqlite::SqliteStorageAdapter;
use std::{path::PathBuf, sync::Arc};

#[allow(dead_code, unused_imports)]
#[path = "../tests/atomic_acceptance_http.rs"]
mod fixture;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let path = PathBuf::from(std::env::var("AGILEPLUS_TEST_DATABASE")?);
    let initialize = !path.exists();
    let database = Arc::new(SqliteStorageAdapter::new(&path)?);
    if initialize {
        fixture::seed(database.as_ref()).await;
    }
    let credentials: Arc<dyn CredentialStore> = Arc::new(InMemoryCredentialStore::new());
    agileplus_api::api_key::import_api_key(
        credentials.as_ref(),
        &std::env::var("AGILEPLUS_TEST_API_KEY")?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let state = AppState::new(
        database,
        Arc::new(GitVcsAdapter::new(std::env::temp_dir())),
        Arc::new(fixture::Noop),
        Arc::new(AppConfig::default()),
        credentials,
    )
    .with_atomic_acceptance();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:39001").await?;
    axum::serve(listener, create_router(state)).await?;
    Ok(())
}
