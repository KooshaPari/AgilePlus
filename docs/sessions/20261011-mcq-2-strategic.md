est# MCQ: 2 strategic decisions for the AgilePlus CI-green campaign

_Prepared 2026-10-11 21:08 PDT. Review and answer at your pace. The fix/rustfmt-ship-command-flow PR will open once you approve the inbox item; this MCQ is for the next-tier decisions._

---

## Q1 — #1110 spec-chain resolution (the campaign's central question)

**Context:** the 4-day-old spec branch (`origin/spec/mature-product-contract-v2`, 267 commits ahead) carries the source-backed mature contract (12 journeys, 92 oracles, source-backed manifest schema 2.1.0). The 3-way merge into main has 7 conflict files (all pre-staged with resolutions), 32 auto-merge clean, 81 added-in-remote (including STABLE.md, which merges cleanly). The merged tree will need a final `cargo fmt` pass (6 spec-side files have format issues; main-side issue is now fixed by the new branch).

- [ Y] **A) A'' — land #1099 first (force-push local rebased ref `15655f07`), then 3-way merge spec branch into main**
  Time: 2.5-4.5h. 1 squash-merge + 1 squash-merge. The #1099 PR is independent and rebases cleanly (0 conflicts, 4 files +49/-43). Lands mature contract, 92 oracles, journeys v2, 6 spec docs, atomic acceptance features. Restores `git show origin/main:spec/product/mature-contract.v1.json` to schema 2.1.0 source-backed.
- [ ] **B) A' — skip #1099, do 3-way merge directly**
      Time: 1.5-2.5h. 1 squash-merge only. Skips #1099 (its vercel.json catch-all + dashboard web improvements land later). Same merge complexity.
- [ ] **C) Hold — wait for spec branch to refresh, do nothing now**
      Time: variable. No progress. Spec branch keeps aging (currently 4d stale, 267 commits ahead). Other 9 pending decisions remain blocked. Nightly stays red on linux until the fmt fix lands.
- [ ] **D) Discard spec branch, re-derive contract from #1088's invalidation baseline**
      Time: 2-3h. Loses provenance (267 commits of work, 12 journeys, 92 oracles, 6 spec docs). Re-derives manifest from main's source-backed rewrite. **Does NOT solve the 92-oracle execution requirement** — would just land an empty schema 2.1.0 with 0 oracles.

**My recommendation: A** — gets the fmt fix in, the #1099 improvements in, and the spec content in, with the smallest worktree-hop count. B is faster but defers #1099 indefinitely. C stalls the campaign. D throws away 267 commits of work.

---

## Q2 — S1 scope and the 92-oracle execution gap

**Context:** STABLE.md codifies S1 (Contract-stable) as requiring:

- 67 requirements ✓ (specified)
- 12 journeys ✓ (specified)
- 92 oracles ✓ (specified, file `verification/semantic-verification-cases.v1.json` lands with the spec branch)
- "lands with **execution evidence**" ✗ — the 92 oracles all have `status: specified_not_executed`
- `agileplus validate` returns 10/10 on fresh checkout ✗

A' (or A'') **lands the spec file but does NOT achieve S1.** The 92 oracles must actually be implemented and executed to produce evidence. Estimated: 92 cases × ~1-2h each = **92-184 hours** of focused test work. Parallelizable to **1-2 months with 2-3 engineers**.

- [ ] **A) A' now, treat 92-oracle execution as a separate post-merge campaign (1-2 month follow-up)**
      Lands the contract; S0 + S1-source land. S1-execution is its own backlog item. The `mature-contract.v1.json` carries `status: accepted_mature_semantic_contract_runtime_evidence_pending` and `mature_complete: false` — honest state, executable contract. **Campaign arc: 2-4 months.**
- [ Y] **B) A' now, then immediately start 92-oracle execution before declaring S1 achieved**
  Lands the contract and begins execution in the same week. Slower to declare S1 but no "we said it was S1 but it isn't" gap. **Campaign arc: 2-4 months, no false claim of S1 along the way.**
- [ ] **C) Skip A' until 92 oracles are executable (i.e., until a runner exists that can execute them)**
      No merge. The spec branch ages. The contract stays `invalidated` on main. **S0 stable, S1 never reached.**
- [ ] **D) Land a subset of oracles first (e.g., 12 journeys × 1 oracle each = 12 cases, ~12-24h) as a pilot, then decide**
      Pilot gives the campaign a concrete S1-progress signal without committing to the full 92. S1 declared "in progress" with 12/92 evidence. **Campaign arc: still 2-4 months, but with intermediate visibility.**

**My recommendation: B** — A' now, then immediately start 92-oracle execution. The honest state is "contract accepted, evidence pending" and we should not artificially separate the two. D is reasonable if you want intermediate visibility.

---

## After you answer

- **Q1=A, Q2=B**: I can execute the A'' plan (force-push #1099, merge, then 3-way merge spec) and immediately begin 92-oracle execution work. Estimated 4-6h for merges + 1-2 months for oracles.
- **Q1=A, Q2=A/D**: same merge, different post-merge posture.
- **Q1=B/D/C**: pause and discuss further.

Other 7 pending decisions (yoke-derive branch, #1098 review, #1090/#1097 close-as-subsumed, #1095/#1096 chain CI, stalled repos) can wait until Q1 lands. None of them block Q1.

_Ledger: tx-agent: jcode, tx-validated: manual, tx-scope: cockpit-MCQ, tx-intent: 2 strategic MCQs for next-tier operator decisions_
