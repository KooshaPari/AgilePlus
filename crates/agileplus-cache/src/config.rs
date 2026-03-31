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
    fn cache_config_new() {
        let config = CacheConfig::new("127.0.0.1".into(), 6380);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 6380);
        assert_eq!(config.pool_size, 16);
        assert_eq!(config.default_ttl_secs, 3600);
        assert_eq!(config.connection_timeout_secs, 5);
    }

    #[test]
    fn cache_config_with_pool_size() {
        let config = CacheConfig::new("localhost".into(), 6379).with_pool_size(32);
        assert_eq!(config.pool_size, 32);
    }

    #[test]
    fn cache_config_with_default_ttl() {
        let config = CacheConfig::new("localhost".into(), 6379).with_default_ttl(7200);
        assert_eq!(config.default_ttl_secs, 7200);
    }

    #[test]
    fn cache_config_redis_url() {
        let config = CacheConfig::new("myhost".into(), 6379);
        assert_eq!(config.redis_url(), "redis://myhost:6379");
    }

    #[test]
    fn cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 6379);
        assert_eq!(config.pool_size, 16);
        assert_eq!(config.default_ttl_secs, 3600);
        assert_eq!(config.connection_timeout_secs, 5);
    }

    #[test]
    fn cache_config_clone() {
        let config = CacheConfig::new("clonehost".into(), 6379).with_pool_size(64);
        let cloned = config.clone();
        assert_eq!(cloned.host, config.host);
        assert_eq!(cloned.port, config.port);
        assert_eq!(cloned.pool_size, config.pool_size);
        assert_eq!(cloned.default_ttl_secs, config.default_ttl_secs);
        assert_eq!(cloned.connection_timeout_secs, config.connection_timeout_secs);
    }

    #[test]
    fn cache_config_debug() {
        let config = CacheConfig::new("debughost".into(), 6379);
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("CacheConfig"));
        assert!(debug_str.contains("debughost"));
    }
}
