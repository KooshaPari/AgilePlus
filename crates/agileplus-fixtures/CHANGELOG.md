# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-fixtures-v0.2.4...agileplus-fixtures-v0.2.5) - 2026-10-15

### Added

- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(008)* Temporal + Hatchet deployment + NATS migration
- *(dashboard)* implement Phase 2 web dashboard and core library stabilization ([#250](https://github.com/KooshaPari/AgilePlus/pull/250))
- *(workflows)* add GitHub workflows and templates
- create agileplus-fixtures shared test crate ([#235](https://github.com/KooshaPari/AgilePlus/pull/235))

### Fixed

- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- *(core)* restore reachable gRPC and MCP runtime
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- restore agileplus crates + dashboard web (without node_modules) ([#578](https://github.com/KooshaPari/AgilePlus/pull/578))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- *(quality)* AgilePlus zero warnings workspace-wide
- *(gitignore)* narrow *.toml → drop blanket, restore agileplus-fixtures/Cargo.toml ([#364](https://github.com/KooshaPari/AgilePlus/pull/364))

### Other

- *(proto,fixtures)* add comprehensive tests
- bump workspace version to 0.2.2
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- *(fmt)* sweep cargo fmt drift across workspace ([#408](https://github.com/KooshaPari/AgilePlus/pull/408))
- update API responses, storybook, and fixtures
- update fixtures and sonar config
- *(AgilePlus)* remove P2P sync components
- remove deprecated config files and GitHub templates
