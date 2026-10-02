# Pass 13 — AgilePlus Methodology Generation Map

**Date:** 2026-09-30  
**Source denominator:** reconciled `spec/mature-product-contract-v1` after merge commit `91cd90c6e8c8e7942b9594138250ead5de2e5b35`.  
**Status:** classification pass; not full SOTA closure.

## Why this pass exists

AgilePlus already contains multiple generations of methodology, runtime, spec and evaluation work. The mature-first program must reconcile them rather than add another parallel layer.

Classification vocabulary:

- **KEEP** — concept/system is still directionally correct.
- **ADAPT** — retain core, revise semantics/interface.
- **MERGE** — combine overlapping generations into one authority.
- **SUPERSEDE** — preserve historically but replace as current doctrine.
- **RETIRE** — remove from active path after migration; preserve history.
- **EXPERIMENT** — useful but not accepted architecture.

## Generation map

| Surface | Current disposition | Reason |
|---|---|---|
| `docs/harmonization/FRAMEWORK_ANALYSIS.md` | **KEEP + REFRESH** | Valuable broad framework decomposition. Needs current-source/version refresh and evidence qualification; should feed SOTA subtree rather than act as architecture authority. |
| `UNIFIED_PM_MODEL.md` | **ADAPT / PARTIAL SUPERSESSION** | Strong synthesis of artifacts/gates/frameworks; runtime premise that Tracera must embed AgilePlus is superseded by ADR-0018. |
| ADR-0012 unified artifact set | **ADAPT** | Single durable authority is valuable; hard-coding `kitty-specs/` as sole eternal root may overfit one framework and conflicts with owner preference for OpenSpec velocity. Need artifact-role authority independent of folder brand. |
| ADR-0013 eight-stage lifecycle | **ADAPT** | A common lifecycle vocabulary is useful, but fixed eight-stage passage is too rigid as universal execution semantics. Preserve adaptive-depth/quick-mode and separate durable state from presentation stages. |
| ADR-0014 unified traceability | **ADAPT** | Alias mapping and bidirectional evidence are valuable. A single RequirementId namespace is too requirement-centric for the broader Tracera product ontology and cross-projection identities. |
| ADR-0015 process machine over checklist | **KEEP CORE / PARTIAL SUPERSESSION** | AgilePlus should enforce execution progression with a process machine. Tracera need not be embedded or subordinate. |
| ADR-0016 five-layer framework stack | **ADAPT** | Useful mental model for outcomes/governance/delivery/execution/evidence. Must not become a mandatory ontology or runtime-ownership hierarchy. |
| ADR-0017 embedding migration | **SUPERSEDE** | Mandatory Tracera embedding conflicts with current owner intent and three-lifetime model. |
| ADR-0018 sibling runtimes | **KEEP / CURRENT** | Current ownership boundary: AgilePlus work truth; Tracera product truth; explicit federation. |
| `handbook/specs/assessment/agent-lab-dossiers-v1.1` | **KEEP + MERGE INTO MACE CORE** | Assignment, epoch, criterion, subject snapshot, claim/lease, grader/evidence concepts directly support the desired autograder architecture. Must reconcile duplicate schema authority with mature contract rather than remain a side dossier. |
| `crates/agileplus-triage::claim` + SQLite claim store | **KEEP + HARDEN** | Real claim/lease primitive exists. Needs explicit attempt/development identity, authority transfer, restart/multi-process tests, stale lease handling and MACE linkage. |
| application triage `AppState` | **ADAPT** | Useful orchestration seam, but comments/code still expose temporary in-memory implementations and shadow graph primitives. Must converge on durable ports rather than duplicate future APIs. |
| worktree isolation | **KEEP AS PROFILED DEFAULT** | Strong agent isolation primitive. Do not assume every read-only/research operation needs a worktree; substantive mutation does. Disk-floor values are environment policy, not universal product ontology. |
| existing `kitty-specs/` | **KEEP AS CURRENT INGEST/COMPATIBILITY SURFACE** | Large installed base. Do not make folder name equal product ontology. Harmonizer should allow alternate methods without duplicate authority. |
| old Spec-Kitty command shims | **RETIRE AFTER COMPATIBILITY WINDOW** | Many are already marked deprecated in favor of `ap`. Keep historical/read compatibility until migration evidence closes. |
| triple/parallel spec roots | **MERGE BY AUTHORITY, NOT JUST PATH** | The true problem is conflicting durable authority, not merely multiple directories. External method artifacts may remain if their authority/derivation is explicit. |
| FR/NFR-only spine | **ADAPT** | Useful requirement IDs, but assignment/product/work ontology is broader than requirements. Preserve IDs as one entity family, not the universal identity type. |
| hash-chained worklogs/event evidence | **KEEP + VERIFY** | Strong transactional/evidence primitive. Must bind exact attempt/candidate/evaluator and integrate with Git ledger rather than replace it. |
| framework-specific personas | **CONDITIONAL** | BMAD role separation may improve depth for complex work; do not impose persona ceremony on small changes. |
| fixed framework-by-change-type selection | **ADAPT INTO ADAPTIVE COMPILER** | Replace static mapping (bug→GSD, feature→Kitty, epic→BMAD) with measurable rigor dimensions: risk, uncertainty, novelty, blast radius, reversibility, security, migration, scope. |

