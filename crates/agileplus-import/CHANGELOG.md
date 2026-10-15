# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-import-v0.2.4...agileplus-import-v0.2.5) - 2026-10-15

### Added

- absorb agileplus-* crates from PhenoDevOps
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(workflows)* add GitHub workflows and templates
- *(import)* add project batch import support ([#181](https://github.com/KooshaPari/AgilePlus/pull/181))

### Fixed

- *(security)* replace deprecated serde_yaml with serde_norway 0.9 to clear RUSTSEC-2024-0449
- *(clippy)* commit the remaining 159-lint fix set; both compile gates now exit 0
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- add ALL missing Cargo.toml files (api-types, error-core, cache, contract-tests, import, integration-tests)
- *(workspace)* scaffold 9 missing critical-path stub crates ([#583](https://github.com/KooshaPari/AgilePlus/pull/583))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- stabilize agileplus and sync with main

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- *(domain,import,dispatch)* cover intent graph, builders, file credentials, WP dependencies, agent harnesses and PR loop
- *(import,graph)* deepen test coverage
- *(import,graph)* add comprehensive tests
- *(import,error-core)* add unit tests for import and error-core crates
- add unit tests for cli, dashboard, plane crates + cleanup
- *(plane)* add unit tests for plane crate
- remove 30 orphan scaffold crates not in workspace
- *(ci)* taplo fmt all TOML + drop cpp from CodeQL matrix ([#799](https://github.com/KooshaPari/AgilePlus/pull/799))
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- *(agileplus-import)* add unit tests for ImportReport ([#590](https://github.com/KooshaPari/AgilePlus/pull/590))
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- remove deprecated config files and GitHub templates
- *(wip)* commit prior-agent seed_bridge refactor and import fix
