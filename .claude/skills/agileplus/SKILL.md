name: agileplus
description: Use for AgilePlus workspace engineering tasks, including CLI-based feature flow, workspace health checks, spec/task package validation, and local MCP helpers.

# AgilePlus dev loop

## When to invoke
- Task explicitly names `spec`, `feature`, `work package`, `worktree`, `quality gate`, `dispatch-mcp`, `CLI`.
- You need to edit or audit `crates/` crates, `kitty-specs/`, or quality checks in the root workspace.
- You are troubleshooting workspace-level build/test failures across Rust and Python helpers.

## Repo facts
- **Project path:** `C:/Users/koosh/Dev/AgilePlus`
- **Workspace type:** Rust Cargo workspace with Python helper MCP + sibling CLI tooling.
- **Core commands:** `cargo build --release`, `cargo test`, `agileplus specify`, `agileplus status`.
- **Quality gate:** `cargo fmt --all`, `cargo clippy --all`, `cargo test --workspace`, `ruff check python/`.
- **MCP helper:** `dispatch-mcp` (Python entrypoint) for dispatch/delegation flows.

## Fast workflow

### New spec-to-track path
1. Create/update the spec in `kitty-specs/<feature-id>/`.
2. Run:
```pwsh
agileplus specify --title "<title>" --description "<description>"
```
3. Run:
```pwsh
/ap-specs <feature-id>
```

### Daily engineering loop
1. Run fast checks:
```pwsh
cargo build --workspace
cargo test --workspace
```
2. For deeper hygiene:
```pwsh
/ap-quality
```
3. Update or verify spec status:
```pwsh
/ap-status <feature-id> --wp <wp-id> --state in_progress
```

## Commands from this skill set
- `ap-specify`
- `ap-status`
- `ap-build`
- `ap-quality`
- `ap-specs`
- `ap-worktree`
- `ap-mcp`

## Quality and diagnostics
- If builds fail, check `.cargo` lockfile drift and crate `Cargo.toml` feature mismatches first.
- If MCP tooling fails, validate entrypoint with `dispatch-mcp --version` and environment variable requirements (`OMNIROUTE_URL` when dispatching tiers).
- Keep work in this project’s branch (or worktree) and avoid touching unrelated directories unless explicitly requested.

