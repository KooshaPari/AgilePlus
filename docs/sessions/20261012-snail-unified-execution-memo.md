# Unified Execution Memo -- AgilePlus CI-Green Campaign (2026-10-12)

## 1. Where we are

- main HEAD is `8ce7a446` -- PR #1114 (rustfmt fix in ship_command_flow.rs) is **landed**; CI green gate from the fmt fix is cleared.
- Spec branch `origin/spec/mature-product-contract-v2 @ 4e9f454b` is 267 commits ahead, source-backed (2.1.0), with 92 oracles specified-not-executed.
- 6 open PRs (#1099, #1097, #1096, #1095, #1090), 5 open issues; only (A) is on the critical path for S1.
- Release-lifecycle v2 audit just completed: 7 overlapping release paths, R1/R2/R3 HIGH-severity, all out of scope for this memo (separate audit, separate fix PRs).
- Two strategic MCQs (Q1 = spec-chain plan, Q2 = 92-oracle execution) remain unanswered. This memo integrates them.

## 2. Q1 -- spec-chain resolution

Four options; the merge shape is the variable.

| Opt | Name                                               | Time     | Trade                                                                                                                     |
| --- | -------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------- |
| A'' | Land #1099 first, then 3-way merge spec into main  | 2.5-4.5h | Highest-leverage code; lands fmt-fix + vercel.json catch-all + dashboard web in same arc; 1 squash-merge + 1 squash-merge |
| A'  | Skip #1099, do 3-way merge directly                | 1.5-2.5h | Faster; defers #1099 indefinitely                                                                                         |
| C   | Hold -- wait for spec branch refresh, do nothing   | variable | Zero progress; nightly stays red on linux; spec branch ages further                                                       |
| D   | Discard spec branch, re-derive from #1088 baseline | 2-3h     | Loses provenance (267 commits, 12 journeys, 92 oracles); ships empty schema 2.1.0 with 0 oracles                          |

**Recommendation: A''.** Largest merge but smallest _worktree-hop count_ (1 squash-merge + 1 squash-merge versus skipping #1099 and re-litigating it later). Restores `git show origin/main:spec/product/mature-contract.v1.json` to schema 2.1.0 source-backed.

**Dependencies on Q2:** A'' lands the **specified** contract only. It does **not** achieve S1 (which requires execution evidence). Any Q2 answer other than C requires Q1 to land first.

**Trigger if approved:** execute A'' in a fresh worktree from main @ `8ce7a446` per the pre-staged 7-step plan (force-push #1099 ref `15655f07` if needed; 3-way merge spec; resolve the 7 conflict files per pre-staged resolutions; cargo fmt cleanup on 6 spec-side files; local validation; push; open A' PR with `--admin --squash --delete-branch`).

## 3. Q2 -- 92-oracle execution scope

Four options; the execution timing is the variable.

| Opt | Name                                                                        | Time                           | Trade                                                                                                         |
| --- | --------------------------------------------------------------------------- | ------------------------------ | ------------------------------------------------------------------------------------------------------------- |
| A   | A' first, treat 92-oracle execution as a separate post-merge campaign       | 1-2 months follow-up           | Contract lands; S1-execution is a separate backlog entry; risk of "we said S1 but it isn't"                   |
| B   | A' first, then immediately start 92-oracle execution before declaring S1    | 1-2 months in same arc         | No false S1 claim; same total time; honest `accepted_mature_semantic_contract_runtime_evidence_pending` state |
| C   | Skip A' until a runner exists that can execute oracles                      | unbounded                      | Spec branch ages; contract stays `invalidated`; S0 stable, S1 never reached                                   |
| D   | Pilot 12 cases (12 journeys × 1 oracle each) as a subset first, then decide | 12-24h pilot + 1-2 months rest | Intermediate visibility signal; S1 declared "in progress" with 12/92 evidence                                 |

**Recommendation: B.** Same total time as A but no false S1 claim along the way. `mature_complete: false` is the correct state; execution evidence is what flips it. D is reasonable if the operator wants intermediate visibility (a third path, not a contradiction of B).

**Tier 0 pilot leverage (if B or D are approved):** 13.5h on 5 cases -- AP-VC-001, AP-RVC-040, AP-RVC-041, AP-RVC-042, AP-RVC-043 -- before committing to the long arc.

**Dependencies on Q1:** Q2=B/D assume Q1 lands first (you can't execute oracles against a contract that isn't on main). Q2=A is technically independent but pointless without Q1.

**Trigger if approved:** kick off tier-0 pilot in a worker lane (parallel with any Q1 PR work) and queue tier-1 cases behind it.

## 4. Cross-decision matrix (Q1 x Q2 = 16 cells)

Recommended cell: **A'' x B** (highlighted).

| Q1 \ Q2 | A: post-merge campaign             | B: same-arc execution                          | C: skip until runner                            | D: pilot then reconcile                  |
| ------- | ---------------------------------- | ---------------------------------------------- | ----------------------------------------------- | ---------------------------------------- |
| **A''** | OK (defer execution); default-safe | **RECOMMENDED** -- max honest throughput       | Inconsistent (lands contract but won't execute) | Acceptable -- pilot 12 cases after merge |
| **A'**  | OK (faster merge, defer execution) | OK -- same outcome, 1 squash-merge, lose #1099 | Inconsistent                                    | Acceptable                               |
| **C**   | Blocked -- contract not on main    | Blocked -- contract not on main                | Stalemate -- nothing happens                    | Blocked                                  |
| **D**   | Possible -- lose 267 commits       | Possible -- lose 267 commits                   | Possible -- loses more                          | Possible                                 |

**Bottom-line impact of recommended cell (A'' x B):**

- Total calendar: ~4-6h for merge + 1-2 months for oracles = ~6-8 weeks campaign arc.
- Throughput: 1 spec-content squash + 1 #1099 squash + 92 oracle evidence files + 1 S1 declaration.
- Honest state: contract = accepted_mature (with `runtime_evidence_pending`); S0 = true; S1 = false until tier-5 done.
- Risk: 92-oracle execution depends on the tulip runner being usable; if runner gaps surface, B degrades to D (pilot-first) without re-deciding Q1.

## 5. Recommended execution order (next 3 actions)

| #   | Action                                                                                                                                                                       | Effort | Owner                        | Dependency                |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ | ---------------------------- | ------------------------- |
| 1   | Force-push #1099 ref `15655f07` (refresh branch if needed)                                                                                                                   | 15-30m | git operator (swarm ghost)   | PR #1114 landed (CLEARED) |
| 2   | Fresh worktree from main @ `8ce7a446`; 3-way merge `origin/spec/mature-product-contract-v2`; apply 7 pre-staged conflict resolutions; cargo fmt cleanup on 6 spec-side files | 2-3h   | merge operator (swarm slate) | Action 1                  |
| 3   | Local validation: `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy`; push; open A' PR with `--admin --squash --delete-branch`                                 | 1-1.5h | merge operator (swarm slate) | Action 2 green            |

**Parallel track (if Q2=B is approved):** spawn tier-0 pilot worker (5 cases, 13.5h) as soon as Action 2 produces a buildable merged tree. Pilot results decide tier-1 dispatch.

## 6. Top 3 risks (recommended cell)

| #   | Risk                                                                                                  | Severity | Mitigation                                                                                                                                                                   |
| --- | ----------------------------------------------------------------------------------------------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| R1  | Spec-side `cargo fmt --check` reveals more than 6 issues (pre-staged list may be incomplete)          | MED      | Run `cargo fmt --check` on a fresh `git checkout origin/spec/mature-product-contract-v2` first (5min); expand the 6-file list before the merge, not after                    |
| R2  | `mature-contract.v1.json` schema 1.0→2.0 lands but `agileplus validate` returns != 10/10 (regression) | MED      | Run `agileplus validate` before opening A' PR; if <10/10, open a follow-up PR with the schema-fix rebase instead of merging a green-but-broken PR                            |
| R3  | 92-oracle tier-0 pilot surfaces a runner gap (no executable harness for the spec format)              | HIGH     | Pre-check tulip runner availability in a 30min scout worker _before_ committing to B; if runner is unusable, downgrade Q2=B to Q2=D (pilot 12 cases manually, defer tier-1+) |

## 7. Open questions for operator

- (none blocking for the A'' plan itself) -- the pre-staged 7-step plan and the 92-oracle tier breakdown are complete.
- Should #1095 (release-closure-20261007) and #1090 (spec rust) be closed-as-subsumed by the A' PR, or kept open for separate work? This decision affects PR-comment hygiene but not the merge.
- Is a tier-0 runner-scout acceptable as a precondition for the Q2=B trigger, or should the pilot be skipped and tier-1 dispatched blind?

_Ledger: tx-agent: jcode, tx-validated: manual, tx-scope: cockpit-aggregated-memo, tx-intent: Unified execution memo integrating Q1 (spec-chain) and Q2 (92-oracle) for the AgilePlus CI-green campaign_
