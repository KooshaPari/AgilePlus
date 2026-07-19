# AgilePlus A+ / SOTA completion

## Goal
Drive AgilePlus (and tracked Phenotype repos) to A+ product completeness and SOTA optimality: CLI/DB/platform/dashboard/publish parity, green platform, dogfooded tracking, deploy/install/publish channels.

## Acceptance criteria
- Single SSOT DB with migrations applied (backlog_items present on default path)
- Workspace CLI matches PATH SDD engine (specify/plan/ship/queue/module/platform)
- Platform stack UP with agileplus-api health
- Dashboard usable against live platform
- Release assets published OR install-from-source path verified
- Self-feature tracked through specify→plan→implement→validate→ship
- Quality: local task lint/test/quality green

## Workstreams
1. CLI un-stub / binary SSOT
2. DB migration apply on default .agileplus
3. Platform API restore + process-compose
4. Docs truth pass
5. Publish/install channel
6. Cross-repo A+ shelf hygiene
