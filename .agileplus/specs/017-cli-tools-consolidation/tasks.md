# Work Packages: CLI Tools Consolidation — Consolidate Across 7 Repositories

**Inputs**: Design documents from `kitty-specs/017-cli-tools-consolidation/`
**Prerequisites**: spec.md, Rust toolchain, TypeScript/Node, Go toolchain
**Scope**: Cross-repo (7 repositories): cliproxyapi-plusplus, agentapi-plusplus, Cmdra, forgecode, thegent-sharecli, thegent-cli-share, thegent-subprocess

---

## WP-001: cliproxyapi-plusplus — Complete LLM Proxy with 8+ Providers

- **State:** planned
- **Sequence:** 1
- **File Scope:** cliproxyapi-plusplus repository (src/, tests/, docs/)
- **Acceptance Criteria:**
  - LLM proxy supporting 8+ providers (OpenAI, Anthropic, Google, Mistral, Cohere, Groq, Ollama, local)
  - Request routing, rate limiting, and retry logic per provider
  - Response streaming support for all providers
  - Provider-agnostic API with consistent error handling
  - Integration hooks for agentapi-plusplus
  - ≥80% test coverage on proxy core
  - All quality checks passing
  - 19 concrete subtasks (T001–T019) each with explicit outputs and acceptance criteria
- **Estimated Effort:** L

Complete cliproxyapi-plusplus as the LLM proxy with support for 8+ providers. This serves as the unified interface for all LLM API calls, handling provider-specific quirks, rate limiting, retries, and streaming. It integrates with agentapi-plusplus for agent-facing API access.

### Subtasks

**T001 — Audit existing cliproxyapi-plusplus codebase**
- [ ] T001a Enumerate all currently implemented providers (by name, version, API variant)
- [ ] T001b Identify missing provider implementations (OpenAI, Anthropic, Google, Mistral, Cohere, Groq, Ollama, local)
- [ ] T001c Audit request/response types for gaps against the unified abstraction below
- [ ] T001d Audit routing layer: current model-to-provider selection logic, fallback chain
- [ ] T001e Audit existing tests: coverage %, missing provider adapter tests, integration test gaps
- [ ] T001f Document audit findings in `cliproxyapi-plusplus/docs/audit-2026-05.md`
- **Output:** Audit report at `cliproxyapi-plusplus/docs/audit-2026-05.md`

**T002 — Design provider abstraction layer**
- [ ] T002a Define `Provider` trait with `chat()`, `embeddings()`, `stream()` async methods
- [ ] T002b Define unified `ProxyRequest` struct (model, messages, temperature, max_tokens, stream)
- [ ] T002c Define unified `ProxyResponse` enum (text, usage, streaming chunks, error)
- [ ] T002d Define per-provider `ProviderConfig` struct (base URL, auth header, rate limit, timeout)
- [ ] T002e Define `ProviderError` enum: NetworkError, AuthError, RateLimitError, ModelNotFoundError, UnknownProviderError
- [ ] T002f Define `RoutingConfig`: model prefix map, fallback chain per model, default provider
- **Output:** New `src/providers/` module with trait + types committed to cliproxyapi-plusplus

**T003 — Implement OpenAI provider adapter**
- [ ] T003a Implement `OpenAIProvider::chat()` → `POST /chat/completions`, non-streaming path
- [ ] T003b Implement `OpenAIProvider::embeddings()` → `POST /embeddings`
- [ ] T003c Implement `OpenAIProvider::stream()` → `POST /chat/completions` with SSE, yields `ProxyResponse::StreamChunk`
- [ ] T003d Map OpenAI error response codes to `ProviderError` variants
- [ ] T003e Add OpenAI to `PROVIDER_REGISTRY` with config validation on startup
- **Acceptance Criteria:** `curl`-equivalent test via mock server passes for all three methods; no `unimplemented!()` left; `cargo clippy -- -D warnings` clean

**T004 — Implement Anthropic provider adapter**
- [ ] T004a Implement `AnthropicProvider::chat()` → `POST /v1/messages`, non-streaming path
- [ ] T004b Implement `AnthropicProvider::stream()` → streaming via `Accept: event-stream` header
- [ ] T004c Map Anthropic error codes to `ProviderError` variants
- [ ] T004d Add Anthropic to `PROVIDER_REGISTRY` with `anthropic-version` header injection
- **Acceptance Criteria:** Both non-streaming and streaming paths pass against mock Anthropic server; `cargo clippy -- -D warnings` clean

