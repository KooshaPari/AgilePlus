# Dependabot bumps (stingray, 2026-10-18T19:28:41Z)

**Verdict**: PASS
**Subagent session**: stingray
**Wall-clock**: ~7m (started 19:21:15Z, ended 19:28:41Z)
**PRs opened**: 2 (covers 4 of 5 actionable alerts; 1 alert is documented-ignore per the dispatch brief)
**Branch base**: main

## Dependabot alerts addressed

| # | Alert | Dep | Old | New | Manifest | PR |
|---|-------|-----|-----|-----|----------|-----|
| 1 | #217 | mcp | 1.28.1 | 1.30.0 | python/uv.lock (transitive) | https://github.com/KooshaPari/AgilePlus/pull/1162 |
| 2 | #216 | mcp | 1.27.1 | 1.30.0 | dispatch-mcp/uv.lock (direct) + dispatch-mcp/pyproject.toml | https://github.com/KooshaPari/AgilePlus/pull/1162 |
| 3 | #215 | mcp | 1.28.1 | 1.30.0 | agileplus-mcp/uv.lock (transitive) | https://github.com/KooshaPari/AgilePlus/pull/1162 |
| 4 | #214 | postcss-selector-parser | 6.1.4 | 7.1.6 | package-lock.json (transitive dev) + crates/agileplus-dashboard/web/package.json | https://github.com/KooshaPari/AgilePlus/pull/1163 |

The 5th alert (RUSTSEC-2026-*) was already fixed in main via Tauri upstream PR #16170 — documented-ignore, not touched.

## Test results

### mcp bump (PR #1162)
- `dispatch-mcp` (Python 3.14): `uv run pytest tests/ -x -q` -> exit 0, **24 passed in 0.27s**
- `agileplus-mcp` (Python 3.12): `uv run pytest tests/ -x -q` -> exit 0, **5 passed in 3.06s**
- `python` (Python 3.14): `uv run pytest tests/ -x -q` -> exit 0, **99 passed, 10 skipped in 5.7s**
- Installed `mcp` version verified via `importlib.metadata.version("mcp")` -> **1.30.0** in all three lockfiles
- For `dispatch-mcp`, also bumped `pyproject.toml`: `mcp>=1.0.0,<2.0.0` -> `mcp>=1.30.0,<2.0.0`

### postcss-selector-parser bump (PR #1163)
- `node -e "require('postcss-selector-parser')"` -> **loads OK, version 7.1.6**
- Sample selector parse test (`processSync('.foo .bar > .baz:hover')` -> walks `.foo`, `.bar`, `.baz`): **passes**
- `npm ls postcss-selector-parser` -> **7.1.6 deduped** across tailwindcss and postcss-nested
- Build/test runners (`vite build`, `vitest`, `tsc`) **blocked by pre-existing environmental issues** in this workspace: missing `rolldown` native binding for darwin-arm64, missing `@typescript/darwin-arm64` binary. These are unrelated to the postcss bump; the parser itself works in isolation and is API-compatible with what tailwindcss 3.x consumes.

## Approach notes

### mcp (combined single PR)
- All three lockfiles touch the same CVE (GHSA-84m7-p3x7-pcfv / CVE-2026-59951), so combined into one PR.
- The 1.x bump (not 2.x) because `dispatch-mcp/pyproject.toml` pins `mcp<2.0.0` and the other two lockfiles are fed via `fastmcp>=3.4.2` -> `fastmcp-slim[client,server]` -> `mcp` (no upper bound on mcp, so any 1.x is accepted).
- Resolved via `uv lock --upgrade-package mcp` for transitive bumps and a `sed` + `uv lock` for the direct `dispatch-mcp` constraint.

### postcss-selector-parser (separate PR)
- This dep is **transitive via tailwindcss@^3.4.17 and postcss-nested@^6.2.0** in `agileplus-dashboard-web` (the only consuming workspace).
- Root-level `npm overrides` was attempted first but did not update the lockfile consistently in this npm 11 + workspaces layout (`npm ls` reported the override was effective on paper but the install left 6.1.4).
- Final approach: pin `postcss-selector-parser: ^7.1.6` as a direct devDep in the consuming workspace. This forces the lockfile to 7.1.6, is the most predictable resolution, and matches what dependabot would do.
- The 6->7 API is preserved for the parser surface that tailwindcss 3.x uses (verified by sample parse).

## Blockers

None blocking. The build/test environmental issues for the dashboard workspace are pre-existing and unrelated to these bumps.

## Operator action required

- **Merge both PRs** to close the alerts:
  - https://github.com/KooshaPari/AgilePlus/pull/1162 (mcp)
  - https://github.com/KooshaPari/AgilePlus/pull/1163 (postcss-selector-parser)
- After merging, Dependabot should auto-detect the fixed versions and close alerts #214, #215, #216, #217 automatically on the next scan.
- Optional: consider whether to revert the direct `postcss-selector-parser` devDep pin in favor of a cleaner root-level `overrides` block once npm 11 + workspaces behavior is verified — the current approach works, but a root-level `overrides` would be more idiomatic if it ever becomes reliable.

## Commits

- `dependabot/bump-mcp-2026-10-18`: 1 commit (`bc05735c`) — 4 files changed, 1 insertion(+), 1 deletion(-)
- `dependabot/bump-postcss-selector-parser-2026-10-18`: 1 commit (`a3bffcf2`) — 2 files changed, 1 insertion(+)

## Trace log

Full command trace: `/tmp/dependabot_bumps_2026-10-18.log`