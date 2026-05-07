# AgilePlus AGENTS.MD

## Project Overview
AgilePlus is the Phenotype-org spec-driven development framework. Rust CLI + workspace for managing specs, work packages, and project governance. CLI: `agileplus <command>`

## Stack
- Language: Rust
- Build: Cargo workspace
- CLI: Custom typer-based CLI
- Spec storage: `agileplus/kitty-specs/`

## Key Commands
- `cargo build --release`
- `cargo test`
- `agileplus specify --title "<title>" --description "<desc>"`
- `agileplus status <feature-id> --wp <wp-id> --state <state>`

## Quality Gates
- `cargo check --workspace --all-targets`
- `cargo test --workspace`
- `ruff check src/`
- `ty check src/`

## Branch Discipline
- Feature work: `AgilePlus-wtrees/<subject>/`
- Canonical: bare repo — always work from worktree
- Branch naming: `chore/`, `feat/`, `fix/` prefixes

## Governance Integration
- Specs: `agileplus/kitty-specs/<feature-id>/`
- Worklog: `worklog.md`

## Repo Structure
- `agileplus/` — AgilePlus monorepo (separate git, bare + worktrees)
- `kitty-specs/` — Root-level specs (legacy, read-only)
- Individual repos: `Agentora/`, `pheno/`, etc.
