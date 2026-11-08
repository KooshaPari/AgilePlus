---
work_package_id: "WP22"
title: "heliosApp: Critical Test Coverage + agentapi++ Dedup"
lane: "planned"
dependencies: ["WP11"]
subtasks: ["T066", "T067", "T068"]
history:
  - date: "2026-03-03"
    event: "created"
    by: "spec-kitty.tasks"
---

# WP22: heliosApp — Critical Test Coverage + agentapi++ Dedup

**Implementation command**: `spec-kitty implement WP22`

## Objective

Close the test coverage gap in heliosApp where 11 of 17 runtime modules have zero tests, and eliminate the full inner-directory duplication in agentapi++. heliosApp has excellent code quality (0 `any`, 0 `console.log`) but the entire audit subsystem (2,577 LOC, 12 files) and protocol bus (805 LOC) are completely untested. agentapi++ contains ~50% duplicate LOC from an inner copy of itself.

## Context

- heliosApp: 78K TS, 8 god modules, 0 `any` casts, 0 `console.log` — code quality is high but test coverage is critically low
- Audit subsystem: 2,577 LOC across 12 files with zero test coverage — this is the persistence and replay layer
- Protocol bus: 805 LOC, the IPC backbone for inter-component messaging — untested
- WP11 is building audit-core on Emmett — tests should target the Emmett-backed implementation
- agentapi++: 16,804 LOC total but contains `agentapi-plusplus/agentapi-plusplus/` inner duplicate (~50% of LOC)
- See `kitty-specs/002-phenotype-modular-arch/research/codebase-quality-analysis.md` for metrics

## Subtasks

### T066: Test audit subsystem (2,577 LOC, 12 files, 0 tests)

**Steps**:
1. Create test files for each audit module, prioritized by criticality:
   - **P0 (core correctness)**: `ledger.test.ts` (351L source), `sqlite-store.test.ts` (293L), `replay.test.ts` (149L)
   - **P1 (data flow)**: `sink.test.ts` (282L), `api.test.ts` (265L), `bus-subscriber.test.ts` (176L), `export.test.ts` (176L)
   - **P2 (supporting)**: `ring-buffer.test.ts` (219L), `event.test.ts` (212L), `audit-sink.test.ts` (210L), `retention.test.ts` (152L), `snapshot.test.ts` (92L)
2. For ledger tests: verify event append, ordering, idempotency, and recovery after crash
3. For sqlite-store tests: verify persistence round-trip, schema migration, concurrent access
4. For replay tests: verify event replay produces identical state, handles corrupted events gracefully
5. Important: WP11 is building audit-core on Emmett — write tests against the Emmett-backed implementation where available, with adapters for the current implementation as fallback
6. Use existing heliosApp test patterns and framework (vitest or equivalent)

**Validation**:
- 100% file coverage for `audit/` — every source file has a corresponding test file
- All tests pass
- Coverage >=80% for P0 files (ledger, sqlite-store, replay)

### T067: Test protocol bus + critical runtime modules

**Steps**:
1. Create `protocol/bus.test.ts` for the IPC backbone (805L):
   - Test message routing between registered handlers
   - Test subscription and unsubscription lifecycle
   - Test topic filtering and wildcard matching
   - Test error propagation when handlers throw
   - Test concurrent message delivery ordering guarantees
2. Create test files for untested config modules (5 files):
   - Test config validation rules (required fields, type coercion, defaults)
   - Test config merging precedence (file < env < CLI args)
   - Test error messages for invalid configurations
3. Create test files for untested lanes modules (13 files):
   - Test lane orchestration: creation, activation, deactivation, cleanup
   - Test lane isolation — operations in one lane do not affect another
4. Create test files for untested policy modules (5 files):
   - Test policy evaluation against sample agent actions
   - Test policy composition (allow + deny rules)
5. Create test files for untested sessions modules (4 files):
   - Test session creation, persistence, and restoration
   - Test session expiry and cleanup

**Validation**:
- `protocol/` has >=1 test file with >=10 test cases
- `config/` has >=1 test file
- All tests pass

### T068: agentapi++ — Remove inner directory duplication

**Steps**:
1. Identify the duplication: agentapi++ contains `agentapi-plusplus/agentapi-plusplus/` as a full inner copy
2. Compare outer and inner directories to determine which is canonical:
   - Check git history — which was committed first?
   - Check imports — which do other repos reference?
   - Check CI/build scripts — which path do they use?
3. Likely outcome: the outer directory is canonical. Delete the inner `agentapi-plusplus/agentapi-plusplus/` directory
4. Search for and update any references to the inner path:
   - CI workflow files (`.github/workflows/`)
   - Build scripts and Makefiles
   - Import paths in Go files
   - Documentation references
5. Verify the repo still builds and tests pass after deletion

**Validation**:
- `find . -name 'agentapi-plusplus' -type d` returns only the repo root
- `go build ./...` passes
- `go test ./...` passes
- No CI scripts reference the deleted inner path

## Definition of Done

- [ ] All 12 audit files have corresponding test files
- [ ] Audit P0 files (ledger, sqlite-store, replay) have >=80% coverage
- [ ] Protocol bus has >=10 test cases covering routing, subscription, and error handling
- [ ] Config and policy modules each have >=1 test file
- [ ] agentapi++ inner directory duplication removed (~8K LOC deleted)
- [ ] All tests pass across heliosApp and agentapi++

## Risks

- **WP11 Emmett migration**: If WP11 significantly changes the audit subsystem API, tests written here may need rewriting. Mitigate by writing tests against the interface/contract, not implementation details.
- **Protocol bus concurrency**: The bus likely uses async patterns that are hard to test deterministically. Use controlled schedulers or explicit flush/drain in tests.
- **agentapi++ canonical determination**: If both inner and outer directories have diverged, a merge may be needed instead of a simple delete. Check file-level diffs before deleting.

## Reviewer Guidance

- Verify audit tests cover failure modes (corrupted data, full disk, concurrent writes) not just happy paths
- Check that protocol bus tests verify ordering guarantees — this is the IPC backbone and ordering bugs cause cascading failures
- For T068, confirm the reviewer checks `git log` on both directories to verify which is canonical before approving deletion
- Ensure test files follow existing heliosApp test patterns (describe/it structure, fixture management, cleanup)
