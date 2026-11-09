# Work Package Index: eco-007 Schema Versioning and API Evolution

**Feature**: eco-007-schema-evolution
**Total WPs**: 7 | **Total Subtasks**: 29
**MVP Scope**: WP01 → WP02 → WP03 → WP04 = SQLite schema versioning + proto versioning + event versioning + CI enforcement (4 WPs)
**Full Scope**: + WP05 (Migration Runner) + WP06 (Plugin Versioning) + WP07 (Rollback & Audit)

---

## What / Why

AgilePlus is a 24-crate Rust workspace with event-sourced state, SQLite persistence, gRPC inter-service communication, and a plugin architecture. Without disciplined schema versioning, any schema change risks:

- **Event store breakage**: Old events become unreadable when their schema evolves without a version header
- **gRPC client breakage**: Proto changes silently break all existing clients (mobile, web, CLI)
- **Migration chaos**: Ad-hoc SQLite migrations cause data loss or inconsistency
- **Plugin drift**: Storage and VCS adapters compiled against old schemas fail at runtime

This spec implements a layered versioning strategy:
1. **SQLite layer**: Versioned migrations with rollback support
2. **Proto/gRPC layer**: Versioned API packages with breaking-change detection
3. **Event layer**: Schema version in every event header, enabling forward-compatible replay
4. **Plugin layer**: Version negotiation between adapters and core

The result is an evolution-safe system where clients, plugins, and event consumers can upgrade on their own schedule without coordination.

---

## Dependency Graph

```
WP01 (SQLite Schema Versioning)
├── WP02 (Proto Versioning & CI)    [depends: none]
│   ├── WP03 (Event Schema Version)  [depends: WP01]
│   └── WP04 (Breaking Change CI)    [depends: WP02]
└── WP05 (Migration Runner)         [depends: WP01]
    └── WP06 (Plugin Versioning)     [depends: WP05]
└── WP07 (Rollback & Audit)          [depends: WP05]
```

**Parallelizable**: WP03, WP04, WP05 all become available after WP01. WP06 depends on WP05. WP07 depends on WP05.

---

## Phase 1 — SQLite Schema Versioning

### WP01: SQLite Schema Version Table and Migration Infrastructure (5 subtasks, ~300 lines)

**Goal**: Establish a versioned migration baseline for SQLite — every table change goes through a versioned migration file, and the current version is tracked in a `schema_version` table.
**Priority**: P1 | **Dependencies**: none
**FRs**: FR-SEV-01, FR-SEV-07
**Files touched**: `libs/plugin-sqlite/`, `migrations/`

Subtasks:
- [ ] T001: Create `migrations/` directory at repo root with `0001_initial_schema.sql` as the baseline
- [ ] T002: Implement `schema_version` table: columns `version INTEGER PRIMARY KEY`, `applied_at TEXT`, `description TEXT`, `checksum TEXT`
- [ ] T003: Implement `Migration` Rust struct with fields: version, description, up_sql, down_sql, checksum
- [ ] T004: Implement `MigrationLoader` — reads all `migrations/*.sql` files sorted by filename version prefix
- [ ] T005: Implement `SchemaVersionManager` — query/set current version, detect pending migrations

---

## Phase 2 — Proto Versioning

### WP02: Proto Versioned Packages and buf CI Integration (4 subtasks, ~200 lines)

**Goal**: Version all proto packages with a suffix and wire `buf` breaking-change detection into CI.
**Priority**: P1 | **Dependencies**: none
**FRs**: FR-SEV-02, FR-SEV-05, FR-SEV-08
**Files touched**: `proto/`, `.github/workflows/`

Subtasks:
- [ ] T006: Audit all proto files and add version suffix to package names (e.g., `agileplus.v1` → `agileplus.v2`)
- [ ] T007: Configure `buf.yaml` with `version: v2` and breaking-change enforcement
- [ ] T008: Create `.bufbreaking.json` config that allows additive-only changes (no field removal, no type changes)
- [ ] T009: Implement `buf.gen.yaml` update to generate versioned gRPC client stubs

---

## Phase 3 — Event Schema Versioning

