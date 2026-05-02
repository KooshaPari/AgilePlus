---
spec_id: eco-003
slug: eco-003-circular-dep-resolution
title: Circular Dependency Resolution
status: in_progress
created_at: "2026-03-29T00:00:00Z"
type: operational
---

# Circular Dependency Resolution — Work Packages

**Spec**: eco-003-circular-dep-resolution
**Type**: Operational
**Created**: 2026-03-29
**Total WPs**: 8 | **Phases**: 4 (Audit, Break Critical Cycles, Prevention, Governance)

---

## What

The AgilePlus workspace contains two confirmed Cargo.toml-level circular dependencies that prevent independent crate compilation and may cause incremental build failures. Additionally, module-level `pub use` chains are unverified. This spec defines a full cycle audit, architectural remediation, CI prevention gate, and documentation so the workspace remains cycle-free.

## Why

Circular dependencies between crates violate the hexagonal architecture constraint that domain is the base of all dependency direction. They block independent compilation of `agileplus-api`/`agileplus-dashboard` and `agileplus-agent-review`/`agileplus-agent-service`, create slow iterative builds, and make the event-sourcing hash chain fragile under concurrent refactoring. Preventing cycles is prerequisite to the hexagonal migration (eco-004) and any future crates.io publishing.

**Confirmed cycles:**
1. `agileplus-api → agileplus-dashboard → (api deps: domain/events/sqlite) → agileplus-api` — Cargo.toml cycle
2. `agileplus-agent-review → agileplus-agent-service → agileplus-agent-review` — Cargo.toml cycle

---

## WP Index

| WP | Title | Priority | Effort | Depends |
|----|-------|----------|--------|---------|
| WP-ECO301 | Full Circular Dependency Audit | HIGH | 1–2 calls | — |
| WP-ECO302 | Analyze `agileplus-api` ↔ `agileplus-dashboard` Cycle | CRITICAL | 2–3 calls | WP-ECO301 |
| WP-ECO303 | Break `agileplus-api` ↔ `agileplus-dashboard` Cargo.toml Cycle | CRITICAL | 4–6 calls | WP-ECO302 |
| WP-ECO304 | Break `agileplus-agent-review` ↔ `agileplus-agent-service` Cycle | HIGH | 3–4 calls | WP-ECO301 |
| WP-ECO305 | Add `cargo cycle-detect` to CI | HIGH | 1–2 calls | WP-ECO303, WP-ECO304 |
| WP-ECO306 | Dependency Governance Documentation | MEDIUM | 1 call | WP-ECO303, WP-ECO304 |
| WP-ECO307 | Circular Dep CI Linter (Advanced) | MEDIUM | 3–5 calls | WP-ECO306 |
| WP-ECO308 | Update Hexagonal Architecture Spec (eco-004 Cross-Reference) | LOW | 1 call | WP-ECO307 |

---

## Phase 1 — Audit

### WP-ECO301 — Full Circular Dependency Audit

**What:** Enumerate all Rust crate-level and module-level circular dependencies in the AgilePlus workspace using `cargo tree --duplicates`, `grep` for `pub use`, and manual `Cargo.toml` analysis.

**Why:** The current audit only confirmed two Cargo.toml cycles. Module-level cycles (intra-crate `pub use` chains) and workspace member ordering issues are unverified. A complete inventory is prerequisite for all downstream work.

**Priority:** HIGH
**Dependencies:** None
**Effort:** ~1–2 tool calls

#### Acceptance Criteria

- `cargo tree --duplicates -e normal` output is clean (0 duplicate packages indicating false deps)
- All crates with `path = "../..."` deps listed in a table: `Consumer → Supplier → Cycle`
- Module-level `pub use` chains audited; table of 3+ hop re-exports produced
- Results committed to `evidence_ledger.jsonl` and this tasks.md updated

#### Verification Checklist

