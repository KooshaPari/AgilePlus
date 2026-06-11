# AgilePlus — AI-Native Project Management Platform

## Overview

AgilePlus is an AI-native, spec-driven, local-first project-management platform. The primary
implementation is a Rust Cargo workspace (~20 active crates, hexagonal architecture) with a
web dashboard (Askama + React/TS) and an Electrobun desktop app. It is one of the org's three
PM frontend candidates (with Tracera and Planify).

## Architecture

Hexagonal (ports-and-adapters): `agileplus-domain` and `agileplus-application` have no framework
deps; everything else is an adapter. Key crates:

- **Core**: `agileplus-domain`, `agileplus-application`
- **Interfaces**: `agileplus-cli`/`-subcmds`, `agileplus-api`, `agileplus-grpc`/`-proto`, `agileplus-dashboard` (web + desktop)
- **Persistence/transport**: `agileplus-sqlite`, `agileplus-events`/`-nats`, `agileplus-sync`, `agileplus-p2p`
- **Integrations**: `agileplus-github`, `agileplus-plane`, `agileplus-import`, `agileplus-git`
- **Cross-cutting**: `agileplus-governance`, `agileplus-config`, `agileplus-cache`, `agileplus-telemetry`, `agileplus-triage`, `agileplus-graph`
- **Tests/bench**: `agileplus-contract-tests`, `agileplus-integration-tests`, `agileplus-benchmarks`, `agileplus-fixtures`
- **Python**: `python/phenotype_traceability/` package; `agileplus-mcp/` is a separate FastMCP server.

Build: `cargo build --workspace`. CLI: `cargo install --path crates/agileplus-cli`. Dashboard: `cd crates/agileplus-dashboard/web && bun run dev`.

## Branch Discipline

- `main` is protected. All changes via PR.
- Branch naming: `feat/`, `fix/`, `chore/`, `ci/`, `docs/` prefixes.
- Keep PRs small and focused.
- Feature work in worktrees: `AgilePlus-wtrees/<topic>/`

## Encoding

All files must be UTF-8. No BOM.

## Bootstrap Status

- ✅ `.github/workflows/trufflehog.yml` — secrets scanning (pinned to SHA)
- ✅ `FUNDING.yml` — GitHub Sponsors
- ✅ `SECURITY.md` — vulnerability reporting
- ✅ `.github/dependabot.yml` — automated dependency updates
- ✅ `deny.toml` — cargo-deny advisories config
- ✅ `gitleaks.toml` — gitleaks config
- ✅ `rust-toolchain.toml` — nightly channel (MSRV enforcement added to CI)
- ✅ Branch protection — configured on KooshaPari/AgilePlus
- ❌ SBOM — not yet generated

## CI/CD Security (2026-05-05 Audit)

All GitHub Actions are now pinned to full commit SHAs:
- `returntocorp/semgrep-action@713efdd345f3035192eaa63f56867b88e63e4e5d` (v1) in `sast-quick.yml`
- `pre-commit/action@2c7b3805fd2a0fd8c1884dcaebf91fc102a13ecd` (v3.0.1) in `security-guard.yml`
- `trufflesecurity/trufflehog@3fc0c2aa6648d54242e4af6fbfde0701796e4fb0` (already pinned) in `sast-quick.yml`
- `actions/checkout@v6` — consider pinning to SHA in future pass

## Parallel Work Policy

Use many background agents for broad audits and repo sweeps when work can be split safely.

Why: this workspace is a multi-repo lab with many incomplete projects, and the fastest way to extend the DAG is to have workers scout, rank, and patch different repos in parallel instead of serially waiting on one lane.

How to apply: prefer concurrent agents for discovery/ranking across repos, while the parent agent keeps synthesis, concrete edits, and task tracking coordinated.

## Repo Audit Findings (2026-05-05)

### Workspace Structure
- `agileplus/` and `AgilePlus/` are the **same directory** (case-insensitive macOS filesystem, same inode). The canonical remote is `KooshaPari/AgilePlus`.
- The bare-repo pattern means direct commits to `main` are blocked; all changes flow through PRs.
- Worktrees: `AgilePlus-wtrees/<topic>/` (note: `AgilePlus-wtr/` exists with inconsistent naming — should be consolidated).

### Python Projects Across Phenotype Repos (~15 identified)
- Active Python repos: `pheno/`, `phenoAI/`, `PhenoAgent/`, `phenoData/`, `phenoDesign/`, `phenoUtils/`, `phenoXdd/`, `phenoShared/`, `cheap-llm-mcp/`, `agileplus-mcp/`, `dispatch-mcp/`, `phenotype-shared/`, `agentops-policy-federation/`
- Gaps: several missing `pytest` in CI; some missing `uv.lock` (see tasks #228, #229)

### Rust Workspaces Across Phenotype Repos (~17+ identified)
- Key multi-crate workspaces: `FocalPoint/`, `PhenoMCP/`, `PhenoProject/`, `PolicyStack/`, `thegent/`, `Parpoura/`
- Scaffolding pattern: many repos have scaffolded `crates/<name>/` and `libs/<name>/` dirs excluded from workspace until populated

### CLAUDE.md Coverage
- ~100 repos have CLAUDE.md
- 7 active repos **missing** CLAUDE.md: `agileplus-agents` (Rust, Cargo.toml), `agileplus-mcp` (Python, pyproject.toml), `bdd-integration` (Rust, Cargo.toml), `dispatch-mcp` (Python, pyproject.toml), `pheno-cli` (Go, go.mod), `phenoShared/` (Python, likely fork of `PhenoShared/`), `phenotype-shared/` (Python, likely fork)
- Both `phenoShared/` and `PhenoShared/` exist — verify which is canonical before adding CLAUDE.md

### Stale/Orphaned Directories
- `.archive/` — flagged for deletion (task #233)
- `Tracera-corrupt-20260501201930` — recovery artifact, candidate for removal
- `AgilePlus-wtr/` — inconsistent worktree naming
- `ValidationKit/`, `PhenoContracts/`, `PhenoControl/`, `PhenoEvents/` — no CLAUDE.md, no README; assess whether active or archive candidates
