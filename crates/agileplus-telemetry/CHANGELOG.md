# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-telemetry-v0.2.4...agileplus-telemetry-v0.2.5) - 2026-10-15

### Added

- wtree-sprawl cleanup — core-mcp-runtime and CI workflow refinements
- *(dashboard)* theme system, i18n scaffolding, skeleton/empty-state components, CI improvements
- *(governance)* SpecKitty rubric — catalog + parser (P2.1+P2.2) ([#893](https://github.com/KooshaPari/AgilePlus/pull/893))
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(FR-AGP-015)* OpenTelemetry tracing — init_subscriber, OTLP adapter, request-span middleware ([#615](https://github.com/KooshaPari/AgilePlus/pull/615))
- *(telemetry)* add init_telemetry, TelemetryGuard, AgilePlusMetrics, trace_layer ([#333](https://github.com/KooshaPari/AgilePlus/pull/333))
- *(workflows)* add GitHub workflows and templates
- modernize AgilePlus tooling (axum 0.8, reqwest 0.13, workspace deps)
- *(WP15)* add OpenTelemetry OTLP traces, custom metrics, and structured JSON logs
- *(WP10)* telemetry adapter with OpenTelemetry integration
- *(WP01)* scaffold agileplus-core Rust workspace with 7 crates

### Fixed

- *(fmt)* wrap serde_norway call sites that overflowed the 100-char line limit
- *(security)* replace deprecated serde_yaml with serde_norway 0.9 to clear RUSTSEC-2024-0449
- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- *(clippy)* clear all 50 workspace lints so the Rust job's clippy gate passes
- *(telemetry)* fix 4 failing adapter tests (noop parent assertions, OTLP tokio runtime)
- fix compilation errors in governance and telemetry test files
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- *(core)* restore reachable gRPC and MCP runtime
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- bump MSRV pin 1.85 → 1.88 (unblocks verify gate) ([#701](https://github.com/KooshaPari/AgilePlus/pull/701))
- *(workspace)* scaffold 9 missing critical-path stub crates ([#583](https://github.com/KooshaPari/AgilePlus/pull/583))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- *(tests)* repair P1 doctest + wiremock failures so HOOKS_SKIP=1 isn't needed ([#407](https://github.com/KooshaPari/AgilePlus/pull/407))
- *(quality)* resolve agileplus-telemetry lifetimes + agileplus-plane module paths
- stabilize agileplus and sync with main
- resolve Rust compilation errors across workspace ([#33](https://github.com/KooshaPari/AgilePlus/pull/33))
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete

### Other

- *(agileplus-telemetry)* cover telemetry_layer fallback and all log output tags
- *(agileplus-p2p)* NATS error conversion and EventBatch rejection
- *(git,p2p,telemetry)* deepen coverage
- *(git,p2p,telemetry)* deepen coverage
- bump workspace version to 0.2.2
- chore(deps)(deps): bump the major group across 1 directory with 2 updates ([#1080](https://github.com/KooshaPari/AgilePlus/pull/1080))
- *(deps)* bump the major group with 21 updates
- audit remediation phase 2 — fuzz targets, memory profiling, demo mode, onboarding, i18n, themes
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- *(fmt)* sweep cargo fmt drift across workspace ([#408](https://github.com/KooshaPari/AgilePlus/pull/408))
- remove deprecated config files and GitHub templates
- fix duplicate workspace member, add agileplus-github, clean unused imports
