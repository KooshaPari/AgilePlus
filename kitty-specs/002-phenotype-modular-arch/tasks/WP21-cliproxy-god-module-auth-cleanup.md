---
work_package_id: "WP21"
title: "cliproxyapi++: God Module Split + Auth Consolidation"
lane: "planned"
dependencies: ["WP06"]
subtasks: ["T063", "T064", "T065"]
history:
  - date: "2026-03-03"
    event: "created"
    by: "spec-kitty.tasks"
---

# WP21: cliproxyapi++ — God Module Split + Auth Consolidation

**Implementation command**: `spec-kitty implement WP21`

## Objective

Break apart the largest god modules in cliproxyapi++ and consolidate the duplicated auth directory. kiro_executor.go at 4,691 LOC is 3x the next largest executor, auth sprawls 21K LOC across 14 providers, and multiple functions exceed 200 lines. This WP reduces file sizes, eliminates dead code, and enforces complexity ceilings.

## Context

- cliproxyapi++ has 15 executor files totaling 15,403 LOC with kiro_executor.go as the worst offender
- 21,441 LOC of auth code across 14 provider subdirectories, 6 distinct auth mechanisms
- Functions exceeding 200 lines: `executeClaudeNonStream` (205L), `ExecuteStream` (198L), `Execute` (149L)
- `internal/auth/` (~600L) appears dead — superseded by `pkg/llmproxy/auth/`
- `config/config.go` (2,266L) and `sdk/cliproxy/service.go` (1,777L) are additional god modules
- Depends on WP06 (executor core extraction) — splits happen after interface is defined
- See `kitty-specs/002-phenotype-modular-arch/research/codebase-quality-analysis.md` for metrics

## Subtasks

### T063: Split kiro_executor.go (4,691 LOC)

**Steps**:
1. Analyze kiro_executor.go to identify natural seam boundaries: execution flow, session management, protocol handling, auth flows
2. Split into four files:
   - `kiro_executor.go` — main `Execute`/`Stream` entry points (<800 LOC)
   - `kiro_session.go` — session creation, renewal, lifecycle management
   - `kiro_protocol.go` — Kiro-specific protocol marshaling and response handling
   - `kiro_auth.go` — AWS STS/SSO/OIDC flows (absorbs code from `auth/kiro/sso_oidc.go` (1,489L), `auth/kiro/oauth_web.go` (912L), and `auth/kiro/aws.go` (597L) after WP07 auth consolidation)
3. Split antigravity_executor.go (1,783L) into:
   - `antigravity_executor.go` — main executor logic
   - `antigravity_auth.go` — provider-specific auth
   - `antigravity_format.go` — format conversion between provider wire formats
4. Ensure all split files are in the same package — no import changes required for callers
5. Run full test suite after each file extraction

**Validation**:
- No executor file exceeds 1,000 LOC
- `go build ./...` passes
- `go test ./...` passes with no regressions

### T064: Delete dead internal/auth/ directory (~600 LOC)

**Steps**:
1. Inventory `internal/auth/` — contains claude, copilot, gemini subdirectories
2. Confirm these duplicate `pkg/llmproxy/auth/` by comparing function signatures and behavior
3. Run `grep -r 'internal/auth' --include='*.go'` across the entire repo to confirm zero imports
4. If zero imports: delete `internal/auth/` directory entirely
5. If any imports found: update them to point to `pkg/llmproxy/auth/` equivalents, then delete

**Validation**:
- `grep -r 'internal/auth' --include='*.go'` returns 0 results
- `go build ./...` passes
- No test regressions

### T065: Split remaining god functions and files

**Steps**:
1. Split `antigravity_executor.go::executeClaudeNonStream` (205 lines):
   - Extract format conversion into `antigravity_format.go::convertClaudeResponse`
   - Extract retry logic into a helper using retry-go (from WP06)
2. Split `antigravity_executor.go::ExecuteStream` (198 lines):
   - Extract SSE event parsing into `antigravity_stream.go::parseSSEEvents`
   - Extract stream chunk assembly into `antigravity_stream.go::assembleStreamChunks`
3. Split `claude_executor.go::ExecuteStream` (148 lines):
   - Extract streaming response parser into `claude_stream.go::parseStreamResponse`
4. Split `config/config.go` (2,266L) into:
   - `config.go` — core config loading, merging, and access (<800L)
   - `config_models.go` — model/provider type definitions and defaults
   - `config_validation.go` — validation rules, constraint checking
5. Split `sdk/cliproxy/service.go` (1,777L) into:
   - `service.go` — server setup, lifecycle, dependency injection (<600L)
   - `service_handlers.go` — HTTP handler registrations and route definitions
   - `service_middleware.go` — middleware chain (auth, logging, rate limiting)
6. Run tests after each split

**Validation**:
- No function exceeds 100 lines
- No file exceeds 1,500 LOC
- `go build ./...` passes
- `go test ./...` passes with no regressions

## Definition of Done

- [ ] kiro_executor.go split into 4 files, each <1,000 LOC
- [ ] antigravity_executor.go split into executor + auth + format files
- [ ] `internal/auth/` deleted with zero import breakage
- [ ] `config/config.go` split into 3 files
- [ ] `sdk/cliproxy/service.go` split into 3 files
- [ ] No function >100 lines, no file >1,500 LOC
- [ ] `go build ./...` and `go test ./...` pass

## Risks

- **Merge conflicts with WP06/WP07**: This WP touches many of the same files as executor extraction and auth consolidation. Coordinate sequencing — WP06 lands first, then WP07, then WP21.
- **Kiro auth absorption timing**: T063 absorbs kiro auth files that WP07 may restructure. If WP07 changes auth directory layout, T063 must adapt to the new locations.
- **Hidden coupling in god functions**: Functions >200 lines often have non-obvious shared state. Extract carefully and verify with integration tests, not just unit tests.

## Reviewer Guidance

- Verify that each split file has a clear single responsibility — no "misc" or "utils" catch-all files
- Check that no split introduced new public API surface — all splits should be package-internal reorganization
- Confirm `internal/auth/` deletion is safe by reviewing the grep results for zero references
- Ensure config split preserves the exact same config loading behavior — no field reordering or default changes
