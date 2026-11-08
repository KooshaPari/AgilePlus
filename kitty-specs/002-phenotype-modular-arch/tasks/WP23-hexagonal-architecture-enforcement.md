---
work_package_id: WP23
title: 'Hexagonal Architecture Enforcement (Cross-Repo)'
lane: planned
dependencies: [WP06, WP09]
subtasks: [T069, T070, T071]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
---

# WP23: Hexagonal Architecture Enforcement (Cross-Repo)

**Implementation command**: `spec-kitty implement WP23 --base WP06,WP09`

## Objective

Introduce hexagonal port/adapter boundaries across all repos where domain logic currently makes direct infrastructure calls (HTTP, file I/O, sandbox syscalls, cloud SDKs, ORM queries). This enforces the dependency inversion principle at repo boundaries and enables testability via port mocking.

## Context

- Multiple repos violate hexagonal architecture by embedding infrastructure calls directly in domain logic
- heliosCLI: `web_search.rs` makes direct HTTP calls; `codex-rs/state/` and `codex-rs/secrets/` perform file I/O in domain; no `SandboxPort` trait for sandbox policy (seatbelt/landlock/windows)
- cliproxy++: executors construct `http.Client` inline; executors accept concrete `*config.Config` instead of interface; `internal/thinking/` duplicates provider switch logic instead of using a `ThinkingBehavior` port
- thegent: 8+ files with direct `httpx` calls outside adapters (doctor/checks.py, tools/borrow.py, etc.); `clode_main.py` uses inline `os.execvpe()` instead of a `ProcessExecutionPort`
- heliosApp: desktop `stores/` has direct IPC coupling; `integrations/` embedded in runtime instead of port extraction
- portage: `gke.py` (1,023L) and `daytona.py` (1,081L) mix cloud SDK calls in domain; CLI `tasks.py`/`jobs.py` bypasses `use_cases/`
- trace: handlers import `gorm.DB` directly; `temporal_repository.go` (1,095L) mixes query logic with business logic

## Subtasks

### T069: heliosCLI hexagonal ports

**Steps**:
1. Define `HttpPort` trait in `codex-rs/core/` — `web_search.rs` consumes it instead of direct reqwest calls
2. Define `StatePort` and `SecretsPort` traits — `codex-rs/state/` and `codex-rs/secrets/` become adapters implementing these traits; domain code depends only on the trait
3. Define `SandboxPort` trait abstracting seatbelt (macOS), landlock (Linux), and Windows sandbox policy behind a unified interface
4. Update all call sites to accept `dyn Port` or generic `impl Port` parameters
5. Add mock implementations for each port in test modules

**Validation**:
- `cargo build --workspace` succeeds
- `cargo test --workspace` passes
- No direct `reqwest::` usage in domain modules (only in adapter impls)
- No direct `std::fs::` usage in domain modules for state/secrets
- Each port has at least one mock implementation used in tests

### T070: cliproxy++ hexagonal ports

**Steps**:
1. Define `HTTPClient` interface — executors accept it instead of constructing `http.Client` inline
2. Define `ConfigProvider` interface — executors accept it instead of concrete `*config.Config`
3. Extract `ThinkingBehavior` port from `internal/thinking/` — eliminate duplicated provider switch statements
4. Update all executor constructors to accept interfaces
5. Add mock implementations for testing

**Validation**:
- `go build ./...` succeeds
- `go test ./...` passes
- `grep -r 'http.Client{' internal/runtime/executor/` returns 0 matches
- All executors accept interface parameters, not concrete types

### T071: thegent + portage + trace hexagonal ports

**Steps**:
1. thegent: Route all direct `httpx` calls in doctor/checks.py, tools/borrow.py, and 6+ other files through the existing port system
2. thegent: Extract `ProcessExecutionPort` — `clode_main.py` calls port instead of inline `os.execvpe()`
3. portage: Define `CloudProviderPort` interface — `gke.py` and `daytona.py` become adapters behind the port
4. portage: Ensure CLI `tasks.py`/`jobs.py` routes through `use_cases/` layer instead of bypassing it
5. trace: Introduce repository layer — handlers no longer import `gorm.DB` directly
6. trace: Split `temporal_repository.go` (1,095L) — separate query/persistence from business logic

**Validation**:
- All test suites pass in thegent, portage, and trace
- `grep -r 'httpx\.' src/thegent/ --include='*.py'` only matches files under adapters/
- `grep -r 'gorm.DB' internal/handler/` returns 0 matches in trace
- `temporal_repository.go` split into files each <500 LOC

## Definition of Done

- [ ] All 6 repos have port traits/interfaces for infrastructure dependencies
- [ ] No direct HTTP, file I/O, or cloud SDK calls in domain modules
- [ ] Each port has mock implementations used in tests
- [ ] All test suites pass across all affected repos
- [ ] No public API changes to external consumers

## Risks

- **Cross-repo coordination complexity**: Changes span 6 repos and must not break inter-repo contracts. Sequence carefully per subtask.
- **Performance overhead from dynamic dispatch**: Port traits using `dyn` dispatch add vtable indirection. Profile hot paths and use `impl Trait` (monomorphization) where performance-critical.
- **Incomplete port coverage**: Some infrastructure calls may be missed in initial audit. Use grep/ripgrep sweeps post-implementation to verify.

## Reviewer Guidance

- Verify that port trait definitions live in domain/core modules, not in adapter modules
- Check that adapter implementations are in dedicated adapter directories
- Confirm no domain module has `use reqwest`, `use httpx`, `import gorm`, or direct cloud SDK imports
- Verify mock implementations are non-trivial and actually used in tests