- [ ] `cargo tree --duplicates -e normal 2>&1` runs without error in workspace root
- [ ] No duplicate packages flagged in output (false deps = unused transitive deps)
- [ ] `grep -rn "pub use" crates/*/src libs/*/src --include="*.rs"` produces no chains ≥3 hops unaccounted for
- [ ] `CYCLE_TABLE.md` exists in this spec dir with findings committed
- [ ] `evidence_ledger.jsonl` has entry for WP-ECO301 audit results
- [ ] Both confirmed cycles (api/dashboard, review/service) listed in CYCLE_TABLE.md
- [ ] New cycles (if any) added to CYCLE_TABLE.md with "NEW" tag

---

## Phase 2 — Break Critical Cycles

### WP-ECO302 — Analyze `agileplus-api` ↔ `agileplus-dashboard` Cycle

**What:** Deep-dive into the `agileplus-api` ↔ `agileplus-dashboard` Cargo.toml cycle — what types are shared, why dashboard exists as a dep of api, and whether the cycle can be broken by extracting shared types.

**Why:** `agileplus-api/src/router.rs` (line 101–103) calls `agileplus_dashboard::app_state::DashboardStore::seeded()` and `agileplus_dashboard::routes::router(dashboard_state)`. The dashboard runs as a standalone Axum server. The cycle exists because api mounts dashboard's router as a sub-router. The correct fix is NOT to remove the runtime coupling but to restructure the dependency direction.

**Priority:** CRITICAL
**Dependencies:** WP-ECO301
**Effort:** ~2–3 tool calls

#### Acceptance Criteria

- `crates/agileplus-api/src/router.rs` usages of `agileplus_dashboard` fully enumerated with line numbers
- Root cause classified: (a) dashboard types leak into api types, (b) api just mounts dashboard router at runtime, or (c) both
- 3 candidate resolutions documented with trade-offs:
  - **Flip:** dashboard implements a trait from api; api has zero dashboard deps
  - **Extract:** shared dashboard-agnostic types → `agileplus-domain` or new `agileplus-shared-types`
  - **Eliminate:** remove dashboard from workspace; publish as separate crate
- ADR drafted in `../../docs/changes/eco-003-dashboard-cycle-adr.md`

#### Verification Checklist

- [ ] `grep -n "agileplus_dashboard" crates/agileplus-api/src/router.rs` output shows all usages with line numbers
- [ ] `crates/agileplus-dashboard/src/lib.rs` exports enumerated — dashboard types vs. router functions separated
- [ ] Root cause is explicitly stated at top of ADR document
- [ ] ADR has at least 3 options with trade-off table (pros/cons/risk)
- [ ] Recommended resolution is clearly marked in ADR
- [ ] ADR committed to `docs/changes/eco-003-dashboard-cycle-adr.md`
- [ ] `evidence_ledger.jsonl` entry links to ADR commit

---

### WP-ECO303 — Break `agileplus-api` ↔ `agileplus-dashboard` Cargo.toml Cycle

**What:** Remove the `agileplus-dashboard` path dep from `agileplus-api/Cargo.toml` and refactor the router so api does not depend on the dashboard crate at compile time.

**Why:** The Cargo.toml cycle prevents independent compilation. The architectural fix is to have dashboard expose a **trait** (e.g., `DashboardPlugin`) that api's router registers at runtime via a feature-gated `dyn` call — breaking the compile-time dep while preserving the runtime mount.

**Priority:** CRITICAL
**Dependencies:** WP-ECO302 (must know which types to extract/abstract first)
**Effort:** ~4–6 tool calls

#### Acceptance Criteria

- `agileplus-api/Cargo.toml` no longer contains `agileplus-dashboard` in `[dependencies]`
- `agileplus-dashboard` implements a `DashboardPlugin` trait defined in `agileplus-api` or a shared location
- `agileplus-api/src/router.rs` uses `axum::Router::merge()` with a `Box<dyn DashboardPlugin>` — zero compile-time dep on dashboard types
- Both crates compile independently: `cargo check -p agileplus-api -p agileplus-dashboard`
- Tests pass: `cargo test -p agileplus-api -p agileplus-dashboard`

