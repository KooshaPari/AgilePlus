# AgilePlus Canonical Semantic Decisions

**Status:** accepted for the mature-product specification/recovery program  
**Date:** 2026-10-01  
**Scope:** semantic authority only. This document does not claim runtime implementation.

## 1. Product boundary

AgilePlus is the canonical development/work control system for a repository-scoped change effort. It owns durable development identity, specification revision, decomposition, assignment, attempts, claims/leases, candidate production, independent evaluation, work evidence, work acceptance and promotion handoff.

Tracera owns accepted product truth and product-level evidence applicability. Therefore:

```text
AgilePlus accepted work ≠ Tracera satisfied product obligation
```

## 2. Canonical identity model

The mature semantic model is:

```text
Development
  ├─ SpecRevision*
  ├─ WorkPackage*
  │   └─ Assignment*
  │       ├─ CriterionBinding*
  │       └─ Attempt*
  │           ├─ ClaimLease*
  │           ├─ ExecutionResource*
  │           └─ Candidate*
  ├─ Evaluation*
  │   ├─ CriterionResult*
  │   └─ EvidenceReceipt*
  └─ Promotion*
```

Existing `Feature` is a compatibility projection/type of Development for feature-shaped work. WorkPackage remains the durable DAG/decomposition unit. Assignment freezes an executable contract. Attempt is one worker episode. Claim/lease is temporary execution authority and never durable work identity. Candidate is exact produced output. Evaluation is an independent grader run bound to exact Assignment/SpecRevision/Candidate/Evaluator.

No future specification may use worker id, worktree path, branch, claim id, PR id, or Feature id as a universal substitute for these distinct identities.

## 3. Adaptive lifecycle: partial order, not mandatory waterfall

ADR-0013's fixed eight-stage spine is **superseded as a mandatory FSM**.

The canonical mature lifecycle consists of semantic milestones with dependencies:

- **IntentKnown** — the change/development objective and authority are identifiable.
- **ContractAccepted** — an accepted SpecRevision exists with non-empty applicable acceptance expectations.
- **Executable** — decomposition/context/dependencies/authority are sufficient for an Assignment.
- **Attempting** — one or more Attempts may execute under valid claims/resources.
- **CandidateProduced** — exact candidate identity exists.
- **Evaluated** — independent evaluation exists for the exact candidate and accepted contract.
- **AcceptedWork** — all mandatory criteria and transition governance are satisfied.
- **Promoted** — accepted work has been integrated/released/handed off as applicable.
- **Archived** — durable history/recovery state is retained.

These are a **partial order**. Research, clarification, design, planning and decomposition are artifact roles/activities that may occur and recur whenever dependencies demand them. They are not universal ceremonial states.

Profiles may project familiar workflows (OpenSpec, Spec Kit, BMAD, bugfix, incident, experiment, release) onto these milestones. A profile may collapse optional activities but cannot skip the semantic prerequisites for acceptance.

## 4. Canonical artifact roles, not canonical filesystem roots

ADR-0012's rule that `kitty-specs/<feature-id>/` is the sole permanent authority is **superseded**.

Canonical authority belongs to typed artifact identities and accepted revisions, not a framework-specific directory.

Core roles include:
- Intent;
- Research;
- Requirement/Scenario;
- Architecture/Design;
- Decision/ADR;
- Contract/Interface;
- Plan;
- WorkPackage/DAG;
- Assignment;
- Criterion;
- Fixture;
- Oracle;
- Candidate;
- Evaluation;
- EvidenceReceipt;
- Promotion/Release record;
- Retrospective/Decision history.

Repository profiles define materialized paths. Harmonizers import/map external frameworks without creating competing semantic authorities. Source provenance and aliases remain explicit.

## 5. Traceability authority

ADR-0014's RequirementId-only universal namespace is **superseded**.

Every durable object type has its own stable identity. Typed links connect them. Requirement ids remain canonical requirement identities, not universal graph identities.

AgilePlus owns work traceability:

```text
intent → accepted spec revision → work package → assignment
→ attempt → candidate → evaluation → evidence → promotion
```

Tracera owns product traceability and independently admits/rejects AgilePlus evidence for product obligations.

Derived trace files are projections, never silent sources of accepted intent.

## 6. Acceptance authority

ADR-0019 remains authoritative.

Acceptance requires both:
1. correctness proof against non-empty accepted criteria for the exact candidate; and
2. governance permission for the transition.

Review approval, CI green, policy presence, work completion, merge, or agent self-report cannot individually award terminal accepted work.

Explicit non-success states include at minimum:
- Unsatisfied;
- Inconclusive;
- Unknown;
- NotConfigured;
- Stale;
- Unauthorized;
- Cancelled/Aborted where evaluation did not complete.

Empty applicable criteria can never yield accepted work.

## 7. Audit integrity

Historical audit entries retain **audit-hash-v1** semantics: transition-core fields are hash-covered; evidence_refs/event_id/archive metadata are not retroactively claimed as covered.

The mature contract requires **audit-hash-v2** for future stronger receipts:
- explicit schema version;
- canonical serialization;
- exact immutable identities for evidence/evaluation references;
- covered-field declaration;
- cross-version chain verification;
- no rewriting of v1 history.

Until v2 exists, documentation must not claim the v1 chain attests mutable fields outside its actual envelope.

## 8. Worker replacement

Worker replacement is first-class:
- replacement creates a new Attempt;
- prior Attempt remains durable;
- claims are re-issued explicitly;
- authority is not inherited merely by claim transfer;
- accepted Assignment may remain the same only if its contract has not changed;
- changed instructions/criteria create a new accepted SpecRevision and, where execution basis changes, a new Assignment;
- evaluation binds the exact resulting Candidate.

## 9. Finality rule

These semantic decisions are final for this recovery program unless a later explicitly accepted ADR supersedes them with source-backed rationale. Runtime divergence is implementation debt, not permission to reinterpret the specification.