**T005 — Implement Google provider adapter**
- [ ] T005a Implement `GoogleProvider::chat()` → `POST /v1beta/models/{model}:generateContent`
- [ ] T005b Implement `GoogleProvider::stream()` → streaming via `alt=sse` parameter
- [ ] T005c Map Google error codes to `ProviderError` variants
- [ ] T005d Add Google to `PROVIDER_REGISTRY`
- **Acceptance Criteria:** chat + stream pass against mock server; `cargo clippy -- -D warnings` clean

**T006 — Implement Mistral provider adapter**
- [ ] T006a Implement `MistralProvider::chat()` → `POST /v1/chat/completions`
- [ ] T006b Implement `MistralProvider::stream()` → SSE streaming
- [ ] T006c Map Mistral error codes to `ProviderError` variants
- [ ] T006d Add Mistral to `PROVIDER_REGISTRY`
- **Acceptance Criteria:** chat + stream pass against mock server; `cargo clippy -- -D warnings` clean

**T007 — Implement Cohere provider adapter**
- [ ] T007a Implement `CohereProvider::chat()` → `POST /v1/chat`
- [ ] T007b Implement `CohereProvider::stream()` → SSE streaming via `POST /v1/chat/stream`
- [ ] T007c Map Cohere error codes to `ProviderError` variants
- [ ] T007d Add Cohere to `PROVIDER_REGISTRY`
- **Acceptance Criteria:** chat + stream pass against mock server; `cargo clippy -- -D warnings` clean

**T008 — Implement Groq provider adapter**
- [ ] T008a Implement `GroqProvider::chat()` → `POST /v1/chat/completions` (Groq API-compatible with OpenAI)
- [ ] T008b Implement `GroqProvider::stream()` → SSE streaming
- [ ] T008c Map Groq error codes to `ProviderError` variants
- [ ] T008d Add Groq to `PROVIDER_REGISTRY`
- **Acceptance Criteria:** chat + stream pass against mock server; `cargo clippy -- -D warnings` clean

**T009 — Implement Ollama provider adapter**
- [ ] T009a Implement `OllamaProvider::chat()` → `POST /api/chat`
- [ ] T009b Implement `OllamaProvider::stream()` → SSE via `POST /api/chat` with `stream: true`
- [ ] T009c Map Ollama error codes to `ProviderError` variants
- [ ] T009d Add Ollama to `PROVIDER_REGISTRY`
- **Acceptance Criteria:** chat + stream pass against local Ollama mock server; `cargo clippy -- -D warnings` clean

**T010 — Implement local/openai-compatible provider adapter**
- [ ] T010a Implement `LocalProvider::chat()` → `POST /v1/chat/completions` against any OpenAI-compatible endpoint
- [ ] T010b Implement `LocalProvider::stream()` → SSE streaming
- [ ] T010c Add configurable base URL (no auth by default for local setups)
- [ ] T010d Add LocalProvider to `PROVIDER_REGISTRY`
- **Acceptance Criteria:** chat + stream pass against local test server (llama.cpp / ollama); `cargo clippy -- -D warnings` clean

**T011 — Implement request routing: model-to-provider selection and fallback chain**
- [ ] T011a Parse incoming `model` field (e.g., `gpt-4o`, `claude-3-5-sonnet`, `gemini-1.5-pro`)
- [ ] T011b Implement prefix-based routing table: `gpt-*` → OpenAI, `claude-*` → Anthropic, `gemini-*` → Google, `mistral-*` → Mistral, `cohere-*` → Cohere, `groq/*` → Groq, `ollama/*` → Ollama, `local/*` → Local
- [ ] T011c Implement fallback chain: configurable ordered list of providers per model (e.g., `[OpenAI, Groq]` for `gpt-4o`)
- [ ] T011d Return `ModelNotFoundError` only after all fallbacks exhausted
- [ ] T011e Expose routing diagnostics via `GET /admin/routes` (model → provider mapping + health)
- **Acceptance Criteria:** Unit test covers all 8 provider routes + all 8 fallback chains; routing latency < 1 ms p99