#### Verification Checklist

- [ ] `agileplus-dashboard` absent from `crates/agileplus-api/Cargo.toml` `[dependencies]` section
- [ ] `DashboardPlugin` trait defined in `agileplus-api/src/routes/dashboard_plugin.rs` (pub)
- [ ] `agileplus-dashboard` implements `DashboardPlugin`; its own `routes::router()` remains internal
- [ ] `agileplus-api/src/router.rs` uses `Box<dyn DashboardPlugin>` via `Router::merge()` with no `use` of `agileplus_dashboard` types
- [ ] `cargo check -p agileplus-api -p agileplus-dashboard` succeeds without errors
- [ ] `cargo test -p agileplus-api -p agileplus-dashboard` passes (0 test failures)
- [ ] `cargo tree -e normal -p agileplus-api` shows no path to `agileplus-dashboard`

---

### WP-ECO304 — Break `agileplus-agent-review` ↔ `agileplus-agent-service` Cycle

**What:** Break the Cargo.toml circular dep between `agileplus-agent-review` and `agileplus-agent-service`.

**Why:** `agent-service` depends on both `agent-dispatch` and `agent-review`; `agent-review` transitively depends back on service via `agent-dispatch`. The architectural pattern here differs from the api/dashboard case — these are gRPC service crates. The fix is to extract shared request/response types into `agent-dispatch` (which has no outbound deps) and make both service and review depend only on dispatch.

**Priority:** HIGH
**Dependencies:** WP-ECO301
**Effort:** ~3–4 tool calls

#### Acceptance Criteria

- Neither `agent-review` nor `agent-service` Cargo.toml lists the other in `[dependencies]`
- Shared types (proto-generated structs used by both) live in `agent-dispatch`
- Both crates compile: `cargo check -p agileplus-agent-review -p agileplus-agent-service`
- gRPC contract test `cargo test -p agileplus-contract-tests` passes

#### Verification Checklist

- [ ] `agileplus-agent-review/Cargo.toml` has no `agileplus-agent-service` in `[dependencies]`
- [ ] `agileplus-agent-service/Cargo.toml` has no `agileplus-agent-review` in `[dependencies]`
- [ ] Shared proto-generated message types reside in `agent-dispatch` and are imported by both crates
- [ ] `cargo check -p agileplus-agent-review -p agileplus-agent-service` succeeds without errors
- [ ] `cargo test -p agileplus-contract-tests` passes (0 test failures)
- [ ] `cargo tree -e normal -p agileplus-agent-review` shows no path to `agileplus-agent-service`
- [ ] `cargo tree -e normal -p agileplus-agent-service` shows no path to `agileplus-agent-review`

---

## Phase 3 — Prevention

### WP-ECO305 — Add `cargo cycle-detect` to CI

**What:** Add a CI gate that fails if any new Cargo.toml dependency introduces a cycle.

**Why:** Manual audit is brittle and decays. A deterministic check run on every PR prevents regression.

**Priority:** HIGH
**Dependencies:** WP-ECO303, WP-ECO304
**Effort:** ~1–2 tool calls

#### Acceptance Criteria

- `.github/workflows/cycle-detect.yml` exists and is enabled in the `spec/eco-003-expand` branch
- Workflow uses `cargo cyclonedds` or a custom script to emit a non-zero exit code on any new cycle
- Workflow runs on `pull_request` and `push` to `main`
- PR that introduces a cycle is blocked; CI failure is clear and actionable

#### Verification Checklist

- [ ] `.github/workflows/cycle-detect.yml` file exists and uses `cargo tree` for graph analysis
- [ ] Workflow triggers on `pull_request` and `push` to `main` (not just workflow_dispatch)
- [ ] Workflow emits non-zero exit code when cycles are detected
- [ ] Workflow error message includes cycle path (e.g., `A → B → C → A`)
- [ ] Workflow succeeds on current codebase post WP-ECO303 and WP-ECO304
- [ ] `just lint-cycles` (or equivalent) available as local dev command
- [ ] PR introducing a synthetic cycle is blocked (verified via test PR)

