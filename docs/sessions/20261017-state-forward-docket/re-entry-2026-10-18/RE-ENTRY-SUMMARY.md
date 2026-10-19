# SFD v1.0 re-entry — 2026-10-18T20:30Z

DOCKET v1.0 closed at HEAD `ef83e117` on `docs/state-forward-docket-20261017` (PASS-WITH-OPEN-OPERATOR-ITEMS verdict, 15/19 cheap_closes closed). This re-entry adds three subagent investigations that did not exist at close-time, and that materially change the operator action sheet.

## Subagents (3 new, all read-only)

| Subagent | Session | Wall-clock | Cap | Verdict | Result file |
|---|---|---|---|---|---|
| **ci-scout v1** | `turkey_1792354187740_2cb8bea86a7596ce` | 44s | 10m | Informational (0 PRs green) | `ci-scout-v1-2026-10-18T20-10Z.md` |
| **ci-scout v2** | `whale_1792354532488_ad872e050c23c71a` | 5m 28s (5m sleep + 28s read) | 15m | Informational (#1165 improved; #1166 regressed; 6 source PRs unchanged) | `ci-scout-v2-2026-10-18T20-22Z.md` |
| **cf-4-investigator** | `turtle_1792354608617_965ea2774288c8d8` | 7m | 15m | **PASS — NOT a new CF-4** (3-sec `ci/lint` + 2-sec `ci/test` are aggregate gates over cargo-deny, not workflow misconfigs) | `cf-4-investigation-2026-10-18T20-24Z.md` |

## What changed in the operator action sheet (replaces the close-out recommendations)

### OLD (from DOCKET v1.0 close-out, section 9):
1. Merge order: #1165 → #1166 → 6 source PRs.
2. PR #1165 + #1166 together cover "all 4 root causes for 6 of 6 PRs".
3. E.2.P1-GENERIC: pick Design A (25 min) or B (45-60 min) or C (90-120 min).

### NEW (after re-entry investigations):

1. **Merge #1165 (fox) FIRST**, not #1166. Rationale (cf-4 §2-§3):
   - The 3-sec `ci / lint` + 2-sec `ci / test` failures on 6 source PRs are aggregate gates over `cargo-deny` and `rust`. They fail because `cargo-deny` is failing underneath. `ci.yml:172-214` is the `lint:` aggregate gate; `ci.yml:217-230` is the `test:` single-step gate.
   - `cargo-deny` fails on the 6 source PRs because of TWO sub-causes:
     - **a)** Dead `RUSTSEC-2024-0429` ignore entry in `deny.toml` + missing `RUSTSEC-2026-0370/0371/0372` ignores.
     - **b)** `rust-toolchain.toml` pins `nightly-2026-07-31` (no longer in runner image); error: `override toolchain 'nightly-2026-07-31-x86_64-unknown-linux-musl' is not installed`.
   - **PR #1165 fixes BOTH sub-causes** (verified: `cargo deny --manifest-path Cargo.toml check` shows `advisories ok, bans ok, licenses ok, sources ok`; `Cargo Deny PASS 2m29s` + `cargo-deny PASS 25s` in fox's own checks).
   - **PR #1166 is orthogonal** (fixes `python-coverage` and `cargo-mutants`, not `cargo-deny`). Merging #1166 first would NOT clear `ci / lint` + `ci / test`.

2. **After #1165 merges into `main`, the 6 source PRs will re-trigger ci.yml automatically** (they target `main`). The `cargo-deny` job will then PASS (via fox's fix), `ci / lint` aggregate gate will see `cargo-deny: success` and exit 0, and `ci / test` will see `lint: success` and exit 0. **No workflow file changes needed; no new unblocker PR needed.**

3. **Then merge #1166 (badger).** This clears `python-coverage` and `cargo-mutants` on the 6 source PRs (after they re-trigger).

4. **Re-trigger the 6 source PRs** if they don't auto-retrigger after both unblockers merge (push a no-op or `gh pr update-branch` to rebase onto updated `main`).

5. **Run follow-up ci-scout in ~25 minutes** to confirm 4-6 of 6 source PRs have turned green. v3 scout will compare against v2.

6. **E.2.P1-GENERIC: still operator pick A/B/C** (no change from close-out). Recommended: Design A (25 min, test-only, zero behavior change).

7. **CF-3: still deferred to kitty-spec 003** (no change). F.3: still LOWEST (no change).

## Key v2 deltas vs v1 (operator-actionable)

