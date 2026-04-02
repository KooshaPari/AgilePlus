# Polyrepo Next Tranche WBS

Updated: 2026-04-02

## Purpose

This is the execution WBS for the next polyrepo stabilization tranche after the initial PR-creation
wave. It supersedes the older quarter-scale plan for immediate execution order and reflects the live
repo, worktree, stash, and PR geometry on 2026-04-02.

## Execution Update 2026-04-02

### Runtime and Evidence

- Captured a fresh AgilePlus event snapshot to `/tmp/agileplus-events-latest.csv`.
- Current event history is effectively empty for the tranche; the snapshot contains only one recent
  row:
  - `feature|2|state_transitioned|2026-04-02T05:16:03.479064+00:00`
- This confirms that runtime transitions and tranche progress are still under-recorded in the live
  AgilePlus surfaces.

### phenotype-infrakit

- Base checkout at the shelf root is still polluted by sibling repo state and cannot be treated as a
  safe edit surface.
- `.worktrees/feat/cache-adapter-impl` is detached `HEAD` and requires explicit classification before
  use.
- `.worktrees/feat/phenotype-crypto-complete` contains real implementation drift on
  `feat/crypto-complete-rebased`:
  - modified `Cargo.toml`
  - modified `crates/phenotype-crypto/Cargo.toml`
  - modified `crates/phenotype-crypto/src/lib.rs`
  - untracked `encryption.rs`, `hash.rs`, `keys.rs`, `signatures.rs`, and `tests/`
- No stash entries were observed.

### thegent

- `platforms/worktrees/thegent/pr-876-fix` is the active non-root lane and sits
  `ahead 29, behind 7` relative to `origin/chore/sync-docs-security-deps`.
- `platforms/thegent-pr882` remains stale metadata and still reports shelf-root pollution through
  relative paths, so it should not be used as an active edit surface.
- Two stashes remain on `main` and must be preserved until they are reclassified:
  - `stash@{0}` `feat(thegent): enhance phench, governance providers, observability`
  - `stash@{1}` `docs(prd): expand PRD with 5 feature epics (#888)`

### agentapi-plusplus

- Root checkout still mixes intended governance or chat edits with massive tracked deletions under
  `docs/node_modules/**`.
- The `docs/node_modules` churn is consistent with generated-install artifact cleanup, not with
  intentional source changes.
- First non-PR action remains classification and hygiene, not branch publication.

### cloud

- Preserved local branch `cloud/chore/plan-sync-20260402` still contains only two plan-file updates:
  - `plans/gastown-town-centric-refactor.md`
  - `plans/product-analytics-improvements.md`
- No extra local diff exists outside that preserved branch.
- Publish remains blocked by `403` against `Kilo-Org/cloud`.

### koosha-portfolio

- Still not a git repo.
- Current visible surface is only `.next/`, `styles/globals.css`, and `.DS_Store`.
- It remains a boundary folder and stays out of the PR queue until explicit onboarding.

## Phase 1: Active PR Lane Completion

### 1.1 AgilePlus split PR tranche

Objective:
close the already-open split lanes without reintroducing mixed `main` state.

Work packages:
- `WP1.1.1` PR `#274` governance lane follow-through
  - keep fixes limited to GitHub ruleset baseline and governance docs
  - do not mix runtime, CLI, or worklog changes into this lane
- `WP1.1.2` PR `#275` runtime-local-deploy follow-through
  - keep fixes limited to local deploy workflow surfaces and supporting scripts
- `WP1.1.3` PR `#276` CLI event-flow follow-through
  - keep fixes limited to `validate`, `ship`, and `retrospective` command behavior
- `WP1.1.4` PR `#278` docs/worklog/spec backfill follow-through
  - keep fixes limited to worklog, validation, spec, task, research, and plan files

Blockers:
- dirty `AgilePlus/main` remains a recovery surface and must not become the active edit lane again

### 1.2 heliosCLI active PR lanes

Objective:
keep all follow-up work on the already-open review lanes instead of the noisy root checkout.

Work packages:
- `WP1.2.1` compare PR `#182` against the clean `chore/governance-pr-ready` lane
- `WP1.2.2` keep root-checkout nested surfaces out of all review work
- `WP1.2.3` continue code-lane follow-up from the live PR branch, not from `main`

Blockers:
- root checkout still carries nested untracked surfaces and cannot be treated as the clean source of truth

### 1.3 heliosApp live lane

Objective:
keep the existing federation PR reviewable without opening a duplicate lane.

Work packages:
- `WP1.3.1` reconcile local drift around `deps-changelog.json` with PR `#362`
- `WP1.3.2` keep follow-up work on `feat/fix-typescript-vite-federation`
- `WP1.3.3` split only if governance or CI follow-up would otherwise mix with app changes

Blockers:
- local drift is small, but broadening the live PR unnecessarily would recreate mixed scope

### 1.4 cliproxyapi-plusplus live lane

Objective:
push PR `#942` until only external quota or broad repo debt remains.

Work packages:
- `WP1.4.1` clear branch-local CI and formatting failures
- `WP1.4.2` separate repo code debt from external billing or quota failures
- `WP1.4.3` stop once remaining failures are clearly outside the branch-local scope

