---
id: FR-AUDIT-001
title: Portfolio Audit KooshaPari Legacy + Pheno SDK
status: specified
priority: P3
created: 2026-03-25
category: audit
owner: kooshapari
source: kitty-specs/portfolio-audit-kooshapari-2026
---

# FR-AUDIT-001: Portfolio Audit KooshaPari Legacy + Pheno SDK

## Description

Establish tracked program to inventory, assess, and modernize KooshaPari-era and CodeProjects-local work (2023–2026), with Pheno SDK as primary monolith input and Phenotype `libs/*` as extraction targets.

## Scope

- GitHub org `KooshaPari` (249 repos): triage by last push, archive status, overlap with `Phenotype/repos`
- Local `CodeProjects/*` (KooshaPari/Dino, archive, orphans, Dev, learning)
- Canonical SDK tree: `Phenotype/repos/worktrees/phenoSDK/main`

## Acceptance Criteria

1. [ ] Inventory artifacts committed or referenced from `docs/reports/`
2. [ ] Per-repo/per-cluster backlog: hexagonal boundaries, CI/QA gaps, decomposition candidates
3. [ ] Pheno SDK: language map, hot-spot modules, proposed package split
4. [ ] Optional acceleration: PyO3/Rust/Zig where profiling justifies; default Python ports with thin native adapters

## Work Packages

- WP-A1: Freeze org inventory + local path map
- WP-A2: Pheno SDK structure + tokei baseline + test/lint smoke
- WP-A3: Crosswalk `phenoSDK` packages to `libs/python/phenotype-sdk`, etc.
- WP-A4: AgilePlus `plan` + `research` for top three extraction epics

## Non-Goals

- Rewriting Pheno SDK in one pass
- Resolving GitHub Actions billing on KooshaPari account

## Notes

Original: `kitty-specs/portfolio-audit-kooshapari-2026/`
