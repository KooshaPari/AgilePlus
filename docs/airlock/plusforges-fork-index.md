# PlusForges Member Fork Index Docket

**Date:** 2026-07-29
**Auditor:** Forge-Code (autonomous consolidation pass)
**Status:** MIGRATION PROPOSED — awaiting squash approval per fork

---

## Members in Scope

| Repo | Default | Branches | Last Push | Local Clone | Upstream |
|---|---|---|---|---|---|
| `agentapi-plusplus` | main | 1 | 2026-07-29 | ❌ | `coder/agentapi` |
| `cliproxyapi-plusplus` | main | 4 | 2026-07-29 | ✅ `/repos/cliproxyapi-plusplus` | `router-for-me/CLIProxyAPI` |
| `context-mode-plusplus` | main | 1 | 2026-07-29 | ❌ | (upstream-fork) |
| `PlusForges` | main | 1 | 2026-06-25 | ❌ | (meta-repo) |

## MIGRATE — semantic content mapping

These are **independent forks of distinct upstreams** — they are NOT duplicates of each other. `PlusForges` is the meta-index repo that lists them; the actual forks live in their own repos.

| Source | Target | Rule | Notes |
|---|---|---|---|
| `PlusForges` | pointer | meta | README-only list linking out to all `*+-plusplus` forks |
| `agentapi-plusplus` | canonical | n/a | Fork of `coder/agentapi` |
| `cliproxyapi-plusplus` | canonical | n/a | Fork of `router-for-me/CLIProxyAPI`; local clone has origin+upstream remotes |
| `context-mode-plusplus` | canonical | n/a | Upstream-fork; standalone |

## STATE — current branches

```
agentapi-plusplus:           main (single branch)
cliproxyapi-plusplus:        main + 3 develop branches
context-mode-plusplus:       main (single branch)
PlusForges:                  main (README-only)
```

## ABSORBED — confirmed content states

- **`agentapi-plusplus`** → independent fork; no absorption expected.
- **`cliproxyapi-plusplus`** → independent fork; no absorption expected.
- **`context-mode-plusplus`** → independent fork; no absorption expected.
- **`PlusForges`** → meta-pointer; no novel content.

## SUPERSEDES — receipts preserved

Each fork retains its own history. No archive repos in this family.

## PROPOSED MUTATIONS (NOT EXECUTED — pending approval)

1. `agentapi-plusplus` → squash to 1 commit on `main`.
2. `cliproxyapi-plusplus` → squash to 1 commit on `main`.
3. `context-mode-plusplus` → squash to 1 commit on `main`.
4. `PlusForges` → leave as meta-pointer README (no squash needed; 1 branch total).

**Upstream sync verification (recommended before squash):**

```
git remote add upstream https://github.com/coder/agentapi.git
git remote add upstream https://github.com/router-for-me/CLIProxyAPI.git
git fetch upstream --quiet
git rev-list --count upstream/main ^main    # unique upstream commits
git rev-list --count main ^upstream/main    # unique local commits
```

For each fork, this verifies whether local is ahead/behind upstream. The user's stance is "Plus" forks carry KooshaPari patches on top of upstream.

## LOCAL CHECKOUT COVERAGE

- `/repos/cliproxyapi-plusplus` (origin/upstream): 4/4 branches local ✅
- Other 3 forks: single-branch only; no local clone needed.

## RISK CLASS

**Low.** Small forks; no cross-fork conflicts.

## NEXT CHECKPOINT

User must approve:
- (a) upstream-sync verification per fork
- (b) squash of each fork to 1 commit on `main` (separately or together)
