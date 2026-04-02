# Tasks: Polyrepo Ecosystem Stabilization

## Phase 1: Immediate (Days 1-7) — Stop the Bleeding

### P1.1: Close/merge 10 open PRs in phenotype-infrakit
- [ ] PR #544: Workspace stabilization — review and merge
- [ ] PR #553: Gitignore + test-infra — review and merge
- [ ] PR #554: Workspace restructuring — review and merge
- [ ] PR #557: String compression (zstd) — review and merge
- [ ] PR #558: Builder derive macro — review and merge
- [ ] PR #559: Shared config implementation — review and merge
- [ ] PR #560: ADR-015 crate org guidelines — merge (docs only)
- [ ] PR #561: Health checker with timeout — review and merge
- [ ] PR #562: Error core layered types — review and merge
- [ ] PR #563: Test infrastructure utilities — review and merge

### P1.2: Delete 8 obvious test/typo repos
- [ ] agentapi-deprec (deprecated, replaced by plusplus)
- [ ] tehgent (typo of thegent)
- [ ] BytePort-TestPortfolio (test artifact)
- [ ] Byteport-TestZip (test artifact)
- [ ] P2 (placeholder)
- [ ] Tokn (truncated name)
- [ ] argisexec (typo/abbrev)
- [ ] acp (ambiguous)

### P1.3: Clean 22 GB build artifacts locally
- [ ] `rm -rf heliosCLI/bazel-*` (~30 GB savings)
- [ ] `rm -rf */node_modules` (~5 GB savings)
- [ ] `rm -rf */.venv` (~3 GB savings)
- [ ] `cargo clean` in workspace target (~1.5 GB savings)
- [ ] Delete all `.log` files at shelf root (~200 MB)

### P1.4: Enforce .gitignore across 9 cloned repos
- [ ] phenotype-infrakit: Add target/, *.log to .gitignore
- [ ] AgilePlus: Add target/, .venv/, __pycache__/ to .gitignore
- [ ] thegent: Add node_modules/, .venv/, target/ to .gitignore
- [ ] heliosCLI: Add bazel-*, target/ to .gitignore
- [ ] heliosApp: Add node_modules/, dist/ to .gitignore
- [ ] agentapi-plusplus: Add node_modules/, dist/ to .gitignore
- [ ] cliproxyapi-plusplus: Verify .gitignore completeness
- [ ] cloud: Add .next/, node_modules/ to .gitignore
- [ ] agent-wave: Verify .gitignore completeness

### P1.5: Set up org-level .github repo with reusable workflows
- [ ] Create github.com/KooshaPari/.github repo
- [ ] Move 32 workflow files from shelf root to .github/workflows/
- [ ] Create reusable ci-rust.yml workflow
- [ ] Create reusable ci-python.yml workflow
- [ ] Create reusable ci-typescript.yml workflow
- [ ] Create reusable ci-go.yml workflow
- [ ] Create reusable security.yml workflow
- [ ] Create reusable publish.yml workflow
- [ ] Create reusable docs.yml workflow
- [ ] Create reusable release.yml workflow
- [ ] Update all active repos to reference org workflows

### P1.6: Audit and enrich 35 AgilePlus specs
- [ ] Audit all 35 specs for completeness (spec.md, plan.md, tasks.md, research.md)
- [ ] Enrich spec 005 (heliosApp) with plan, tasks, research
- [ ] Enrich spec 006 (heliosCLI) with plan, tasks, research
- [ ] Enrich spec 007 (thegent) with plan, tasks, research
- [ ] Enrich spec 012 (portfolio triage) with audit findings
- [ ] Enrich spec 013 (infrakit stabilization) with audit findings
- [ ] Create spec 021 (this spec) with full stabilization plan
- [ ] Update worklog with audit findings

### P1.7: Establish worktree discipline
- [ ] Document worktree rules in WORKTREES.md
- [ ] Clean empty worktree directories (docs/, infrastructure/, phenotype-errors/)
- [ ] Investigate cache-adapter-impl worktree (detached HEAD?)
- [ ] Merge or close phenotype-crypto-complete worktree
- [ ] Document maximum 3 concurrent worktrees per repo rule

