# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-plane-v0.2.4...agileplus-plane-v0.2.5) - 2026-10-15

### Added

- *(tests)* add SQLite + plane crate coverage tests
- *(tests)* add plane crate extended tests
- *(agileplus)* shard #956 triage + #954 governance-dashboard onto main ([#958](https://github.com/KooshaPari/AgilePlus/pull/958))
- *(dashboard)* wire PlaneSyncDaemon into API + fix cfg.interval field rename
- *(plane)* add PlaneSyncDaemon — polling background loop
- *(observability)* convert 12 unstructured tracing sites to structured fields (L13)
- *(dashboard)* theme system, i18n scaffolding, skeleton/empty-state components, CI improvements
- shared-core traceability spine + docs consolidation (Phase 3)
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(plane)* FR-AGP-018 Plane.so integration adapter ([#620](https://github.com/KooshaPari/AgilePlus/pull/620))
- *(agileplus-plane)* implement required methods and test infrastructure ([#295](https://github.com/KooshaPari/AgilePlus/pull/295))
- *(workflows)* add GitHub workflows and templates
- modernize AgilePlus tooling (axum 0.8, reqwest 0.13, workspace deps)
- *(004)* add Plane.so Module & Cycle sync (WP06)
- *(WP08)* implement Plane.so bidirectional sync with state mapping, webhooks, and retry queue
- *(WP18)* Add agileplus-plane crate for Plane.so sync

### Fixed

- *(clippy)* commit the remaining 159-lint fix set; both compile gates now exit 0
- *(clippy)* clear all 50 workspace lints so the Rust job's clippy gate passes
- *(plane)* de-flake token bucket capacity test
- *(plane)* serialize daemon environment config tests
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- *(core)* restore reachable gRPC and MCP runtime
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- *(dashboard)* remove manifest conflict markers ([#952](https://github.com/KooshaPari/AgilePlus/pull/952))
- *(spdx)* restore Rust sources corrupted by #760 ([#768](https://github.com/KooshaPari/AgilePlus/pull/768))
- bump MSRV pin 1.85 → 1.88 (unblocks verify gate) ([#701](https://github.com/KooshaPari/AgilePlus/pull/701))
- correct Rust edition to 2021 and restore missing dependencies ([#645](https://github.com/KooshaPari/AgilePlus/pull/645))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- *(tests)* repair P1 doctest + wiremock failures so HOOKS_SKIP=1 isn't needed ([#407](https://github.com/KooshaPari/AgilePlus/pull/407))
- *(quality)* AgilePlus zero warnings workspace-wide
- *(quality)* resolve agileplus-telemetry lifetimes + agileplus-plane module paths
- stabilize CI for AgilePlus (lint and tests) ([#162](https://github.com/KooshaPari/AgilePlus/pull/162))
- stabilize agileplus and sync with main
- resolve Rust compilation errors across workspace ([#33](https://github.com/KooshaPari/AgilePlus/pull/33))
- resolve CI failures for clippy, typos, ruff, and audit
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- *(api,plane)* close uncovered evidence-lookup, shipping-gate, and update branches
- *(coverage)* deepen application, api middleware, and plane resource coverage
- *(plane)* cover untouched outbound push and inbound delete branches
- *(plane)* cover daemon loop, sync adapter, runtime env paths, queue store
- *(events,plane)* deepen coverage
- *(agileplus-plane)* add integration tests for all public modules
- add unit tests for cli, dashboard, plane crates + cleanup
- *(plane)* add unit tests for plane crate
- bump workspace version to 0.2.2
- *(deps)* bump the major group with 21 updates
- auto-commit daemon 2026-08-04T22:45:53Z
- auto-commit daemon 2026-08-04T09:29:59Z
- audit phase 2b — EventBus pub/sub, fuzz targets, structured tracing
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- *(fmt)* sweep cargo fmt drift across workspace ([#408](https://github.com/KooshaPari/AgilePlus/pull/408))
- remove deprecated config files and GitHub templates
- fix duplicate workspace member, add agileplus-github, clean unused imports
