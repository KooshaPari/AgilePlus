//! Cache configuration.

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub host: String,
    pub port: u16,
    pub pool_size: u32,
    pub default_ttl_secs: u64,
    pub connection_timeout_secs: u64,
}

impl CacheConfig {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            pool_size: 16,
            default_ttl_secs: 3600,
            connection_timeout_secs: 5,
        }
    }

    pub fn with_pool_size(mut self, size: u32) -> Self {
        self.pool_size = size;
        self
    }

    pub fn with_default_ttl(mut self, secs: u64) -> Self {
        self.default_ttl_secs = secs;
        self
    }

    pub fn redis_url(&self) -> String {
        format!("redis://{}:{}", self.host, self.port)
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::new("localhost".into(), 6379)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_new_localhost() {
        let default = CacheConfig::default();
        let explicit = CacheConfig::new("localhost".into(), 6379);
        assert_eq!(default.host, explicit.host);
        assert_eq!(default.port, explicit.port);
        assert_eq!(default.pool_size, explicit.pool_size);
        assert_eq!(default.default_ttl_secs, explicit.default_ttl_secs);
        assert_eq!(
            default.connection_timeout_secs,
            explicit.connection_timeout_secs
        );
    }

    #[test]
    fn builder_preserves_unrelated_fields() {
        let config = CacheConfig::new("cache.internal".into(), 6380)
            .with_pool_size(4)
            .with_default_ttl(60);

        assert_eq!(config.host, "cache.internal");
        assert_eq!(config.port, 6380);
        assert_eq!(config.pool_size, 4);
        assert_eq!(config.default_ttl_secs, 60);
        assert_eq!(config.connection_timeout_secs, 5);
    }

    #[test]
    fn redis_url_uses_host_and_port() {
        assert_eq!(CacheConfig::default().redis_url(), "redis://localhost:6379");
        assert_eq!(
            CacheConfig::new("dragonfly.local".into(), 6381).redis_url(),
            "redis://dragonfly.local:6381"
        );
    }
}
