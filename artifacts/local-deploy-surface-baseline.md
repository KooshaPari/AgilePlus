# Local Deploy Surface Baseline

**Worktree:** `AgilePlus/.worktrees/chore-runtime-local-deploy-clean`

**Branch:** `agileplus/chore/runtime-local-deploy-clean`

**Command:** `scripts/dev-up.sh` (Process Compose bring-up using `.agileplus/runtime/local-ports.env`)

**Result:**
* Local stack started (observed `docker-compose` tasks 1/4 through 3/4 in the runtime log)
* Events near `2026-04-02T05:16:03.479064+00:00` were recorded; snapshot captured at `/tmp/agileplus-events-latest.csv`
* `scripts/local-health-check.sh` and `scripts/authkit-smoke.sh` now exist in the clean runtime worktree as repeatable local validation entrypoints
* This evidence snapshot still lacks a fresh run of those scripts against a live local stack and a real provider domain

**Next step:** rerun `scripts/dev-up.sh`, then execute `scripts/local-health-check.sh` and `scripts/authkit-smoke.sh` with real environment values so WP04 records verified local health and provider metadata evidence instead of only script availability
