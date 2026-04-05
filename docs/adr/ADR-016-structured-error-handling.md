# ADR-016: Structured Error Handling Strategy

**Status**: Proposed

**Date**: 2026-04-05

**Context**: AgilePlus needs a consistent error handling strategy that works across all layers: domain logic, adapters, CLI, MCP server, and API. Errors must be user-friendly for CLI output, machine-parseable for API responses, and actionable for debugging. The system must support error codes for programmatic handling, localization, and proper error chain preservation.

---

## Decision Drivers

| Driver | Priority | Notes |
|--------|----------|-------|
| Debuggability | High | Clear error context for troubleshooting |
| User Experience | High | Human-readable messages without jargon |
| API Consistency | High | Structured error responses across all entry points |
| Error Code Stability | Medium | Codes must be stable across versions |
| Error Recovery | Medium | Distinguish recoverable vs fatal errors |

---

## Options Considered

### Option 1: ad-hoc Error Strings

**Description**: Simple `Result<T, String>` or `anyhow::Error` throughout the codebase.

**Pros**:
- Simple to implement
- Flexible

**Cons**:
- No error codes for programmatic handling
- No structured data for API responses
- Inconsistent error formats
- Difficult to localize

**Performance Data**:
| Metric | Value | Source |
|--------|-------|--------|
| Implementation time | ~1 day | Initial estimate |

### Option 2: Thiserror with Domain Error Codes

**Description**: Use `thiserror` crate with custom error types, each having a unique error code.

**Description**: Structured errors using `thiserror` with:
- Unique error codes per error variant
- Structured context (fields relevant to each error)
- Error chain preservation via `source()`
- Automatic `Display` and `Debug` implementations

**Pros**:
- Compile-time error exhaustiveness
- Structured error context
- Error chain preservation
- Works with `?` operator

**Cons**:
- Requires discipline to maintain error codes
- Can become verbose for complex error hierarchies

### Option 3: Custom Error Enum with Codes + anyhow for Context

**Description**: Hybrid approach: domain errors are typed enums with codes; library/framework errors use `anyhow` for context.

**Pros**:
- Typed domain errors for business logic
- Flexible context for integration code
- Best of both worlds

**Cons**:
- Two error handling patterns in codebase
- Requires clear guidelines on when to use which

---

## Decision

**Chosen Option**: Option 2 - Thiserror with Domain Error Codes

**Rationale**: AgilePlus is a domain-driven system where errors have business meaning (e.g., `FeatureNotFound`, `InvalidStateTransition`). Using typed errors with codes provides the best debugging experience while enabling structured API responses and error code stability.

**Evidence**: This pattern is used by established Rust projects (rustls, tokio, clap) for good reason.

---

## Error Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Error Hierarchy                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│                     ┌──────────────────┐                        │
│                     │   AgilePlusError │                        │
│                     │   (error codes)  │                        │
│                     └────────┬─────────┘                        │
│                              │                                   │
│        ┌─────────────────────┼─────────────────────┐            │
│        │                     │                     │            │
│        ▼                     ▼                     ▼            │
│ ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│ │  DomainError │    │ AdapterError │    │  ConfigError │       │
│ │  (thiserror) │    │  (thiserror) │    │  (thiserror) │       │
│ └──────────────┘    └──────────────┘    └──────────────┘       │
│        │                     │                     │            │
│        ▼                     ▼                     ▼            │
│ ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│ │FeatureNotFnd │    │   GitError   │    │  EnvVarError │       │
│ │InvalidTrans  │    │  SQLiteError │    │  ParseError  │       │
│ │CycleViolatn  │    │   LlmError   │    │  ValidError  │       │
│ └──────────────┘    └──────────────┘    └──────────────┘       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Error Code Ranges

| Range | Category | Examples |
|-------|----------|----------|
| 1000-1999 | Domain Errors | FeatureNotFound, InvalidStateTransition |
| 2000-2999 | Adapter Errors | GitError, SQLiteError, LlmError |
| 3000-3999 | Config Errors | EnvVarError, ParseError, ValidError |
| 4000-4999 | Validation Errors | SlugFormatError, TitleLengthError |
| 5000-5999 | Permission Errors | UnauthorizedError, ForbiddenError |

