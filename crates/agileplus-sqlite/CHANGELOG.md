# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-sqlite-v0.2.4...agileplus-sqlite-v0.2.5) - 2026-10-15

### Added

- *(sqlite)* implement atomic acceptance transaction
- *(sqlite)* implement canonical atomic acceptance port
- *(sqlite)* commit terminal acceptance atomically
- *(acceptance)* commit work state audit event and receipt atomically
- *(evaluation)* bind terminal grades to frozen assignment criteria
- *(execution)* implement active assignment supersession in SQLite
- *(execution)* add idempotent revisions and assignment supersession
- *(mace)* wire attempt lifecycle persistence
- *(mace)* add controlled attempt lifecycle update
- *(mace)* implement execution record port for SQLite
- *(mace)* expose execution repository
- *(mace)* add SQLite execution record repository
- *(mace)* register execution record migration
- *(mace)* add immutable execution record schema
- *(tests)* coverage tests for traceability, git, CLI, sqlite
- wtree-sprawl cleanup — core-mcp-runtime and CI workflow refinements
- *(cli)* wire full-deps SDD surface and restore platform dogfood path ([#917](https://github.com/KooshaPari/AgilePlus/pull/917))
- *(dashboard)* theme system, i18n scaffolding, skeleton/empty-state components, CI improvements
- *(governance)* SpecKitty rubric — catalog + parser (P2.1+P2.2) ([#893](https://github.com/KooshaPari/AgilePlus/pull/893))
- shared-core traceability spine + docs consolidation (Phase 3)
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- *(reconciliation)* apply stash #2 — WIP at09367ce37
- *(triage)* apply stash — claim.rs, edge_cases, hybrid_pipeline, dedup
- *(intent)* add new mcp-intent crate, domain modules, and SQL migrations
- *(intent)* integrate intent graph, builder, mcp-intent stub, and CLI command
- *(agileplus-sqlite)* add L2 #38 migration for worklog/trace/gate/run/scope tables (renumbered to 024) + triage bloom/embeddings (l2-38 worktree)
- *(agileplus-cli)* add ap trace link + ap dashboard (L2 #40 worktree)
- *(trace-validator+governance+ci)* P0 closure + SOTA upgrade suite
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(dashboard)* FR-AGP-014 live dashboard wired to SQLite API
- *(FR-AGP-017)* rule-based triage engine for synced GitHub items ([#618](https://github.com/KooshaPari/AgilePlus/pull/618))
- *(sync-persistence)* wire GitHub sync to SQLite via PersistSyncedStories use case (FR-AGP-013) ([#610](https://github.com/KooshaPari/AgilePlus/pull/610))
- *(requirements)* ingest FR/NFR catalogs as Epics/Stories with Tracera cross-refs ([#609](https://github.com/KooshaPari/AgilePlus/pull/609))
- *(sqlite)* SQLite-backed repository for User/Project/Epic/Story ([#602](https://github.com/KooshaPari/AgilePlus/pull/602))
- *(api)* bootstrap agileplus-api axum HTTP server ([#587](https://github.com/KooshaPari/AgilePlus/pull/587))
- *(domain)* scaffold agileplus-domain with all aggregate types and port traits ([#581](https://github.com/KooshaPari/AgilePlus/pull/581))
- *(workflows)* add GitHub workflows and templates
- *(import)* add project batch import support ([#181](https://github.com/KooshaPari/AgilePlus/pull/181))
- modernize AgilePlus tooling (axum 0.8, reqwest 0.13, workspace deps)
- multi-project dashboard, Plane.so integration, dogfood seeding ([#17](https://github.com/KooshaPari/AgilePlus/pull/17))
- *(004)* add Plane.so Module & Cycle sync (WP06)
- *(004)* add CLI cycle subcommands (WP04)
- *(004)* implement StoragePort + SQLite adapter for Modules & Cycles (WP02)
- *(WP21)* add criterion performance benchmarks for all subsystems
- *(WP03)* add SQLite schema extensions for event sourcing
- *(WP01)* add domain model extensions for platform completion
- *(WP06)* implement SQLite adapter with full StoragePort trait
- *(WP01)* scaffold agileplus-core Rust workspace with 7 crates

### Fixed

- *(sqlite)* update stale migration_tracking_records_names test to expect 031
- *(clippy)* commit the remaining 159-lint fix set; both compile gates now exit 0
- *(test)* import sqlite params for rollback witness
- *(evaluation)* encode criterion mandatory flag explicitly
- *(test)* import evaluation result from execution domain
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- revert unintended sqlite changes from governance commit
- close feedback loops — verify decomposition with real tests + E2E
- *(core)* restore reachable gRPC and MCP runtime
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- *(dashboard)* remove manifest conflict markers ([#952](https://github.com/KooshaPari/AgilePlus/pull/952))
- align StoragePort trait renames and resolve all merge conflicts for clean build
- *(cli)* unblock workspace build — rebase onto post-#893 merged main ([#897](https://github.com/KooshaPari/AgilePlus/pull/897))
- *(sqlite)* register migrations 024-026 + harden parse_up + fix upsert_requirement_id ([#896](https://github.com/KooshaPari/AgilePlus/pull/896))
- *(spdx)* restore Rust sources corrupted by #760 ([#768](https://github.com/KooshaPari/AgilePlus/pull/768))
- bump MSRV pin 1.85 → 1.88 (unblocks verify gate) ([#701](https://github.com/KooshaPari/AgilePlus/pull/701))
- *(sqlite)* remove duplicate anyhow dep key in Cargo.toml
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- implement evidence-backed validate policies ([#424](https://github.com/KooshaPari/AgilePlus/pull/424))
- stabilize agileplus and sync with main
- resolve compilation errors and apply formatting standards ([#154](https://github.com/KooshaPari/AgilePlus/pull/154))
- resolve clippy warnings in dashboard and git snapshot
- resolve Rust compilation errors across workspace ([#33](https://github.com/KooshaPari/AgilePlus/pull/33))
- resolve CI failures for clippy, typos, ruff, and audit
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete

### Other

- PR #1096 close operator acceptance and durable Git/SQLite promotion recovery
- *(contract)* 3-way merge spec/mature-product-contract-v2 into main (A'' step 2)
- Make promotion resumable across Git and SQLite; repair dependency and browser CI gates
- Connect operator dashboard to canonical acceptance and durable receipt lookup
- *(rust)* apply cargo fmt
- *(sqlite)* prove late acceptance write failure rolls back
- *(sqlite)* restore canonical atomic acceptance witnesses
- *(rust)* apply cargo fmt
- *(sqlite)* rustfmt canonical acceptance imports
- *(sqlite)* remove duplicate atomic acceptance impl
- *(sqlite)* keep stronger canonical atomic acceptance module
- *(sqlite)* align atomic acceptance with rustfmt
- *(sqlite)* prove atomic acceptance commit replay and rollback
- *(sqlite)* remove superseded weaker acceptance transaction
- *(rust)* apply cargo fmt
- *(rust)* apply cargo fmt
- *(evaluation)* enforce receipt-only satisfied grades
- *(rust)* apply cargo fmt
- *(execution)* cover idempotent revisions and atomic supersession
- *(execution)* register active-assignment invariant migration
- *(execution)* enforce one active assignment per WP
- *(execution)* preserve replacement attempt history and candidate binding
- *(migrations)* account for execution-record migration
- *(rust)* apply cargo fmt
- *(fmt)* format the workspace with CI's exact toolchain
- *(sqlite)* salvage wave-9 repository unit tests
- *(agileplus-sqlite)* cover migration, rebuild, and row-mapping error paths
- *(sqlite)* deepen coverage
- *(sqlite)* fix crud_features test
- *(sqlite)* add 30 tests for sqlite crate (schema, CRUD)
- bump workspace version to 0.2.2
- configure cargo-dist for shell+powershell installers
- *(sqlite)* comprehensive repository unit tests for 8 modules
- add integration tests for sqlite and domain
- *(sqlite)* comprehensive repository tests for audit, backlog, evidence, governance
- *(governance)* comprehensive unit tests
- *(sqlite)* add 12 comprehensive epic repository tests
- add 12 features repository tests covering CRUD, state, labels, and filtering
- add 29 work_packages CRUD tests covering all public API functions
- apply cargo fmt to all crates
- decompose oversized modules + implement CLI --json + fix desktop reads
- *(deps)* bump the major group with 21 updates
- update Cargo.lock and minor crate adjustments
- audit remediation phase 2 — fuzz targets, memory profiling, demo mode, onboarding, i18n, themes
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 5 medium stragglers (L5-110.3) ([#766](https://github.com/KooshaPari/AgilePlus/pull/766))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- Merge branch 'chore/l5-87-focus-repo-specs-2026-06-11'
- Merge branch 'chore/agileplus-reconciliation-2026-06-12'
- Merge branch 'feature/agileplus-sota-wraps-2026-06-12'
- Merge chore/L4-031-triage-port-adapter-2026-06-11
- *(forge-commit)* 21 files from agent worktree
- *(docs)* add CARGO-WORKSPACE.md and per-crate README public API indexes (L3 #45)
- *(governance)* add CODEOWNERS, CONTRIBUTING, SECURITY, FUNDING (L2 #30)
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- extend seed-requirements to 6 catalogs (PhenoMCP + PhenoObservability) ([#613](https://github.com/KooshaPari/AgilePlus/pull/613))
- remove deprecated config files and GitHub templates
- fix duplicate workspace member, add agileplus-github, clean unused imports
