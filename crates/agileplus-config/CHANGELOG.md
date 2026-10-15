# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-config-v0.2.4...agileplus-config-v0.2.5) - 2026-10-15

### Added

- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))

### Fixed

- *(security)* replace deprecated serde_yaml with serde_norway 0.9 to clear RUSTSEC-2024-0449
- *(clippy)* commit the remaining 159-lint fix set; both compile gates now exit 0
- fix compilation errors in governance and telemetry test files
- resolve CI release build failures

### Other

- PR #1096 close operator acceptance and durable Git/SQLite promotion recovery
- Make promotion resumable across Git and SQLite; repair dependency and browser CI gates
- *(fmt)* format the workspace with CI's exact toolchain
- *(config,nats)* add comprehensive tests
- *(config)* add config_builder macro tests
- PII sweep — redact names, emails, paths across all files
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- extract config_builder! macro to agileplus-config, dedup 3 config sites ([#607](https://github.com/KooshaPari/AgilePlus/pull/607))
