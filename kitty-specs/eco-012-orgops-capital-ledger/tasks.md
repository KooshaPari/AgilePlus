# Work Packages — OrgOps Capital Ledger (spec eco-012)

**Feature**: eco-012-orgops-capital-ledger
**Generated**: 2026-03-31
**Total WPs**: 8 | **Total Subtasks**: 38

## Overview

| Phase | WPs | Priority | Parallelizable |
|-------|-----|----------|----------------|
| Foundation | WP01-WP02 | P1 | WP01 → WP02 |
| Secrets & Profiles | WP03-WP04 | P1 | Parallel |
| Git & Worktrees | WP05 | P1 | After WP01 |
| CLI Integration | WP06 | P2 | After WP01-WP03 |
| Org Config | WP07 | P1 | Parallel with WP01 |
| Daemon & Monitoring | WP08 | P3 | After WP06 |

## Dependency Graph

```
WP07 (capital.toml) ──→ WP01 (capital crate) ──→ WP06 (CLI wiring)
                            │                          │
                            ├──→ WP03 (secrets)        │
                            │                          │
                            └──→ WP02 (ledger)    ──→ WP08 (daemon)
                                                      │
WP04 (profiles)  ─────────────────────────────────────┘
WP05 (git-core)  ─────────────────────────────────────┘
```

---

## WP07 — Org-Level Capital Configuration
**Priority**: P1 | **Est. lines**: ~50 (TOML) + ~200 (parser)
**Dependencies**: none
**Goal**: Create `repos/capital.toml` with full org resource inventory and the parser to read it.

- [ ] T001: Design and write `capital.toml` schema with accounts for LLM, cloud, auth, browser profiles
- [ ] T002: Implement `CapitalToml` parser in `phenotype-capital::registry` (serde Deserialize)
- [ ] T003: Implement `Account::validate()` trait method for each account type (api_key, subscription, always_free)
- [ ] T004: Add tests for TOML parsing, account resolution, and tag filtering

**Files**:
- `repos/capital.toml`
- `crates/phenotype-capital/src/registry.rs`

---

## WP01 — phenotype-capital Crate Foundation
**Priority**: P1 | **Est. lines**: ~450
**Dependencies**: WP07
**Goal**: Create the `phenotype-capital` crate with SQLite-backed capital ledger, reusing `agileplus-sqlite` patterns.

- [ ] T005: Scaffold `crates/phenotype-capital/` with Cargo.toml (deps: rusqlite, serde, tokio, phenotype-cost-core)
- [ ] T006: Implement `CapitalLedger` struct with SQLite connection (WAL mode, same pattern as SqliteStorageAdapter)
- [ ] T007: Implement `ResourceAccount` entity (id, name, type, env_var, rate_limit, budget, freshness_status)
- [ ] T008: Implement `SecretEntry` entity (id, account_id, env_var, value_hash, last_validated, status, rotation_interval)
- [ ] T009: Implement `ConsumptionRecord` entity (id, account_id, agent_id, tokens_used, api_calls, timestamp)
- [ ] T010: Implement CRUD operations: create/read/update accounts, secrets, consumption records
- [ ] T011: Implement `BudgetManager` integration — check consumption against `phenotype-cost-core::BudgetManager`
- [ ] T012: Implement SQLite migrations (CREATE TABLE for accounts, secrets, consumption, allocations)
- [ ] T013: Add comprehensive tests for all CRUD ops and budget enforcement

**Files**:
- `crates/phenotype-capital/Cargo.toml`
- `crates/phenotype-capital/src/lib.rs`
- `crates/phenotype-capital/src/ledger.rs`
- `crates/phenotype-capital/src/migrations.rs`

---

## WP02 — Consumption Tracking & Budget Enforcement
**Priority**: P1 | **Est. lines**: ~300
**Dependencies**: WP01
**Goal**: Implement real-time consumption tracking and budget enforcement using `phenotype-cost-core`.

- [ ] T014: Implement `record_consumption(account_id, tokens, api_calls)` — appends to SQLite
- [ ] T015: Implement `check_budget(project_id, account_id)` — returns remaining vs. allocated
- [ ] T016: Implement `enforce_budget()` — returns `Result<(), CostError::BudgetExceeded>` before API calls
- [ ] T017: Implement `daily_summary(account_id)` — aggregate consumption for current day
- [ ] T018: Implement `project_allocation(project_id)` — which resources a project is allocated
- [ ] T019: Add tests for budget enforcement, aggregation, and allocation queries

**Files**:
- `crates/phenotype-capital/src/consumption.rs`

---

## WP03 — Secret Lifecycle Management
**Priority**: P1 | **Est. lines**: ~350
**Dependencies**: WP01
**Goal**: Implement secret validation, rotation, and propagation (SQLite + .env export).

