# AgilePlus Identity and Authority Crosswalk

**Date:** 2026-09-30  
**Status:** proposed reconciliation map; does not yet change runtime schemas.

## Principle

Do not collapse distinct lifetimes into one Feature/WP/Requirement id.

## Current / candidate identities

| Identity | Lifetime / meaning | Current AgilePlus evidence | Mature disposition |
|---|---|---|---|
| Feature | durable change/feature container | PRD/domain/FSM | KEEP/ADAPT as one Development scope, not universal product identity |
| WorkPackage | durable decomposed unit of work | PRD/specs/graph/storage | KEEP |
| RequirementId | requirement/acceptance reference | FR/NFR + trace | KEEP as requirement identity, not universal namespace |
| ClaimId | temporary resource lease | claim runtime | KEEP; execution authority only |
| AgentId | worker identity | claim/runtime | KEEP; not work identity |
| Worktree path/branch | isolated execution location | git/worktree runtime | KEEP as Attempt resource, not identity |
| GovernanceContract | transition/evidence policy | domain/evaluator | ADAPT into assignment/evaluation policy |
| Evidence | work/governance proof | domain/storage | KEEP but distinguish work evidence from Tracera product evidence/admission |
| DevelopmentId | durable development/change effort | not yet canonical as named type | ADD/derive; may map 1:1 to Feature for simple work, but not necessarily forever |
| AssignmentId | locked executable assignment/epoch | shared agent-lab prior art, not proven AgilePlus runtime | ADAPT/ADD after comparison |
| AttemptId | one worker execution attempt | implicit across agents/worktrees/runs | ADD explicitly |
| CriterionId | reusable acceptance predicate | governance rules/FRs partially overlap | MERGE/ADAPT, avoid duplicate evaluator ontology |
| EvaluationId | exact grader run | not clearly canonical | ADD/derive |
| CandidateId | exact produced candidate | commits/branches/artifacts currently implicit | ADD/reference external immutable identity |
| SpecRevisionId | accepted assignment/spec basis | spec hash exists; no mature explicit identity yet | ADAPT |
| Epoch | frozen assignment/profile/evaluator/scope basis | shared portfolio prior art | ADAPT candidate, not current repo authority |

## Recommended nesting

```text
DevelopmentId
  ├ accepted SpecRevisionId
  ├ WorkPackage*
  │   └ Assignment*
  │       ├ Criterion*
  │       └ Attempt*
  │           ├ Claim/Lease*
  │           ├ Worktree/Branch
  │           └ Candidate*
  └ Evaluation*
      └ Evidence*
```

This is conceptual, not yet a mandatory relational schema.

## Feature mapping

For current AgilePlus compatibility:
- existing Feature may initially serve as DevelopmentId for feature-sized changes;
- quick fixes/maintenance/experiments must not require inventing fake product features merely to obtain a durable development identity;
- therefore DevelopmentId should eventually be the more general durable work/change identity, with Feature as a typed projection/class where useful.

## WorkPackage vs Assignment

WorkPackage answers:
> what bounded durable work unit exists in the development plan/DAG?

Assignment answers:
> what exact executable contract is handed to a worker/grader at a particular accepted revision?

One WP can produce multiple assignments over time as:
- scope is clarified;
- verifier changes;
- an attempt fails;
- a different implementation strategy is selected.

Do not mutate historical assignment semantics silently.

## Attempt vs Claim

Attempt:
- worker execution episode;
- durable record even after death/failure.

Claim:
- temporary authority to mutate/use a resource.

Attempt may own several claims.
Claim expiry does not erase Attempt.
Claim transfer does not automatically transfer every approval/credential/authority held by the old worker.

## Evidence boundary

AgilePlus evidence can establish:
- assignment criterion result;
- work execution receipt;
- CI/review/governance result.

Tracera decides whether evidence is admissible/current for a product obligation/configuration.

Therefore:
`AgilePlus Work Done` != `Tracera Product Satisfied`.

## Existing GovernanceContract

Do not create an entirely separate grading system until GovernanceContract/PolicyEvaluator are mapped.

Likely split:
- reusable Criterion;
- Assignment criterion binding;
- Evaluator implementation/version;
- Evaluation result/evidence.

The current transition-oriented governance contract can remain a policy projection over these concepts.

## Next implementation archaeology

Locate exact persisted/runtime representations for:
- Feature;
- WorkPackage;
- GovernanceContract;
- Evidence;
- audit/event;
- spec hash/revision;
- agent run/worktree;
- review/evaluator output.

Then identify the smallest additive identities needed for the worker-replacement vertical witness.
