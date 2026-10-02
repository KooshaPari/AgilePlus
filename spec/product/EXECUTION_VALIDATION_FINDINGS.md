# AgilePlus Execution/Validation False-Green Findings

**Date:** 2026-09-30  
**Status:** blocking findings for worker-replacement/MACE vertical slice.

## F1 — WorkPackage Done is not bound to an immutable attempt/evaluation

Current `implement` path:
1. creates/reuses worktree;
2. sets WP Doing;
3. dispatches agent and receives ephemeral job_id;
4. runs review loop;
5. on Approved, sets WP Review then Done;
6. appends audit transition with empty evidence_refs;
7. cleans worktree.

Missing durable binding:
- AttemptId;
- exact assignment/spec revision;
- exact candidate/tree/commit set;
- evaluator/version;
- evaluation result;
- evidence receipt.

Therefore current `Done` means the orchestration review loop returned Approved, not that an immutable assignment/candidate/evaluation tuple is durably recorded.

## F2 — resume is WP/worktree state, not attempt recovery

`--resume` reuses a worktree for a WP in Doing state.

It does not reconstruct:
- which prior attempt owned it;
- whether the prior worker died vs is still active;
- which candidate was last verified;
- which verified sub-results survive;
- which credentials/claims remain valid.

Do not call this worker-replacement semantics yet.

## F3 — audit completion entry has empty evidence_refs

The Done audit entry currently uses `evidence_refs: vec![]`.

Even after audit hash-boundary correction, work completion has no explicit evidence link in this path.

## F4 — HTTP validation has vacuous green

API `trigger_validate` computes:
`compliant = satisfied_rules == total_rules`.

When no rules exist, 0 == 0 and API reports compliant=true. A unit test explicitly pins this behavior.

Under MACE doctrine, absence of criteria/proof must not manufacture green.

Expected mature result should be Unknown/NotConfigured/NotEvaluated unless an explicit policy declares an empty contract sufficient.

## F5 — CLI and HTTP validation semantics diverge

CLI validation includes richer:
- evidence parsing by FR/type;
- active policy rules;
- built-in policy mapping;
- threshold/metric evaluation;
- custom evaluator failure behavior.

HTTP endpoint independently reimplements a simpler presence loop.

This creates two potential truths for "governance validation."

## Required correction direction

1. Extract one application/domain evaluation service used by CLI/API/MCP.
2. Add explicit evaluation result states, including NotConfigured/Unknown.
3. Persist EvaluationId + candidate/assignment/spec/evaluator binding.
4. Completion audit references immutable evaluation/evidence receipt.
5. WP Done remains work state; Tracera product acceptance remains separate.
6. Resume/replacement uses Attempt history + Claim/Lease state, not WP/worktree presence alone.

## Negative controls

- no governance rules => not green by default;
- evidence from wrong WP/assignment/candidate => not admitted;
- stale prior attempt evaluation => does not green new candidate;
- review Approved but evaluation missing => work cannot claim verified completion;
- API/CLI/MCP same fixture => same semantic result;
- worker A dies; worker B resumes => A history retained and B receives distinct AttemptId.