**T012 — Implement rate limiting: per-provider limits, token bucket algorithm**
- [ ] T012a Implement per-provider token bucket: `requests_per_minute` + `tokens_per_minute` limits
- [ ] T012b Load rate limit config from `providers.toml` (no hardcoded values)
- [ ] T012c Return HTTP 429 with `Retry-After` header when bucket exhausted
- [ ] T012d Emit metrics: `proxy_rate_limit_hits_total{provider}`, `proxy_rate_limit_remaining{provider}`
- [ ] T012e Add integration test: exhaust bucket, verify 429 returned, verify recovery after window
- **Acceptance Criteria:** Rate limiting engages correctly for all 8 providers; `cargo clippy -- -D warnings` clean

**T013 — Implement retry logic: exponential backoff + circuit breaker**
- [ ] T013a Implement exponential backoff: base_delay=500ms, max_delay=30s, jitter=±20%, max_retries=3
- [ ] T013b Implement circuit breaker: open after 5 consecutive errors per provider, half-open after 60s
- [ ] T013c Detect retryable status codes (429, 500, 502, 503, 504) and non-retryable (401, 403, 404)
- [ ] T013d Implement per-request timeout (default: 60s, configurable per provider)
- [ ] T013e Add metrics: `proxy_retries_total{provider,result}`, `proxy_circuit_breaker_state{provider}`
- [ ] T013f Add integration test: simulate 429/503, verify retry + fallback chain fires correctly
- **Acceptance Criteria:** Retries fire on 429/5xx; circuit opens after 5 consecutive failures; `cargo clippy -- -D warnings` clean

**T014 — Implement response streaming: unified SSE format across all providers**
- [ ] T014a Implement `stream_to_sse()` adapter: normalize all provider streaming to SSE `data: {...}` format
- [ ] T014b Implement `SseChunk` enum: `Delta(String)`, `Usage { prompt_tokens, completion_tokens }`, `Done`, `Error { code, message }`
- [ ] T014c Wire `stream_to_sse()` into all 8 provider `stream()` implementations
- [ ] T014d Add streaming integration test: send requests to all 8 providers, verify SSE format + JSON parsing
- [ ] T014e Document SSE format in `docs/streaming-protocol.md`
- **Acceptance Criteria:** SSE output format is identical across all 8 providers; no provider-specific quirks leak into client; `curl` demo works for all 8 providers

**T015 — Define integration API for agentapi-plusplus**
- [ ] T015a Define `ProxyClient` struct with typed `chat()`, `embeddings()`, `stream()` methods callable from agentapi
- [ ] T015b Define `ProxyClientConfig`: base URL of cliproxyapi, API key, timeout, fallback chain
- [ ] T015c Define error types: map proxy errors to agentapi-compatible `AgentProxyError`
- [ ] T015d Publish client library: `cliproxyapi-plusplus/libs/client/` as npm package or TypeScript SDK
- [ ] T015e Add integration test: agentapi calls cliproxyapi client → routed to correct provider → response returned
- **Acceptance Criteria:** agentapi-plusplus can call cliproxyapi via typed client with no raw HTTP in agentapi handlers; `cargo clippy -- -D warnings` clean

**T016 — Write unit tests for all provider adapters (target: ≥80% coverage on proxy core)**
- [ ] T016a Mock HTTP responses per provider using `wiremock` or equivalent; implement mock for all 8 providers
- [ ] T016b Write table-driven tests per adapter: valid request → correct serialized body + headers
- [ ] T016c Write error path tests per adapter: 401/403/429/5xx mapped to correct `ProviderError` variants
- [ ] T016d Write coverage report: `cargo tarpaulin` output must show ≥80% on `src/providers/`
- **Acceptance Criteria:** Coverage ≥80% on `src/providers/` and `src/routing/`; all 8 adapters tested; no `todo!()` left

