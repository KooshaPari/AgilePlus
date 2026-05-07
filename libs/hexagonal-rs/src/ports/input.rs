//! Input port traits (use case drivers)

use async_trait::async_trait;

/// Input port trait - called by driving adapters
#[async_trait]
pub trait InputPort<TInput, TResult>: Send + Sync {
    /// Execute the use case
    async fn execute(&self, input: TInput) -> TResult;
}
