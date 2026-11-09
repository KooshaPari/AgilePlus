---
work_package_id: WP08
title: 'cliproxyapi++: Plugin System + Hexagonal Ports'
lane: planned
dependencies: []
subtasks: [T019, T020, T021, T022]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
- date: '2026-03-03'
  event: revised
  by: agent
  note: 'Incorporate external-library-leverage research: tmaxmax/go-sse, coder/websocket for custom executor streaming'
---

# WP08: cliproxyapi++ — Plugin System + Hexagonal Ports

**Implementation command**: `spec-kitty implement WP08 --base WP06`

## Objective

Implement the two-tier plugin architecture for cliproxyapi++ executors. Formalize executor and translator interfaces as hexagonal ports. Add translator matrix builder to reduce N×M duplication.

## Context

- executor-core extracted in WP06 provides BifrostAdapter (standard providers) and ThinCustomExecutor base (Kiro/Copilot/Cursor)
- Bifrost handles streaming for standard providers; custom executors (Kiro, Copilot, Cursor) need their own streaming
- Goal: allow third-party executors as WASM plugins without modifying core routing
- Translator matrix (input format × output format) currently duplicated across executors
- **Library research**: `kitty-specs/002-phenotype-modular-arch/research/external-library-leverage.md` — **tmaxmax/go-sse** (2-5K LOC saved) and **coder/websocket** (1-3K LOC saved) for streaming in custom executors and WASM plugins

## Subtasks

### T019: Tier 1 — Executor interface + init() registration

**Steps**:
1. Create `plugin/registry.go`:
   ```go
   var executorRegistry = make(map[string]ExecutorFactory)

   type ExecutorFactory func(config map[string]any) (ExecutorInterface, error)

   func RegisterExecutor(name string, factory ExecutorFactory) {
       executorRegistry[name] = factory
   }
   ```
2. Each built-in executor registers via `init()`:
   ```go
   func init() {
       plugin.RegisterExecutor("claude", func(cfg map[string]any) (ExecutorInterface, error) {
           return NewClaudeExecutor(cfg)
       })
   }
   ```
3. Update routing to discover executors from registry instead of hardcoded switch

**Validation**: All 15 executors register via init() and are discoverable

### T020: Tier 2 — Extism host for executor .wasm

**Steps**:
1. Add `github.com/extism/go-sdk` dependency
2. Create `plugin/wasm_host.go`:
   - Load `.wasm` executor plugins from configurable directory
   - Validate plugin exports match ExecutorContract proto schema
   - Wrap in WasmExecutor implementing ExecutorInterface
3. Sandbox: memory limit (256MB), timeout (60s), no fs access
4. Register WASM executors in same registry as Tier 1
5. Create test .wasm plugin (minimal echo executor)

**Validation**: WASM executor loads, registers, handles requests; failure doesn't crash host

### T021: Translator matrix builder + streaming library integration

**Library guidance**:
- **tmaxmax/go-sse** — use for SSE streaming in custom executors (Kiro, Copilot, Cursor) and WASM plugin streaming bridges. Replaces all custom SSE parsers and reconnect logic. Provides typed event handling, automatic reconnection, and last-event-ID tracking. Estimated LOC savings: 2-5K. Note: provider-specific JSON envelope parsing is still per-provider.
- **coder/websocket** — use for WebSocket streaming where providers use WS instead of SSE (e.g., some Cursor endpoints). Replaces custom WebSocket frame parsers. Estimated LOC savings: 1-3K.
- Bifrost (from WP06) already handles streaming for the 9 standard providers; go-sse and coder/websocket are only needed for the custom executors and WASM plugin host.

**Steps**:
1. Add dependencies:
   - `github.com/tmaxmax/go-sse`
   - `github.com/coder/websocket`
2. Analyze current translator duplication (input format × output format conversions)
3. Create `translator/matrix.go`:
   - Define `Translator` interface: `Translate(from Format, to Format, data []byte) ([]byte, error)`
   - Build translator registry mapping (Format, Format) → Translator
   - Codegen or builder pattern to reduce N×M to N+M implementations
4. Create `translator/streaming.go`:
   - `SSEStreamReader` wrapping go-sse client for SSE-based custom executors
   - `WSStreamReader` wrapping coder/websocket for WebSocket-based custom executors
   - Both implement a common `StreamReader` interface producing `StreamChunk`
   - Wire into ThinCustomExecutor base (from WP06) as the default streaming layer
5. Replace hardcoded format conversions in executors with matrix lookups
6. Wire streaming readers into WASM plugin host (T020) for plugin streaming support

**Validation**: All format conversions work through the matrix; no duplicated conversion code; custom executors use go-sse/websocket for streaming

### T022: Formalize executor + translator as hexagonal ports

**Steps**:
1. Create `ports/` package:
   - `executor_port.go` — ExecutorPort interface (same as ExecutorInterface but named as port)
   - `translator_port.go` — TranslatorPort interface
   - `routing_port.go` — RoutingPort for request dispatch
2. Create `adapters/` for concrete implementations
3. Ensure all external I/O goes through ports (HTTP calls, auth, storage)

**Validation**: Clear separation between domain logic and external dependencies

## Definition of Done

- [ ] Plugin registry with init() registration for all built-in executors
- [ ] Extism host loads and executes .wasm executor plugins
- [ ] Translator matrix reduces format conversion duplication
- [ ] Executor and translator formalized as hexagonal ports
- [ ] All tests pass; no routing behavior changes

## Risks

- **Extism Go SDK maturity**: Test thoroughly; fall back to HashiCorp go-plugin if Extism is unstable.
- **Translator matrix complexity**: Start simple (map-based registry) before adding codegen.

## Reviewer Guidance

- Verify WASM sandbox constraints are enforced
- Check translator matrix covers all existing format pairs
- Ensure hexagonal ports don't over-abstract simple operations