**T017 — Write integration tests: end-to-end proxy flow with mock providers**
- [ ] T017a Set up mock HTTP servers per provider using `wiremock` with realistic response fixtures
- [ ] T017b Test T001 → T015 flow end-to-end: `ProxyRequest` → router → correct provider → normalized response
- [ ] T017c Test fallback chain: primary provider returns 429 → fallback provider handles
- [ ] T017d Test circuit breaker: primary provider returns 5xx × 5 → circuit opens → fallback used
- [ ] T017e Test streaming: all 8 providers produce parseable SSE; delta + done chunks present
- **Acceptance Criteria:** All integration tests green; mock servers reset between tests; no shared mutable state

**T018 — Add documentation: provider config, routing, rate limiting, streaming**
- [ ] T018a Write `docs/providers.md`: all 8 providers, required env vars/API keys, model names, rate limits
- [ ] T018b Write `docs/routing.md`: model prefix routing table, fallback chain config, diagnostic endpoint usage
- [ ] T018c Write `docs/streaming.md`: SSE format reference, client-side parsing example, error framing
- [ ] T018d Write `docs/agent-integration.md`: ProxyClient API, TypeScript SDK usage, error handling
- [ ] T018e Verify all docs build under `cargo doc --no-deps` with no warnings
- **Acceptance Criteria:** All 4 docs present and build-clean; `cargo doc` produces no warnings

**T019 — Run quality gate: tests, linter, formatter, type checker**
- [ ] T019a `cargo test --all-features` passes (no failures, no ignored failures)
- [ ] T019b `cargo clippy -- -D warnings` returns zero warnings
- [ ] T019c `cargo fmt -- --check` returns zero diffs
- [ ] T019d `cargo audit` returns zero vulnerabilities
- [ ] T019e Coverage ≥80% on `src/providers/` + `src/routing/` confirmed by `cargo tarpaulin`
- **Acceptance Criteria:** All 5 quality checks pass; PR cannot be opened with failures

### Dependencies
- None (can start independently)

### Risks & Mitigations
- Provider API changes: Abstract provider interfaces, test against provider SDKs
- Rate limit complexity: Use well-tested rate limiting library, document limits per provider

---

## WP-002: agentapi-plusplus — Complete HTTP API for CLI Agents

- **State:** planned
- **Sequence:** 2
- **File Scope:** agentapi-plusplus repository (src/, tests/, docs/)
- **Acceptance Criteria:**
  - Complete HTTP API for CLI agent interactions (dispatch, status, results)
  - No functionality duplication with cliproxyapi-plusplus (clear boundary: proxy vs. agent API)
  - Authentication and authorization for API endpoints
  - WebSocket support for real-time agent status updates
  - Integration with cliproxyapi-plusplus for LLM calls
  - ≥80% test coverage on API handlers
  - All quality checks passing
- **Estimated Effort:** M

Complete agentapi-plusplus as the HTTP API for CLI agent interactions. This API handles agent dispatch, status queries, and result retrieval — distinct from cliproxyapi-plusplus which handles LLM proxying. The two components integrate but have clear boundaries.

### Subtasks
- [ ] T016 Audit current agentapi-plusplus: existing endpoints, gaps, overlap with cliproxyapi
- [ ] T017 Define API boundary: what belongs in agent API vs. LLM proxy
- [ ] T018 Implement agent dispatch endpoint: create agent task, assign to cliproxyapi
- [ ] T019 Implement agent status endpoint: query running/complete/failed agents
- [ ] T020 Implement agent results endpoint: retrieve agent output, artifacts
- [ ] T021 Implement authentication: API key validation, role-based access
- [ ] T022 Implement WebSocket endpoint: real-time agent status streaming
- [ ] T023 Integrate with cliproxyapi-plusplus: route LLM calls through proxy
- [ ] T024 Write unit tests for API handlers (target: ≥80% coverage)
- [ ] T025 Write integration tests: full agent lifecycle via HTTP API
- [ ] T026 Add API documentation: OpenAPI spec, endpoint examples
- [ ] T027 Run quality checks: `cargo test` / `npm test`, linter, formatter

### Dependencies
- WP-001 (cliproxyapi-plusplus integration API defined)

### Risks & Mitigations
- API boundary confusion: Clear documentation, separate codebases, integration tests verify boundaries
- WebSocket scalability: Document connection limits, implement connection pooling

---

## WP-003: Cmdra — Universal CLI Framework Completion

