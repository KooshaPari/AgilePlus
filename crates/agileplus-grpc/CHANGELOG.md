# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-grpc-v0.2.4...agileplus-grpc-v0.2.5) - 2026-10-15

### Added

- *(acceptance)* wire authenticated gRPC and repair transport test compilation
- *(tests)* add transport-layer tests + fix CI coverage threshold
- *(agileplus-mcp)* ProjectScope per-session repo-root binding + repo-local artifact paths ([#1077](https://github.com/KooshaPari/AgilePlus/pull/1077))
- wtree-sprawl cleanup — core-mcp-runtime and CI workflow refinements
- *(dashboard)* theme system, i18n scaffolding, skeleton/empty-state components, CI improvements
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(FR-AGP-011)* tonic WorkItemsService + agileplus-proto stub crate ([#617](https://github.com/KooshaPari/AgilePlus/pull/617))
- *(workflows)* add GitHub workflows and templates
- modernize AgilePlus tooling (axum 0.8, reqwest 0.13, workspace deps)
- *(WP14)* gRPC server and MCP integration bridging Rust and Python
- *(WP01)* scaffold agileplus-core Rust workspace with 7 crates

### Fixed

- *(grpc)* collapse single-arm match in command post-transition logic
- *(grpc)* fail closed on terminal acceptance and shipping
- *(grpc)* satisfy clippy governance transition mapping
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- comprehensive tests, BDD fixes, and gRPC command implementations
- truthfulness pass — MCP dispatch, CI gates, desktop types, docs accuracy
- *(grpc)* report persisted work package counts ([#1069](https://github.com/KooshaPari/AgilePlus/pull/1069))
- *(core)* restore reachable gRPC and MCP runtime
- bump MSRV pin 1.85 → 1.88 (unblocks verify gate) ([#701](https://github.com/KooshaPari/AgilePlus/pull/701))
- *(grpc)* add tonic-build to build-dependencies; gate protoc on availability ([#594](https://github.com/KooshaPari/AgilePlus/pull/594))
- *(workspace)* scaffold 9 missing critical-path stub crates ([#583](https://github.com/KooshaPari/AgilePlus/pull/583))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- stabilize agileplus and sync with main
- rustfmt extra blank line and update deny.toml to v2 format
- Windows unused import, grpc test unused import, cargo-deny flag
- unused import in pact_schema test and cargo-deny config path
- commit Cargo.lock and suppress result_large_err clippy lint
- resolve CI failures for clippy, typos, ruff, and audit
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete
- resolve CI failures for buf format, taplo format, and buf lint
- add missing fields to pact_schema test after WP01 domain model extensions
- add TransitionResult, FromStr for FeatureState, fix test case assertions

### Other

- *(grpc)* use nonterminal command for invalid-transition oracle
- *(rust)* apply cargo fmt
- *(grpc)* prove stale governance and divergent replay fail closed
- *(rust)* apply cargo fmt
- *(grpc)* route validate through canonical acceptance use case
- *(grpc)* depend on canonical acceptance application use case
- *(grpc)* align generic dispatch with canonical acceptance boundary
- *(rust)* apply cargo fmt
- *(grpc)* reject transport-local terminal transitions
- *(fmt)* format the workspace with CI's exact toolchain
- *(grpc)* apply rustfmt to the relocated helper tests
- *(grpc)* exercise the core RPC handlers and the server process
- *(grpc)* cover server scope, evidence, and error mapping
- *(grpc)* add 188 tests for grpc crate
- bump workspace version to 0.2.2
- configure cargo-dist for shell+powershell installers
- *(api,grpc)* improve unit test coverage
- remove 55 orphaned source files (~6.2k LOC)
- vendor agileplus boundary types
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- remove deprecated config files and GitHub templates
