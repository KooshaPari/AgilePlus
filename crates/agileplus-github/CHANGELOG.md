# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-github-v0.2.4...agileplus-github-v0.2.5) - 2026-10-15

### Added

- wtree-sprawl cleanup — core-mcp-runtime and CI workflow refinements
- *(observability)* convert 12 unstructured tracing sites to structured fields (L13)
- *(governance)* SpecKitty rubric — catalog + parser (P2.1+P2.2) ([#893](https://github.com/KooshaPari/AgilePlus/pull/893))
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(sync-persistence)* wire GitHub sync to SQLite via PersistSyncedStories use case (FR-AGP-013) ([#610](https://github.com/KooshaPari/AgilePlus/pull/610))
- *(agileplus-github)* add sync_repository capstone service
- *(agileplus-github)* add GitHub → domain mapping module ([#596](https://github.com/KooshaPari/AgilePlus/pull/596))
- *(workflows)* add GitHub workflows and templates
- *(WP19)* Add agileplus-github crate for GitHub Issues sync

### Fixed

- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- fix github test failures (serde default for labels, merged state mapping)
- fix compilation errors in governance and telemetry test files
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- *(core)* restore reachable gRPC and MCP runtime
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- bump MSRV pin 1.85 → 1.88 (unblocks verify gate) ([#701](https://github.com/KooshaPari/AgilePlus/pull/701))
- *(workspace)* scaffold 9 missing critical-path stub crates ([#583](https://github.com/KooshaPari/AgilePlus/pull/583))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- stabilize agileplus and sync with main
- resolve Rust compilation errors across workspace ([#33](https://github.com/KooshaPari/AgilePlus/pull/33))
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- *(agileplus-github)* cover HTTP request/failure paths and sync adapter decision table
- bring forward uncommitted changes
- PII sweep — redact names, emails, paths across all files
- *(github)* fix unused mut warning in map tests
- *(github)* add unit tests
- bump workspace version to 0.2.2
- wtree-sprawl cleanup — core-mcp-runtime linear2 (MCP runtime+CI fixes, 72 commits unique to local) ([#1060](https://github.com/KooshaPari/AgilePlus/pull/1060))
- *(deps)* bump the major group with 21 updates
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- remove deprecated config files and GitHub templates
- fix duplicate workspace member, add agileplus-github, clean unused imports
