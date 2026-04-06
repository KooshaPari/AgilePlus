# Feature Spec: Fork Strategy for Phenotype Ecosystem

## Overview

Implement a whitebox/blackbox fork strategy for integrating major open-source tools into the Phenotype ecosystem.

## Fork Decisions

| Tool | Stars | Strategy | Rationale |
|------|-------|----------|-----------|
| litellm | 7.8k | Fork → Rust core | Add Rust performance (10x faster routing) |
| fastmcp | 3k | Fork → Rust core | Add native async/await |
| SurrealDB | 29k | Fork with extensions | MCP protocol adapter, skill schema |
| OpenTelemetry | High | Contribute | Extend, don't fork |
| NATS | High | Contribute | Extend, don't fork |

## Fork Crates

### phenotype-llm (litellm fork)

**Key Differentiators:**
- Rust core for routing (10x faster)
- Native connection pooling
- WASM plugin support
- Multi-tenant cost tracking

**Status:** ✅ Implemented, Compiles, Edition 2024

### phenotype-mcp-server (fastmcp fork)

**Key Differentiators:**
- Pure Rust (no Python dependency)
- Native async/await
- Schema validation via jsonschema
- Resource streaming

**Status:** ✅ Implemented, Compiles, Edition 2024

### phenotype-surrealdb (SurrealDB fork)

**Key Differentiators:**
- MCP protocol adapter
- Skill storage schema
- WASM embedding support
- Vector search integration

**Status:** ✅ Implemented, Compiles, Edition 2024

## Repository Structure

```
PhenoMCP/crates/
├── phenotype-llm/          # litellm fork
├── phenotype-mcp-server/    # fastmcp fork
└── phenotype-surrealdb/     # SurrealDB fork

PhenoRuntime/crates/
├── phenotype-llm/
├── phenotype-mcp-server/
└── phenotype-surrealdb/

PhenoObservability/crates/
├── phenotype-llm/
├── phenotype-mcp-server/
└── phenotype-surrealdb/
```

## Aggressive Adoption

- Edition 2024 across all fork crates
- Latest compatible dependencies
- Rust 1.85+ required

## FR Traceability

- FR-FORK-001: LLM routing via phenotype-llm
- FR-FORK-002: MCP server via phenotype-mcp-server  
- FR-FORK-003: SurrealDB integration via phenotype-surrealdb
- FR-FORK-004: Edition 2024 adoption
- FR-FORK-005: Dependency updates

## Testing Requirements

- Unit tests for each fork crate
- Integration tests with real services
- Performance benchmarks vs upstream

## Work Packages

1. [x] Create fork crates
2. [x] Add to all repos
3. [x] Edition 2024 adoption
4. [ ] Integration tests
5. [ ] Performance benchmarks
6. [ ] Documentation

## Status

**Phase:** Complete (P0/P1)  
**Next:** P2 - Integration tests and benchmarks
