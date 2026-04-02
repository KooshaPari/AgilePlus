---
work_package_id: WP05
title: Governance Enforcement Rollout
feature: # Polyrepo Ecosystem Stabilization
feature_slug: polyrepo-mixed-tranche-wave-1
sequence: 5
state: planned
created_at: 2026-04-02T00:00:00Z
dependencies: [WP01, WP02, WP03]
phase: Phase 2 - Dependency Gated Governance
priority: P1
---

# Work Package: Governance Enforcement Rollout

## Feature
# Polyrepo Ecosystem Stabilization (`polyrepo-mixed-tranche-wave-1`)

## Objectives

Convert stabilized active-lane truth into enforcement-ready governance state for the active repos.

## Success Criteria

- Active repos have a recorded governance gate status of `ready`, `hold`, or `blocked`.
- Required status-check names and ruleset baseline state are recorded per repo.
- Each non-ready repo has one explicit next action and blocker class.

## File Scope

- `artifacts/github-ruleset-governance-baseline.md`
- `artifacts/local-pr-readiness-audit.md`
- `artifacts/polyrepo-next-tranche-wbs-20260402.md`
- `kitty-specs/021-polyrepo-ecosystem-stabilization/plan.md`

## Evidence

- per-repo governance gate checklist
- required-check manifest with exact workflow names
- blocker map for repos not yet enforcement-ready

## Stop Conditions

- every active repo is marked `ready`, `hold`, or `blocked` with evidence
- governance rollout does not start before `WP01`, `WP02`, and `WP03` are complete or
  blocker-finalized

## Instructions

Use stabilized PR and branch truth from the dependent WPs. Do not guess required checks from stale
or mixed repo state.
