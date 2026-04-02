---
work_package_id: WP06
title: Intake and Boundary Normalization
feature: # Polyrepo Ecosystem Stabilization
feature_slug: polyrepo-mixed-tranche-wave-1
sequence: 6
state: planned
created_at: 2026-04-02T00:00:00Z
dependencies: []
phase: Phase 1 - Parallel Classification
priority: P1
---

# Work Package: Intake and Boundary Normalization

## Feature
# Polyrepo Ecosystem Stabilization (`polyrepo-mixed-tranche-wave-1`)

## Objectives

Classify external intake and boundary surfaces so the next wave starts from explicit ownership and
scope, not ambiguous shelf edges.

## Success Criteria

- External intake wave 1 targets are classified as `import-now`, `watch`, `archive`, or `boundary`.
- Boundary surfaces such as `koosha-portfolio` have one explicit treatment and next artifact.
- Unresolved intake or boundary items are blocker-finalized with class `access` or `boundary`.

## File Scope

- `artifacts/next-target-list.md`
- `artifacts/local-pr-readiness-audit.md`
- `artifacts/polyrepo-next-tranche-wbs-20260402.md`
- tranche classification notes under `docs/` if needed

## Evidence

- updated intake table with status and next artifact
- boundary treatment log with final disposition and reason
- tranche note describing how classifications affect the next wave

## Stop Conditions

- no intake or boundary candidate remains unclassified at the next manager checkpoint
- unresolved cases are explicitly blocked, not left as implied follow-up

## Instructions

This WP is classification-first. Do not start onboarding or publishing work inside this lane.
