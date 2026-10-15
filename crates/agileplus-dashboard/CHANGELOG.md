# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-dashboard-v0.2.4...agileplus-dashboard-v0.2.5) - 2026-10-15

### Added

- *(tests)* add dashboard crate coverage tests
- wtree-sprawl cleanup — core-mcp-runtime and CI workflow refinements
- *(agileplus)* shard #956 triage + #954 governance-dashboard onto main ([#958](https://github.com/KooshaPari/AgilePlus/pull/958))
- *(dashboard)* wire seed_bridge → agileplus-api HTTP calls ([#939](https://github.com/KooshaPari/AgilePlus/pull/939))
- *(dashboard)* wire PlaneSyncDaemon into API + fix cfg.interval field rename
- *(dashboard)* instantiate GovernanceClient + PlaneClient at startup
- *(dashboard)* wire live governance + plane status endpoints
- *(dashboard)* routes/mod — register /governance/:endpoint + seed_bridge toggle
- *(dashboard)* routes/mod — register the /governance/:endpoint router
- *(dashboard)* wire agileplus-governance + agileplus-plane deps
- *(AgilePlus)* storage_port backdrop/backlog + dashboard app_state wiring
- *(dashboard-web)* TS7 + @typescript/native-preview experimental typecheck ([#924](https://github.com/KooshaPari/AgilePlus/pull/924))
- *(dashboard)* wire Plane.so settings, agent sessions, and absorbed frontend attempts ([#927](https://github.com/KooshaPari/AgilePlus/pull/927))
- *(observability)* convert 12 unstructured tracing sites to structured fields (L13)
- phase-2b audit remediation — REPL, splash screen, accessibility CI, infra CI
- phase-2 audit remediation — fuzzing targets, memory profiling, demo mode
- phase-1 audit remediation — CLI self-update, Electrobun shell, settings UI
- add fuzzing harness (L22) and onboarding tour (L71/L72)
- *(dashboard)* theme system, i18n scaffolding, skeleton/empty-state components, CI improvements
- *(desktop-electrobun)* wire AgilePlus icons into win/linux build config ([#888](https://github.com/KooshaPari/AgilePlus/pull/888))
- integrate Electrobun native window for AgilePlus dashboard ([#860](https://github.com/KooshaPari/AgilePlus/pull/860))
- shared-core traceability spine + docs consolidation (Phase 3)
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- *(observability)* wire OpenTelemetry startup
- *(desktop)* native Electrobun shell (HMR) for AgilePlus ([#633](https://github.com/KooshaPari/AgilePlus/pull/633))
- *(brand)* AI-coded SVG brand base + ico/png/jpg exports for AgilePlus ([#632](https://github.com/KooshaPari/AgilePlus/pull/632))
- *(dashboard)* unify native frontend — 4-view app with full component library wiring ([#627](https://github.com/KooshaPari/AgilePlus/pull/627))
- *(dashboard)* FR-AGP-014 live dashboard wired to SQLite API
- *(dashboard)* replace mock health with real checkers (final P3)
- *(dashboard)* add SSE v1 stream alias ([#334](https://github.com/KooshaPari/AgilePlus/pull/334))
- *(dashboard)* add health page route ([#303](https://github.com/KooshaPari/AgilePlus/pull/303))
- *(dashboard)* add SSE stream, /health, /features/:id routes and state transitions ([#335](https://github.com/KooshaPari/AgilePlus/pull/335))
- *(008)* Temporal + Hatchet deployment + NATS migration
- *(dashboard)* implement Phase 2 web dashboard and core library stabilization ([#250](https://github.com/KooshaPari/AgilePlus/pull/250))
- *(logger)* add Sentry error tracking integration
- *(workflows)* add GitHub workflows and templates
- create agileplus-fixtures shared test crate ([#235](https://github.com/KooshaPari/AgilePlus/pull/235))
- *(cli)* add list command for feature database query ([#213](https://github.com/KooshaPari/AgilePlus/pull/213))
- *(seed)* add project seeding script for all Phenotype ecosystem repos ([#212](https://github.com/KooshaPari/AgilePlus/pull/212))
- *(dashboard)* implement feature events and media endpoints
- *(evidence)* add real evidence bundle infrastructure ([#199](https://github.com/KooshaPari/AgilePlus/pull/199))
- *(dashboard)* rich UI overhaul - lightbox, settings endpoints, media gallery ([#180](https://github.com/KooshaPari/AgilePlus/pull/180))
- add /hub ecosystem services page to AgilePlus dashboard ([#184](https://github.com/KooshaPari/AgilePlus/pull/184))
- *(import)* add project batch import support ([#181](https://github.com/KooshaPari/AgilePlus/pull/181))
- *(dashboard)* real agent activity, config forms, evidence viewer, clickable timeline, rich UI ([#178](https://github.com/KooshaPari/AgilePlus/pull/178))
- modernize AgilePlus tooling (axum 0.8, reqwest 0.13, workspace deps)
- *(dashboard)* merge responsive Plane and filter updates
- multi-project dashboard, Plane.so integration, dogfood seeding ([#17](https://github.com/KooshaPari/AgilePlus/pull/17))
- wire dashboard routes and update template assets
- *(WP13)* add SSE, drag-drop, state transitions, agent activity, toasts
- *(WP12)* add agileplus-dashboard crate with Askama templates and htmx integration

### Fixed

- let optional locale catalogs typecheck
- distinguish disconnected API from empty project data
- *(clippy)* commit the remaining 159-lint fix set; both compile gates now exit 0
- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- *(dashboard)* release the state read guard before the second transition request
- *(dashboard)* accept the real services settings form
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- *(tests)* fix evidence and settings test assertions
- *(tests)* fix dashboard test compilation errors
- *(core)* restore reachable gRPC and MCP runtime
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- *(dashboard)* remove manifest conflict markers ([#952](https://github.com/KooshaPari/AgilePlus/pull/952))
- *(cli)* register cockpit reader filter ([#951](https://github.com/KooshaPari/AgilePlus/pull/951))
- *(ci)* restore AgilePlus quality gates ([#937](https://github.com/KooshaPari/AgilePlus/pull/937))
- *(dashboard)* fail closed on agent config load
- *(dashboard)* fail closed on config save failures
- *(dashboard)* load deployed credential configuration
- *(dashboard)* share secure health persistence
- *(dashboard)* preserve config on health migration failure
- *(dashboard)* secure health config writes
- *(credentials)* harden migration durability
- *(dashboard)* preserve config on migration failure
- *(dashboard)* migrate legacy Plane credential config
- *(credentials)* migrate legacy Plane secrets safely
- *(credentials)* require secure operator key lifecycle
- *(credentials)* prevent dashboard secret persistence
- *(dashboard)* add agileplus-dashboard crate to workspace members ([#898](https://github.com/KooshaPari/AgilePlus/pull/898))
- resolve committed merge-conflict markers from bad pr-769 landing ([#891](https://github.com/KooshaPari/AgilePlus/pull/891))
- *(web)* resolve tailwind v3/v4 conflict and pin real playwright version
- *(deploy)* target agileplus dashboard ([#876](https://github.com/KooshaPari/AgilePlus/pull/876))
- update postcss config for tailwindcss v4 compatibility ([#862](https://github.com/KooshaPari/AgilePlus/pull/862))
- *(spdx)* restore Rust sources corrupted by #760 ([#768](https://github.com/KooshaPari/AgilePlus/pull/768))
- *(dashboard)* add nav + 4 views with real data (6 epics / 152 stories) ([#628](https://github.com/KooshaPari/AgilePlus/pull/628))
- align domain models and restore dashboard templates ([#585](https://github.com/KooshaPari/AgilePlus/pull/585))
- restore agileplus crates + dashboard web (without node_modules) ([#578](https://github.com/KooshaPari/AgilePlus/pull/578))
- restore deleted agileplus-dashboard web frontend ([#577](https://github.com/KooshaPari/AgilePlus/pull/577))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- *(dashboard)* resolve route conflicts with agileplus-api router ([#404](https://github.com/KooshaPari/AgilePlus/pull/404))
- *(quality)* AgilePlus zero warnings workspace-wide
- *(workspace)* restore disabled crates and fix invalid Rust edition
- post-merge seed bridge, MockStorage, and dashboard compilation fixes
- dashboard service control integration + safe restart command registry + governance docs
- format agileplus dashboard hub projects block
- *(dashboard)* replace stub implementations with working service controls and settings ([#183](https://github.com/KooshaPari/AgilePlus/pull/183))
- *(dashboard)* populate event timeline links from WorkPackage data ([#182](https://github.com/KooshaPari/AgilePlus/pull/182))
- align AgilePlus dashboard expectations and template comparisons
- *(dashboard)* resolve all compiler errors in agileplus-dashboard ([#186](https://github.com/KooshaPari/AgilePlus/pull/186))
- stabilize agileplus and sync with main
- resolve Rust compilation errors across workspace ([#33](https://github.com/KooshaPari/AgilePlus/pull/33))
- add missing project_for_feature and feature_counts_for_project methods ([#22](https://github.com/KooshaPari/AgilePlus/pull/22))
- replace fake SpecKitty project with real Phenotype org repos ([#21](https://github.com/KooshaPari/AgilePlus/pull/21))
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete

### Other

- PR #1096 close operator acceptance and durable Git/SQLite promotion recovery
- detect HTML response on dashboard API paths
- Repair release CI and record passing real operator recovery journey
- Make promotion resumable across Git and SQLite; repair dependency and browser CI gates
- Connect operator dashboard to canonical acceptance and durable receipt lookup
- *(fmt)* format the workspace with CI's exact toolchain
- *(dashboard)* salvage wave-9 route-failure and seeding test files
- *(dashboard)* cover runtime-resolved routes, disk evidence, and live clients
- *(dashboard,api)* deepen coverage
- bring forward uncommitted changes
- PII sweep — redact names, emails, paths across all files
- add unit tests for cli, dashboard, plane crates + cleanup
- bump workspace version to 0.2.2
- configure cargo-dist for shell+powershell installers
- *(dashboard)* comprehensive coverage
- update lock files after dependency changes
- cargo-dist init, fix repo URL consistency, track desktop Cargo.toml
- remove 55 orphaned source files (~6.2k LOC)
- *(deps)* bump the major group with 21 updates
- resolve triple merge conflict in dashboard Cargo.toml (keep agileplus-api dep)
- auto-commit daemon 2026-08-06T03:01:34Z
- auto-commit daemon 2026-08-05T08:27:35Z
- *(api/dashboard)* break workspace cycle — api no longer depends on dashboard
- auto-commit daemon 2026-08-04T09:29:59Z
- sync Cargo.lock and crate adjustments
- audit phase 2b — EventBus pub/sub, fuzz targets, structured tracing
- audit remediation phase 2 — fuzz targets, memory profiling, demo mode, onboarding, i18n, themes
- API clean recovery and credential hardening
- *(deploy)* Vercel deploy + PR previews for AgilePlus ([#853](https://github.com/KooshaPari/AgilePlus/pull/853))
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 5 medium stragglers (L5-110.3) ([#766](https://github.com/KooshaPari/AgilePlus/pull/766))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- Merge branch 'chore/fix-doc-links'
- *(forge-commit)* 17 files from agent worktree
- *(docs)* add CARGO-WORKSPACE.md and per-crate README public API indexes (L3 #45)
- *(governance)* add CODEOWNERS, CONTRIBUTING, SECURITY, FUNDING (L2 #30)
- *(dashboard)* FR-DASHBOARD-HEALTH-001 traceability ([#691](https://github.com/KooshaPari/AgilePlus/pull/691))
- *(domain)* add FR-DOMAIN-001 traceability for Event and Snapshot ([#686](https://github.com/KooshaPari/AgilePlus/pull/686))
- *(docs)* fix broken rich-media stub links in journey-traceability
- *(fmt)* sweep cargo fmt drift across workspace ([#408](https://github.com/KooshaPari/AgilePlus/pull/408))
- update API responses, storybook, and fixtures
- update fixtures and sonar config
- remove deprecated config files and GitHub templates
- *(dashboard)* add 35 unit tests for routes helpers and process detector
- *(work-audit)* fix template-commons and tokenledger paths — final state
- add portfolio-audit-kooshapari-2026 kitty-specs
- add docs-site scaffold and verification harness
- *(work-audit)* fix heliosApp path to correct location
- Merge remote-tracking branch 'origin/fix/post-merge-seed-and-mock-storage'
- *(wip)* commit prior-agent seed_bridge refactor and import fix
- Merge pull request #206 from KooshaPari/fix/dashboard-service-control-expansion
- update ADR, process docs, and kitty-specs
- Replace Dragonfly and PostgreSQL with OrbStack containers ([#149](https://github.com/KooshaPari/AgilePlus/pull/149))
