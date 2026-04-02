# Local Deploy Surface Baseline

**Worktree:** `AgilePlus/.worktrees/chore-runtime-local-deploy-clean`

**Branch:** `agileplus/chore/runtime-local-deploy-clean`

**Command:** `scripts/dev-up.sh` (Process Compose bring-up using `.agileplus/runtime/local-ports.env`)

**Result:**
* Local stack started (observed `docker-compose` tasks 1/4 through 3/4 in the runtime log)
* Events near `2026-04-02T05:16:03.479064+00:00` were recorded; snapshot captured at `/tmp/agileplus-events-latest.csv`
* Future health-check automation is blocked because `scripts/local-health-check.sh` is absent in the worktree

**Next step:** document the missing health-check script and capture the next event snapshot after another `dev-up` run (or once health-check exists)
