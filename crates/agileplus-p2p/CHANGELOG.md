# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-p2p-v0.2.4...agileplus-p2p-v0.2.5) - 2026-10-15

### Added

- absorb agileplus-* crates from PhenoDevOps
- *(observability)* convert 12 unstructured tracing sites to structured fields (L13)
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(p2p)* vector-clock replication + delta merge ([#599](https://github.com/KooshaPari/AgilePlus/pull/599))
- *(agileplus-p2p)* implement mDNS peer discovery ([#591](https://github.com/KooshaPari/AgilePlus/pull/591))
- apply governance standards ([#285](https://github.com/KooshaPari/AgilePlus/pull/285))
- *(workflows)* add GitHub workflows and templates
- *(WP16)* add agileplus-p2p crate with Tailscale peer discovery and vector clock sync
- *(WP17)* add git-backed state export, import, and merge conflict resolution
- *(WP16)* add agileplus-p2p crate with Tailscale peer discovery and vector clock sync

### Fixed

- *(clippy)* commit the remaining 159-lint fix set; both compile gates now exit 0
- fix github test failures (serde default for labels, merged state mapping)
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- add missing Cargo.toml for nats, p2p, sync crates (CI hotfix)
- *(spdx)* restore Rust sources corrupted by #760 ([#768](https://github.com/KooshaPari/AgilePlus/pull/768))
- *(agileplus-domain)* derive Default on BacklogSort ([#675](https://github.com/KooshaPari/AgilePlus/pull/675))
- bump MSRV pin 1.85 → 1.88 (unblocks verify gate) ([#701](https://github.com/KooshaPari/AgilePlus/pull/701))
- *(workspace)* scaffold 9 missing critical-path stub crates ([#583](https://github.com/KooshaPari/AgilePlus/pull/583))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- *(deps)* close 6 RUSTSEC advisories — rustls-webpki + time bumps ([#389](https://github.com/KooshaPari/AgilePlus/pull/389))
- stabilize agileplus and sync with main
- Windows unused import, grpc test unused import, cargo-deny flag
- resolve remaining CI failures (time crate, unused var, Windows build)
- resolve CI failures for clippy, typos, ruff, and audit
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete

### Other

- PR #1096 close operator acceptance and durable Git/SQLite promotion recovery
- Make promotion resumable across Git and SQLite; repair dependency and browser CI gates
- *(api,nats,p2p)* salvage wave-9 leftovers; add complete workspace coverage tooling
- *(agileplus-p2p)* Tailscale local API client over a real UNIX socket
- *(agileplus-p2p)* NATS error conversion and EventBatch rejection
- *(agileplus-p2p)* vector clock partial order, merge laws, serde, fetch errors
- *(git,p2p,telemetry)* deepen coverage
- *(git,p2p,telemetry)* deepen coverage
- *(p2p)* add 75 integration tests for p2p crate
- bump workspace version to 0.2.2
- remove 30 orphan scaffold crates not in workspace
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 5 medium stragglers (L5-110.3) ([#766](https://github.com/KooshaPari/AgilePlus/pull/766))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- Merge branch 'chore/fix-doc-links'
- Merge chore/L1-001-sota-2026-06-11-from-wt
- *(docs)* add CARGO-WORKSPACE.md and per-crate README public API indexes (L3 #45)
- *(governance)* add CODEOWNERS, CONTRIBUTING, SECURITY, FUNDING (L2 #30)
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- *(docs)* fix broken rich-media stub links in journey-traceability
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- *(AgilePlus)* remove P2P sync components
- *(agileplus-sync, agileplus-p2p)* add comprehensive cargo tests ([#247](https://github.com/KooshaPari/AgilePlus/pull/247))
- remove deprecated config files and GitHub templates
- fix duplicate workspace member, add agileplus-github, clean unused imports
- Merge branch '003-agileplus-platform-completion-WP18'
