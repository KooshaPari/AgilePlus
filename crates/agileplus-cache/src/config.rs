//! Cache configuration.

use agileplus_config::config_builder;

config_builder! {
    #[derive(Clone, Debug)]
    pub struct CacheConfig {
        (str)  pub host: String = "localhost".to_string(),
        (val)  pub port: u16 = 6379,
        (val)  pub pool_size: u32 = 16,
        (val)  pub default_ttl_secs: u64 = 3600,
        (val)  pub connection_timeout_secs: u64 = 5,
    }
}

impl CacheConfig {
    /// Construct with explicit host and port; all other fields use defaults.
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port, ..Self::default() }
    }

    /// Back-compat alias for [`with_default_ttl_secs`].
    #[inline]
    pub fn with_default_ttl(self, secs: u64) -> Self {
        self.with_default_ttl_secs(secs)
    }

    /// Redis connection URL.
    pub fn redis_url(&self) -> String {
        format!("redis://{}:{}", self.host, self.port)
    }
}
