# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-cache-v0.2.4...agileplus-cache-v0.2.5) - 2026-10-15

### Added

- absorb agileplus-* crates from PhenoDevOps
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(cache)* add health_check, InMemoryCacheStore, generic ProjectionCache ([#288](https://github.com/KooshaPari/AgilePlus/pull/288))
- *(workflows)* add GitHub workflows and templates
- *(WP04)* add agileplus-cache crate with Dragonfly/Redis support

### Fixed

- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- add ALL missing Cargo.toml files (api-types, error-core, cache, contract-tests, import, integration-tests)
- bump MSRV pin 1.85 → 1.88 (unblocks verify gate) ([#701](https://github.com/KooshaPari/AgilePlus/pull/701))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- *(quality)* AgilePlus zero warnings workspace-wide
- *(quality)* zero warnings + cache trait dyn compat (AgilePlus hardening)
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete

### Other

- chore(deps)(deps): bump the major group across 1 directory with 9 updates
- *(fmt)* format the workspace with CI's exact toolchain
- *(agileplus-cache)* cover redis-backed adapters, pool/health, and store boundaries
- *(integration-tests,cache)* add comprehensive tests
- *(cache)* add 62 unit tests for cache crate
- *(plane)* add unit tests for plane crate
- remove 30 orphan scaffold crates not in workspace
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- extract config_builder! macro to agileplus-config, dedup 3 config sites ([#607](https://github.com/KooshaPari/AgilePlus/pull/607))
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- *(fmt)* sweep cargo fmt drift across workspace ([#408](https://github.com/KooshaPari/AgilePlus/pull/408))
- remove deprecated config files and GitHub templates