- **State:** planned
- **Sequence:** 3
- **File Scope:** Cmdra repository (src/, tests/, docs/)
- **Acceptance Criteria:**
  - Complete CLI framework with command registration, argument parsing, and help generation
  - Plugin system for CLI extensions
  - Consistent command patterns: subcommands, flags, positional args
  - Adoption by all other CLI tools in this spec (cliproxyapi, agentapi, forgecode)
  - ≥80% test coverage on framework core
  - All quality checks passing
- **Estimated Effort:** L

Complete Cmdra as the universal CLI framework adopted by all CLI tools in this consolidation. Cmdra provides command registration, argument parsing, help generation, and a plugin system for extensions. All other CLI tools migrate to use Cmdra as their framework.

### Subtasks
- [ ] T028 Audit current Cmdra: existing framework code, gaps, plugin system status
- [ ] T029 Complete command registration: hierarchical commands, subcommands, aliases
- [ ] T030 Complete argument parsing: flags, positional args, validation, defaults
- [ ] T031 Implement help generation: auto-generated from command metadata, formatted output
- [ ] T032 Implement plugin system: discover, load, and register CLI plugins
- [ ] T033 Implement consistent command patterns: before/after hooks, error handling
- [ ] T034 Implement configuration loading: per-command config, global config, env vars
- [ ] T035 Write unit tests for framework core (target: ≥80% coverage)
- [ ] T036 Write integration tests: register commands, parse args, execute with plugins
- [ ] T037 Add comprehensive rustdoc with CLI framework usage guide
- [ ] T038 Run quality checks: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`

### Dependencies
- None (can start in parallel with WP-001)

### Risks & Mitigations
- Framework adoption resistance: Provide migration guides, backward-compatible adapters
- Plugin system complexity: Start simple (file-based discovery), add advanced features incrementally

---

## WP-004: forgecode — Git Workflow Framework Completion

- **State:** planned
- **Sequence:** 4
- **File Scope:** forgecode repository (src/, tests/, docs/)
- **Acceptance Criteria:**
  - Complete git workflow framework: branch management, PR creation, review loops
  - Integrated with Cmdra CLI framework (forgecode commands use Cmdra)
  - Conventional commit enforcement
  - Worktree management for parallel development
  - ≥80% test coverage on workflow core
  - All quality checks passing
- **Estimated Effort:** M

Complete forgecode as the git workflow framework, integrated with Cmdra. forgecode handles branch management, PR creation, review loops, and worktree operations — all accessible through Cmdra-based CLI commands.

### Subtasks
- [ ] T039 Audit current forgecode: existing workflows, gaps, Cmdra integration status
- [ ] T040 Complete branch workflow: create branch from spec, checkout, merge to target
- [ ] T041 Complete PR workflow: create PR with structured description, set reviewers
- [ ] T042 Implement review loop: poll for reviews, feed comments back to agent, re-push
- [ ] T043 Implement conventional commit enforcement: validate commit messages, auto-fix
- [ ] T044 Implement worktree management: create, list, cleanup worktrees
- [ ] T045 Integrate with Cmdra: register forgecode commands as Cmdra plugins
- [ ] T046 Write unit tests for git workflows (target: ≥80% coverage)
- [ ] T047 Write integration tests with real git repositories (temp repos)
- [ ] T048 Add documentation: workflow configuration, Cmdra integration guide
- [ ] T049 Run quality checks: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`

### Dependencies
- WP-003 (Cmdra framework available for integration)

### Risks & Mitigations
- Git operation complexity: Use git2 crate, test on macOS and Linux
- Review loop reliability: Implement timeout, max retry count, graceful failure

---

## WP-005: Deduplicate thegent-sharecli vs thegent-cli-share — Merge or Delete One

- **State:** planned
- **Sequence:** 5
- **File Scope:** thegent-sharecli repository, thegent-cli-share repository
- **Acceptance Criteria:**
  - Decision documented: which repo is kept, which is archived, rationale
  - All functionality from both repos preserved in the kept implementation
  - Kept implementation integrated with Cmdra CLI framework
  - Archived repo README documents migration path
  - No broken references in active tooling
  - All quality checks passing on merged implementation
- **Estimated Effort:** S

