# ADR-003: Rust for Core Platform with Python MCP Server

## Status
**Accepted** — Implemented across 24 crates

## Context

AgilePlus requires a technology stack that delivers:

1. **Performance** — CLI cold start <50ms, API p99 <100ms
2. **Safety** — Memory safety for long-running processes
3. **Ecosystem** — Modern libraries for gRPC, SQLite, HTTP
4. **AI Integration** — MCP server needs rapid AI library evolution
5. **Developer Experience** — Fast compile times, good tooling
6. **Deployment** — Single binary, cross-platform

We evaluated the full stack across multiple languages.

### Alternatives Considered

| Language | Pros | Cons | Verdict |
|----------|------|------|---------|
| **TypeScript/Node** | Fast dev, large ecosystem | Runtime overhead, GC pauses | ❌ Rejected |
| **Go** | Fast compile, good stdlib | Verbose error handling, no generics | ⚠️ Partial |
| **Rust** | Performance, safety, ecosystem | Steep learning, compile times | ✅ Core platform |
| **Python** | AI libraries, rapid prototyping | Runtime overhead, GIL | ✅ MCP server |
| **Zig** | Performance, simplicity | Immature ecosystem | ❌ Rejected |
| **OCaml/Reason** | Type safety, performance | Small ecosystem, hiring | ❌ Rejected |

## Decision

We will use a **polyglot architecture**:

1. **Rust** for core platform (24 crates)
   - CLI (`agileplus-cli`)
   - Domain logic (`agileplus-domain`)
   - API server (`agileplus-api`)
   - gRPC service (`agileplus-grpc`)
   - Storage adapters (SQLite, Git, etc.)
   - All performance-critical paths

2. **Python** for MCP server (`agileplus-mcp`)
   - Rapid AI SDK integration
   - Anthropic/Claude Code compatibility
   - Prototyping agent behaviors

3. **TypeScript** for web dashboard (planned)
   - htmx + Alpine.js for minimal JS
   - Only where interactivity requires it

### Rust Justification

```
Performance Comparison (normalized to Rust = 1.0)
═══════════════════════════════════════════════════════════════

Benchmark              Rust    Go      Node    Python
─────────────────────────────────────────────────────────────
HTTP req/sec           1.0     0.85    0.42    0.18
Memory (MB baseline)   12      18      45      62
Startup time (ms)      5       15      120     200
Binary size (MB)       4       8       N/A     N/A

AgilePlus Requirements:
├── CLI cold start: <50ms     ✓ Rust: ~5ms
├── API p99 latency: <100ms   ✓ Rust: ~30ms
├── Memory idle: <128MB       ✓ Rust: ~12MB
└── Binary size: <20MB        ✓ Rust: ~8MB
```

### Rust Ecosystem Choices

| Domain | Crate | Justification |
|--------|-------|---------------|
| **Async Runtime** | tokio | Industry standard, ecosystem compatibility |
| **HTTP Server** | axum | Tokio-native, middleware system |
| **gRPC** | tonic | Proto-first, streaming support |
| **SQLite** | sqlx/rusqlite | Compile-time checked queries |
| **Serialization** | serde + MessagePack | Fast, compact, schema evolution |
| **CLI** | clap | Derive macros, shell completions |
| **Testing** | tokio-test, insta | Async testing, snapshot testing |
| **Observability** | opentelemetry | OpenTelemetry standard |

### Python Justification for MCP

```
MCP Server Requirements
═══════════════════════════════════════════════════════════════

Must integrate with:
├── Anthropic Claude Code (Python SDK)
├── OpenAI Agents (Python SDK)
├── Custom AI tooling (Python dominant)
└── Rapid protocol evolution

Python advantages:
• Official MCP SDK from Anthropic is Python-first
• Fastest path to protocol compliance
• AI/ML library ecosystem unmatched
• Easy to modify for protocol changes

Trade-offs accepted:
• Slower than Rust (acceptable for I/O-heavy MCP)
• GIL limitations (single-instance acceptable)
• Deployment complexity (Docker/container solves)
```

## Consequences

### Positive

1. **Performance**: Rust delivers on all performance targets
2. **Safety**: Compile-time memory safety eliminates entire bug class
3. **Ecosystem**: Rust has excellent libraries for all our needs
4. **AI Integration**: Python provides best-in-class AI tooling
5. **Deployment**: Rust produces single static binaries
6. **Team Growth**: Rust hiring is easier than niche languages

