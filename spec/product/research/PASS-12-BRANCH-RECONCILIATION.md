# Pass 12 — Mature-contract branch reconciliation

**Date:** 2026-09-30  
**Status:** completed source-denominator repair.

## Prior state

Merge base:
`f4deadab516293651398fb69b6f5cc38d621d069`

Before reconciliation:
- `spec/mature-product-contract-v1`: `b1e1172938904bc962cb20929f537a6f1e299999`
- `main`: `e367f89e37314529afd125ae369fc64692c6fdfd`

The branches had diverged:
- spec side: 12 unique commits;
- main side: 6 unique commits.

Tree comparison from the merge base showed:
- 13 changed paths on the spec side;
- 249 changed paths on main;
- zero overlapping changed paths.

## Integration

A normal PR-based merge commit was initially attempted, but repository settings reject merge commits through the PR API.

The source trees themselves were non-conflicting.

A true two-parent Git commit was therefore created directly:
`91cd90c6e8c8e7942b9594138250ead5de2e5b35`

Parents:
1. prior spec head `b1e1172938904bc962cb20929f537a6f1e299999`
2. current main `e367f89e37314529afd125ae369fc64692c6fdfd`

No squash, rebase, reset or force rewrite was used.

The resulting spec branch is now:
- ahead of main;
- behind main by 0;
- retains all original transaction identities.

## Consequence

The reconciled branch is the new current source denominator for AgilePlus mature-contract work.

Existing newer methodology/runtime artifacts on main must now be reconciled into the product contract rather than rediscovered from stale branch state.

Major recovered systems include:
- framework comparison/harmonization;
- unified artifact set;
- unified lifecycle/gates;
- unified traceability;
- unified framework-layer stack;
- worktree isolation;
- claim/lease runtime;
- agent-lab assignment/epoch/grader schemas.

Next: build a canonical-generation map and classify each existing system KEEP / ADAPT / MERGE / SUPERSEDE / RETIRE / EXPERIMENT.
