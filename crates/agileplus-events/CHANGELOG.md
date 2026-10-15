# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-events-v0.2.4...agileplus-events-v0.2.5) - 2026-10-15

### Added

- *(events)* add EventBus pub/sub + proptest for 6 domain enums (L26 + L21)
- shared-core traceability spine + docs consolidation (Phase 3)
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(events)* typed DomainEvent enum, EventEnvelope, and EventHandler port ([#600](https://github.com/KooshaPari/AgilePlus/pull/600))
- *(events)* add in-memory event stores ([#282](https://github.com/KooshaPari/AgilePlus/pull/282))
- *(workflows)* add GitHub workflows and templates
- *(WP02)* add agileplus-events crate with event sourcing engine

### Fixed

- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- *(core)* restore reachable gRPC and MCP runtime
- *(fmt)* normalize to stable rustfmt + untrack node_modules ([#1045](https://github.com/KooshaPari/AgilePlus/pull/1045))
- *(fmt)* reorder domain_event re-exports ([#1039](https://github.com/KooshaPari/AgilePlus/pull/1039))
- *(fmt)* format smoke test imports for nightly rustfmt ([#1038](https://github.com/KooshaPari/AgilePlus/pull/1038))
- *(nightly)* format all modified files for nightly rustfmt compliance ([#1014](https://github.com/KooshaPari/AgilePlus/pull/1014))
- *(test)* import DomainEvent from domain_event module (matches EventEnvelope::new signature) ([#1012](https://github.com/KooshaPari/AgilePlus/pull/1012))
- *(events)* format all files, add full exports, remove unused Arc ([#1011](https://github.com/KooshaPari/AgilePlus/pull/1011))
- *(events)* format all modified files for nightly rustfmt ([#1010](https://github.com/KooshaPari/AgilePlus/pull/1010))
- *(events)* remove unused std::sync::Arc import from bus.rs ([#1009](https://github.com/KooshaPari/AgilePlus/pull/1009))
- *(events)* format re-exports and remove dangling comments ([#1008](https://github.com/KooshaPari/AgilePlus/pull/1008))
- *(events)* add PartialEq+Eq derives to domain_event structs for DomainEvent comparison ([#1006](https://github.com/KooshaPari/AgilePlus/pull/1006))
- *(events)* rewrite DomainEvent as tuple variants wrapping domain_event structs ([#1005](https://github.com/KooshaPari/AgilePlus/pull/1005))
- *(events)* add missing DomainEvent variants and make SpyPublisher::emitted() sync ([#1004](https://github.com/KooshaPari/AgilePlus/pull/1004))
- *(events)* format domain_event re-exports across multiple lines for rustfmt ([#996](https://github.com/KooshaPari/AgilePlus/pull/996))
- *(events)* export domain_event types, add uuid, remove phenotype-error-core ([#993](https://github.com/KooshaPari/AgilePlus/pull/993))
- *(events)* export domain_event types, fix uuid dep, remove phenotype-error-core ([#992](https://github.com/KooshaPari/AgilePlus/pull/992))
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- *(w18b)* repoint phenoShared git deps to DOMAIN_ROLES owners ([#763](https://github.com/KooshaPari/AgilePlus/pull/763))
- correct Rust edition to 2021 and restore missing dependencies ([#645](https://github.com/KooshaPari/AgilePlus/pull/645))
- restore agileplus crates + dashboard web (without node_modules) ([#578](https://github.com/KooshaPari/AgilePlus/pull/578))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- stabilize CI for AgilePlus (lint and tests) ([#162](https://github.com/KooshaPari/AgilePlus/pull/162))
- resolve CI failures for clippy, typos, ruff, and audit
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- *(events)* cover bus tag arms, load error paths, and boundary behavior
- *(events,plane)* deepen coverage
- *(events)* add unit tests
- bump workspace version to 0.2.2
- add integration tests for cli, events, git, domain
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- remove deprecated config files and GitHub templates
