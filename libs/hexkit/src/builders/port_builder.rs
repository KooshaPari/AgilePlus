//! Port builder for fluent configuration

use crate::error::{HexkitError, HexkitResult};

#[derive(Debug, Clone, Default)]
pub struct PortConfig {
    pub cache_enabled: bool,
    pub timeout_secs: Option<u64>,
    pub retry_enabled: bool,
    pub max_retries: Option<u32>,
}

#[derive(Clone, Default)]
pub struct PortBuilder<P> {
    config: PortConfig,
    _phantom: std::marker::PhantomData<P>,
}

impl<P> PortBuilder<P> {
    pub fn new() -> Self {
        Self {
            config: PortConfig::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.config.cache_enabled = enabled;
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.config.timeout_secs = Some(secs);
        self
    }

    pub fn with_retry(mut self, enabled: bool) -> Self {
        self.config.retry_enabled = enabled;
        self
    }

    pub fn with_max_retries(mut self, max: u32) -> Self {
        self.config.max_retries = Some(max);
        self
    }

    pub fn build(self) -> HexkitResult<PortConfig> {
        if self.config.retry_enabled && self.config.max_retries.is_none() {
            return Err(HexkitError::BuilderValidation(
                "retry enabled but max_retries not set".into(),
            ));
        }
        Ok(self.config)
    }
}

#[derive(Clone, Default)]
pub struct ContextBuilder<Storage, Vcs, Agent> {
    storage: Option<Storage>,
    vcs: Option<Vcs>,
    agent: Option<Agent>,
}

impl<Storage, Vcs, Agent> ContextBuilder<Storage, Vcs, Agent> {
    pub fn new() -> Self {
        Self {
            storage: None,
            vcs: None,
            agent: None,
        }
    }

    pub fn with_storage(mut self, storage: Storage) -> Self {
        self.storage = Some(storage);
        self
    }
    pub fn with_vcs(mut self, vcs: Vcs) -> Self {
        self.vcs = Some(vcs);
        self
    }
    pub fn with_agent(mut self, agent: Agent) -> Self {
        self.agent = Some(agent);
        self
    }

    pub fn build(self) -> HexkitResult<(Storage, Vcs, Agent)> {
        let storage = self
            .storage
            .ok_or_else(|| HexkitError::BuilderValidation("storage not set".into()))?;
        let vcs = self
            .vcs
            .ok_or_else(|| HexkitError::BuilderValidation("vcs not set".into()))?;
        let agent = self
            .agent
            .ok_or_else(|| HexkitError::BuilderValidation("agent not set".into()))?;
        Ok((storage, vcs, agent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_builder() {
        let builder = PortBuilder::<()>::new();
        let config = builder
            .with_cache(true)
            .with_timeout(30)
            .with_retry(true)
            .with_max_retries(3)
            .build()
            .unwrap();
        assert!(config.cache_enabled);
        assert_eq!(config.timeout_secs, Some(30));
        assert!(config.retry_enabled);
        assert_eq!(config.max_retries, Some(3));
    }

    #[test]
    fn test_port_builder_invalid_retry() {
        let builder = PortBuilder::<()>::new().with_retry(true);
        let res = builder.build();
        assert!(res.is_err());
    }

    #[test]
    fn context_builder_test() {
        let ctx = ContextBuilder::<i32, String, bool>::new()
            .with_storage(42)
            .with_vcs("git".to_string())
            .with_agent(true)
            .build()
            .unwrap();
        assert_eq!(ctx.0, 42);
        assert_eq!(ctx.1, "git");
        assert!(ctx.2);
    }

    #[test]
    fn context_builder_missing_vcs() {
        let res = ContextBuilder::<i32, String, bool>::new()
            .with_storage(42)
            .with_agent(true)
            .build();
        assert!(res.is_err());
    }
}
