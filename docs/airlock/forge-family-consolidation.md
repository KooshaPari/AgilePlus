# Forge Family Consolidation Docket

**Date:** 2026-07-29
**Auditor:** Forge-Code (autonomous consolidation pass)
**Status:** MIGRATION PROPOSED — feature-branch imports + squash pending approval

---

## Members in Scope

| Repo | Default | Branches | Last Push | Local Clone | Action |
|---|---|---|---|---|---|
| `forgecode` | main | 10 | 2026-07-29 | ✅ `/repos/forgecode` | CANONICAL |
| `pheno-forge-smoke` | main | 4 | 2026-07-28 | ❌ | MERGE INTO forgecode |
| `pheno-forge-plugins` | main | 4 | 2026-07-22 | ❌ | MERGE INTO forgecode |
| `forgecode-tmp` | main | 1 | 2026-07-29 | ❌ | docket + retire |
| `phenoForge` | main | 1 | 2026-07-22 | ❌ | KEEP (distinct project) |
| `PlusForges` | main | 1 | 2026-06-25 | ❌ | KEEP (meta-pointer) |
| `MCPForge` | main | 18 | 2026-07-29 | ❌ | KEEP (distinct fork) |
| `Tasken-phenoforge-final` | chore/governance-baseline | 5 | 2026-07-16 | ❌ | docket + retire |
| `dinoforge-packs-archive-2026-07-14` | main | 1 | 2026-07-15 | ❌ | docket + retire |
| `DINOForge-UnityDoorstop` | master | 1 | 2026-06-21 | ❌ | KEEP (archived; distinct project) |

## MIGRATE — semantic content mapping

### A. Active merges into `forgecode`

| Source | Target | Rule | Notes |
|---|---|---|---|
| `pheno-forge-smoke` | `forgecode/crates/pheno-forge-smoke/` | absorbed | "End-to-end smoke binary for the 4-PR forgecode improvement stack (ADR-096) — loads libpheno_bridge via libloading and exercises composite routing" |
| `pheno-forge-plugins` | `forgecode/crates/pheno-forge-plugins/` (or `plugins/`) | absorbed | "Sidecar bundle for antinomyhq/forgecode — 6 plugins (memory + config + tracing) brought up as per-machine systemd units" |

### B. Retired/archived receipts

| Source | Target | Rule | Notes |
|---|---|---|---|
| `forgecode-tmp` | `forgecode` (history receipt) | snapshot | 1 branch, recovery snapshot |
| `Tasken-phenoforge-final` | retired | snapshot | 5 branches, archive of deleted `Tasken-phenoforge-final` from 2026-07-14 |
| `dinoforge-packs-archive-2026-07-14` | retired | snapshot | 1 branch, archive of deleted `dinoforge-packs` |

### C. NOT consolidated (distinct projects)

| Source | Action | Why |
|---|---|---|
| `phenoForge` | keep separate | "Rust-native task runner with parallel execution, dependency graph resolution, hot reload" — distinct project despite name overlap |
| `PlusForges` | keep as meta-pointer README | "Meta-repo of all KooshaPari 'Plus' forks" |
| `MCPForge` | keep separate | Fork of `isaacphi/mcp-language-server` (Go LSP); distinct upstream from `antinomyhq/forgecode` (Rust) |
| `DINOForge-UnityDoorstop` | keep archived | Fork of `NeighTools/UnityDoorstop` (C#/Unity DLL-injector); distinct project |

## STATE — current branches

```
forgecode (10 branches):
  main, plus 9 develop branches (forge/origin/upstream remotes)

pheno-forge-smoke (4 branches):
  main + 3 branches (smoke binary evolution)

pheno-forge-plugins (4 branches):
  main + 3 branches (plugin sidecar evolution)

forgecode-tmp (1):
  main (recovery snapshot 2026-07-29)

MCPForge (18):
  main + 17 (LSP fork evolution)
```

## ABSORBED — confirmed content states

- **`pheno-forge-smoke`** → content lives as `bin/pheno-forge-smoke` per its description; targeted for `forgecode/crates/pheno-forge-smoke/`.
- **`pheno-forge-plugins`** → content lives as sidecar systemd units; targeted for `forgecode/plugins/` or `forgecode/crates/pheno-forge-plugins/`.

## SUPERSEDES — receipts preserved

- `forgecode-tmp` — preserved as 1-branch receipt.
- `Tasken-phenoforge-final` — preserved as 5-branch archive.
- `dinoforge-packs-archive-2026-07-14` — preserved as 1-branch archive.
- `DINOForge-UnityDoorstop` — already preserved (archived flag = true).

## PROPOSED MUTATIONS (NOT EXECUTED — pending approval)

### Phase 1: feature-branch imports (NO force-push)
1. Create `forgecode` branch `feat/import-pheno-forge-smoke` from `main`.
2. Cherry-pick `pheno-forge-smoke`'s commits into `forgecode/crates/pheno-forge-smoke/`.
3. Create `forgecode` branch `feat/import-pheno-forge-plugins` from `main`.
4. Cherry-pick `pheno-forge-plugins`'s commits into `forgecode/crates/pheno-forge-plugins/`.
5. Push both as feature branches (no force-push).
6. Verify CI on each branch.

### Phase 2: PRs and merge
1. PR each feature branch against `forgecode/main`.
2. Wait for user sign-off on each PR.

### Phase 3: SQUASH (per-group approval)
1. After PR merges, squash `forgecode` to 1 commit on `main`.
2. Squash `MCPForge` to 1 commit on `main` (independent).

## LOCAL CHECKOUT COVERAGE

- `/repos/forgecode` (fork/origin/upstream): 10/10 branches local ✅
- Local clones of `pheno-forge-smoke`, `pheno-forge-plugins` required for cherry-pick — to be set up at `/repos/pheno-forge-smoke` and `/repos/pheno-forge-plugins` (shallow clones). **Safe: read-only fetches.**

## RISK CLASS

**Medium.** The `pheno-forge-smoke` import is low-risk (binary only). The `pheno-forge-plugins` import is medium-risk — it integrates with `antinomyhq/forgecode` upstream and may have already been partially merged there. Verify by `git log --oneline forgecode/main ^antinomyhq/forgecode/main | wc -l` (count already-merged commits).

## NEXT CHECKPOINT

User must approve:
- (a) starting feature-branch imports for `pheno-forge-smoke` and `pheno-forge-plugins` (no force-push)
- (b) final squash of `forgecode` and `MCPForge`