---

### WP-ECO306 — Dependency Governance Documentation

**What:** Document the dependency rules for AgilePlus crates so future contributors know the correct way to introduce a new dep.

**Why:** Circular deps arise from unclear ownership boundaries. A written policy (part of `AGENTS.md` or a new `docs/guides/dependency-governance.md`) makes the correct pattern explicit and enforceable.

**Priority:** MEDIUM
**Dependencies:** WP-ECO303, WP-ECO304
**Effort:** ~1 tool call

#### Acceptance Criteria

- `docs/guides/dependency-governance.md` created in the spec worktree
- Section: "The Dependency Rule" — domain is the base, all other crates depend on domain (not each other)
- Section: "Breaking Cycles" — 3 patterns (trait extraction, type push-down, shared crate)
- Section: "Anti-patterns" — direct cross-crate path deps between feature crates
- Linked from `AGENTS.md` and `CLAUDE.md`

#### Verification Checklist

- [ ] `docs/guides/dependency-governance.md` exists and is non-empty
- [ ] "The Dependency Rule" section explicitly states domain is the base of all dependency direction
- [ ] "Breaking Cycles" section documents at least 3 patterns with examples
- [ ] "Anti-patterns" section flags direct cross-crate path deps between non-shared crates
- [ ] Reference to `docs/guides/dependency-governance.md` added to `AGENTS.md`
- [ ] Reference to `docs/guides/dependency-governance.md` added to `CLAUDE.md`
- [ ] `evidence_ledger.jsonl` entry records doc creation

---

### WP-ECO307 — Circular Dep CI Linter (Advanced)

**What:** Implement a Rust-based or Python-based linter that parses `Cargo.toml` files and emits warnings/errors for cycles before they enter the codebase.

**Why:** The CI workflow from WP-ECO305 uses `cargo tree` at the workspace level. A dedicated linter can also catch intra-crate module cycles (`pub use` chains) and produce structured output (JSON) for GitHub annotations.

**Priority:** MEDIUM
**Dependencies:** WP-ECO306
**Effort:** ~3–5 tool calls

#### Acceptance Criteria

- `tools/cycle-checker/` crate or script published to workspace
- Parses all `Cargo.toml` files, builds dep graph, detects SCCs (strongly connected components)
- Emits SARIF or GitHub workflow commands output for annotations
- Integrated into `justfile` as `just lint-cycles` and into pre-commit hook
- Zero false positives on current codebase

#### Verification Checklist

- [ ] `tools/cycle-checker/` directory exists with runnable source (Rust binary or Python script)
- [ ] Tool accepts workspace root as argument; outputs cycle list or empty on success
- [ ] Tool detects both Cargo.toml-level and `pub use` module-level cycles
- [ ] Tool outputs SARIF or structured JSON for GitHub annotations
- [ ] `just lint-cycles` target exists in `justfile` and calls the tool
- [ ] Pre-commit hook (`.git/hooks/pre-commit` or config) invokes cycle checker
- [ ] Tool returns 0 on current codebase, 1 when a synthetic cycle is introduced

---

## Phase 4 — Governance

### WP-ECO308 — Update Hexagonal Architecture Spec (eco-004 Cross-Reference)

**What:** Update the hexagonal architecture specification (eco-004) to explicitly reference the cycle-breaking patterns established in eco-003, and add the dependency rule as an architectural invariant.

**Why:** eco-004 already defines ports and adapters. eco-003 establishes concrete cycle-breaking tactics. The two specs are complementary; eco-004 should codify the cycle-free dependency constraint as a first-class architectural rule.

**Priority:** LOW
**Dependencies:** WP-ECO307
**Effort:** ~1 tool call

