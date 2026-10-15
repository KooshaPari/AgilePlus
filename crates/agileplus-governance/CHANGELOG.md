# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/KooshaPari/AgilePlus/releases/tag/agileplus-governance-v0.1.0) - 2026-10-15

### Added

- *(agileplus)* shard #956 triage + #954 governance-dashboard onto main ([#958](https://github.com/KooshaPari/AgilePlus/pull/958))
- *(governance)* rubric v2 content-probe rules + --probes CLI flag ([#902](https://github.com/KooshaPari/AgilePlus/pull/902))
- *(governance)* SpecKitty → AgilePlus migration — scoring engine + ap rubric + 14 cursor shims ([#901](https://github.com/KooshaPari/AgilePlus/pull/901))
- *(governance)* SpecKitty rubric — catalog + parser (P2.1+P2.2) ([#893](https://github.com/KooshaPari/AgilePlus/pull/893))
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- *(trace-validator+governance+ci+sota)* wave-7..24 SOTA wrap suite (P0 FR-024 closure + mise+justfile+Taskfile + governance gates)
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))

### Fixed

- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- *(clippy)* clear all 50 workspace lints so the Rust job's clippy gate passes
- *(lint)* clear all 46 ruff errors across the python tree
- resolve CI release build failures
- *(core)* restore reachable gRPC and MCP runtime
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- *(dashboard)* remove manifest conflict markers ([#952](https://github.com/KooshaPari/AgilePlus/pull/952))
- resolve committed merge-conflict markers from bad pr-769 landing ([#891](https://github.com/KooshaPari/AgilePlus/pull/891))
- bump MSRV pin 1.85 → 1.88 (unblocks verify gate) ([#701](https://github.com/KooshaPari/AgilePlus/pull/701))

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- *(governance)* cover client transport failures and env overrides
- *(governance,triage)* deepen coverage
- bring forward uncommitted changes
- PII sweep — redact names, emails, paths across all files
- *(governance)* add 150 integration tests across 10 files (2652 lines)
- *(governance)* add unit tests for governance crate
- *(governance)* comprehensive unit tests
- chore(governance-deps)(deps): bump the governance group ([#770](https://github.com/KooshaPari/AgilePlus/pull/770))
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 5 medium stragglers (L5-110.3) ([#766](https://github.com/KooshaPari/AgilePlus/pull/766))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- extract config_builder! macro to agileplus-config, dedup 3 config sites ([#607](https://github.com/KooshaPari/AgilePlus/pull/607))
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
