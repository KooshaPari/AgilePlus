# Polyrepo Ecosystem Stabilization & Optimization

## Meta

- **ID**: 021-polyrepo-ecosystem-stabilization
- **Title**: Full 247-Repo Ecosystem Audit, Stabilization, and Optimization
- **Created**: 2026-04-02
- **State**: specified
- **Scope**: Shelf-level (cross-repo, all 247 repos)
- **Priority**: P0 — Critical
- **Depends On**: 012-github-portfolio-triage, 013-phenotype-infrakit-stabilization, eco-001-worktree-remediation, eco-002-branch-consolidation

## Context

The Phenotype ecosystem has exploded to **247 repositories** across the KooshaPari GitHub organization. Only **9 repos are cloned locally** consuming **89 GB** (22 GB in build artifacts alone). The ecosystem spans 15+ languages with 65 Rust, 45 Python, 30 TypeScript, and 25 Go repos.

This growth was organic and rapid (~100 repos created in March 2026 alone) without systematic governance, resulting in:
- **Fragmented state**: 7 of 9 local repos have uncommitted changes, 10 open PRs in phenotype-infrakit alone
- **Stale artifacts**: 22 GB in build artifacts, 50+ stale branches, empty worktree directories
- **Incomplete specs**: AgilePlus has 35 specs, many with only `spec.md` files (no plans, tasks, or research)
- **Missing infrastructure**: No org-level CI/CD, no package publishing, no docs federation
- **Off-main repos**: thegent, heliosApp, heliosCLI canonical repos are NOT on `main`
- **Disk crisis**: 89 GB local, dominated by heliosCLI (39 GB bazel) and AgilePlus (20 GB venv)

This spec orchestrates a **4-phase stabilization plan** targeting full ecosystem governance within one quarter, reducing from 247 to ~190 managed repos while establishing durable infrastructure.

## Problem Statement

1. **247 repos unmanageable**: No grouping strategy, no clear ownership signals, no systematic governance
2. **Local state degraded**: 89 GB disk, 7/9 repos dirty, worktrees in disarray
3. **AgilePlus incomplete**: 35 specs, ~15 with only spec.md, worklog 29 days stale
4. **No shared infrastructure**: CI/CD not distributed, no package registry, no docs federation
5. **Agent context loss**: 247 repos overwhelm agent context windows, agents struggle with breadth
6. **In-progress work orphaned**: 50+ tasks across repos at various completion states, no tracking

## Goals

- Group all 247 repos into **6 logical clusters** for manageable stabilization
- Reduce portfolio to **~190 repos** through archival, consolidation, and merging
- Reduce local disk from **89 GB → 20 GB** (77% reduction)
- Complete **all open PRs** across 9 cloned repos (10 in infrakit, 5 in thegent, etc.)
- Enrich **all incomplete AgilePlus specs** with plans, tasks, and research
- Establish **org-level CI/CD** with reusable workflows
- Set up **package publishing** for npm, PyPI, crates.io, Go modules
- Create **docs federation** hub at docs.phenotype.dev
- Return **all canonical repos to main** branch
- Establish **worktree discipline** with documented rules

## Non-Goals

- Rewriting any core application logic
- Migrating repos to different languages
- Creating new features (stabilization only)
- Deleting any repository (archive only, preserve history on git)

## Cluster Definitions

### Cluster 1: Core Platform (13 repos — HIGHEST PRIORITY)
phenotype-infrakit, AgilePlus, thegent, heliosCLI, heliosApp, phenotype-hub, cloud, cliproxyapi-plusplus, agentapi-plusplus, agent-wave, forgecode, phench, bifrost

### Cluster 2: Agent Orchestration (8 repos)
thegent (shared), thegent-plugin-host, agileplus-mcp, agileplus-agents, agentapi-plusplus (shared), agent-wave (shared), forgecode (shared), forgecode-fork

### Cluster 3: SDK & Developer Tools (16 repos)
pheno-core (TS), pheno-llm (TS), pheno-resilience (TS), phenosdk (Python), pheno-core (Python), pheno-llm (Python), pheno-mcp (Python), pheno-agents (Python), pheno-atoms (Python), phenotype-config-core (Rust), nexus (Rust), heliosHarness, phenotype-go-sdk, phenotype-types, phenotype-config-ts, phenotype-validation