Resolve the duplication between thegent-sharecli and thegent-cli-share by auditing both, deciding which to keep, merging functionality, and archiving the other. The kept implementation is migrated to use Cmdra as its CLI framework.

### Subtasks
- [ ] T050 Audit thegent-sharecli: features, code quality, dependencies, test coverage
- [ ] T051 Audit thegent-cli-share: features, code quality, dependencies, test coverage
- [ ] T052 Compare feature overlap: identify unique features in each, common functionality
- [ ] T053 Make keep/archive decision with documented rationale
- [ ] T054 Merge unique features from archived repo into kept repo
- [ ] T055 Migrate kept repo to use Cmdra CLI framework
- [ ] T056 Archive the superseded repo with migration README
- [ ] T057 Search all active repos for references to archived repo
- [ ] T058 Update references to use the kept implementation
- [ ] T059 Write tests for merged functionality (target: ≥80% coverage)
- [ ] T060 Run quality checks on merged implementation

### Dependencies
- WP-003 (Cmdra framework available for migration)

### Risks & Mitigations
- Functionality loss during merge: Comprehensive feature audit (T050-T052) before decision
- Broken references: Systematic search across all repos before archival

---

## WP-006: thegent-subprocess — Subprocess Management for thegent

- **State:** planned
- **Sequence:** 6
- **File Scope:** thegent-subprocess repository (src/, tests/, docs/), thegent main repository
- **Acceptance Criteria:**
  - Complete subprocess management library: spawn, monitor, communicate, cleanup
  - Integrated with Cmdra CLI framework (subprocess commands via Cmdra)
  - Integrated with thegent main application
  - Stream handling: stdin, stdout, stderr with buffering
  - Timeout and signal handling
  - ≥80% test coverage on subprocess core
  - All quality checks passing
- **Estimated Effort:** M

Complete thegent-subprocess as the subprocess management library for thegent. This handles spawning, monitoring, and communicating with subprocesses — essential for agent execution, tool invocation, and CLI command orchestration.

### Subtasks
- [ ] T061 Audit current thegent-subprocess: existing code, gaps, integration status
- [ ] T062 Implement subprocess spawning: configurable command, environment, working directory
- [ ] T063 Implement stream handling: stdin write, stdout/stderr read, buffering
- [ ] T064 Implement subprocess monitoring: PID tracking, exit code capture, status polling
- [ ] T065 Implement timeout handling: configurable timeouts, graceful termination
- [ ] T066 Implement signal handling: SIGTERM, SIGKILL, signal forwarding
- [ ] T067 Implement subprocess cleanup: kill on parent exit, resource cleanup
- [ ] T068 Integrate with Cmdra: register subprocess commands as Cmdra plugins
- [ ] T069 Integrate with thegent: subprocess management available to thegent core
- [ ] T070 Write unit tests for subprocess operations (target: ≥80% coverage)
- [ ] T071 Write integration tests: spawn real subprocesses, verify communication
- [ ] T072 Add documentation: subprocess configuration, stream handling guide
- [ ] T073 Run quality checks across all components

### Dependencies
- WP-003 (Cmdra framework available for integration)

### Risks & Mitigations
- Subprocess reliability: Test on macOS and Linux, handle platform-specific behaviors
- Resource leaks: Implement strict cleanup, test parent exit scenarios

---

## Dependency & Execution Summary

```
WP-001 (cliproxyapi-plusplus LLM proxy) ───── first, no deps
WP-002 (agentapi-plusplus HTTP API) ────────── depends on WP-001
WP-003 (Cmdra CLI framework) ──────────────── first, no deps (parallel with WP-001)
WP-004 (forgecode git workflows) ───────────── depends on WP-003
WP-005 (Deduplicate sharecli) ──────────────── depends on WP-003
WP-006 (thegent-subprocess) ────────────────── depends on WP-003
```

**Parallelization**: WP-001 and WP-003 can run in parallel (independent codebases). WP-004, WP-005, and WP-006 can run in parallel after WP-003. WP-002 depends on WP-001.

**MVP Scope**: WP-001 alone provides LLM proxy. WP-003 provides CLI framework. WP-001 + WP-003 + WP-002 provides the core API stack.
