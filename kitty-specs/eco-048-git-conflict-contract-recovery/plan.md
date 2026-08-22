# Plan: Git Conflict Contract Recovery

1. Preserve the original PR #1022 and classify its four file deltas.
2. Recompose only the conflict-parser and exact-path test contract from current
   `main` in an additive worktree.
3. Prove the parser against the four output encodings and the real divergent
   merge flow.
4. Record unrelated baseline failures as distinct repair lanes.
5. Publish a draft PR, complete semantic review, then require hosted CI and
   governance gates before promotion.

## Scope Decision

The hook/configuration and lockfile deltas were intentionally excluded because
they do not establish a causal dependency on conflict reporting. The original
branch remains reachable evidence for later review.
