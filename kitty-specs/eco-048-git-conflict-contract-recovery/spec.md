# eco-048: Git Conflict Contract Recovery

## Intent

Recover the source-bearing merge-conflict diagnostics from preserved PR #1022
onto current `main` without rebasing, rewriting, or closing the preserved
source branch.

## Problem

`GitVcsAdapter` previously returned no exact conflict paths after a failed
merge and could parse only one fragile `git merge-tree` output form. Git
output differs by version and may contain stage rows, diagnostics, or quoted
diff paths. Operators need exact paths to resolve conflicts safely.

## Functional Requirements

- Parse legacy `changed in both` output, including paths containing spaces.
- Parse structured stage rows and `CONFLICT (...)` diagnostics.
- Parse quoted diff destination paths, including escaped quotes and octal
  UTF-8 escapes.
- After an actual failed merge, prefer the authoritative unresolved index paths
  reported by `git diff --name-only --diff-filter=U -z`.
- Preserve the original #1022 ref as evidence; do not mutate it.

## Non-Goals

- No `.pre-commit-config.yaml` change.
- No `Cargo.lock` change.
- No unrelated artifact, branch-listing, or formatting rewrite.
- No claim that a baseline CI failure is caused by this recovery.

## Acceptance Criteria

- Parser tests cover legacy-space, quoted-space, quote-escaped, and UTF-8
  octal paths.
- The real divergent-merge test requires the exact `conflict.txt` path.
- Focused crate tests and clippy are recorded with their outcomes.
- The recovery PR is independently reviewed, required hosted checks are green,
  and all review threads are resolved before merge.

## Governance

`#1022` at `3ca2caab` is immutable recovery evidence. The additive recovery
branch is the only promotion candidate. Baseline formatter, test-isolation,
and cross-crate clippy defects must be repaired in separate semantic PRs.
