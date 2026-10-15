# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-graph-v0.2.4...agileplus-graph-v0.2.5) - 2026-10-15

### Added

- *(AgilePlus)* graph layer hardening + intent-graph matrix projection
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- add agileplus-convoy, -factory, -hook, -pipeline, -refinery crates and triage modules
- *(agileplus-graph)* stabilize UUID graph store API ([#326](https://github.com/KooshaPari/AgilePlus/pull/326))
- *(workflows)* add GitHub workflows and templates
- *(WP05)* add agileplus-graph crate with Neo4j graph layer

### Fixed

- *(ci)* unblock rust linker, ruff py311 target, cargo-machete unused deps
- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- *(clippy)* clear all 50 workspace lints so the Rust job's clippy gate passes
- resolve CI release build failures
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- stabilize CI for AgilePlus (lint and tests) ([#162](https://github.com/KooshaPari/AgilePlus/pull/162))
- resolve CI failures for clippy, typos, ruff, and audit
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete

### Other

- PR #1096 close operator acceptance and durable Git/SQLite promotion recovery
- Make promotion resumable across Git and SQLite; repair dependency and browser CI gates
- *(import,graph)* deepen test coverage
- *(import,graph)* add comprehensive tests
- *(graph)* add unit tests for graph crate
- remove 55 orphaned source files (~6.2k LOC)
- *(deps)* bump the major group with 21 updates
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 5 medium stragglers (L5-110.3) ([#766](https://github.com/KooshaPari/AgilePlus/pull/766))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- update agileplus-git, agileplus-graph, agileplus-triage from working tree
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- *(agileplus-graph)* migrate to UUID-based graph API ([#336](https://github.com/KooshaPari/AgilePlus/pull/336))
- remove deprecated config files and GitHub templates
