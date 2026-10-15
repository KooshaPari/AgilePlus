# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-subcmds-v0.2.4...agileplus-subcmds-v0.2.5) - 2026-10-15

### Added

- wtree-sprawl cleanup — core-mcp-runtime and CI workflow refinements
- *(cli)* wire full-deps SDD surface and restore platform dogfood path ([#917](https://github.com/KooshaPari/AgilePlus/pull/917))
- *(intent)* integrate intent graph, builder, mcp-intent stub, and CLI command
- *(workflows)* add GitHub workflows and templates
- modernize AgilePlus tooling (axum 0.8, reqwest 0.13, workspace deps)
- wire dashboard routes and update template assets
- *(WP14)* add platform up/down/status/logs, events query, and dashboard CLI commands
- *(WP10)* add sync push/pull/auto/status/resolve CLI commands
- *(WP18)* add device discover, sync, and status CLI commands
- *(WP14)* add platform up/down/status/logs, events query, and dashboard CLI commands
- *(WP10)* add sync push/pull/auto/status/resolve CLI commands
- *(WP20)* Add agileplus-subcmds crate with 25 hidden sub-commands

### Fixed

- resolve CI release build failures
- *(core)* restore reachable gRPC and MCP runtime
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- *(dashboard)* remove manifest conflict markers ([#952](https://github.com/KooshaPari/AgilePlus/pull/952))
- *(platform)* honor resolved runtime endpoint
- *(platform)* load resolved runtime health target
- *(runtime)* enforce generated proto and resolved endpoint contract
- *(spdx)* restore Rust sources corrupted by #760 ([#768](https://github.com/KooshaPari/AgilePlus/pull/768))
- *(agileplus-subcmds)* re-add to workspace members + re-create tracera_bridge module (T-9)
- bump MSRV pin 1.85 → 1.88 (unblocks verify gate) ([#701](https://github.com/KooshaPari/AgilePlus/pull/701))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- stabilize CI for AgilePlus (lint and tests) ([#162](https://github.com/KooshaPari/AgilePlus/pull/162))
- stabilize agileplus and sync with main
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete
- wire dashboard/platform CLI subcommands, fix BDD test fields and race condition - 622 tests passing

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- *(agileplus-subcmds)* cover dashboard CLI parsing, dispatch, and launcher failures
- *(agileplus-subcmds)* add integration tests for registry, events, dashboard, audit, sync
- PII sweep — redact names, emails, paths across all files
- remove 55 orphaned source files (~6.2k LOC)
- decompose application lib.rs + git lib.rs + remove dead platform dir
- remove platform subcommand (ADR-018 item 4)
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- remove deprecated config files and GitHub templates
- fix duplicate workspace member, add agileplus-github, clean unused imports
- Merge branch '003-agileplus-platform-completion-WP19'
- Merge branch '003-agileplus-platform-completion-WP18'
