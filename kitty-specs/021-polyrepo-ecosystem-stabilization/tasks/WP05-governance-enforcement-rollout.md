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

## Current Gate Status

| Repo | Target PR(s) | Gate Status | Blocker Class | Notes |
| AgilePlus | #274, #275, #276, #278, #279 | blocked | review | Multiple split lanes still under review; no single lane has merged, so governance gate remains blocked until at least one PR demonstrates clean state changes. |
| heliosCLI | #182, #184 | blocked | review | Governance baseline lanes intentionally target `main`; waiting on policy/CD gate plus remaining review threads before the repo can claim `ready`. |
| heliosApp | #362 | blocked | review | Large federation/backfill change still needs CI/merge cleanup; treat as blocked until the PR is rebased/CI-green. |
| cliproxyapi-plusplus | #942 | blocked | external-provider | SAST workflows include Snyk steps that currently fail due to billing limits; gating blocked on Snyk quota. |
| phenodocs | #119 | blocked | ci | Required-check manifest is new and still awaiting remote verification and matching workflow names before the gate can flip. |
| agentapi-plusplus | #438 | blocked | ci | SAST/self-merge governance additions rely on pinned workflows; gate stays blocked until updated jobs fully pass. |
| phenotype-infrakit | (recovery PRs) | blocked | review | Multiple infra PRs remain open; treat staged recovery work as blocked until PRs merge and leave only one canonical lane. |

Update this table each poll cycle. When a repo transitions to `ready`, record the merge commit and the verified required checks list.