#### Acceptance Criteria

- `kitty-specs/eco-004-hexagonal-migration/tasks.md` updated with new section: "Dependency Cycle Invariant"
- eco-003 cited as the canonical reference for cycle-breaking tactics
- `AGENTS.md` updated to note the cross-reference

#### Verification Checklist

- [ ] "Dependency Cycle Invariant" section added to `kitty-specs/eco-004-hexagonal-migration/tasks.md`
- [ ] Section explicitly cites eco-003 and its cycle-breaking patterns
- [ ] Section states the invariant: "No crate in the workspace may depend (directly or transitively) on itself"
- [ ] `AGENTS.md` has a cross-reference to eco-004's Dependency Cycle Invariant
- [ ] Both spec docs reference each other bidirectionally

---

## Dependency Graph (WP Ordering)

```
WP-ECO301 ──► WP-ECO302 ──► WP-ECO303 ──► WP-ECO305
    │                                                     │
    │                                                     ▼
    ▼                                                 WP-ECO306
WP-ECO304 ────────────────────────────────────────────────► WP-ECO307
                                                                  │
                                                                  ▼
                                                              WP-ECO308
```

| WP | Blocked By | Blocks |
|----|-----------|--------|
| WP-ECO301 | — | WP-ECO302, WP-ECO304 |
| WP-ECO302 | WP-ECO301 | WP-ECO303 |
| WP-ECO303 | WP-ECO302 | WP-ECO305, WP-ECO306 |
| WP-ECO304 | WP-ECO301 | WP-ECO305, WP-ECO306 |
| WP-ECO305 | WP-ECO303, WP-ECO304 | — |
| WP-ECO306 | WP-ECO303, WP-ECO304 | WP-ECO307 |
| WP-ECO307 | WP-ECO306 | WP-ECO308 |
| WP-ECO308 | WP-ECO307 | — |

---

## Verification Gate

All items below must be true before eco-003 is marked **COMPLETED** in `meta.json`.

- [ ] `cargo tree --duplicates -e normal` runs clean (0 false-dep duplicates)
- [ ] No cycles in `cargo tree -e normal --no-dev-dependencies --workspace`
- [ ] Both `cargo check -p agileplus-api -p agileplus-dashboard` pass independently
- [ ] Both `cargo check -p agileplus-agent-review -p agileplus-agent-service` pass independently
- [ ] `cargo test --workspace` passes (0 failures introduced by eco-003 changes)
- [ ] `.github/workflows/cycle-detect.yml` exists, enabled, and passes on current branch
- [ ] `docs/guides/dependency-governance.md` exists and is linked from `AGENTS.md` and `CLAUDE.md`
- [ ] `kitty-specs/eco-004-hexagonal-migration/tasks.md` updated with "Dependency Cycle Invariant"
- [ ] `meta.json` status updated from `"in_progress"` to `"completed"`

---

## Status Summary

| WP | Title | Priority | Effort | Status |
|----|-------|----------|--------|--------|
| WP-ECO301 | Full Circular Dependency Audit | HIGH | 1–2 calls | ⬜ |
| WP-ECO302 | Analyze `agileplus-api` ↔ `agileplus-dashboard` | CRITICAL | 2–3 calls | ⬜ |
| WP-ECO303 | Break `agileplus-api` ↔ `agileplus-dashboard` | CRITICAL | 4–6 calls | ⬜ |
| WP-ECO304 | Break `agileplus-agent-review` ↔ `agileplus-agent-service` | HIGH | 3–4 calls | ⬜ |
| WP-ECO305 | Cycle-detect CI | HIGH | 1–2 calls | ⬜ |
| WP-ECO306 | Dependency Governance Doc | MEDIUM | 1 call | ⬜ |
| WP-ECO307 | Advanced Cycle Linter | MEDIUM | 3–5 calls | ⬜ |
| WP-ECO308 | eco-004 Cross-Reference | LOW | 1 call | ⬜ |
