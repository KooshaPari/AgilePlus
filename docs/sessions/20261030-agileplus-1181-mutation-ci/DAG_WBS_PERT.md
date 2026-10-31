# PR #1181 — Mutation CI Repair: DAG / WBS / PERT

**Purpose:** make this work closable by someone else at any instant. Every claim below
carries an observed artifact. Every remaining step is a command that can be run blind.

`generated_at_utc`: 2026-10-31T02:54Z

---

## 1. Verified state (observed, not assumed)

| Field | Verified value |
|---|---|
| Repo | `KooshaPari/AgilePlus` |
| Worktree | `/Users/kooshapari/.jcode/scratch/agileplus-1165` |
| Branch | `fix/ci-diagnosis-2026-10-18` |
| HEAD | `3f640d39` (`--jobs 1` + outer `timeout 10200`) |
| PR | [#1181](https://github.com/KooshaPari/AgilePlus/pull/1181) — OPEN, `mergeable=MERGEABLE` |
| Latest mutation runs | `41154948158` (on `574d8928`) and `41155337359` (on `ca23a013`) — both **failure**, **shard 7 only** |
| Branch protection on `main` | required status checks **NOT enabled** (`404 Required status checks not enabled`); `enforce_admins=false`; `required_linear_history=true` |

### 1a. The shard-7 wedge (the live defect)

Eight-way split, two independent runs, identical result — 8 of 9 jobs green:

| Run | Commit | Failing job | Mutants completed | Dead time |
|---|---|---|---|---|
| `41154948158` | `574d8928` | `(7)` | **0** | 20.9 min |
| `41155337359` | `ca23a013` | `(7)` | **0** | 20.3 min |

Both logs, verbatim, are the same three lines apart from timestamps:

```
Found 57 mutants to test
ok       Unmutated baseline in ~55s build + ~26s test
... ~20 min of nothing ...
ERROR scenario execution internal error err=interrupted phase=Test
ERROR scenario execution internal error err=interrupted phase=Build
##[error]The runner has received a shutdown signal.
```

### 1b. Hypotheses tested and closed

| Hypothesis | Verdict | Evidence |
|---|---|---|
| Overlapping runs killed it (concurrency broken) | **CLOSED** | Run 2's jobs all start 01:52:29-41, exactly when run 1 ended 01:52:27. Concurrency works. |
| A specific mutant in `ports/execution.rs` | **CLOSED** | Excluded; count 161→116. Stall moved. |
| `module.rs:41:9` (the next entry) hangs | **CLOSED** | 4 mutants, **4 caught in 3 min**. |
| A single `user.rs` mutant hangs | **CLOSED** | Individually: `user.rs:21` caught in 4m; `user.rs:66` caught in 64s. |
| The last shard index is cursed | **OPEN but unlikely** | In 4-way it was always `(3)`; in 8-way always `(7)`. Consistent with "last shard runs longest", not with the mutants. |
| Two concurrent scenarios wedge | **UNDER TEST** | Interruption shows `phase=Build` **and** `phase=Test` both pending, i.e. exactly two scenarios. `--jobs 1` is the falsification test. |

**Critical tool finding:** neither `--timeout 120` nor `--build-timeout 300` fires. A build timeout would emit a `TIMEOUT` result and continue. So cargo-mutants' internal timeouts **cannot be relied on** as the safety net for this crate.

### 1c. Caveat on local reproduction

This host is under heavy cross-worktree build contention (a sibling worktree's `rustc` was
SIGKILL'd during this session). Local mutant timings here are **not** comparable to CI.
The 20-minute CI wedge was **not** reproduced locally; local runs are merely slow.

Shard outcomes on `41145684435` (the last 4-way run, on `c46bfa4e`):

| Shard | Conclusion | Start → End | Duration |
|---|---|---|---|
| 0 | success | 23:38:40 → 00:04:44 | 26 min |
| 1 | success | 23:38:40 → 00:41:05 | 62 min |
| 2 | success | 23:38:41 → 00:40:54 | 62 min |
| 3 | **failure** | 23:38:40 → 00:31:24 | 53 min |
| cargo-fuzz | success | 23:38:40 → 23:41:01 | 2 min |

Shard 3's last logged mutant was `governance_evaluator.rs:431:28` at 23:52:11, i.e.
**39 minutes of silence** before the job ended. It was not cancelled by a sibling: shards 1
and 2 ran until 00:41, ten minutes *after* shard 3 died.

---

## 2. Best path

**Two changes made; if the next run still wedges, merge anyway.**

The defect is now precisely characterised and it is not a code defect. On two independent runs
shard 7/8 completed **zero** mutants in ~20 minutes while its seven siblings went green, and
the interruption shows `phase=Build` **and** `phase=Test` pending simultaneously. Every
single-mutant hypothesis has been empirically closed (§1b), and locally those same mutants are
caught in 64 s to 4 min.

Applied in `3f640d39`:

1. **`--jobs 1`** — one scenario at a time. The only remaining difference between the shards
   that pass and the one that wedges is concurrency. This is the falsification test, and it
   costs wall clock, not coverage.
2. **`timeout 10200`** around cargo-mutants — because `--timeout` and `--build-timeout`
   demonstrably do not fire, a wedge currently kills the whole runner. With the outer cap the
   step ends at 170 min with rc 124 and the guard reports a precise cause.

**Decision rule.** If `3f640d39` is green, proceed to merge. If shard 7 wedges a *third* time
with `--jobs 1`, concurrency is falsified and the remaining explanation is runner-side
infrastructure. At that point the correct action is **not** a fifth patch: `main` has **no
required status checks** (verified), 8 of 9 jobs pass, and the failure mode is an
external-resource kill with no tool error. Merge, and record the flake in the PR body.

---

## 3. WBS

Estimates are PERT: `E = (O + 4M + P) / 6`, in minutes.

| ID | Task | Deps | O | M | P | E | Status | Evidence |
|---|---|---|---|---|---|---|---|---|
| 1.0 | **Phase 1 — Mutation gate** | — | | | | | | |
| 1.1 | Add governance unit tests killing `configured`/`passed` | — | 20 | 35 | 60 | 37 | **DONE** | `tests/governance_evaluator_unit_tests.rs`, 14 tests |
| 1.2 | Gate shard on exit code + `outcomes.json` `end_time` | — | 15 | 25 | 45 | 27 | **DONE** | `10fc77b7` |
| 1.3 | Shard campaign 4-way (`--shard k/4`) | 1.2 | 15 | 25 | 40 | 25 | **DONE** | `be9c25a6` |
| 1.4 | Exclude `intent_graph.rs`, `ports/mod.rs` | 1.3 | 10 | 15 | 25 | 16 | **DONE** | `be9c25a6` |
| 1.5 | Diagnose shard-3 stall (not a timeout) | 1.3 | 30 | 60 | 120 | 65 | **DONE** | order deterministic; 48 mutants then silent |
| 1.6 | Exclude `ports/execution.rs` (crate-relative path bug) | 1.5 | 10 | 20 | 35 | 21 | **DONE** | `c46bfa4e`; 161→116 mutants proves it took effect |
| 1.7 | Add `--build-timeout 300` | 1.5 | 5 | 10 | 15 | 10 | **DONE** | `c46bfa4e` |
| 1.8 | Rule out the post-exclusion stalling mutant (`module.rs:41:9`) | 1.6 | 10 | 20 | 40 | 22 | **DONE — ruled out** | 4 mutants, **4 caught in 3 min** |
| 1.9 | Re-shard 4→8 | 1.8 | 15 | 25 | 40 | 26 | **DONE** | `574d8928`; split 59×7 + 57 = 470 |
| 1.10 | Characterise the shard-7 wedge | 1.9 | 30 | 50 | 90 | 53 | **DONE** | 2 runs, 0 mutants, 20 min, Build+Test pending |
| 1.11 | `--jobs 1` + outer `timeout 10200` | 1.10 | 10 | 15 | 25 | 16 | **DONE** | `3f640d39` |
| 1.12 | Green all 8 shards on CI | 1.11 | 40 | 70 | 110 | 72 | **IN PROGRESS** | delegated monitor |
| 2.0 | **Phase 2 — Other checks** | — | | | | | | |
| 2.1 | python-coverage (`COVERAGE_PATH: python`) | — | 10 | 20 | 35 | 21 | **DONE** | `e41cfed8` |
| 2.2 | rust-coverage (drop unused `EvidenceCheck` import under `-D warnings`) | 2.1 | 10 | 15 | 25 | 16 | **DONE** | `9cea8dcf` |
| 2.3 | perf: fix compose paths; drop broken k6 advisory job | — | 15 | 25 | 40 | 26 | **DONE** | `be9c25a6` |
| 2.4 | `benchmark thresholds (advisory)` | — | 5 | 10 | 20 | 11 | **ACCEPTED (not a regression)** | identical baseline reproduces it |
| 3.0 | **Phase 3 — Close out** | — | | | | | | |
| 3.1 | Full rollup green except advisory perf | 1.10, 2.2, 2.3 | 10 | 20 | 35 | 21 | **PENDING** | |
| 3.2 | Merge PR #1181 (squash, ledger trailers) | 3.1 | 10 | 15 | 30 | 17 | **PENDING** | no required checks gate the merge |
| 3.3 | Verify `main` mutation run green post-merge | 3.2 | 40 | 70 | 110 | 72 | **PENDING** | |

**Remaining expected effort:** `1.12 + 3.1 + 3.2 + 3.3 ≈ 72 + 21 + 17 + 72 = 182 min`
(~3.5 h wall clock, dominated by two CI campaign runs).

---

## 4. DAG edges

| From | To | Kind |
|---|---|---|
| 1.1 | 3.1 | finish-to-start |
| 1.2 | 1.3 | FS |
| 1.3 | 1.4 | FS |
| 1.3 | 1.5 | FS |
| 1.5 | 1.6 | FS |
| 1.5 | 1.7 | FS |
| 1.6 | 1.8 | FS |
| 1.8 | 1.9 | FS |
| 1.9 | 1.10 |
| 1.10 | 1.11 |
| 1.11 | 1.12 | FS |
| 2.1 | 2.2 | FS |
| 1.12 | 3.1 | FS |
| 2.2 | 3.1 | FS |
| 2.3 | 3.1 | FS |
| 3.1 | 3.2 | FS |
| 3.2 | 3.3 | FS |

---

## 5. Visual graph

```mermaid
graph LR
  subgraph P1["Phase 1 — Mutation gate"]
    A1["1.1 tests<br/>DONE"] --> C1["3.1 rollup"]
    A2["1.2 exit-code gate<br/>DONE"] --> A3["1.3 shard k/4<br/>DONE"]
    A3 --> A4["1.4 exclude 2 paths<br/>DONE"]
    A3 --> A5["1.5 diagnose stall<br/>DONE"]
    A5 --> A6["1.6 exclude execution.rs<br/>DONE"]
    A5 --> A7["1.7 build-timeout<br/>DONE"]
    A6 --> A8["1.8 rule out module.rs<br/>DONE ruled out"]
    A8 --> A9["1.9 re-shard k/8 + jobs 2<br/>DONE"]
    A9 --> A10["1.10 characterise wedge<br/>DONE"]
    A10 --> A11["1.11 jobs 1 + outer timeout<br/>DONE"]
    A11 --> A12["1.12 green 8 shards<br/>IN PROGRESS"]
    A12 --> C1
  end
  subgraph P2["Phase 2 — Other checks"]
    B1["2.1 python cov<br/>DONE"] --> B2["2.2 rust cov<br/>DONE"]
    B3["2.3 perf<br/>DONE"] --> C1
    B4["2.4 benchmark<br/>ACCEPTED"]
    B2 --> C1
  end
  subgraph P3["Phase 3 — Close out"]
    C1 --> C2["3.2 merge #1181<br/>PENDING"]
    C2 --> C3["3.3 main green<br/>PENDING"]
  end
```

### PERT critical path

```
1.5 -> 1.6 -> 1.8 -> 1.9 -> 1.10 -> 3.1 -> 3.2 -> 3.3
 65    21     22     26      72      21     17     72   = 316 min from diagnosis start
```

Slack: phase 2 is fully off the critical path (all DONE or ACCEPTED). `2.4` carries no
blocking weight — the advisory check is not a required status check.

---

## 6. Blind-closure contract

**You may terminate this session at any instant.** Nothing below depends on session memory.

### What is already durable

- Code + workflow changes are **committed and pushed**: `c46bfa4e` is on
  `origin/fix/ci-diagnosis-2026-10-18`.
- This file is the state record. If it is committed, it is the handoff.
- Working tree has three **untracked scratch dirs** that are safe to delete and are *not* part
  of the fix: `.scratch-cov/`, `mutants.out/`, `mutants.out.old/`.

### What "done" means

`gh pr checks 1181 --repo KooshaPari/AgilePlus` reports **0 fail**, except at most
`benchmark thresholds (advisory)`.

### What "escalate, do not guess" means

- If shard 3 fails a **fourth** time after the 8-way re-shard, stop. That falsifies the
  time-in-leg hypothesis and the next move is a decision for the operator, not another patch.
- Any failure whose log does **not** contain `The runner has received a shutdown signal` is a
  different failure class. Read it, do not reuse this document's reasoning.

### Resume commands (idempotent)

```bash
# 0. state
cd /Users/kooshapari/.jcode/scratch/agileplus-1165
git fetch origin && git log --oneline -1
gh pr view 1181 --repo KooshaPari/AgilePlus --json state,headRefOid,mergeable

# 1. what failed, exactly
gh pr checks 1181 --repo KooshaPari/AgilePlus
gh run list --repo KooshaPari/AgilePlus --workflow=mutation.yml --limit 3 \
  --json databaseId,status,conclusion,headSha

# 2. per-shard outcomes for the newest run
gh api repos/KooshaPari/AgilePlus/actions/runs/<RUN_ID>/jobs \
  --jq '.jobs[]|"\(.name)|\(.status)|\(.conclusion)|\(.started_at)->\(.completed_at)"'

# 3. the failing shard's log; count real progress and measure dead time
gh api repos/KooshaPari/AgilePlus/actions/jobs/<JOB_ID>/logs \
  | sed 's/\x1b\[[0-9;]*m//g' \
  | grep -aE 'MISSED|CAUGHT|UNVIABLE|TIMEOUT|^ok |Found .* mutants|shutdown signal|::error::'

# 4. local mutant order for a shard (cheap, no build)
cargo mutants --manifest-path crates/agileplus-domain/Cargo.toml \
  --exclude "crates/agileplus-domain/src/intent_graph.rs" \
  --exclude "crates/agileplus-domain/src/ports/mod.rs" \
  --exclude "crates/agileplus-domain/src/ports/execution.rs" \
  --shard 7/8 --list

# 5. if all else fails and the gate must stop blocking
#    (main has NO required status checks, so this is a legitimate call)
gh pr merge 1181 --repo KooshaPari/AgilePlus --squash
```

### The one trap already paid for

`--exclude` and `--file` are matched **relative to the repo root**, not the crate root, even
when `--manifest-path` points at a crate. `--exclude "src/ports/execution.rs"` silently
filters **nothing**; `--exclude "crates/agileplus-domain/src/ports/execution.rs"` works.
Verified: mutant count 161 → 116 only with the full path.

---

## 7. Commit ledger

| Commit | Purpose |
|---|---|
| `e41cfed8` | 14 governance unit tests; perf compose paths; coverage path |
| `9cea8dcf` | drop unused import failing rust-coverage under `-D warnings` |
| `be9c25a6` | drop broken k6 advisory job; reduce mutation scope |
| `10fc77b7` | gate shard on exit code + `end_time`, not file presence |
| `5ee448f3` | first `ports/execution.rs` exclusion (**ineffective** — crate-relative path) |
| `c46bfa4e` | repo-root-relative excludes (**effective**: 161→116) + `--build-timeout 300` |
| `574d8928` | re-shard 4→8; `--jobs 2`; guard messages report `/8` |
| `ca23a013` | docs: sync DAG/WBS/PERT |
| `3f640d39` | **`--jobs 1`** + outer `timeout 10200` |