### WP03: Event Schema Version in Event Header (5 subtasks, ~250 lines)

**Goal**: Every stored event carries a `schema_version` field in its JSON/MessagePack envelope, enabling replay of old events against current handlers.
**Priority**: P1 | **Dependencies**: WP01
**FRs**: FR-SEV-03
**Files touched**: `libs/intent-registry/`, `libs/event-sourcing/` (or equivalent domain crate)

Subtasks:
- [ ] T010: Define `EventEnvelope` struct with fields: `event_id`, `schema_version`, `event_type`, `payload`, `timestamp`, `previous_hash`
- [ ] T011: Update all existing event types to be wrapped in `EventEnvelope` with the current `SCHEMA_VERSION` constant
- [ ] T012: Implement `SchemaVersionExtractor` — given a raw event bytes, extract schema_version without full deserialization
- [ ] T013: Implement `UpcastHandler` — given an old-schema event, applies transformation functions to bring it to current schema
- [ ] T014: Write at least 2 upcast functions for existing event types (demonstrating the pattern)

---

## Phase 4 — CI Enforcement

### WP04: Breaking Change CI Gates (4 subtasks, ~150 lines)

**Goal**: Enforce schema discipline in CI — no breaking proto changes, no unversioned migrations.
**Priority**: P1 | **Dependencies**: WP02
**FRs**: FR-SEV-05
**Files touched**: `.github/workflows/`, `migrations/`

Subtasks:
- [ ] T015: Create `.github/workflows/schema-evolution.yml` with `buf breaking` gate on all PRs
- [ ] T016: Add migration-file naming convention check (enforces `NNNN_description.sql` pattern)
- [ ] T017: Add schema-version consistency check (verifies `schema_version` table exists and is queryable)
- [ ] T018: Add event-envelope check to `task quality` (fails if events lack `schema_version` field)

---

## Phase 5 — Migration Runner

### WP05: Migration Runner CLI (4 subtasks, ~300 lines)

**Goal**: Provide `agileplus migrate` CLI commands for applying, inspecting, and rolling back migrations.
**Priority**: P2 | **Dependencies**: WP01
**FRs**: FR-SEV-04, FR-SEV-09
**Files touched**: `agileplus/` (Python MCP server), `libs/plugin-sqlite/`, `apps/`

Subtasks:
- [ ] T019: Implement `agileplus migrate status` — reports current schema version and list of pending migrations
- [ ] T020: Implement `agileplus migrate apply [--dry-run]` — executes exactly one pending migration atomically
- [ ] T021: Implement `agileplus migrate apply --all` — runs all pending migrations in order
- [ ] T022: Implement `agileplus migrate rollback <version>` — reverts to the specified version using down_sql

---

## Phase 6 — Plugin Versioning

### WP06: Plugin Schema Version Negotiation (4 subtasks, ~200 lines)

**Goal**: Storage and VCS plugin adapters declare their compatible schema version range; core fails fast if versions are incompatible.
**Priority**: P2 | **Dependencies**: WP05
**FRs**: FR-SEV-06, FR-SEV-08
**Files touched**: `libs/plugin-*/`, `libs/plugin-system/` (or equivalent)

Subtasks:
- [ ] T023: Define `PluginSchemaManifest` struct: `min_schema_version`, `max_schema_version`, `adapter_type`
- [ ] T024: Update SQLite plugin to emit its manifest on startup; implement version check in core plugin loader
- [ ] T025: Update Git plugin to declare compatible version range
- [ ] T026: Implement `SchemaCompatibilityChecker` — given a plugin manifest and current schema version, returns `Compatible | Incompatible(requires_upgrade)` with actionable message

---

## Phase 7 — Rollback & Audit

### WP07: Rollback Safety, Audit Trail, and Diff Report (3 subtasks, ~200 lines)

**Goal**: Ensure migrations can be safely rolled back, all changes are audited, and schema diffs are generated on migration addition.
**Priority**: P2 | **Dependencies**: WP05
**FRs**: FR-SEV-09, FR-SEV-10
**Files touched**: `migrations/`, `agileplus/`, `docs/reference/`

