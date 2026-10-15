# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-proto-v0.2.4...agileplus-proto-v0.2.5) - 2026-10-15

### Added

- wtree-sprawl cleanup — core-mcp-runtime and CI workflow refinements
- shared-core traceability spine + docs consolidation (Phase 3)
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(FR-AGP-011)* tonic WorkItemsService + agileplus-proto stub crate ([#617](https://github.com/KooshaPari/AgilePlus/pull/617))

### Fixed

- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- resolve CI release build failures
- *(grpc)* report persisted work package counts ([#1069](https://github.com/KooshaPari/AgilePlus/pull/1069))
- *(core)* restore reachable gRPC and MCP runtime

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- *(proto)* drop needless struct update in ProjectScope roundtrip
- *(proto,fixtures)* add comprehensive tests
- bring forward uncommitted changes
- PII sweep — redact names, emails, paths across all files
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
