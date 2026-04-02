//! Default and no-op port implementations

/// A default port with configurable behavior
#[derive(Debug, Clone, Default)]
pub struct DefaultPort {
    pub simulate_latency: bool,
    pub latency_ms: u64,
    pub fail_on_use: bool,
}

impl DefaultPort {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_latency(mut self, ms: u64) -> Self {
        self.simulate_latency = true;
        self.latency_ms = ms;
        self
    }
    pub fn with_failure(mut self, _msg: impl Into<String>) -> Self {
        self.fail_on_use = true;
        self
    }
}

/// No-op port for testing
#[derive(Debug, Clone, Default)]
pub struct NoOpPort;

impl NoOpPort {
    pub fn new() -> Self {
        Self
    }
}
