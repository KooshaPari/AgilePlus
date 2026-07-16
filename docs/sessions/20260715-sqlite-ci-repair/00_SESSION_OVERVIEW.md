# SQLite CI repair

## Goal

Produce a focused, PR-ready repair for the known SQLite test-module and
formatting failure without changing or discarding any existing worktree.

## Preservation snapshot

- Captured: 2026-07-15 (America/Los_Angeles)
- Branch: `fix/sqlite-ci-repair`, created from `origin/main`
- Base: `a83a7677ecacac0a3080e41da312d80def74fee5`
- Working tree before this session document: clean
- Remote: `git@github.com:KooshaPari/AgilePlus.git`
- Existing worktrees retained: 16, including the independently dirty
  `ap-cockpit`, `cli-test-reconcile`, `speckitty-catalog`, and
  `sqlite-test-migration-runner` worktrees.

## Scope

Only the reproducible CI failure in the SQLite crate and directly necessary CI
plumbing are in scope. Existing worktree changes and unrelated branches are
not incorporated.