Blockers:
- broad CI and package-graph failures
- external Snyk or billing noise

### 1.5 phenodocs small governance lane

Objective:
finish the small ruleset-baseline lane with minimal additional churn.

Work packages:
- `WP1.5.1` finish remaining checks on PR `#119`
- `WP1.5.2` keep the branch narrow and reviewable

Blockers:
- none visible locally on the current branch

## Phase 2: Mixed-State Repo Isolation

### 2.1 agentapi-plusplus root-noise triage

Objective:
separate root-checkout noise from the clean draft governance lane.

Work packages:
- `WP2.1.1` decide whether `docs/node_modules` deletions are intentional cleanup or install churn
- `WP2.1.2` keep draft PR `#438` focused on governance and workflow cleanup only
- `WP2.1.3` open a second cleanup branch only if the root-noise decision produces a clean isolated diff

Blockers:
- root `main` remains heavily polluted with tracked deletions and mixed edits

### 2.2 phenotype-infrakit surface isolation

Objective:
identify one true repo surface before any PR-prep resumes.

Work packages:
- `WP2.2.1` separate sibling shelf-path pollution from the real checkout
- `WP2.2.2` classify active versus stale worktrees
- `WP2.2.3` decide whether any live stash is recovery material or discardable noise

Blockers:
- repo-surface pollution still makes PR prep unsafe

## Phase 3: Metadata And Boundary Hygiene

### 3.1 thegent worktree and stash hygiene

Objective:
reduce stale metadata while preserving the separation between code and governance lanes.

Work packages:
- `WP3.1.1` classify `platforms/thegent-pr882` as prunable metadata or an active dependency
- `WP3.1.2` keep the active side lane distinct from governance cleanup
- `WP3.1.3` decide how to handle the main stash stack without collapsing current PR separation

Blockers:
- stale detached worktree metadata
- mixed stash history

### 3.2 cloud access hold

Objective:
preserve the prepared branch but stop work until access exists.

Work packages:
- `WP3.2.1` keep `cloud/chore/plan-sync-20260402` intact locally
- `WP3.2.2` verify no local diff is missing from that preserved branch
- `WP3.2.3` retry publish only after push rights for `Kilo-Org/cloud` change

Blockers:
- `403` on push prevents PR creation from the current auth context

### 3.3 koosha-portfolio boundary confirmation

Objective:
decide whether it becomes a tracked repo or remains an out-of-scope folder.

Work packages:
- `WP3.3.1` confirm repo boundary
- `WP3.3.2` initialize git only if it belongs on the shelf
- `WP3.3.3` add minimal governance and docs only after the boundary is real

Blockers:
- directory is not a git repo today

## Phase 4: Governance Enforcement Rollout

### 4.1 Active-repo required-check enforcement

Objective:
move the repos with stable PR lanes onto enforceable Git ruleset and CI discipline.

Work packages:
- `WP4.1.1` map exact required-check names from real CI runs
- `WP4.1.2` enforce no-force-push and no-`--no-verify` on protected lanes
- `WP4.1.3` require resolved review threads and passing CI before merge
- `WP4.1.4` preserve the billing-only exception path when jobs never start

Dependencies:
- Phase 1 repo lanes must be stable enough that rulesets will not block legitimate repair work

## Phase 5: AgilePlus Runtime And Manager Sync

### 5.1 AgilePlus event and tranche coverage

Objective:
make AgilePlus the live operator surface for the next tranche instead of relying on prose-only drift.

Work packages:
- `WP5.1.1` record real transition events for the active PR lanes
- `WP5.1.2` tie each active repo lane to a concrete feature or work package
- `WP5.1.3` mark blocked lanes by blocker type: local state, CI debt, access, or boundary

### 5.2 Shelf manager artifact upkeep

Objective:
keep the shelf-level readiness and queue views aligned with actual blocked and boundary state.

Work packages:
- `WP5.2.1` keep the local audit ledger in sync with current repo or worktree state
- `WP5.2.2` keep the next-target list synchronized to open PR reality
- `WP5.2.3` reflect `cloud` as access-blocked, `phenotype-infrakit` as recovery-first, and
  `koosha-portfolio` as a non-repo until that changes

## Critical Path

1. Close the active AgilePlus split PR tranche.
2. Reconcile `heliosCLI` and `heliosApp` on their live PR lanes.
3. Finish the branch-local portion of `cliproxyapi-plusplus`.
4. Isolate `agentapi-plusplus` and `phenotype-infrakit`.
5. Clean up `thegent` metadata and stash ownership.
6. Hold `cloud` until access changes and confirm the `koosha-portfolio` boundary.
7. Roll out enforceable governance and update AgilePlus runtime coverage.

## Immediate Next Five

1. Work AgilePlus PRs `#274`, `#275`, `#276`, and `#278` as the current split tranche.
2. Reconcile `heliosCLI` PR `#182` against the clean governance lane.
3. Normalize local scope on `heliosApp` PR `#362`.
4. Triage `agentapi-plusplus` root noise separately from draft PR `#438`.
5. Keep `cliproxyapi-plusplus` moving until only external or repo-wide debt remains.
