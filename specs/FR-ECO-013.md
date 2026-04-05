---
id: FR-ECO-013
title: KooshaPari GitHub Stale Repo Triage
status: specified
priority: P3
created: 2026-03-25
category: maintenance
owner: kooshapari
source: kitty-specs/kooshapari-stale-repo-triage
---

# FR-ECO-KST-001: KooshaPari GitHub Stale Repo Triage

## Description

Drive each repo in `stale_90d_1y`, `stale_1y_2y`, and `stale_over_2y` buckets to a decision: archive, delete, or revive with owner.


## User Stories

### US-1: Maintenance Automation (P1)
**Given** an operations engineer monitoring the system,
**When** maintenance tasks are required,
**Then** automated tools handle routine operations with minimal manual intervention.

### US-2: Ecosystem Health (P2)
**Given** a platform administrator,
**When** ecosystem health degrades,
**Then** early warnings and remediation guidance are provided.

## Acceptance Criteria

- [ ] Decision log (markdown table or CSV) with repo, decision, rationale, date
- [ ] Batch 1: all `stale_over_2y` and `stale_1y_2y` (9 repos) resolved on GitHub
- [ ] No accidental deletion of repos with unique remotes not mirrored in `Phenotype/repos`

## Notes

Original: `kitty-specs/kooshapari-stale-repo-triage/`