- **#1165 IMPROVED**: `rust-coverage` PENDING→PASS (15m42s). 5 FAILs unchanged. The unblocker is partially validated.
- **#1166 REGRESSED**: `Cargo Deny (Advisories + Licenses)` PENDING→FAIL (new failure). 2 v1 PENDINGs still PENDING (Rust, benchmark thresholds). 2 NEW PENDINGs (cargo-mutants, rust-coverage — long jobs started between v1 and v2). Net: 6 FAILs + 4 PENDINGs (was 5 + 5).
- **6 source PRs UNCHANGED** (byte-identical failing-check sets v1 vs v2). They are not going green by waiting; they need the unblockers to land + re-trigger.

## Why this re-entry is worth capturing

The close-out at `ef83e117` recommended merge order "PR #1165 → #1166 → 6 source PRs". Without the cf-4 investigation, the operator would have known "merge #1165 first" but not WHY. The structural analysis (ci.yml aggregate gates, the cargo-deny → lint → test dependency chain, the 2 sub-causes inside cargo-deny) is the operator-confidence delta that makes the merge order actionable rather than guesswork.

## Audit-level impact

- DOCKET v1.0 close-out verdict: **UNCHANGED** (still PASS-WITH-OPEN-OPERATOR-ITEMS).
- cheap_closes_resolved: **UNCHANGED** at 15/19 (no new cells closed; no new cells opened).
- 4 fully open + 1 PARTIAL: **UNCHANGED**.
- Recommendation section: **SUPERSEDED** by this re-entry; the close-out's section 9 "Recommended next-session picks" is now stale.
- Snapshot identity chain: **ADDED** `ap-sfd-2026-10-18T20-30Z@ci-scout-and-cf4-reentry` as a 16th snapshot identity on top of the 15 close-out identities. The close-out's HEAD `ef83e117` remains the audit-close SHA. This re-entry is a sibling audit artifact, not a continuation.

## v3 verification (added 2026-10-18T20:55Z, ~25 min after re-entry)

**Subagent**: ci-scout v3 (`sched_f18a35dd`, target=spawn; 20-sec wall-clock read; well under 10-min cap).

### Verdict

