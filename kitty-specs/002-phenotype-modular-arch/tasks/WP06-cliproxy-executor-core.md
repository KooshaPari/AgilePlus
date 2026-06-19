---
work_package_id: "WP06"
title: "cliproxyapi++: Executor Core Extraction"
lane: "planned"
dependencies: []
subtasks: ["T015", "T016"]
history:
  - date: "2026-03-03"
    event: "created"
    by: "spec-kitty.tasks"
  - date: "2026-03-03"
    event: "revised"
    by: "agent"
    note: "Incorporate external-library-leverage research: Bifrost for standard providers, retry-go, gobreaker, go-retryablehttp"
---

# WP06: cliproxyapi++ — Executor Core Extraction

**Implementation command**: `spec-kitty implement WP06`

## Objective

Define the `ExecutorInterface` port and integrate **maximhq/bifrost** as the primary adapter for all standard LLM providers. Bifrost replaces 9+ custom executors (OpenAI, Anthropic, Gemini, Bedrock, Azure, Cohere, Mistral, Groq, Ollama) with a single unified gateway at 11µs overhead. Keep thin custom executors only for providers Bifrost does not cover (Kiro, Copilot, Cursor). Adopt **avast/retry-go**, **sony/gobreaker**, and **hashicorp/go-retryablehttp** for resilience primitives.

**Estimated LOC savings**: 25-40K (Bifrost) + 1-3K (retry-go) + 0.5-2K (gobreaker).

## Context

- cliproxyapi++ has 15 LLM provider executors (Claude, Codex, Gemini, Kiro, Copilot, etc.)
- Each executor duplicates: retry logic, HTTP request construction, streaming response handling, error wrapping, timeout management
- Code reduction phase already extracted `RefreshWithRetry` and `defaultHttpRequest` — build on that
- See `kitty-specs/002-phenotype-modular-arch/research.md` Decision 5
- **Library research**: `kitty-specs/002-phenotype-modular-arch/research/external-library-leverage.md` — Bifrost identified as top-1 leverage swap (25-40K LOC)

## Subtasks

### T015: Bifrost integration + ExecutorInterface port + resilience primitives

**Steps**:
1. Create `sdk/executor-core/` directory in cliproxyapi++
2. Add Go dependencies:
   - `github.com/maximhq/bifrost` — unified LLM gateway
   - `github.com/avast/retry-go/v4` — generic retry with backoff
   - `github.com/sony/gobreaker` — circuit breaker
   - `github.com/hashicorp/go-retryablehttp` — HTTP retry client
3. Define `ExecutorInterface` as a hexagonal port:
   ```go
   type ExecutorInterface interface {
       Execute(ctx context.Context, req *ExecuteRequest) (*ExecuteResponse, error)
       Stream(ctx context.Context, req *ExecuteRequest) (<-chan StreamChunk, error)
       Name() string
       SupportedModels() []string
       Health() HealthStatus
   }
   ```
4. Implement `BifrostAdapter` wrapping Bifrost as the primary ExecutorInterface implementation:
   - Configure Bifrost with provider credentials for: OpenAI, Anthropic, Gemini, Bedrock, Azure, Cohere, Mistral, Groq, Ollama
   - Map `ExecuteRequest`/`ExecuteResponse` to/from Bifrost's unified request/response types
   - Wire streaming through Bifrost's native streaming support
   - Expose per-provider health via Bifrost's built-in provider health tracking
5. Implement `ThinCustomExecutor` base for providers Bifrost does not cover (Kiro, Copilot, Cursor):
   - Uses `go-retryablehttp` for HTTP transport with automatic retry
   - Uses `retry-go` for non-HTTP retry loops (e.g., token refresh)
   - Uses `gobreaker` for circuit breaking on repeated failures
   - Minimal struct — only provider-specific request/response mapping
6. Create `go.mod` for `sdk/executor-core` (internal module, not yet extracted to separate repo)

**Validation**:
- `go build ./sdk/executor-core/...` passes
- BifrostAdapter can initialize with at least one provider config
- ThinCustomExecutor base compiles with retry-go + gobreaker wired in

### T016: Migrate standard executors to Bifrost; keep Kiro/Copilot/Cursor as custom

**Steps**:
1. **Delete** the 9+ standard provider executor files (Claude, Codex/OpenAI, Gemini, Bedrock, Azure, Cohere, Mistral, Groq, Ollama) — Bifrost replaces them entirely
2. Configure Bifrost provider settings from existing executor configs (API keys, endpoints, model lists)
3. For each deleted executor, verify the BifrostAdapter produces equivalent request/response behavior:
   - Same model name resolution
   - Same streaming chunk format (or mapped to common StreamChunk)
   - Same error codes and retry semantics
4. Implement the 3 custom executors (Kiro, Copilot, Cursor) using `ThinCustomExecutor` base:
   ```go
   type KiroExecutor struct {
       ThinCustomExecutor
       sessionToken string
   }

   func (e *KiroExecutor) Execute(ctx context.Context, req *ExecuteRequest) (*ExecuteResponse, error) {
       return retry.Do(func() (*ExecuteResponse, error) {
           httpReq := e.BuildRequest("POST", kiroEndpoint, req.ToKiroFormat())
           return e.DoRequest(ctx, httpReq)
       }, retry.Attempts(3), retry.Delay(500*time.Millisecond))
   }
   ```
5. Update executor factory/registry:
   - Standard providers → single BifrostAdapter registration (one adapter, multiple provider names)
   - Kiro, Copilot, Cursor → individual ThinCustomExecutor registrations
6. Run full test suite after migration (incremental per-provider, not big-bang)

**Validation**:
- All provider routing works through BifrostAdapter or ThinCustomExecutor
- No duplicated retry/HTTP/streaming code remains
- Custom executor files (Kiro, Copilot, Cursor) are each <200 LOC
- Existing executor tests pass (adapted to new adapter surface)

## Definition of Done

- [ ] `sdk/executor-core` package with ExecutorInterface port, BifrostAdapter, ThinCustomExecutor base
- [ ] Bifrost handles 9+ standard providers (OpenAI, Anthropic, Gemini, Bedrock, Azure, Cohere, Mistral, Groq, Ollama)
- [ ] Only Kiro, Copilot, Cursor remain as thin custom executors
- [ ] `avast/retry-go`, `sony/gobreaker`, `hashicorp/go-retryablehttp` wired into custom executor base
- [ ] No duplicated retry/HTTP/streaming patterns across executors
- [ ] All existing executor tests pass (adapted)
- [ ] `go build ./...` and `go test ./...` pass

## Risks

- **Bifrost provider coverage gaps**: If Bifrost drops support for a provider or has bugs, we may need to temporarily add a custom executor. Monitor Bifrost releases.
- **Bifrost streaming fidelity**: Verify that Bifrost's streaming output matches the exact chunk format executors previously emitted. May need a thin mapping layer.
- **Kiro/Copilot/Cursor edge cases**: These providers have non-standard APIs; the ThinCustomExecutor base must be flexible enough for their auth and streaming patterns.
- **go-retryablehttp license**: MPL-2.0 — confirm license compatibility before adoption.

## Reviewer Guidance

- Verify Bifrost configuration covers all previously-supported models per provider
- Check that BifrostAdapter streaming produces the same StreamChunk shape as old executors
- Confirm custom executors (Kiro, Copilot, Cursor) use retry-go and gobreaker correctly
- Ensure no executor test was deleted or weakened during migration — adapted tests must cover the same behavior
