# ADR-015: Configuration Management Strategy

**Status**: Proposed

**Date**: 2026-04-05

**Context**: AgilePlus needs a unified, consistent approach to configuration that works across CLI, MCP server, API, and dashboard entry points. Configuration sources include environment variables, config files, CLI flags, and remote config services. The system must support development, staging, and production environments with clear precedence rules.

---

## Decision Drivers

| Driver | Priority | Notes |
|--------|----------|-------|
| Environment parity | High | Dev/staging/prod config consistency |
| Security | High | No secrets in config files, proper secret rotation |
| DX (Developer Experience) | High | Easy to set up, clear error messages for misconfiguration |
| Flexibility | Medium | Support custom deployments per organization |
| Testability | Medium | Easy to mock/configure for unit and integration tests |

---

## Options Considered

### Option 1: ENV-first with Config File Override

**Description**: Environment variables are the primary source, with optional YAML/TOML config files for complex structures.

**Pros**:
- 12-factor app compatible
- Easy container orchestration
- No config file drift
- Secrets via secrets managers

**Cons**:
- ENV vars are global (pollution risk)
- No schema validation at startup
- Complex nested structures awkward in ENV

**Performance Data**:
| Metric | Value | Source |
|--------|-------|--------|
| Startup time (empty config) | ~2ms | Local benchmark |

### Option 2: Config File-first with ENV Overrides

**Description**: YAML/TOML config files define the schema, ENV vars override specific values.

**Pros**:
- Schema validation possible (serde, zod)
- Nested structures natural
- Version control friendly (non-secret config)
- IDE support (schema, autocomplete)

**Cons**:
- Config file path management complexity
- Secret values still need ENV or secret stores
- Additional file to manage

### Option 3: Hierarchical Config with Explicit Precedence

**Description**: Layered configuration with clear precedence: Default < Config File < Environment < CLI Flags < Runtime API.

**Description**: Unified config system combining:
- Default values in code
- Project-level `agileplus.yaml`
- User-level `~/.agileplus/config.yaml`
- Environment variables (`AGILEPLUS_*`)
- CLI flags (highest priority)
- Runtime config API for MCP/dynamic updates

**Pros**:
- Maximum flexibility
- Clear precedence rules
- All sources validated
- Works for all entry points

**Cons**:
- Most complex to implement
- Higher cognitive load for users

---

## Decision

**Chosen Option**: Option 3 - Hierarchical Configuration System

**Rationale**: AgilePlus serves multiple entry points (CLI, MCP, API) with varying configuration needs. A hierarchical system with explicit precedence provides the flexibility required for development, staging, and production environments while maintaining consistency across all entry points.

**Evidence**: 12-factor app methodology recommends hierarchical config for testability and environment parity.

---

## Configuration Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Configuration Sources                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐          │
│  │   Default   │    │   Config    │    │  Environment│          │
│  │   Values    │    │    File     │    │  Variables  │          │
│  │  (in code)  │    │  YAML/TOML  │    │  AGILEPLUS_ │          │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘          │
│         │                  │                  │                  │
│         └──────────────────┼──────────────────┘                  │
│                            ▼                                       │
│                 ┌─────────────────────┐                           │
│                 │  Config Aggregator   │                           │
│                 │   (precedence merge) │                           │
│                 └──────────┬──────────┘                           │
│                            │                                       │
│         ┌──────────────────┼──────────────────┐                  │
│         ▼                  ▼                  ▼                  │
│  ┌────────────┐    ┌────────────┐    ┌────────────┐              │
│  │    CLI     │    │    MCP     │    │    API     │              │
│  │   Flags    │    │   Server   │    │  Runtime   │              │
│  └────────────┘    └────────────┘    └────────────┘              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Config File Schema

```yaml
# agileplus.yaml (project-level)
version: "1"

database:
  url: "${AGILEPLUS_DATABASE_URL}"  # ENV interpolation
  pool_size: 5
  timeout_ms: 5000

logging:
  level: "${AGILEPLUS_LOG_LEVEL:-info}"
  format: json

features:
  local_first: true
  agent_mcp: true

# Organization-level overrides
organization:
  id: "${AGILEPLUS_ORG_ID}"
  sso_provider: "${AGILEPLUS_SSO_PROVIDER}"
```

### Rust Config Struct

```rust
// crates/agileplus-config/src/settings.rs

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub logging: LoggingSettings,
    pub features: FeatureFlags,
    pub organization: OrgSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub url: String,
    pub pool_size: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FeatureFlags {
    pub local_first: bool,
    pub agent_mcp: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrgSettings {
    pub id: Option<String>,
    pub sso_provider: Option<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let config = Config::builder()
            // Layer 1: Default values
            .set_default("database.pool_size", 5)?
            .set_default("database.timeout_ms", 5000)?
            .set_default("logging.level", "info")?
            .set_default("logging.format", "json")?
            .set_default("features.local_first", true)?
            
            // Layer 2: Project config file
            .add_source(File::with_name("agileplus").required(false))
            .add_source(File::with_name("~/.agileplus/config").required(false))
            
            // Layer 3: Environment variables (AGILEPLUS_*)
            .add_source(
                Environment::with_prefix("AGILEPLUS")
                    .separator("__")
                    .list_separator(",")
            )
            
            // Layer 4: CLI overrides (applied at runtime)
            .build()?;

        config.try_deserialize()
    }
}
```

### Environment Variable Precedence

| Variable | Config Path | Example |
|----------|-------------|---------|
| `AGILEPLUS_DATABASE__URL` | `database.url` | `sqlite://...` |
| `AGILEPLUS_DATABASE__POOL_SIZE` | `database.pool_size` | `10` |
| `AGILEPLUS_LOGGING__LEVEL` | `logging.level` | `debug` |
| `AGILEPLUS_FEATURES__LOCAL_FIRST` | `features.local_first` | `true` |

Note: Double underscore (`__`) in ENV vars maps to nested config paths.

---

## Implementation Plan

- [ ] Phase 1: Core config crate (`agileplus-config`) with layered loading - Target: 2026-04-15
- [ ] Phase 2: Schema validation with user-friendly error messages - Target: 2026-04-22
- [ ] Phase 3: Secret management integration (Vault, AWS Secrets Manager) - Target: 2026-05-01
- [ ] Phase 4: Runtime config API for MCP/dynamic updates - Target: 2026-05-15

---

## Consequences

### Positive

- Consistent config across all entry points
- Clear precedence rules eliminate ambiguity
- Environment parity between dev/staging/prod
- Easy to test with different configurations
- Secrets management integration ready

### Negative

- Additional complexity in config loading
- User education required on precedence rules
- Schema evolution requires migration strategy

### Neutral

- Config validation happens at startup (not lazy)
- Changes to config files require restart (except MCP runtime config)

---

## References

- [12-factor App: Config](https://12factor.net/config) - Configuration methodology
- [config-rs](https://github.com/mehcode/config-rs) - Rust configuration library
- [serde](https://serde.rs/) - Serialization framework
- [ADR-012: Plugin Architecture](./ADR-012-plugin-architecture.md) - Plugin config patterns
