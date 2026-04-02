# crates Specification

Canonical definition of system behavior for the crates repository.

---

## Repository Overview

**crates/** is a Rust workspace containing shared infrastructure crates extracted from the Phenotype ecosystem.

### Structure

```
crates/
├── phenotype-core/           # Umbrella crate re-exporting all phenotype crates
├── agileplus-*              # AgilePlus monorepo crates
├── phenotype-*              # Phenotype library crates
└── bifrost-*                # Bifrost routing crates
```

---

## phenotype-core — Umbrella Crate

Re-exports all common phenotype crates for easier dependency management.

### Re-exports

| Category | Types |
|----------|-------|
| **Error Handling** | ApiError, DomainError, RepositoryError, StorageError |
| **Configuration** | ConfigLoader, Priority |
| **Event Bus** | EventBus, EventEnvelope, EventId (ULID-based) |
| **Validation** | ValidationRule, RequiredRule |
| **Health** | HealthChecker, HealthStatus |
| **Ports** | Repository, CachePort, SecretPort |
| **Contracts** | InMemoryRepository, InMemoryCache |
| **Async** | AsyncIterator, RetryPolicy |

---

## AgilePlus Crates

The AgilePlus monorepo contains Rust crates for project management and developer tooling.

| Crate | Description |
|-------|-------------|
| **agileplus-api** | REST API for AgilePlus |
| **agileplus-api-types** | Shared type definitions |
| **agileplus-cli** | Command-line interface |
| **agileplus-domain** | Domain models |
| **agileplus-graph** | Graph-based data structures |
| **agileplus-sqlite** | SQLite persistence layer |
| **agileplus-error-core** | Error handling primitives |

---

## Phenotype Library Crates

Shared libraries for the Phenotype ecosystem.

### Core Libraries

| Crate | Description |
|-------|-------------|
| **phenotype-core** | Umbrella crate (see above) |
| **phenotype-config-core** | Configuration management |
| **phenotype-error-core** | Error type definitions |
| **phenotype-event-sourcing** | Append-only event store with SHA-256 hash chains |
| **phenotype-health** | Health check abstraction |
| **phenotype-state-machine** | Generic FSM with transition guards |

### Supporting Libraries

| Crate | Description |
|-------|-------------|
| **phenotype-cache-adapter** | Two-tier LRU + DashMap cache with TTL |
| **phenotype-policy-engine** | Rule-based policy evaluation with TOML config |
| **phenotype-contracts** | Shared traits and types |
| **phenotype-async-traits** | Async trait utilities |
| **phenotype-bdd** | BDD testing framework |
| **phenotype-git-core** | Git operations |
| **phenotype-logging** | Logging utilities |
| **phenotype-telemetry** | Observability primitives |

---

## Style Constraints

- **Line length**: 100 characters
- **Formatter**: `cargo fmt` (mandatory)
- **Type checker**: Rust compiler (strict)
- **Linter**: `cargo clippy` with `-- -D warnings` (zero warnings)
- **File size**: ≤350 lines target, ≤500 lines hard limit

---

## Dependencies

- No inter-crate dependencies; each crate is independently consumable
- All public types must implement `Debug` and `Clone` where practical
- Error types must use `thiserror` with proper `#[from]` conversions
- Workspace-level dependency management in root `Cargo.toml`

---

## Quality Standards

- All linters must pass: `cargo clippy --workspace -- -D warnings`
- All tests must pass: `cargo test --workspace`
- Test-First Mandate: test file must exist before implementation file
- FR Traceability: All tests must reference a Functional Requirement (FR)