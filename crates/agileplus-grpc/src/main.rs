use std::sync::Arc;

use agileplus_grpc::event_bus::EventBus;
use agileplus_grpc::proxy::ProxyRouter;
use agileplus_grpc::runtime::CoreConfig;
use agileplus_sqlite::SqliteStorageAdapter;

fn ensure_database_parent(database: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = database.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let config = CoreConfig::from_env()?;
    ensure_database_parent(&config.database)?;

    let storage = Arc::new(SqliteStorageAdapter::new(&config.database)?);
    let proxy = Arc::new(ProxyRouter::new(None, None).await);
    tracing::info!(bind = %config.bind, database = %config.database.display(), "starting AgilePlus core");

    agileplus_grpc::server::start_server(config.bind, storage, Arc::new(EventBus::new(256)), proxy)
        .await
}

#[cfg(test)]
mod tests {
    #[test]
    fn filename_only_database_path_has_no_parent_directory_to_create() {
        super::ensure_database_parent(std::path::Path::new("agileplus.db")).unwrap();
    }
}