### P1.8: Run cargo fmt && cargo clippy on phenotype-infrakit
- [ ] `cargo fmt` across workspace
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] Fix all clippy warnings
- [ ] Verify all tests pass: `cargo test --workspace`

### P1.9: Commit all dirty files across 9 repos
- [ ] phenotype-infrakit: Commit 8 dirty files (session docs, worklog, new sources)
- [ ] AgilePlus: Commit 28 dirty files (cleanup, deleted workflows)
- [ ] thegent: Commit 4 dirty files (WORKLOG.md, Cargo.toml, CODEOWNERS)
- [ ] heliosCLI: Commit 8 dirty session doc files
- [ ] heliosApp: Commit CLAUDE.md, PR_SUMMARY.md, WORKLOG.md
- [ ] agentapi-plusplus: Commit WORKLOG.md
- [ ] cliproxyapi-plusplus: Commit PLAN.md
- [ ] cloud: Commit 2 plan files

### P1.10: Return canonical repos to main
- [ ] thegent: Merge `refactor/cleanup-error-variants` → main
- [ ] heliosApp: Merge `feat/fix-typescript-vite-federation` → main
- [ ] heliosCLI: Merge `refactor/decouple-harness-crates` → main
- [ ] Verify all repos on main branch

---

## Phase 2: Short-term (Weeks 2-3) — Consolidate and Deduplicate

### P2.1: Merge 15 duplicate repos into 8 targets
- [ ] phenotype-contract + phenotype-contracts → phenotype-contracts
- [ ] phenotype-error-core + phenotype-errors + phenotype-error-macros → phenotype-error-core
- [ ] phenotype-ports-canonical + phenotype-port-traits → phenotype-contracts
- [ ] thegent-plugin-host → thegent/apps/plugin-host
- [ ] forgecode-fork → forgecode (or delete)
- [ ] hexagon-rust → hexagon-rs
- [ ] agileplus-agents → AgilePlus/packages/agents
- [ ] agileplus-mcp → AgilePlus/packages/mcp
- [ ] router-docs → phenotype-hub/docs/
- [ ] FixitGo + FixitRs → fixit (single repo)
- [ ] phenotype-config-loader → phenotype-config-core
- [ ] phenotype-shared-config → phenotype-config-core
- [ ] phenotype-async-traits → phenotype-contracts
- [ ] bifrost-routing + bifrost-routing-backup → bifrost
- [ ] vibeproxy-monitoring-unified (already archived)

### P2.2: Archive 4 odin-* course repos
- [ ] odin-dash → archive
- [ ] odin-TTT → archive
- [ ] odin-library → archive
- [ ] odin-recipes → archive

### P2.3: Move personal repos to separate org
- [ ] Create separate GitHub org or use personal account
- [ ] Move koosha-portfolio
- [ ] Move dotfiles
- [ ] Move vibeproxy (after audit)
- [ ] Remove from local shelf
- [ ] Exclude from CI/CD and AgilePlus tracking

