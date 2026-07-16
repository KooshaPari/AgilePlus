# Research

## Initial findings

- `origin/main` is commit `a83a7677`.
- The workspace includes `crates/agileplus-sqlite`.
- The independent branch `chore/sqlite-test-migration-runner` is not merged and
  contains both committed migration work and uncommitted test-support work; it
  is preserved and excluded from this repair branch.
- CI executes `cargo fmt --all -- --check`, `cargo build --workspace`, and
  `cargo test --workspace` from repository root.

## Investigation method

Reproduce the formatter and SQLite package test commands on the clean
`origin/main` base, then trace any compiler/module error to the module
declaration and its expected source path before editing.

## Reproduction result

- `cargo test -p agileplus-sqlite` passed: 80 tests passed, 0 failed.
- `cargo fmt --all -- --check` failed on the clean base with 84 files needing
  rustfmt normalization. The failures span application, CLI, dashboard,
  domain, governance, SQLite, triage, and traceability crates, so the CI gate
  cannot truthfully be repaired with a SQLite-only formatting change.
- The uncommitted `test_support.rs` in the preserved SQLite runner worktree is
  not referenced by `origin/main` and has no open PR. It is therefore a
  separate unfinished change, not a reproducible main-branch failure.

## Decision

Apply the formatter's mechanical output for all files selected by the existing
workspace gate. Do not copy the unfinished `test_support` module into this
branch; keep it available in its original worktree for its owning slice.

## Adjacent CI repair

`cargo clippy --workspace -- -D warnings` reproduced an independent generated
protobuf build failure: `tonic_build::configure` is absent from `tonic-build`
0.14. Git history identifies the cause as Dependabot PR #839, which upgraded
only `tonic-build` and `prost` to 0.14 while `tonic`, `tonic-health`, and
`prost-build` remained on 0.13. The focused repair restores the internally
compatible 0.13 `tonic-build`/`prost` pair, rather than applying an unrelated
workspace-wide 0.14 API migration.
