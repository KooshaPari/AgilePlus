---
work_package_id: WP07
title: 'cliproxyapi++: Shared Package Extraction (authkit + httpkit)'
lane: planned
dependencies: []
subtasks: [T017, T018]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
- date: '2026-03-03'
  event: revised
  by: agent
  note: 'Incorporate external-library-leverage research: x/oauth2, markbates/goth for auth consolidation'
---

# WP07: cliproxyapi++ — Shared Package Extraction

**Implementation command**: `spec-kitty implement WP07 --base WP02`

## Objective

Extract auth logic to `phenotype-go-authkit` and HTTP helpers to `phenotype-go-httpkit` shared repos. cliproxyapi++ and agentapi++ will consume these as Go module dependencies.

## Context

- Shared repos scaffolded in WP02
- cliproxyapi++ `internal/auth/` has ~4K LOC of token storage, refresh, provider auth — used by 20+ importers within cliproxy++
- HTTP helpers (`defaultHttpRequest`, proxy helpers) identified during code reduction phase
- agentapi++ also needs auth and HTTP patterns (WP14 will consume these)
- **Library research**: `kitty-specs/002-phenotype-modular-arch/research/external-library-leverage.md` — cliproxyapi++ has 3 parallel auth systems that should collapse to **golang.org/x/oauth2** + **markbates/goth** (6-14K LOC saved)

## Subtasks

### T017: Extract auth logic to phenotype-go-authkit (backed by x/oauth2 + Goth)

**Library guidance**:
- **golang.org/x/oauth2** — use as the foundation for all token management. The 3 parallel auth systems in cliproxyapi++ (OAuth2 flows, API key rotation, session-based) should collapse to a single `oauth2.TokenSource` registry. Each provider gets a `TokenSource` implementation; `RefreshWithRetry` becomes a thin wrapper around `oauth2.ReuseTokenSource`. Estimated LOC savings: 3-8K.
- **markbates/goth** — use for web OAuth callback handlers (75+ providers supported out of the box). This replaces custom OAuth callback routes and provider-specific redirect logic. Estimated LOC savings: 3-6K. Note: Goth covers browser-based flows only; machine-to-machine flows still need custom `oauth2.TokenSource` implementations.

**Steps**:
1. Identify all auth-related types in cliproxyapi++ `internal/auth/`:
   - `TokenStore`, `BaseTokenStorage`, `RefreshWithRetry`
   - Provider-specific auth (OAuth2, API key, session-based)
2. Add dependencies to `phenotype-go-authkit`:
   - `golang.org/x/oauth2`
   - `github.com/markbates/goth`
3. Implement `phenotype-go-authkit/` with x/oauth2 as core:
   - `tokenregistry.go` — `TokenSourceRegistry` mapping provider name to `oauth2.TokenSource`; replaces `TokenStore` + `BaseTokenStorage`
   - `refresh.go` — thin wrapper around `oauth2.ReuseTokenSource` with logging; replaces `RefreshWithRetry`
   - `oauth2_web.go` — Goth-backed web OAuth callback handlers (provider init, callback route, session extraction)
   - `apikey.go` — API key management (simple `TokenSource` that returns static keys)
   - `m2m.go` — machine-to-machine `TokenSource` implementations for providers without browser flows
4. Ensure no cliproxyapi++-internal imports in extracted code
5. In cliproxyapi++, replace `internal/auth` imports with `phenotype-go-authkit`:
   ```go
   import authkit "github.com/KooshaPari/phenotype-go-authkit"
   ```
6. Collapse the 3 parallel auth systems to the single `TokenSourceRegistry`
7. Update `go.mod` with new dependency
8. Run `go mod tidy` and verify all imports resolve

**Validation**: `go build ./...` and `go test ./...` pass in both repos; all 3 former auth systems exercised through the unified registry

### T018: Extract HTTP helpers to phenotype-go-httpkit

**Steps**:
1. Identify HTTP helper code:
   - `defaultHttpRequest` function (extracted during code reduction)
   - Proxy request construction helpers
   - Response parsing utilities
   - Cache helpers
2. Copy to `phenotype-go-httpkit/`:
   - `request.go` — defaultHttpRequest, request builders
   - `response.go` — response parsing, error extraction
   - `cache.go` — cache helpers
3. In cliproxyapi++, replace local helpers with httpkit imports
4. Update `go.mod`

**Validation**: `go build ./...` and `go test ./...` pass in both repos

## Definition of Done

- [ ] phenotype-go-authkit contains all shared auth types
- [ ] phenotype-go-httpkit contains all shared HTTP helpers
- [ ] cliproxyapi++ consumes both as Go module dependencies
- [ ] No auth/HTTP duplicated code remains in cliproxyapi++ internals
- [ ] All tests pass in all 3 repos

## Risks

- **internal/ visibility**: Go `internal/` packages can't be imported externally. The extraction must move types OUT of internal/ into the shared repo.
- **Transitive deps**: Shared packages should minimize their own dependencies.

## Reviewer Guidance

- Verify no cliproxyapi++-specific logic leaked into shared packages
- Check that shared package APIs are clean and documented
- Ensure go.sum files are committed
