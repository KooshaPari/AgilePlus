# Local Deploy Surface Baseline

**Worktree:** `AgilePlus/.worktrees/chore-runtime-local-deploy-clean`

**Branch:** `agileplus/chore/runtime-local-deploy-clean`

**Command:** `scripts/dev-up.sh` (Process Compose bring-up using `.agileplus/runtime/local-ports.env`)

**Result:**
* Local stack started (observed `docker-compose` tasks 1/4 through 3/4 in the runtime log)
* Events near `2026-04-02T05:16:03.479064+00:00` were recorded; snapshot captured at `/tmp/agileplus-events-latest.csv`
* `scripts/local-health-check.sh` and `scripts/authkit-smoke.sh` now exist in the clean runtime worktree as repeatable local validation entrypoints
* `scripts/resolve-local-ports.sh`, `scripts/orb-up.sh`, and `scripts/dev-up.sh` now preserve the persisted port map, bootstrap OrbStack containers correctly, and avoid the Process Compose control-port collision that previously blocked detached startup
* The remaining live blockers are deeper services: `neo4j` aborts during startup, and `plane-api` still exits immediately from the stale `.agileplus/plane/apiserver` run path before the health check can pass

**Next step:** repair the `neo4j` and Plane API startup paths inside the runtime worktree, then rerun `scripts/dev-up.sh`, `scripts/local-health-check.sh`, and `scripts/authkit-smoke.sh` so WP04 records verified local health and provider metadata evidence instead of bootstrap-only progress
