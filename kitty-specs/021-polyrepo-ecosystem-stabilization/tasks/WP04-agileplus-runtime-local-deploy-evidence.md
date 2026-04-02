---
work_package_id: WP04
title: AgilePlus Runtime and Local Deploy Evidence
feature: # Polyrepo Ecosystem Stabilization
feature_slug: polyrepo-mixed-tranche-wave-1
sequence: 4
state: planned
created_at: 2026-04-02T00:00:00Z
dependencies: []
phase: Phase 1 - Parallel Runtime Validation
priority: P1
---

# Work Package: AgilePlus Runtime and Local Deploy Evidence

## Feature
# Polyrepo Ecosystem Stabilization (`polyrepo-mixed-tranche-wave-1`)

## Objectives

Capture the first durable local runtime evidence set for AgilePlus and reduce the event-history gap
from prose-only notes to concrete evidence artifacts.

## Success Criteria

- Local AgilePlus bring-up and health verification run on the approved local scripts and produce a
  durable evidence snapshot.
- A fresh event-history snapshot is captured and linked from the tranche artifact.
- The local deploy baseline reflects the exact commands, branch, and health outcome.

## File Scope

- `artifacts/polyrepo-next-tranche-wbs-20260402.md`
- `artifacts/local-deploy-surface-baseline.md`
- `docs/sessions/`
- runtime evidence files under `artifacts/` or `/tmp`

## Evidence

- `/tmp/agileplus-events-latest.csv`
- `artifacts/local-deploy-surface-baseline.md` now records the clean runtime worktree, the `scripts/dev-up.sh` invocation, and the latest snapshot entry.
- `scripts/local-health-check.sh` is currently missing from `AgilePlus/.worktrees/chore-runtime-local-deploy-clean`; document this absence as the blocker to a full health validation.
- health-check output summary
- local deploy evidence note with command paths and result state

## Stop Conditions

- evidence is captured and linked, or the runtime path is blocker-finalized with exact failing
  subsystem
- no unrelated feature work is mixed into runtime validation

## Instructions

Prefer existing local scripts and evidence surfaces. If runtime remains sparse, classify the exact
failure source instead of leaving it implicit.