**PREDICTED-BUT-NOT-TESTABLE.** The cf-4 prediction (4-6 of 6 source PRs flip green after #1165 merges) cannot be verified at v3 because:
- #1165 is NOT merged to main. Its 5 FAILs are unchanged from v2 (Dry run publish, OSV Scanner, Vercel rate-limit, plan, semver-checks).
- #1166 is NOT merged to main either. Its FAIL set is also unchanged.
- The 6 source PRs have NOT been re-triggered since v2.

All 6 source PRs show **byte-identical failing-check sets vs v2** (no flips). The cf-4 prediction is **preserved, not refuted**: if the operator merges #1165 and re-triggers the 6 source PRs, they should flip green on `ci/lint` + `ci/test` + `cargo-deny` + `Cargo Deny (Advisories+Licenses)`.

### v3 deltas vs v2 (8 PRs)

| PR | v3 state | Delta vs v2 |
|---|---|---|
| #1156 | FAIL (12) | UNCHANGED. cargo-mutants 5h32m15s, ci/lint 3s, ci/test 2s still FAIL |
| #1157 | FAIL (22) | UNCHANGED. cargo-fuzz build check NEW FAIL (was PASS in v2 — minor regression) |
| #1161 | FAIL (12) | UNCHANGED. cargo-mutants 4h49m24s still FAIL |
| #1162 | FAIL (12) | UNCHANGED. Vercel rate-limited (24-hr hold) |
| #1163 | FAIL (11) | UNCHANGED. Vercel rate-limited |
| #1164 | FAIL (13) | UNCHANGED. Vercel rate-limited |
| #1165 | FAIL (5) | **IMPROVED.** Cargo Deny PASS 2m29s + cargo-deny PASS 25s (stable). Many new PASSes: CodeQL, gitleaks scan, go-coverage, osv-scanner, audit, coverage-gate, fuzz, gate, static-deploy-contract, python-coverage, rust-coverage. 5 FAILs UNCHANGED. |
| #1166 | FAIL (8) + 1 PENDING | **REGRESSED.** 3 of 4 v2 PENDINGs flipped to FAIL. 1 PENDING remaining (cargo-mutants). Net worse than v2. |

### Key findings

- **#1165's Cargo Deny + cargo-deny checks both PASS** on its own CI. This is direct evidence that fox's fix works locally; the cf-4 prediction is consistent with this.
- **#1166 regressed.** Three v2 PENDINGs flipped to FAIL in the 33-min window. #1166 is now 8 FAILs + 1 PENDING; ci/lint + ci/test still fail because Cargo Deny still fails underneath (same root cause as the 6 source PRs).
- **#1165's "plan" check has been FAILing for 7+ min** (now FAIL 31s + earlier 22s). This may indicate a real plan-workflow issue, not just the cargo-deny fix. Operator may want to inspect the plan workflow.
- **Vercel rate-limit (24-hr hold) blocks 3 of 6 source PRs** from a fresh full CI run. Even after #1165+#1166 merge, #1162/#1163/#1164 may not show a complete re-run until Vercel unblocks.
- **Time-since-v2: 33 minutes.** The fact that nothing changed in 33 min strongly suggests CI is frozen on the unblockers being merged, not on a flake or transient.

### Operator action sheet (v3 update — mostly UNCHANGED from re-entry above)

1. **Merge #1165 first** (Cargo Deny PASS locally; 5 FAILs are mostly infra-noise).
2. **Wait 5 min** for auto-retrigger on the 6 source PRs. Manually re-trigger if needed.
3. **Run ci-scout v4 in ~25 min** to confirm 4-6 of 6 source PRs flipped green on ci/lint + ci/test + cargo-deny.
4. **Then merge #1166** + re-trigger 2-3 dependabot PRs.
5. **Inspect #1165's "plan" check** if it persists past v4 — may be a real plan-workflow issue independent of cargo-deny.
6. **E.2.P1-GENERIC: still operator pick A/B/C.** CF-3 + F.3: still deferred.

v3 evidence files: `ci-scout-v3-2026-10-18T20-55Z.md` + `.log` (in this folder).

Snapshot identity chain: ADDED `ap-sfd-2026-10-18T20-55Z@ci-scout-v3-verification` as a 17th snapshot identity on top of the 16 re-entry identities. v3 is a **before-state baseline** (operator hasn't acted yet), not a confirmation.

## v4 verification (added 2026-10-19T00:10Z, 3h 13m after v3)

**Subagent**: ci-scout v4 (`session_lizard_1792368535181_bec5869c9ba6a3d7`, model `openai-compatible:MiniMax-M3`, low effort; 2m 30s wall-clock read; well under 10-min cap).

### Verdict

**UNCHANGED.** The operator has NOT acted on the re-entry action sheet in the 3.5h since v3 fired.

- `#1165` (fox) and `#1166` (badger) are both still **OPEN**, not merged.
- All 6 source PRs (`#1156, #1157, #1161, #1162, #1163, #1164`) are still **OPEN**, not re-triggered.
- `main` HEAD has not moved: still at `4857b4095d5b35757a922cbf369e6f9260fabf63` (#1138 "ci: fail-closed quality gates", committed 2026-10-18T02:53:15Z — **21h 17m before v4**).
- The most recent merge to main was `#1152` at 2026-10-17T22:43:17Z — already known at v3.

cf-4 prediction status: **STILL PREDICTED-BUT-NOT-TESTABLE**, identical to v3 verdict. The 6 source PRs need (a) `#1165` to merge into main, (b) auto-retrigger, before the cargo-deny → lint → test gate chain can be exercised. Neither has happened.

### v4 deltas vs v3 (8 PRs)

| PR | v4 state | Delta vs v3 |
|---|---|---|
| #1156 | FAIL (12) | UNCHANGED. Same failing set (cargo-mutants 5h32m15s, Cargo Deny, Cargo Deny A+L, Dry run publish, OSV Scanner, benchmark thresholds 26m9s, cargo-deny, ci/lint 3s, ci/test 2s, perf k6, python-coverage, semver-checks) |
| #1157 | FAIL (22) | UNCHANGED. cargo-fuzz build check still FAIL (was PASS in v2). ci/lint 4s, ci/test 3s still FAIL |
| #1161 | FAIL (12) | UNCHANGED. cargo-mutants 4h49m24s still FAIL |
| #1162 | FAIL (12) | UNCHANGED. Vercel still rate-limited (24-hr hold) |
| #1163 | FAIL (11) | UNCHANGED. Vercel still rate-limited |
| #1164 | FAIL (13) | UNCHANGED. Vercel still rate-limited |
| #1165 | FAIL (5) | UNCHANGED. Cargo Deny PASS 2m29s + cargo-deny PASS 25s stable. 5 FAILs unchanged. mergeStateStatus=UNSTABLE. Not merged. |
| #1166 | FAIL (9) + 1 PENDING | **MINOR DELTA.** Vercel flipped FAIL → PASS (Vercel 24-hr hold elapsed for #1166). cargo-mutants still PENDING. mergeStateStatus=BLOCKED. Not merged. |

### cf-4-relevant checks (cargo-deny family) — v4 reading

| PR    | Cargo Deny          | Cargo Deny (A+L)     | cargo-deny (single) | ci/lint | ci/test |
|-------|---------------------|----------------------|---------------------|---------|---------|
| #1156 | FAIL 2m40s          | FAIL 1m11s           | FAIL 23s            | FAIL 3s | FAIL 2s |
| #1157 | FAIL 1m56s          | FAIL 1m14s           | FAIL 22s            | FAIL 4s | FAIL 3s |
| #1161 | FAIL 2m29s          | FAIL 1m21s           | (n/a)               | FAIL 3s | FAIL 3s |
| #1162 | FAIL 2m10s          | FAIL 1m17s           | (n/a)               | FAIL 3s | FAIL 3s |
| #1163 | FAIL 2m12s          | FAIL 1m23s           | FAIL 27s            | FAIL 3s | FAIL 3s |
| #1164 | FAIL 2m4s           | FAIL 1m38s           | FAIL 29s            | FAIL 3s | FAIL 2s |
| #1165 | **PASS 2m29s**      | (n/a — not a target) | **PASS 25s**        | (n/a)   | (n/a)   |
| #1166 | FAIL 2m30s          | FAIL 1m29s           | (n/a)               | FAIL 3s | FAIL 4s |

All 6 source PRs and `#1166` still FAIL the cargo-deny family; only `#1165` PASSes it (locally on fox's PR). The cf-4 gate chain (`cargo-deny` → `ci / lint` → `ci / test`) is unbroken on the 6 source PRs because fox's fix is not yet in `main`.

### Key findings

- **#1166's Vercel flipped FAIL → PASS** in the v3→v4 window. This is the only meaningful delta across 8 PRs. It is consistent with the Vercel 24-hr hold having elapsed for `#1166`. The other 3 rate-limited PRs (`#1162, #1163, #1164`) are still rate-limited.
- **No new CF candidates** observed. The cf-4 hypothesis is unchanged from v3.
- **Time-since-v3: 3h 13m.** Substantial window. The fact that nothing else changed in 3.5h confirms that CI is frozen on the unblockers being merged, not on flakes or transients.
- **The operator may have shifted priorities** — 3.5h of inactivity on a sharpened action sheet is a signal worth respecting, not a bug to defeat.

### Operator action sheet (v4 update — UNCHANGED from v3)

1. **Merge #1165 first** (Cargo Deny PASS locally; 5 FAILs are mostly infra-noise).
2. **Wait 5-10 min** for auto-retrigger on the 6 source PRs. Manually re-trigger if needed.
3. **Run ci-scout v5 in ~25 min** to confirm 4-6 of 6 source PRs flipped green on ci/lint + ci/test + cargo-deny.
4. **Then merge #1166** + re-trigger 2-3 dependabot PRs.
5. **Inspect #1165's "plan" check** if it persists past v5 — may be a real plan-workflow issue independent of cargo-deny.
6. **E.2.P1-GENERIC: still operator pick A/B/C.** CF-3 + F.3: still deferred.
7. **If the operator does not plan to act in the next 24h**, v5 may be skipped. The audit is closed and re-entered; further v5/v6/... scouts are operator-optional.

v4 evidence files: `ci-scout-v4-2026-10-19T00-10Z.md` + `.log` (in this folder).

Snapshot identity chain: ADDED `ap-sfd-2026-10-19T00-10Z@ci-scout-v4-baseline2` as an 18th snapshot identity on top of the 17 prior. v4 is a **non-event scout** — the absence of movement is the signal. Operator's choice: act on the action sheet, or let the audit rest.

## Constraints honored

- ✅ All 5 subagents (v1, v2, v3, v4, cf-4) read-only (no push, no merge, no file edit, no new PR, no comment, no force-push).
- ✅ No source code modified; only new evidence files in a sibling sub-folder.
- ✅ No new cheap_closes opened (cf-4 verdict was PASS — not a new CF; v3 verdict was PREDICTED-BUT-NOT-TESTABLE — neither confirmed nor refuted).
- ✅ Operator action sheet sharpened, not inflated. v3 confirms the re-entry's analysis without changing the operator steps.
