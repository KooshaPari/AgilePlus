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

## Constraints honored

- ✅ All 3 subagents read-only (no push, no merge, no file edit, no new PR, no comment, no force-push).
- ✅ No source code modified; only new evidence files in a sibling sub-folder.
- ✅ No new cheap_closes opened (cf-4 verdict was PASS — not a new CF).
- ✅ Operator action sheet sharpened, not inflated.