### Domain Error Implementation

```rust
// crates/agileplus-domain/src/errors.rs

use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum DomainError {
    #[error("feature not found: {slug}")]
    #[code(1001)]
    FeatureNotFound {
        slug: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("invalid state transition: {from} -> {to} (rule: {rule})")]
    #[code(1002)]
    InvalidStateTransition {
        from: String,
        to: String,
        rule: String,
        entity_id: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("cycle violation: cannot complete cycle '{cycle_name}'")]
    #[code(1003)]
    CycleViolation {
        cycle_name: String,
        blockers: Vec<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("work package not found: {id}")]
    #[code(1004)]
    WorkPackageNotFound {
        id: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("spec validation failed: {message}")]
    #[code(1005)]
    SpecValidationFailed {
        message: String,
        errors: Vec<SpecValidationError>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecValidationError {
    pub field: String,
    pub message: String,
}
```

### API Error Response

```rust
// crates/agileplus-api/src/error.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiError,
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: u32,
    pub message: String,
    pub details: Option<Value>,
    pub domain: String,
    pub trace: Option<String>,  // Stack trace in debug builds
}

impl From<DomainError> for ApiErrorResponse {
    fn from(err: DomainError) -> Self {
        ApiErrorResponse {
            error: ApiError {
                code: err.code(),
                message: err.to_string(),
                details: err.details(),
                domain: "domain".to_string(),
                trace: std::env::var("RUST_BACKTRACE").ok().filter(|_| cfg!(debug_assertions)),
            },
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        }
    }
}
```

### CLI Error Output

```rust
// crates/agileplus-cli/src/error.rs

impl Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Domain(err) => {
                // Pretty error for humans
                write!(f, "{}", style("error:").red().bold())?;
                writeln!(f, "  {}", err.message())?;
                if let Some(hint) = err.hint() {
                    writeln!(f, "  hint: {}", style(hint).cyan())?;
                }
            }
            CliError::Io(err) => {
                write!(f, "{}: {}", style("IO error").red(), err)?;
            }
        }
        Ok(())
    }
}
```

Example CLI output:
```
error: feature not found: user-auth-flow

  The feature 'user-auth-flow' does not exist in this workspace.

  hint: Run 'agileplus feature list' to see available features.
```

---

## Error Recovery Patterns

| Error Type | Recovery Strategy |
|------------|-------------------|
| Network timeout | Retry with exponential backoff |
| Resource not found | Clear error message, suggest alternatives |
| Invalid input | Show validation errors, suggest corrections |
| Permission denied | Explain required permissions |
| State conflict | Show current state, suggest valid transitions |

---

## Implementation Plan

- [ ] Phase 1: Define error code ranges and base error enum - Target: 2026-04-15
- [ ] Phase 2: Implement thiserror errors in domain crate - Target: 2026-04-20
- [ ] Phase 3: API error response format with request IDs - Target: 2026-04-25
- [ ] Phase 4: CLI error formatting with hints - Target: 2026-05-01
- [ ] Phase 5: Error code documentation and tooling - Target: 2026-05-10

---

## Consequences

### Positive

- Structured errors enable programmatic handling
- Error codes provide stable API contracts
- Human-readable messages improve UX
- Error chains preserve debugging context
- Consistent error format across all entry points

### Negative

- Initial investment in error type definitions
- Error code registry requires maintenance
- Need tooling to generate code from error codes

### Neutral

- Error handling is verbose in type definitions
- Some errors don't fit neatly into categories

---

## References

- [thiserror crate](https://github.com/dtolnay/thiserror) - Error handling patterns
- [anyhow crate](https://github.com/dtolnay/anyhow) - Context errors
- [RFC 7807: Problem Details for HTTP APIs](https://tools.ietf.org/html/rfc7807) - Error response standard
- [ADR-012: Plugin Architecture](./ADR-012-plugin-architecture.md) - Plugin error boundaries
