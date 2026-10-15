# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-nats-v0.2.4...agileplus-nats-v0.2.5) - 2026-10-15

### Added

- absorb agileplus-* crates from PhenoDevOps
- *(dashboard)* theme system, i18n scaffolding, skeleton/empty-state components, CI improvements
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(nats)* hexagonal NATS adapter implementing AsyncEventBus port
- *(workflows)* add GitHub workflows and templates
- *(WP06)* add agileplus-nats crate with NATS event bus abstraction

### Fixed

- *(nats)* clear all 42 clippy lints in agileplus-nats (+ its only dependent)
- add missing Cargo.toml for nats, p2p, sync crates (CI hotfix)
- *(workspace)* scaffold 9 missing critical-path stub crates ([#583](https://github.com/KooshaPari/AgilePlus/pull/583))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- stabilize agileplus and sync with main
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- *(api,nats,p2p)* salvage wave-9 leftovers; add complete workspace coverage tooling
- *(agileplus-nats)* envelope strict deserialization — per-field rejection, timestamps, extra fields, malformed bytes
- *(agileplus-nats)* subject edge cases — empty segments, dot edges, chevron semantics, long subjects, round-trips
- *(config,nats)* add comprehensive tests
- *(nats)* add unit tests
- remove 30 orphan scaffold crates not in workspace
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- extract config_builder! macro to agileplus-config, dedup 3 config sites ([#607](https://github.com/KooshaPari/AgilePlus/pull/607))
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- remove deprecated config files and GitHub templates