Subtasks:
- [ ] T027: Implement migration diff report generator (`agileplus migrate diff <version>`) — outputs SQL diff between two versions
- [ ] T028: Enforce every migration file has a PR/issue reference comment at the top of its `up.sql`
- [ ] T029: Write `docs/reference/schema-evolution-audit.md` documenting current schema version, migration history, and event schema registry

---

## Subtask Index

| ID | Description | WP | Parallel |
|----|-------------|-----|----------|
| T001 | Create `migrations/` directory and baseline SQL | WP01 | |
| T002 | `schema_version` table definition | WP01 | |
| T003 | `Migration` Rust struct | WP01 | |
| T004 | `MigrationLoader` — file scanner | WP01 | |
| T005 | `SchemaVersionManager` | WP01 | |
| T006 | Proto version suffix audit and update | WP02 | |
| T007 | `buf.yaml` v2 configuration | WP02 | |
| T008 | `.bufbreaking.json` additive-only config | WP02 | |
| T009 | Versioned stub generation in `buf.gen.yaml` | WP02 | |
| T010 | `EventEnvelope` struct | WP03 | |
| T011 | Wrap existing events in `EventEnvelope` | WP03 | |
| T012 | `SchemaVersionExtractor` | WP03 | |
| T013 | `UpcastHandler` for schema migration | WP03 | |
| T014 | 2+ upcast functions for existing events | WP03 | |
| T015 | Schema evolution CI workflow | WP04 | |
| T016 | Migration-file naming convention check | WP04 | |
| T017 | Schema-version consistency check | WP04 | |
| T018 | Event-envelope check in `task quality` | WP04 | |
| T019 | `agileplus migrate status` | WP05 | |
| T020 | `agileplus migrate apply [--dry-run]` | WP05 | |
| T021 | `agileplus migrate apply --all` | WP05 | |
| T022 | `agileplus migrate rollback` | WP05 | |
| T023 | `PluginSchemaManifest` struct | WP06 | |
| T024 | SQLite plugin manifest emission | WP06 | |
| T025 | Git plugin manifest | WP06 | |
| T026 | `SchemaCompatibilityChecker` | WP06 | |
| T027 | Migration diff report generator | WP07 | |
| T028 | PR/issue reference enforcement in migration files | WP07 | |
| T029 | `schema-evolution-audit.md` documentation | WP07 | |

---

## Acceptance Criteria

| ID | Criterion | Verification |
|----|----------|--------------|
| AC01 | `schema_version` table exists in SQLite and is populated on first run | Manual: fresh DB, query `SELECT * FROM schema_version` |
| AC02 | `buf breaking` passes on additive-only changes; fails on field removal | Manual: add field, run `buf breaking` → pass; remove field → fail |
| AC03 | Events serialized after this spec carry `schema_version` in header | Unit: serialize event, inspect envelope |
| AC04 | `agileplus migrate status` reports current version and pending count | Manual: run command, verify output |
| AC05 | `agileplus migrate apply` executes one migration atomically | Manual: add test migration, apply, verify schema_version updated |
| AC06 | Plugin fails fast with incompatible schema version | Manual: load plugin with wrong version range, verify error |
| AC07 | `agileplus migrate rollback` reverts exactly one migration | Manual: rollback v3, verify schema_version = 2 |
| AC08 | CI workflow fails on breaking proto change | CI: push PR removing a proto field → workflow fails |

---

## Verification Checklist

- [ ] All 29 subtasks implemented and passing unit tests
- [ ] CI workflow passes on GitHub Actions (Linux runner only)
- [ ] Migration runner tested against fresh SQLite database
- [ ] `buf breaking` verified: additive changes pass, breaking changes fail
- [ ] Event envelope schema version verified by unit test
- [ ] Upcast functions tested with historical event fixtures
- [ ] Plugin version negotiation tested with compatible and incompatible manifests
- [ ] Rollback tested: apply 2 migrations, rollback 1, verify schema_version correct
- [ ] `schema-evolution-audit.md` exists and reflects current state
- [ ] No unversioned proto fields added (verified by CI gate)
- [ ] No unversioned migrations added (verified by naming convention check)