## Canonical synthesis direction

AgilePlus should not be a runtime that "runs OpenSpec + Spec Kit + BMAD + GSD."

It should compile a durable development effort into the **minimum sufficient methodology** for the change.

### Core durable objects

Candidate mature core:

```text
Development
SpecRevision
Assignment
Criterion / CriterionInstance
WorkPackage
Dependency
ClaimLease
Attempt
Worker
SubjectSnapshot / Candidate
EvaluationEpoch
Observation / Evidence
Grade
FailureClassification
Decision / Escalation
Receipt
```

Framework-specific artifacts map into these objects.

### Adaptive depth

Inputs:

```text
risk
complexity
uncertainty
novelty
blast radius
reversibility
security
migration
external interface count
number of affected components/repos
human judgment requirement
```

Outputs:

```text
required artifacts
research depth
architecture depth
review roles
oracle rigor
negative controls
work decomposition
parallelism
HITL gates
release/promotion requirements
```

### Context compiler

Context should be compiled from authority, not folder conventions:

- accepted assignment/spec revision;
- resolved portfolio policy;
- product context from Tracera where present;
- relevant repo/code graph;
- architecture/decision lineage;
- prior attempt failures;
- verified sub-results;
- exact next frontier.

## Important current code findings

### Claim primitive exists
`ClaimKind`, `ClaimState`, TTL/heartbeat, resource uniqueness, transfer and SQLite backing already exist.

This should be extended, not re-created.

### Temporary duplicate graph implementation exists
Application triage currently carries a self-contained `WpGraph` because the intended graph crate lacks synchronous topology primitives.

Treat this as declared transition debt and either absorb or remove it when the canonical graph API exists.

### Work-completion semantics need review
`AppState::done` releases a claim and marks a WP done. Mature MACE semantics require a distinction between:
- work execution complete;
- assignment criteria graded;
- development accepted;
- product evidence accepted by Tracera.

Do not overload `done`.

## Next concrete work

1. Promote agent-lab assignment/evaluation schemas into the canonical comparison set.
2. Map those schemas against existing WorkPackage/Feature/GovernanceContract/claim types.
3. Define DevelopmentId vs AttemptId vs ClaimLeaseId.
4. Define spec revision and evaluation epoch binding.
5. Audit `done`/review/ship transitions for self-awarded completion.
6. Replace fixed framework-selection rules with adaptive-depth decision model.
7. Refresh SOTA sources against current upstream methods.
8. Build one worker-replacement vertical witness using the existing claim/worktree runtime.
9. Only then rewrite the mature product contract/schema.

## Non-goal

Do not delete historical framework artifacts merely because they are superseded. They are part of the Git/product-memory ledger.