### Cluster 4: Templates & Hexagonal Kits (7 repos)
templates/ (shelf), kits/ (shelf), hexagon-rs, hexagon-rust (duplicate), phenotype-governance, thegent/templates/, thegent/dotfiles/

### Cluster 5: Peripheral / Archive Candidates (23+ repos)
agentapi-deprec, tehgent, hexagon-rust, odin-*, BytePort-TestPortfolio, Byteport-TestZip, P2, Tokn, argisexec, FixitGo, FixitRs, router-docs, heliosBench, QuadSGM, Kogito, Tossy, Frostify, AppGen, TripleM, Project-Spyn, ssToCal-front, BytePortfolio, agentapi, acp

### Cluster 6: Learning / Personal (6+ repos)
koosha-portfolio, odin-* (shared with C5), KaskMan, dotfiles, vibeproxy, vibeproxy-monitoring-unified

## Technical Approach

### Phase 1: Immediate (Days 1-7) — Stop the Bleeding

| Task | Effort | Dependencies |
|------|--------|-------------|
| P1.1: Close/merge 10 open PRs in phenotype-infrakit | 4h | None |
| P1.2: Delete 8 obvious test/typo repos (agentapi-deprec, tehgent, BytePort-*, P2, Tokn, argisexec, acp) | 1h | None |
| P1.3: Clean 22 GB build artifacts locally | 2h | None |
| P1.4: Enforce .gitignore across 9 cloned repos | 2h | P1.3 |
| P1.5: Set up org-level .github repo with reusable workflows | 4h | None |
| P1.6: Audit and enrich 35 AgilePlus specs | 6h | None |
| P1.7: Establish worktree discipline — document in WORKTREES.md | 2h | None |
| P1.8: Run cargo fmt && cargo clippy on phenotype-infrakit | 2h | P1.1 |
| P1.9: Commit all dirty files across 9 repos | 1h | None |
| P1.10: Return thegent, heliosApp, heliosCLI to main | 4h | P1.1, P1.9 |

### Phase 2: Short-term (Weeks 2-3) — Consolidate and Deduplicate

