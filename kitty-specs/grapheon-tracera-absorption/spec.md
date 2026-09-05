---
spec_id: grapheon-tracera-absorption
state: specified
target_branch: main
---

# Grapheon to Tracera no-loss absorption and retirement

## Problem statement

Grapheon has been renamed and archived before all retirement gates were
proven. The program needs an owner-visible lifecycle record that preserves
every source-bearing ref and artifact, proves Tracera successor behavior and
consumer use, and prevents final retirement claims until independent backup,
restore, and zero-billable-runner evidence exists.

## Scope

- Preserve Grapheon refs, worktrees, history, artifacts, packages,
  deployments, automation, open work, and consumer records without destructive
  Git operations or new checkout sprawl.
- Deliver narrow current-main Tracera successor slices with reviewable commits,
  tests, authenticated consumption, telemetry, compliance, and dogfood proof.
- Reconcile phenotype-registry boundary records and AgilePlus lifecycle state.
- Establish immutable second-cloud backup, independent restore, ref parity, and
  git-fsck evidence before any retirement completion decision.

## Constraints

- No delete, reset, clean, prune, force-push, history rewrite, or unapproved
  archive/rename operation.
- Hosted checks must not use billable runners; local evidence must remain
  distinct from hosted CI evidence.
- Historical source-bearing records remain immutable and separately indexed.

## Acceptance criteria

1. A machine-verifiable inventory covers all Grapheon refs, worktrees, history,
   artifacts, automation, packages, deployments, open work, and consumers.
2. Tracera successor changes are merged from current `main` with review, green
   required checks, and captured runtime/telemetry/compliance/dogfood evidence.
3. Every identified consumer is repointed, owner-approved retired, or retained
   with an explicit blocker and provenance record.
4. AgilePlus and phenotype-registry records agree with the live repository and
   preserve historical evidence.
5. An immutable second-cloud copy is independently restored and verified for
   bundle hash, ref/object parity, and `git fsck --full` success.
6. Sponsor approval is recorded only after criteria 1-5 pass; until then the
   disposition is `NO-GO`.