### P2.4: Set up GitHub Packages for @phenotype/*
- [ ] Configure npm scope @phenotype/*
- [ ] Set up publishing workflow
- [ ] Publish first package
- [ ] Verify installation from GitHub Packages

### P2.5: Set up PyPI publishing for phenotype-*
- [ ] Configure PyPI project
- [ ] Set up publishing workflow
- [ ] Publish first package
- [ ] Verify installation from PyPI

### P2.6: Complete phenotype-infrakit Phase 3 (performance)
- [ ] Performance benchmarks for all crates
- [ ] Optimize hot paths
- [ ] Document performance characteristics

### P2.7: Complete AgilePlus Phase 3 (governance)
- [ ] Implement policy rules
- [ ] Set up evidence evaluation
- [ ] Complete governance enforcement

### P2.8: Distribute base templates to all active repos
- [ ] Create base AGENTS.md template
- [ ] Create base CLAUDE.md template
- [ ] Create base README.md template
- [ ] Distribute to all ~190 active repos
- [ ] Verify template adoption

---

## Phase 3: Medium-term (Weeks 4-6) — Build Auxiliary Infrastructure

### P3.1: Create SDK monorepo (phenotype-sdk)
- [ ] Create phenotype-sdk repo structure
- [ ] Move packages/pheno-* into phenotype-sdk/packages/
- [ ] Move python/pheno-* into phenotype-sdk/python/
- [ ] Set up workspace configuration
- [ ] Configure publishing for all packages

### P3.2: Set up docs federation (VitePress hub)
- [ ] Configure phenodocs as federation hub
- [ ] Add thegent/docs/ as source
- [ ] Add AgilePlus/docs/ as source
- [ ] Add heliosCLI/docs/ as source
- [ ] Add phenotype-infrakit/docs/ as source
- [ ] Deploy to docs.phenotype.dev

### P3.3: Implement health check pattern
- [ ] Define /health endpoint standard
- [ ] Implement in all services
- [ ] Set up health monitoring

### P3.4: Set up Sentry for all production services
- [ ] Configure Sentry projects
- [ ] Add Sentry SDK to all services
- [ ] Set up alerting rules

### P3.5: Complete thegent Phase 3 (memory)
- [ ] Implement memory layer
- [ ] Cross-platform integration
- [ ] Testing and documentation

### P3.6: Complete heliosCLI Phase 2 (sandboxing)
- [ ] Implement sandboxing
- [ ] Security review
- [ ] Testing and documentation

### P3.7: Archive 11 low-signal personal projects
- [ ] heliosBench → archive
- [ ] QuadSGM → archive
- [ ] Kogito → archive
- [ ] Tossy → archive
- [ ] Frostify → archive
- [ ] AppGen → archive
- [ ] TripleM → archive
- [ ] Project-Spyn → archive
- [ ] ssToCal-front → archive
- [ ] BytePortfolio → archive
- [ ] agentapi → archive

### P3.8: Split phenotype-infrakit into 3 workspaces (optional)
- [ ] core workspace (contracts, errors)
- [ ] runtime workspace (event-sourcing, cache, state-machine)
- [ ] tools workspace (policy-engine, validation)
- [ ] Update downstream consumers

---

## Phase 4: Long-term (Weeks 7-12) — Full Ecosystem Stabilization

### P4.1: Complete thegent Phase 4 (cross-platform)
- [ ] Cross-platform integration
- [ ] Testing across platforms
- [ ] Documentation

### P4.2: Complete phenotype-infrakit Phase 4 (enterprise)
- [ ] Enterprise features
- [ ] Performance optimization
- [ ] Documentation

### P4.3: Set up artifact storage and retention policies
- [ ] Configure GitHub Actions cache (30 days)
- [ ] Configure GitHub Releases (permanent)
- [ ] Configure GHCR (90 days)
- [ ] Configure S3/GitHub Pages for benchmarks

### P4.4: Implement template versioning and distribution
- [ ] Define versioning scheme (1.0 → 1.1 quarterly)
- [ ] Create template registry
- [ ] Implement scaffolding CLI
- [ ] Set up template testing CI

### P4.5: Clone and onboard remaining ~200 repos
- [ ] Systematic clone of all GitHub repos
- [ ] Add AGENTS.md, CLAUDE.md, README.md where missing
- [ ] Set up docs/sessions/ directories
- [ ] Verify git health

### P4.6: Full CI/CD coverage across all active repos
- [ ] Verify all repos reference org workflows
- [ ] Fix any CI failures
- [ ] Set up branch protection rules
- [ ] Configure required status checks

### P4.7: Governance audit — verify compliance
- [ ] Check all repos for AGENTS.md
- [ ] Check all repos for CLAUDE.md
- [ ] Check all repos for docs/sessions/
- [ ] Verify CI/CD passing
- [ ] Verify no dirty files on main
- [ ] Generate compliance report

### P4.8: Performance benchmarks and optimization report
- [ ] Run benchmarks across all crates
- [ ] Document performance characteristics
- [ ] Identify optimization opportunities
- [ ] Create optimization roadmap