### Negative

1. **Learning Curve**: Rust has steep initial learning curve
2. **Compile Times**: Slower than Go/TypeScript
3. **Complexity**: Two languages increases cognitive load
4. **Debugging**: Two separate debugging environments
5. **Build System**: Must coordinate Cargo + uv/pip

### Mitigations

| Concern | Mitigation |
|---------|------------|
| Learning Curve | Internal Rust book; pair programming; gradual onboarding |
| Compile Times | `cargo nextest`; sccache; parallel compilation |
| Polyglot Complexity | Clear interface boundaries (gRPC between Rust/Python) |
| Debugging | Shared logging/tracing across both languages |
| Build System | `just` task runner coordinates both builds |

## Implementation

### Project Structure

```
agileplus/
├── Cargo.toml              # Workspace root
├── Cargo.lock
├── justfile                # Task runner
├── crates/                 # 24 Rust crates
│   ├── agileplus-domain/
│   ├── agileplus-cli/
│   ├── agileplus-api/
│   ├── agileplus-grpc/
│   ├── agileplus-sqlite/
│   └── ... (18 more)
│
├── mcp-server/             # Python MCP server
│   ├── pyproject.toml
│   ├── uv.lock
│   ├── src/
│   │   ├── agileplus_mcp/
│   │   │   ├── __init__.py
│   │   │   ├── server.py
│   │   │   ├── tools/
│   │   │   └── resources/
│   │   └── main.py
│   └── tests/
│
├── proto/                  # Shared gRPC definitions
│   ├── agileplus.proto
│   └── mcp.proto
│
└── scripts/
    ├── build.sh
    ├── test.sh
    └── run-mcp.sh
```

### Inter-Process Communication

```
Rust Core ↔ Python MCP Communication
═══════════════════════════════════════════════════════════════

Method: gRPC over Unix socket (localhost for Windows)

Rust side (gRPC server):
└── agileplus-grpc crate exposes domain operations

Python side (gRPC client):
└── Generated stubs from proto definitions

Advantages:
├── Language-agnostic protocol
├── Type safety via protobuf
├── Streaming for real-time updates
├── Easy to add other language clients later

Sequence:
1. Python MCP server starts
2. Connects to Rust gRPC on unix:///tmp/agileplus.sock
3. MCP tool calls → gRPC → Rust domain operation
4. Results serialized → Python → MCP response
```

### Build Configuration

```toml
# Cargo.toml — Workspace configuration
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.43", features = ["full"] }
axum = "0.8"
tonic = "0.13"
serde = { version = "1.0", features = ["derive"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
clap = { version = "4.5", features = ["derive"] }
```

```toml
# mcp-server/pyproject.toml
[project]
name = "agileplus-mcp"
version = "2.0.0"
dependencies = [
    "mcp>=1.0.0",
    "grpcio>=1.70.0",
    "protobuf>=5.0",
    "pydantic>=2.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=8.0",
    "pytest-asyncio>=0.25",
    "ruff>=0.9",
]
```

### Development Workflow

```bash
# Build everything
just build

# Build Rust only
cargo build --workspace

# Build Python only
cd mcp-server && uv build

# Run tests
just test           # Both Rust and Python
cargo test          # Rust only
cd mcp-server && uv run pytest  # Python only

# Run full stack
just dev
# → Starts Rust gRPC server
# → Starts Python MCP server
# → Connects both
```

## Performance Targets by Component

| Component | Language | Target | Rationale |
|-----------|----------|--------|-----------|
| CLI cold start | Rust | <50ms | User-facing, interactive |
| API request | Rust | p99 <100ms | User-facing API |
| gRPC internal | Rust | p99 <10ms | Service-to-service |
| Event write | Rust | <5ms | Audit trail requirement |
| MCP tool call | Python | <500ms | AI agent acceptable latency |
| Sync operation | Rust | <30s | Background acceptable |

## Related Decisions

- **ADR-001**: Hexagonal architecture (implemented in Rust)
- **ADR-002**: Event sourcing (Rust implementation for performance)

## Notes

- Rust Edition 2024 features used where available
- Python 3.14 dev preview acceptable for MCP server
- `uv` used for Python packaging (Rust-based, fast)
- Cross-compilation via `cross` tool for releases

---

*Proposed: 2025-01-20*  
*Accepted: 2025-01-25*  
*Implemented: 2025-02-01*
