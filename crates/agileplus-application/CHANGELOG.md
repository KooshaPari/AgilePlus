# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-application-v0.2.4...agileplus-application-v0.2.5) - 2026-10-15

### Added

- *(acceptance)* commit work state audit event and receipt atomically
- *(application)* export terminal acceptance use case
- *(application)* centralize terminal feature acceptance
- *(tests)* add coverage tests for application, specify, trace, validate
- *(governance)* SpecKitty rubric — catalog + parser (P2.1+P2.2) ([#893](https://github.com/KooshaPari/AgilePlus/pull/893))
- shared-core traceability spine + docs consolidation (Phase 3)
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- *(reconciliation)* apply stash #2 — WIP at09367ce37
- *(reconciliation)* apply stash — claim.rs, git claim_bound, triage port adapter, QA gates
- *(triage,cli)* add dedup/claim/repo_introspect + dag subcommands + import-dagctl migration
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(workspace)* use phenotype error and logging crates
- *(errors)* add DomainError + AppError with PhenotypeErrorKind lifts
- *(sync-persistence)* wire GitHub sync to SQLite via PersistSyncedStories use case (FR-AGP-013) ([#610](https://github.com/KooshaPari/AgilePlus/pull/610))
- *(application)* hexagonal application/use-case layer ([#603](https://github.com/KooshaPari/AgilePlus/pull/603))

### Fixed

- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- *(tests)* fix parallel_layers_diamond assertion
- *(tests)* fix topo_sort and parallel_layers assertions
- *(tests)* add tempfile dep + fix mut borrow in application tests
- *(core)* restore reachable gRPC and MCP runtime
- *(nightly)* format all modified files for nightly rustfmt compliance ([#1014](https://github.com/KooshaPari/AgilePlus/pull/1014))
- *(test)* case-insensitive Intent parsing, format cfg(test) indentation ([#1013](https://github.com/KooshaPari/AgilePlus/pull/1013))
- *(events)* format all modified files for nightly rustfmt ([#1010](https://github.com/KooshaPari/AgilePlus/pull/1010))
- *(events)* add missing DomainEvent variants and make SpyPublisher::emitted() sync ([#1004](https://github.com/KooshaPari/AgilePlus/pull/1004))
- *(test)* use std::sync::Mutex for SpyPublisher (publish is sync, cannot await RwLock) ([#1003](https://github.com/KooshaPari/AgilePlus/pull/1003))
- *(test)* use tokio::sync::RwLock (not std) for async test code ([#1002](https://github.com/KooshaPari/AgilePlus/pull/1002))
- *(test)* use std::sync::Mutex for SpyPublisher events ([#1000](https://github.com/KooshaPari/AgilePlus/pull/1000))
- *(test)* use std::sync::RwLock in SpyPublisher (publish is sync) ([#999](https://github.com/KooshaPari/AgilePlus/pull/999))
- *(test)* remove .await from non-async publish fn body ([#997](https://github.com/KooshaPari/AgilePlus/pull/997))
- *(domain)* remove async from SpyPublisher::publish to match non-async trait ([#991](https://github.com/KooshaPari/AgilePlus/pull/991))
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- align StoragePort trait renames and resolve all merge conflicts for clean build
- bump MSRV pin 1.85 → 1.88 (unblocks verify gate) ([#701](https://github.com/KooshaPari/AgilePlus/pull/701))

### Other

- Make promotion resumable across Git and SQLite; repair dependency and browser CI gates
- *(rust)* apply cargo fmt
- *(application)* export promotion preflight
- *(promotion)* centralize exact-candidate preflight
- *(rust)* apply cargo fmt
- *(acceptance)* state transactional limitation explicitly
- *(rust)* apply cargo fmt
- *(application)* export shared acceptance oracle
- *(acceptance)* centralize exact-candidate oracle
- *(fmt)* format the workspace with CI's exact toolchain
- *(coverage)* deepen application, api middleware, and plane resource coverage
- *(agileplus-application)* cover stale-id upsert, terminal story states, claim release scope
- *(agileplus-application)* cover unexercised use-case paths and error propagation
- bring forward uncommitted changes
- *(application)* add 76 tests for application crate (dto_serialization + error_types)
- PII sweep — redact names, emails, paths across all files
- *(application)* add unit tests for dto module
- bump workspace version to 0.2.2
- *(application)* add comprehensive test coverage
- add swarm worker test coverage for CLI, application, retrospective, and fix mojibake truncate assertions
- apply cargo fmt to all crates
- decompose application lib.rs + git lib.rs + remove dead platform dir
- restore lib.rs to pre-PR-1000 state (tokio::sync::RwLock for SpyPublisher) ([#1001](https://github.com/KooshaPari/AgilePlus/pull/1001))
- sync Cargo.lock and crate adjustments
- Remove dead phenoShared dependency from agileplus-application ([#871](https://github.com/KooshaPari/AgilePlus/pull/871))
- *(ci)* taplo fmt all TOML + drop cpp from CodeQL matrix ([#799](https://github.com/KooshaPari/AgilePlus/pull/799))
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 5 medium stragglers (L5-110.3) ([#766](https://github.com/KooshaPari/AgilePlus/pull/766))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- Merge branch 'temp-recon2' into chore/agileplus-reconciliation-2026-06-12
- Merge chore/L1-001-sota-2026-06-11-from-wt
- *(application)* cover use-cases ([#710](https://github.com/KooshaPari/AgilePlus/pull/710))
- *(docs)* add CARGO-WORKSPACE.md and per-crate README public API indexes (L3 #45)
- *(governance)* add CODEOWNERS, CONTRIBUTING, SECURITY, FUNDING (L2 #30)
- *(agileplus)* add justfile ([#692](https://github.com/KooshaPari/AgilePlus/pull/692))
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
