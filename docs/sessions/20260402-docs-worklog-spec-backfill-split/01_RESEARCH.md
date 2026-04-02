# Research

## Local Split Findings

- The original `agileplus/docs/worklog-and-spec-backfill` branch still carried mixed repo state from
  local `main`, including DB and script changes that did not belong in a docs-only review.
- The clean replay target is `origin/main` via
  `AgilePlus/.worktrees/docs-worklog-and-spec-backfill-clean`.
- Documentation-only scope for this replay is:
  - `.work-audit/worklog.md`
  - `worklog.md`
  - `kitty-specs/*` validation, spec, task, research, and plan files introduced or updated by the
    mixed local commit
- Captured `/tmp/agileplus-events-latest.csv` to show the current runtime event backlog via
  `sqlite3 AgilePlus/.agileplus/agileplus.db "select entity_type,entity_id,event_type,timestamp from events order by timestamp desc limit 200;"`.
- `scripts/dev-up.sh` now brings up the local stack from `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.worktrees/chore-runtime-local-deploy-clean`; `scripts/local-health-check.sh` and `scripts/authkit-smoke.sh` are now present there, and the bootstrap glue has been repaired to preserve the persisted port map, launch OrbStack correctly, and avoid a hardcoded Process Compose control-port collision.
- WP04 is now blocked on real runtime failures rather than missing script surfaces: `neo4j` aborts during startup, and `plane-api` still exits from the stale `.agileplus/plane/apiserver` run path.

## Decision

Replay only docs and ledger files into the clean worktree, then publish that branch as the final
AgilePlus split PR for this tranche.