| Task | Effort | Dependencies |
|------|--------|-------------|
| P2.1: Merge 15 duplicate repos into 8 targets | 12h | Phase 1 |
| P2.2: Archive 4 odin-* course repos | 1h | Phase 1 |
| P2.3: Move personal repos to separate org | 2h | Phase 1 |
| P2.4: Set up GitHub Packages for @phenotype/* | 4h | P2.1 |
| P2.5: Set up PyPI publishing for phenotype-* | 4h | P2.1 |
| P2.6: Complete phenotype-infrakit Phase 3 (performance) | 8h | P1.8 |
| P2.7: Complete AgilePlus Phase 3 (governance) | 10h | P1.6 |
| P2.8: Distribute base templates to all active repos | 4h | P2.1 |

### Phase 3: Medium-term (Weeks 4-6) — Build Auxiliary Infrastructure

| Task | Effort | Dependencies |
|------|--------|-------------|
| P3.1: Create SDK monorepo (phenotype-sdk) | 8h | P2.4, P2.5 |
| P3.2: Set up docs federation (VitePress hub) | 6h | Phase 2 |
| P3.3: Implement health check pattern across all services | 4h | Phase 2 |
| P3.4: Set up Sentry for all production services | 4h | Phase 2 |
| P3.5: Complete thegent Phase 3 (memory) | 12h | Phase 2 |
| P3.6: Complete heliosCLI Phase 2 (sandboxing) | 10h | Phase 2 |
| P3.7: Archive 11 low-signal personal projects | 2h | P2.3 |
| P3.8: Split phenotype-infrakit into 3 workspaces (optional) | 8h | P2.6 |

### Phase 4: Long-term (Weeks 7-12) — Full Ecosystem Stabilization

| Task | Effort | Dependencies |
|------|--------|-------------|
| P4.1: Complete thegent Phase 4 (cross-platform) | 12h | P3.5 |
| P4.2: Complete phenotype-infrakit Phase 4 (enterprise) | 12h | P3.8 |
| P4.3: Set up artifact storage and retention policies | 4h | Phase 3 |
| P4.4: Implement template versioning and distribution | 6h | Phase 3 |
| P4.5: Clone and onboard remaining ~200 repos | 8h | Phase 3 |
| P4.6: Full CI/CD coverage across all active repos | 10h | P4.5 |
| P4.7: Governance audit — verify compliance | 6h | P4.6 |
| P4.8: Performance benchmarks and optimization report | 8h | P4.2 |

## Success Criteria

- **Repo count**: 247 → ~190 (23% reduction)
- **Local disk**: 89 GB → 20 GB (77% reduction)
- **Build artifacts**: 22 GB → 1 GB (95% reduction)
- **Open PRs**: 15+ → 0 across all cloned repos
- **Incomplete specs**: ~15 → 0 (all specs have plan, tasks, research)
- **CI coverage**: ~30% → 100% across active repos
- **Published packages**: 0 → 20+ across npm, PyPI, crates.io, Go
- **Docs federation**: None → Live at docs.phenotype.dev
- **All canonical repos on main**: Yes
- **Worktree discipline**: Documented and enforced

## Work Packages

| ID | Description | Cluster | State |
|----|-------------|---------|-------|
| WP001 | Phase 1 execution (Days 1-7) | All | specified |
| WP002 | Phase 2 execution (Weeks 2-3) | All | specified |
| WP003 | Phase 3 execution (Weeks 4-6) | All | specified |
| WP004 | Phase 4 execution (Weeks 7-12) | All | specified |
| WP005 | Repo merge: phenotype-error-core + errors + error-macros | C1 | specified |
| WP006 | Repo merge: phenotype-contract + contracts | C1 | specified |
| WP007 | Repo merge: thegent-plugin-host → thegent | C2 | specified |
| WP008 | Repo merge: FixitGo + FixitRs → fixit | C3 | specified |
| WP009 | Repo merge: hexagon-rust → hexagon-rs | C4 | specified |
| WP010 | Repo merge: agileplus-agents + agileplus-mcp → AgilePlus | C2 | specified |
| WP011 | SDK monorepo creation (phenotype-sdk) | C3 | specified |
| WP012 | Docs federation setup (VitePress hub) | All | specified |
| WP013 | Org-level CI/CD (.github repo) | All | specified |
| WP014 | Package publishing pipeline (4 ecosystems) | All | specified |
| WP015 | Worktree remediation and discipline | All | specified |
| WP016 | Branch consolidation (50+ stale branches) | All | specified |
| WP017 | Disk cleanup and artifact management | All | specified |
| WP018 | AgilePlus spec enrichment (15 incomplete specs) | All | specified |
| WP019 | Personal repo migration to separate org | C6 | specified |
| WP020 | Archive 23 peripheral repos | C5 | specified |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Breaking changes during crate merges | Medium | High | Full test suite before each merge |
| CI/CD disruption during workflow migration | Medium | Medium | Parallel run old + new for 1 week |
| Disk cleanup breaking builds | Low | Medium | Backup before cleanup, document restore |
| Repo archival removing needed history | Low | High | Full backup before archival, GitHub archive |
| Spec completion blocking downstream work | Medium | High | Prioritize specs by dependency order |
| Agent context loss during long phases | Medium | Medium | Session documentation at each phase boundary |
| Scope creep from new features | High | Medium | Strict scope enforcement, defer to future specs |

## Traces

- Related: 012-github-portfolio-triage
- Related: 013-phenotype-infrakit-stabilization
- Related: eco-001-worktree-remediation
- Related: eco-002-branch-consolidation
- Related: 001-spec-driven-development-engine
- Related: 003-agileplus-platform-completion
- Related: 005-heliosapp-completion
- Related: 006-helioscli-completion
- Related: 007-thegent-completion
- Strategy: docs/stabilization/STRATEGY.md
