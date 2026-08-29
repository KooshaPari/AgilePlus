use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreConfig {
    pub bind: SocketAddr,
    pub database: PathBuf,
}

impl CoreConfig {
    pub fn from_values(bind: Option<&str>, database: Option<&str>) -> Result<Self, String> {
        let bind = bind
            .unwrap_or("127.0.0.1:50051")
            .parse::<SocketAddr>()
            .map_err(|error| format!("invalid core bind address: {error}"))?;
        if !bind.ip().is_loopback() {
            return Err("plaintext AgilePlus core must bind to a loopback address".to_owned());
        }

        Ok(Self {
            bind,
            database: PathBuf::from(database.unwrap_or(".agileplus/agileplus.db")),
        })
    }

    pub fn from_env() -> Result<Self, String> {
        Self::from_values(
            std::env::var("AGILEPLUS_GRPC_BIND").ok().as_deref(),
            std::env::var("AGILEPLUS_DB_PATH").ok().as_deref(),
        )
    }
}