- [ ] T020: Implement `validate_secret(entry)` — ping service endpoint, update freshness status
- [ ] T021: Implement `validate_all()` — iterate all secrets, return health report
- [ ] T022: Implement `rotate_secret(entry)` — write-test-swap pattern with rollback
- [ ] T023: Implement `export_env(project_path)` — write `.env` from SQLite secrets for a project
- [ ] T024: Implement `import_env(project_path)` — read existing `.env` into SQLite (migration path)
- [ ] T025: Implement secret freshness detection — flag keys older than rotation_interval
- [ ] T026: Add tests for validation, rotation, export, and freshness detection

**Files**:
- `crates/phenotype-capital/src/secrets.rs`

---

## WP04 — phenotype-profiles Crate (Browser UA)
**Priority**: P1 | **Est. lines**: ~300
**Dependencies**: none (parallel with WP01-WP03)
**Goal**: Create `phenotype-profiles` for persistent Chrome user-data-dir management.

- [ ] T027: Scaffold `crates/phenotype-profiles/` with Cargo.toml (deps: serde, dirs, rusqlite)
- [ ] T028: Implement `ProfileManager` — create/list/delete Chrome user-data-dir profiles
- [ ] T029: Implement `AuthSession` detection — check if GitHub/Supabase/etc auth is still valid
- [ ] T030: Implement `refresh_on_failure` — detect stale auth, attempt re-auth via saved cookies
- [ ] T031: Implement profile encryption at rest (AES-256 via existing `phenotype-crypto` or `ring`)
- [ ] T032: Add tests for profile CRUD, auth detection, and refresh

**Files**:
- `crates/phenotype-profiles/Cargo.toml`
- `crates/phenotype-profiles/src/lib.rs`
- `crates/phenotype-profiles/src/chrome.rs`
- `crates/phenotype-profiles/src/auth.rs`
- `crates/phenotype-profiles/src/vault.rs`

---

## WP05 — phenotype-git-core Worktree Extension
**Priority**: P1 | **Est. lines**: ~250
**Dependencies**: none (parallel)
**Goal**: Extend the stub `phenotype-git-core` with gix-based worktree management.

- [ ] T033: Implement `create_worktree(project, branch)` — gix worktree creation at `.worktrees/<project>/<branch>`
- [ ] T034: Implement `list_active()` — scan `.worktrees/` and parse gix state
- [ ] T035: Implement `prune_stale(max_age_days)` — remove worktrees older than threshold
- [ ] T036: Implement `canonical_release_track(project)` — detect which release branch canonical is on
- [ ] T037: Add tests for worktree lifecycle (create, list, prune, release tracking)

**Files**:
- `crates/phenotype-git-core/src/lib.rs`
- `crates/phenotype-git-core/src/worktree.rs`

---

## WP06 — AgilePlus CLI Integration
**Priority**: P2 | **Est. lines**: ~300
**Dependencies**: WP01, WP03
**Goal**: Wire `agileplus capital` and `agileplus secrets` CLI subcommands.

- [ ] T038: Implement `agileplus capital status` — show all accounts, budgets, freshness
- [ ] T039: Implement `agileplus capital check` — validate all secrets, return health report
- [ ] T040: Implement `agileplus capital export-env` — write .env for current project
- [ ] T041: Implement `agileplus secrets list` — show secret inventory with rotation state
- [ ] T042: Implement `agileplus secrets rotate --all` — rotate all stale secrets
- [ ] T043: Implement `agileplus worktree create/list/prune` — worktree management
- [ ] T044: Implement `agileplus profiles list` — show browser UA profiles + auth state

**Files**:
- `AgilePlus/crates/agileplus-cli/src/commands/capital.rs`
- `AgilePlus/crates/agileplus-cli/src/commands/secrets.rs`
- `AgilePlus/crates/agileplus-cli/src/commands/worktree.rs`
- `AgilePlus/crates/agileplus-cli/src/commands/profiles.rs`

---

## WP08 — Daemon & Process-Compose Integration
**Priority**: P3 | **Est. lines**: ~200
**Dependencies**: WP06
**Goal**: Optional background daemon for continuous secret monitoring.

- [ ] T045: Implement `agileplus daemon capital --interval 300` — periodic validation loop
- [ ] T046: Add process-compose.yml entry for capital-monitor service
- [ ] T047: Implement evidence_ledger.jsonl integration — log all validation/rotation events
- [ ] T048: Add daemon health check endpoint

**Files**:
- `AgilePlus/crates/agileplus-cli/src/commands/daemon.rs`
- `repos/process-compose.yml` (amend)
