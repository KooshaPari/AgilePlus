---
work_package_id: WP03
title: Secondary PR Lane Cleanup
feature: # Polyrepo Ecosystem Stabilization
feature_slug: polyrepo-mixed-tranche-wave-1
sequence: 3
state: planned
created_at: 2026-04-02T00:00:00Z
dependencies: []
phase: Phase 1 - Mixed Tranche Bootstrap
priority: P0
---

# Work Package: Secondary PR Lane Cleanup

## Feature
# Polyrepo Ecosystem Stabilization (`polyrepo-mixed-tranche-wave-1`)

## Objectives

Reduce the remaining active PR or recovery lanes outside AgilePlus and Helios to one clean lane
decision per repo.

## Success Criteria

- `cliproxyapi-plusplus` PR `#942`, `phenodocs` PR `#119`, and `agentapi-plusplus` draft PR `#438`
  are each advanced or blocker-finalized.
- Authored branch-local work is separated from generated, root-checkout, or worktree metadata
  noise.
- Remaining failures are classified with exact blocker classes and next actions.

## File Scope

- active PR branch files in `cliproxyapi-plusplus`
- governance and ruleset files in `phenodocs`
- governance and docs files in `agentapi-plusplus`, excluding `.worktrees/` metadata
- PR-relevant recovery files in `phenotype-infrakit`

## Evidence

- per-repo status summary in `artifacts/polyrepo-next-tranche-wbs-20260402.md`
- diff summary proving only scoped files changed
- CI or blocker outputs for each repo lane

## Stop Conditions

- each targeted lane is either ready for merge or blocker-finalized
- no `.worktrees/` metadata is normalized or committed as part of this WP

## Instructions

Do not treat shelf-root or generated artifact churn as authored branch work unless the branch
itself requires it.
