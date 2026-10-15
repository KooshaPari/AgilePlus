# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-git-v0.2.4...agileplus-git-v0.2.5) - 2026-10-15

### Added

- *(tests)* coverage tests for traceability, git, CLI, sqlite
- *(agileplus-mcp)* ProjectScope per-session repo-root binding + repo-local artifact paths ([#1077](https://github.com/KooshaPari/AgilePlus/pull/1077))
- wtree-sprawl cleanup — core-mcp-runtime and CI workflow refinements
- *(governance)* SpecKitty rubric — catalog + parser (P2.1+P2.2) ([#893](https://github.com/KooshaPari/AgilePlus/pull/893))
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- *(reconciliation)* apply stash — claim.rs, git claim_bound, triage port adapter, QA gates
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(api)* bootstrap agileplus-api axum HTTP server ([#587](https://github.com/KooshaPari/AgilePlus/pull/587))
- *(workflows)* add GitHub workflows and templates
- *(WP07)* implement Git adapter with full VcsPort trait
- *(WP01)* scaffold agileplus-core Rust workspace with 7 crates

### Fixed

- *(tests)* repair 3 CI test failures -- stdin EPIPE, missing init.defaultBranch
- *(clippy)* clear all 50 workspace lints so the Rust job's clippy gate passes
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- close feedback loops — verify decomposition with real tests + E2E
- *(core)* restore reachable gRPC and MCP runtime
- *(git)* report exact merge conflict paths
- *(git)* head.shorthand() returns Result<&str, Error> not Result<Option<&str>, Error> ([#990](https://github.com/KooshaPari/AgilePlus/pull/990))
- *(git)* handle Result from head.shorthand() for git2 0.21 ([#989](https://github.com/KooshaPari/AgilePlus/pull/989))
- *(git)* remove orphaned get_feature_history code to fix compilation ([#988](https://github.com/KooshaPari/AgilePlus/pull/988))
- *(git)* add type annotations to tree.walk closure ([#987](https://github.com/KooshaPari/AgilePlus/pull/987))
- *(git)* handle Result from commit.summary() ([#986](https://github.com/KooshaPari/AgilePlus/pull/986))
- *(git)* use explicit match for commit.summary() to fix macOS type inference ([#985](https://github.com/KooshaPari/AgilePlus/pull/985))
- *(git)* use map_or for commit.summary() to fix macOS compilation ([#984](https://github.com/KooshaPari/AgilePlus/pull/984))
- *(git)* remove extra blank line for nightly rustfmt ([#983](https://github.com/KooshaPari/AgilePlus/pull/983))
- *(git)* format get_feature_history for nightly rustfmt ([#981](https://github.com/KooshaPari/AgilePlus/pull/981))
- *(git)* fix get_feature_history for git2 0.21 API ([#980](https://github.com/KooshaPari/AgilePlus/pull/980))
- *(git)* restore scan_all_features/get_feature_history and fix tests ([#979](https://github.com/KooshaPari/AgilePlus/pull/979))
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- *(dashboard)* remove manifest conflict markers ([#952](https://github.com/KooshaPari/AgilePlus/pull/952))
- *(git)* ship merge via temp worktree when target is locked ([#920](https://github.com/KooshaPari/AgilePlus/pull/920))
- *(deps)* resolve 8 Dependabot security alerts ([#869](https://github.com/KooshaPari/AgilePlus/pull/869))
- *(spdx)* restore Rust sources corrupted by #760 ([#768](https://github.com/KooshaPari/AgilePlus/pull/768))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- *(deps)* clear AgilePlus security advisories ([#418](https://github.com/KooshaPari/AgilePlus/pull/418))
- *(deps)* close 6 RUSTSEC advisories — rustls-webpki + time bumps ([#389](https://github.com/KooshaPari/AgilePlus/pull/389))
- stabilize CI for AgilePlus (lint and tests) ([#162](https://github.com/KooshaPari/AgilePlus/pull/162))
- stabilize agileplus and sync with main
- resolve clippy warnings in dashboard and git snapshot
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete
- *(WP07)* add path traversal guard and validate worktree WP IDs

### Other

- PR #1096 close operator acceptance and durable Git/SQLite promotion recovery
- *(contract)* 3-way merge spec/mature-product-contract-v2 into main (A'' step 2)
- chore(deps)(deps): bump the major group across 1 directory with 9 updates
- Make promotion resumable across Git and SQLite; repair dependency and browser CI gates
- *(git)* merge exact candidate commit by SHA
- *(git)* cover merge, branch, remote, and worktree error paths
- *(git,p2p,telemetry)* deepen coverage
- *(git,p2p,telemetry)* deepen coverage
- bring forward uncommitted changes
- PII sweep — redact names, emails, paths across all files
- *(git)* add unit and integration tests for agileplus-git
- bump workspace version to 0.2.2
- *(agileplus-git)* add comprehensive unit tests for materialize
- apply cargo fmt to all crates
- *(git)* decompose lib.rs 607→300 lines (target ≤350)
- decompose application lib.rs + git lib.rs + remove dead platform dir
- decompose oversized modules + implement CLI --json + fix desktop reads
- chore(deps)(deps): bump the major group with 4 updates ([#1075](https://github.com/KooshaPari/AgilePlus/pull/1075))
- *(deps)* bump gix 0.86.0 -> 0.87.1
- *(deps)* bump the major group with 21 updates
- *(ci)* taplo fmt all TOML + drop cpp from CodeQL matrix ([#799](https://github.com/KooshaPari/AgilePlus/pull/799))
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- update agileplus-git, agileplus-graph, agileplus-triage from working tree
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- combined CI hygiene + workspace audit (v2, resolves conflicts) ([#564](https://github.com/KooshaPari/AgilePlus/pull/564))
- remove deprecated config files and GitHub templates
- add cargo-deny configuration and fix security dependencies
