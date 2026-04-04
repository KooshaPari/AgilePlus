# Feature Spec: phenotype-retry Implementation

## Feature Slug
`phenotype-retry-completion`

## Status
Draft → Ready for Specification

## Problem Statement

The `phenotype-retry` crate exists as a skeleton with only a placeholder comment. It has:
- Full dependency configuration (backoff, async-trait, tokio, thiserror)
- Workspace integration
- No implementation, tests, or documentation

This crate is essential for the Phenotype ecosystem as retry logic is a cross-cutting concern used by:
- HTTP clients (phenotype-http-client-core)
- Event bus (phenotype-event-bus)
- Health checks (phenotype-health background checks)
- Any async operation requiring resilience

## Goals

1. Provide a type-safe, async-first retry framework
2. Support multiple backoff strategies (exponential, fixed, linear, custom)
3. Integrate with phenotype-telemetry for observability
4. Support conditional retry based on error types
5. Follow BDD/TDD - specs first, then tests, then implementation

## Non-Goals

- Sync retry (async-only for consistency with ecosystem)
- Complex circuit breaker logic (use phenotype-sentinel for that)
- HTTP-specific retry logic (that's for phenotype-http-client-core)

## Design Specification

### Core Types

```rust
/// Retry policy configuration
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff: BackoffStrategy,
    pub retry_if: Box<dyn Fn(&Error) -> bool + Send + Sync>,
}

/// Backoff strategies
pub enum BackoffStrategy {
    Fixed { delay: Duration },
    Linear { base: Duration },
    Exponential { base: Duration, max: Duration },
    Custom(Box<dyn Fn(u32) -> Duration + Send + Sync>),
}

/// Retry context passed to operations
pub struct RetryContext {
    pub attempt: u32,
    pub max_attempts: u32,
    pub elapsed: Duration,
    pub last_error: Option<Error>,
}
```

### Traits

```rust
#[async_trait]
pub trait Retryable<T, E> {
    async fn retry(&self, policy: &RetryPolicy) -> Result<T, RetryError<E>>;
}
```

### Error Types

```rust
#[derive(Debug, Error)]
pub enum RetryError<E> {
    #[error("Max retry attempts ({attempts}) exceeded")]
    Exceeded { attempts: u32, last_error: E },
    
    #[error("Retry cancelled")]
    Cancelled,
}
```

## BDD Scenarios

### Scenario 1: Exponential Backoff Retry
```gherkin
Given an async operation that fails 2 times then succeeds
And a retry policy with exponential backoff starting at 100ms
When I execute the operation with retry
Then it should succeed on the 3rd attempt
And the delays between attempts should be approximately 100ms, 200ms
```

### Scenario 2: Non-Retryable Errors
```gherkin
Given an async operation that fails with a non-retryable error
And a retry policy that only retries on Io errors
When I execute the operation with retry
Then it should fail immediately without retry
```

### Scenario 3: Max Attempts Exhausted
```gherkin
Given an async operation that always fails
And a retry policy with max_attempts = 3
When I execute the operation with retry
Then it should fail with RetryError::Exceeded after 3 attempts
And the error should contain the last error
```

### Scenario 4: Telemetry Integration
```gherkin
Given an async operation with retry policy
And a telemetry hook is configured
When the operation retries
Then telemetry should record retry_count and retry_duration_ms
```

## Test Plan

1. Unit tests for each backoff strategy calculation
2. Integration tests with mock async operations
3. Property-based tests for retry exhaustion
4. Test for proper error propagation
5. Test for cancellation safety

## Integration Points

- **phenotype-telemetry**: Record retry metrics
- **phenotype-error-core**: Use standardized error codes
- **phenotype-async-traits**: Potentially extend with retry-aware traits

## Tasks

1. [ ] Write BDD specs in comments (this document)
2. [ ] Create test module with failing tests
3. [ ] Implement RetryPolicy and BackoffStrategy
4. [ ] Implement retry logic with tokio::time::sleep
5. [ ] Add telemetry integration
6. [ ] Add documentation and examples
7. [ ] Verify all tests pass
8. [ ] Run cargo clippy and fix warnings

## Acceptance Criteria

- [ ] All BDD scenarios have corresponding tests
- [ ] Test coverage > 80%
- [ ] Documentation includes examples
- [ ] No clippy warnings
- [ ] Integrates with phenotype-telemetry (optional but recommended)

## Related

- phenotype-error-core (error types)
- phenotype-telemetry (metrics)
- phenotype-http-client-core (consumer)
- phenotype-event-bus (consumer)
