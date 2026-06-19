# AgilePlus AGENTS.MD

## Project Overview
AgilePlus is the Phenotype-org spec-driven development framework. Rust CLI + workspace for managing specs, work packages, and project governance. CLI: `agileplus <command>`

## Stack
- Language: Rust
- Build: Cargo workspace (members added when source exists; scaffolded crates/libs excluded until populated)
- CLI: Custom typer-based CLI
- Spec storage: `kitty-specs/` (inside the agileplus workspace)

## Key Commands
- `cargo build --release`
- `cargo test`
- `agileplus specify --title "<title>" --description "<desc>"`
- `agileplus status <feature-id> --wp <wp-id> --state <state>`

## Quality Gates
- `cargo clippy --all` + `cargo fmt --all`
- `cargo test --workspace`
- `cargo deny check licenses` (configured via `deny.toml`)
- `ruff check python/` (Python quality)

## Branch Discipline
- Feature work: `<repo>-wtrees/<subject>/` (e.g., `AgilePlus-wtrees/<topic>/`)
- Canonical `AgilePlus/` = bare repo (main only, no direct commits)
- Tracked workspace: `agileplus/` (lowercase; actual git worktree)
- Branch naming: `chore/`, `feat/`, `fix/` prefixes

## Governance Integration
- Specs: `kitty-specs/<feature-id>/` (relative to agileplus workspace root)
- Worklog: `AgilePlus/.work-audit/worklog.md`

## Repo Structure
- `agileplus/` — **primary tracked workspace** (lowercase; all actual source lives here)
- `AgilePlus/` — bare git repo (remote: KooshaPari/AgilePlus; commits only via PR merge)
- `*-wtrees/` — feature worktrees; safe to work in directly
- `kitty-specs/` — root-level spec archive (legacy, read-only)
- Individual repos: `Agentora/`, `pheno/`, etc.

## Important Notes
- **Never commit directly to `AgilePlus/` main** — it is bare. All changes go through PRs.
- **Do not use `AgilePlus-wtrees/<subject>/`** — the worktree convention is `<repo>-wtrees/` (lowercase repo name).
- See `agileplus/CLAUDE.md` for detailed workspace structure, bootstrap status, and agent operating notes.
