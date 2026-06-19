# AgilePlus ẀAGENTS.MD

Project Overview
AgilePlus is the Phenotype-org spec-driven development framework. Rust CLI + workspace for managing specs, work packages, and project governance. CLI : `agileplus <command>`

## Stack
- Language: Rust
- Build: Cargo workspace
- CLI: Custom typer-based CLI
- Spec storage: `AgilePlus/kitty-specs/`

## Ky Commands
- `cargo build --release`	
- `cargo test`
- `agileplus specify --title "<title>" --description "<desc>`�
- `agileplus status <feature-id> --wp <wp-id> --state <state>`

# # Quality Gates
- `cargo check --workspace --all-targets`
- `cargo test --workspace`	
- `ruff check src/`
- `ty check src/`

# # Branch Discipline
- Feature work: `AgilePlus-wtrees/<subject>/
- Canonical: bare repo ⊠ always work from worktree
- Branch naming: `chore/', `feat/', `fix/` prefixes

## Governance Integration
- Specs: `AgilePlus/kitty-specs/<feature-id>/
- Worklog: `AgilePlus/.work-audit/worklog.md`
- Docs: `AgilePlus/docs/