# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-api-v0.2.4...agileplus-api-v0.2.5) - 2026-10-15

### Added

- *(api)* add explicit 422 acceptance precondition error
- *(acceptance)* wire authenticated gRPC and repair transport test compilation
- *(acceptance)* commit work state audit event and receipt atomically
- wtree-sprawl cleanup — core-mcp-runtime and CI workflow refinements
- *(AgilePlus)* storage_port backdrop/backlog + dashboard app_state wiring
- *(AgilePlus)* graph layer hardening + intent-graph matrix projection
- *(cli)* wire full-deps SDD surface and restore platform dogfood path ([#917](https://github.com/KooshaPari/AgilePlus/pull/917))
- *(observability)* convert 12 unstructured tracing sites to structured fields (L13)
- shared-core traceability spine + docs consolidation (Phase 3)
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(FR-AGP-015)* OpenTelemetry tracing — init_subscriber, OTLP adapter, request-span middleware ([#615](https://github.com/KooshaPari/AgilePlus/pull/615))
- *(api-auth)* implement FR-AGP-012 bearer-token / API-key auth middleware ([#612](https://github.com/KooshaPari/AgilePlus/pull/612))
- *(api)* wire application use-cases to AppState composition root ([#606](https://github.com/KooshaPari/AgilePlus/pull/606))
- *(api)* bootstrap agileplus-api axum HTTP server ([#587](https://github.com/KooshaPari/AgilePlus/pull/587))
- *(AgilePlus)* add BacklogItemResponse and enhance CI
- *(dashboard)* implement Phase 2 web dashboard and core library stabilization ([#250](https://github.com/KooshaPari/AgilePlus/pull/250))
- *(workflows)* add GitHub workflows and templates
- create agileplus-fixtures shared test crate ([#235](https://github.com/KooshaPari/AgilePlus/pull/235))
- *(seed)* add project seeding script for all Phenotype ecosystem repos ([#212](https://github.com/KooshaPari/AgilePlus/pull/212))
- *(import)* add project batch import support ([#181](https://github.com/KooshaPari/AgilePlus/pull/181))
- modernize AgilePlus tooling (axum 0.8, reqwest 0.13, workspace deps)
- multi-project dashboard, Plane.so integration, dogfood seeding ([#17](https://github.com/KooshaPari/AgilePlus/pull/17))
- *(004)* merge WP06 - Plane.so Module & Cycle sync
- *(004)* add Plane.so Module & Cycle sync (WP06)
- *(004)* add dashboard Module & Cycle views (WP05)
- *(004)* implement StoragePort + SQLite adapter for Modules & Cycles (WP02)
- *(WP15)* add OpenTelemetry OTLP traces, custom metrics, and structured JSON logs
- *(WP11)* add REST API, API key auth, SSE streaming, and health endpoint
- *(WP15)* API layer and credential management
- *(WP01)* scaffold agileplus-core Rust workspace with 7 crates

### Fixed

- *(security)* replace deprecated serde_yaml with serde_norway 0.9 to clear RUSTSEC-2024-0449
- *(lint)* unblock CI clippy with manual_contains, dead_code, useless_conversion, and rustfmt
- *(api)* align acceptance 422, governance vacuous-success, and update stale tests
- *(governance)* per-rule satisfaction and by-type-label evidence fallback
- *(clippy)* allow dead code on test-only helpers and reserved functions
- *(release)* enforce operator origin boundary and expose acceptance rejection
- *(api)* use stable character predicate in CORS validation
- *(api)* replace wildcard CORS with explicit browser-origin allowlist
- *(test)* use axum-test TestServer constructor correctly
- *(api)* map acceptance validation to precondition response
- *(api)* prevent terminal-state feature creation
- *(api)* block generic terminal feature transitions
- *(api)* evaluate governance through storage value
- *(acceptance)* block review-only terminal completion
- *(governance)* reject vacuous zero-rule compliance
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- *(core)* restore reachable gRPC and MCP runtime
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- *(dashboard)* remove manifest conflict markers ([#952](https://github.com/KooshaPari/AgilePlus/pull/952))
- align StoragePort trait renames and resolve all merge conflicts for clean build
- *(api)* populate worktree commit field
- *(api)* mount and complete production readiness routes
- *(api)* type credential store and map import errors
- *(api)* authenticate protected routes from credential store
- *(dashboard)* preserve config on migration failure
- *(credentials)* require secure operator key lifecycle
- *(credentials)* prevent dashboard secret persistence
- *(credentials)* encrypt persisted secrets
- *(api)* align OpenAPI info version with 0.2.1 release ([#921](https://github.com/KooshaPari/AgilePlus/pull/921))
- bump MSRV pin 1.85 → 1.88 (unblocks verify gate) ([#701](https://github.com/KooshaPari/AgilePlus/pull/701))
- *(agileplus-domain)* remove duplicate domain.rs (E0761) ([#589](https://github.com/KooshaPari/AgilePlus/pull/589))
- *(workspace)* scaffold 9 missing critical-path stub crates ([#583](https://github.com/KooshaPari/AgilePlus/pull/583))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- *(tests)* repair P1 doctest + wiremock failures so HOOKS_SKIP=1 isn't needed ([#407](https://github.com/KooshaPari/AgilePlus/pull/407))
- *(quality)* AgilePlus zero warnings workspace-wide
- *(quality)* resolve agileplus-api to_yaml() + agileplus-sync async-nats dep
- *(gitignore)* narrow *.toml → drop blanket, restore agileplus-fixtures/Cargo.toml ([#364](https://github.com/KooshaPari/AgilePlus/pull/364))
- post-merge seed bridge, MockStorage, and dashboard compilation fixes
- implement missing StoragePort methods in test MockStorage impls ([#204](https://github.com/KooshaPari/AgilePlus/pull/204))
- stabilize CI for AgilePlus (lint and tests) ([#162](https://github.com/KooshaPari/AgilePlus/pull/162))
- stabilize agileplus and sync with main
- resolve compilation errors and apply formatting standards ([#154](https://github.com/KooshaPari/AgilePlus/pull/154))
- resolve clippy warnings in dashboard and git snapshot
- resolve Rust compilation errors across workspace ([#33](https://github.com/KooshaPari/AgilePlus/pull/33))
- resolve remaining CI failures (time crate, unused var, Windows build)
- resolve CI failures for clippy, typos, ruff, and audit
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete
- update API mock storage for module/cycle StoragePort methods

### Other

- PR #1096 close operator acceptance and durable Git/SQLite promotion recovery
- PR #1095 close private operator origin boundary and repair acceptance diagnostics
- *(spec)* cargo fmt pass on spec-side files (A'' step 4)
- Connect operator dashboard to canonical acceptance and durable receipt lookup
- *(rust)* apply cargo fmt
- *(api)* prove failed acceptance is 422 and non-mutating
- *(api)* prove HTTP acceptance through real SQLite transaction
- *(rust)* apply cargo fmt
- *(rust)* apply cargo fmt
- *(rust)* apply cargo fmt
- *(api)* reject direct terminal WP transition
- *(rust)* apply cargo fmt
- *(governance)* route HTTP validation through shared evaluator
- *(governance)* use shared result semantics
- *(fmt)* format the workspace with CI's exact toolchain
- *(api,plane)* close uncovered evidence-lookup, shipping-gate, and update branches
- *(coverage)* deepen application, api middleware, and plane resource coverage
- *(api,nats,p2p)* salvage wave-9 leftovers; add complete workspace coverage tooling
- *(api)* salvage worker's VCS contract and binary lifecycle coverage
- *(agileplus-api)* cover storage failures, service probes, and binary startup
- *(agileplus-api)* cover unexercised routes, error paths, and rejection shapes
- *(api)* cover auth middleware and SSE streaming over TCP
- *(dashboard,api)* deepen coverage
- *(api)* add unit tests to agileplus-api crate
- bump workspace version to 0.2.2
- configure cargo-dist for shell+powershell installers
- *(api,grpc)* improve unit test coverage
- remove 55 orphaned source files (~6.2k LOC)
- *(deps)* bump the major group with 21 updates
- *(api/dashboard)* break workspace cycle — api no longer depends on dashboard
- *(test)* migrate VcsPort trait impl to async_trait macro
- gitignore SQLite WAL files, update storage test
- *(api)* update integration test assertions
- audit phase 2b — EventBus pub/sub, fuzz targets, structured tracing
- *(credentials)* prove router rotation and configured file backend
- *(api)* prove credential-store key rotation
- *(ci)* taplo fmt all TOML + drop cpp from CodeQL matrix ([#799](https://github.com/KooshaPari/AgilePlus/pull/799))
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 5 medium stragglers (L5-110.3) ([#766](https://github.com/KooshaPari/AgilePlus/pull/766))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- Merge branch 'chore/fix-doc-links'
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- *(docs)* fix broken rich-media stub links in journey-traceability
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- *(deps)* remove dead utoipa-axum dep (zero source uses; clears RUSTSEC-2024-0436 paste transitive)
- *(fmt)* sweep cargo fmt drift across workspace ([#408](https://github.com/KooshaPari/AgilePlus/pull/408))
- update API responses, storybook, and fixtures
- *(AgilePlus)* remove P2P sync components
- remove deprecated config files and GitHub templates
- Merge remote-tracking branch 'origin/fix/post-merge-seed-and-mock-storage'
- fix duplicate workspace member, add agileplus-github, clean unused imports
