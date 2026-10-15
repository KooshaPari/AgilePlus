# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-sync-v0.2.4...agileplus-sync-v0.2.5) - 2026-10-15

### Added

- absorb agileplus-* crates from PhenoDevOps
- *(dashboard)* theme system, i18n scaffolding, skeleton/empty-state components, CI improvements
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- *(plane)* FR-AGP-018 Plane.so integration adapter ([#620](https://github.com/KooshaPari/AgilePlus/pull/620))
- *(agileplus-sync)* add SyncOrchestrator with bidirectional sync, NATS feature gate ([#291](https://github.com/KooshaPari/AgilePlus/pull/291))
- *(workflows)* add GitHub workflows and templates
- *(WP09)* add agileplus-sync crate with conflict detection, resolution, and NATS integration

### Fixed

- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- *(nats)* clear all 42 clippy lints in agileplus-nats (+ its only dependent)
- block comment detection in repl + version bump to 0.2.4
- add protobuf CI deps + fix sync tokio for tests
- resolve CI release build failures
- add missing Cargo.toml for nats, p2p, sync crates (CI hotfix)
- *(spdx)* restore Rust sources corrupted by #760 ([#768](https://github.com/KooshaPari/AgilePlus/pull/768))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- *(deps)* close 6 RUSTSEC advisories — rustls-webpki + time bumps ([#389](https://github.com/KooshaPari/AgilePlus/pull/389))
- *(quality)* AgilePlus zero warnings workspace-wide
- *(quality)* resolve agileplus-api to_yaml() + agileplus-sync async-nats dep
- stabilize agileplus and sync with main
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- bring forward uncommitted changes
- *(sync,error-core)* add comprehensive tests
- *(plane)* add unit tests for plane crate
- bump workspace version to 0.2.2
- remove 30 orphan scaffold crates not in workspace
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- *(fmt)* sweep cargo fmt drift across workspace ([#408](https://github.com/KooshaPari/AgilePlus/pull/408))
- *(AgilePlus)* remove P2P sync components
- *(agileplus-sync, agileplus-p2p)* add comprehensive cargo tests ([#247](https://github.com/KooshaPari/AgilePlus/pull/247))
- remove deprecated config files and GitHub templates
