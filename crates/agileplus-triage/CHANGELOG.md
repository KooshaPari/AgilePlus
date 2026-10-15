# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-triage-v0.2.4...agileplus-triage-v0.2.5) - 2026-10-15

### Added

- wtree-sprawl cleanup — core-mcp-runtime and CI workflow refinements
- *(agileplus)* shard #956 triage + #954 governance-dashboard onto main ([#958](https://github.com/KooshaPari/AgilePlus/pull/958))
- *(dashboard)* theme system, i18n scaffolding, skeleton/empty-state components, CI improvements
- *(governance)* SpecKitty rubric — catalog + parser (P2.1+P2.2) ([#893](https://github.com/KooshaPari/AgilePlus/pull/893))
- shared-core traceability spine + docs consolidation (Phase 3)
- *(agileplus)* consolidate all branches — full spec→test→traceability e2e
- add agileplus-triage lsh+tree_sitter, agileplus-witness, phenotype-dep-guard, phenotype-mcp-sdk-rs, phenotype-sandbox crates
- add agileplus-convoy, -factory, -hook, -pipeline, -refinery crates and triage modules
- *(reconciliation)* apply stash — claim.rs, git claim_bound, triage port adapter, QA gates
- *(triage)* apply stash — claim.rs, edge_cases, hybrid_pipeline, dedup
- *(intent)* integrate intent graph, builder, mcp-intent stub, and CLI command
- *(triage)* add minhash, bloom, embeddings, hybrid_pipeline, ast_tokenize (audit recs #1-#5)
- *(triage)* add minhash, bloom, embeddings, hybrid_pipeline, ast_tokenize (audit recs #1-#5)
- *(agileplus-sqlite)* add L2 #38 migration for worklog/trace/gate/run/scope tables (renumbered to 024) + triage bloom/embeddings (l2-38 worktree)
- *(agileplus-cli)* add 5 traceability subcommands from L2 #25 worktree (gate-add, gate-run, run-record, scope-status, sidecar-status)
- *(pheno-ssot-template)* SSOT trait + struct-and-render pattern (V18 worktree)
- *(triage,cli)* add dedup/claim/repo_introspect + dag subcommands + import-dagctl migration
- final-wave libification + OTel + SOTA (9 commits, 30+ files) ([#708](https://github.com/KooshaPari/AgilePlus/pull/708))
- *(FR-AGP-017)* rule-based triage engine for synced GitHub items ([#618](https://github.com/KooshaPari/AgilePlus/pull/618))
- *(workflows)* add GitHub workflows and templates
- modernize AgilePlus tooling (axum 0.8, reqwest 0.13, workspace deps)
- *(WP17)* Add agileplus-triage crate with classifier, backlog, and router

### Fixed

- *(clippy)* commit the remaining 159-lint fix set; both compile gates now exit 0
- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- *(clippy)* clear all 50 workspace lints so the Rust job's clippy gate passes
- *(triage)* correct empirical_fp formula in bloom filter
- block comment detection in repl + version bump to 0.2.4
- resolve CI release build failures
- *(core)* restore reachable gRPC and MCP runtime
- *(nightly)* pin toolchain to nightly-2026-07-31 and normalize formatting ([#976](https://github.com/KooshaPari/AgilePlus/pull/976))
- *(dashboard)* remove manifest conflict markers ([#952](https://github.com/KooshaPari/AgilePlus/pull/952))
- *(spdx)* restore Rust sources corrupted by #760 ([#768](https://github.com/KooshaPari/AgilePlus/pull/768))
- *(agileplus)* address 5 gemini-code-assist bot review comments
- *(audit)* apply W4 SOTA findings across 12 crates
- *(triage)* use Self:: for band_hash in LshIndex::insert/query
- *(workspace)* scaffold 9 missing critical-path stub crates ([#583](https://github.com/KooshaPari/AgilePlus/pull/583))
- *(rust)* tighten deny.toml wildcards policy in AgilePlus
- stabilize agileplus and sync with main
- resolve CI failures for clippy, typos, ruff, and audit
- resolve CI failures for domain deps, MSRV, fmt, audit, and machete

### Other

- *(fmt)* format the workspace with CI's exact toolchain
- *(triage)* wave-10 engine integration coverage
- *(triage)* wave-10 ast_tokenize integration coverage
- *(triage)* wave-10 embeddings integration coverage
- *(governance,triage)* deepen coverage
- *(triage)* add 32 integration tests for triage crate
- *(triage)* add unit tests for dedup, repo_introspect, claim, ast_tokenize, lib
- bump workspace version to 0.2.2
- remove 55 orphaned source files (~6.2k LOC)
- *(triage)* satisfy clippy baseline
- audit remediation phase 2 — fuzz targets, memory profiling, demo mode, onboarding, i18n, themes
- absorb 2 protected-pattern branches (L5-110.5) ([#769](https://github.com/KooshaPari/AgilePlus/pull/769))
- absorb 24 large w35/w36 stragglers (L5-110.4) ([#767](https://github.com/KooshaPari/AgilePlus/pull/767))
- absorb 35 small stragglers (L5-110.2) ([#765](https://github.com/KooshaPari/AgilePlus/pull/765))
- *(license)* add SPDX-License-Identifier to source files (rebased on main 2026-06-18) ([#760](https://github.com/KooshaPari/AgilePlus/pull/760))
- *(workflows)* pin remaining tag-based action references to SHAs
- *(workflows)* standardize to ci/audit/deny/scorecard/release
- update agileplus-git, agileplus-graph, agileplus-triage from working tree
- Merge branch 'chore/l5-87-focus-repo-specs-2026-06-11'
- Merge branch 'chore/agileplus-reconciliation-2026-06-12'
- Merge branch 'feature/agileplus-sota-suite-2026-06-12'
- Merge branch 'chore/l5-88-focus-repo-readme-agents-2026-06-11'
- add pre-push hook, sweep-stale-worktrees.sh, branch-hygiene CI gate
- *(focus-repos)* add SPEC.md to all focus repos (L5 #87)
- *(merge)* stage residual modifications from L-branch merges
- Merge chore/L4-032-triage-libification-2026-06-11
- *(forge-commit)* 13 files from agent worktree
- *(docs)* add CARGO-WORKSPACE.md and per-crate README public API indexes (L3 #45)
- *(AgilePlus)* CI fixes — explicit workspace members, workflow bugfixes ([#582](https://github.com/KooshaPari/AgilePlus/pull/582))
- remove deprecated config files and GitHub templates
