# ADR-0019: Acceptance Authority and Terminal-State Gates

## Status

Accepted for the mature-product recovery program on 2026-09-30.

## Context

AgilePlus currently contains two partially overlapping acceptance systems.

### Runtime governance validation
`GovernanceContract` rules declare required evidence/policies. The CLI `validate`
command evaluates these records and may transition a Feature to `Validated`.

### Traceability acceptance model
`traceability-core::AcceptanceContract` contains criterion-level test/evidence
references and a `ProgressionGate` model. Its implementation correctly rejects empty
criterion sets and requires every criterion to be Covered.

Repository search shows the latter is currently used primarily in the shared crate,
tests, and architecture documents rather than as the live terminal-state authority.

Meanwhile:
- implementation workflow can move WP `Review -> Done` on review-loop approval;
- audit entries can contain no evidence refs;
- historical CLI flags could bypass validation/policies;
- ship historically tolerated merge errors.

The mature MACE model requires an independent, explicit grading boundary.

## Decision

### 1. Separate assignment acceptance from governance policy

**AcceptanceContract / grading**
answers:

> Did this candidate satisfy the accepted assignment criteria?

It owns:
- criterion identity/revision;
- candidate/subject snapshot;
- evaluator/oracle identity;
- criterion outcomes;
- evidence references;
- evaluation epoch;
- grade/receipt.

**GovernanceContract**
answers:

> Given the accepted grade/evidence, is this lifecycle transition permitted under policy?

It owns:
- policy requirements;
- required evidence classes;
- approvals/authority;
- security/compliance/release constraints;
- transition-specific policy.

Neither substitutes for the other.

### 2. Terminal work states require both

Authoritative transition to a terminal accepted work state requires:
1. non-empty acceptance criteria;
2. an evaluation/grade bound to the exact candidate and criterion revision;
3. every mandatory criterion accepted;
4. governance policy for that transition satisfied;
5. valid claim/authority where the transition requires one.

A PR review approval alone is not sufficient.

### 3. Work execution completion and accepted completion are distinct

A worker may finish execution and produce a reviewable candidate.

That state is not automatically `Done`.

Until the domain model gains a richer state vocabulary, current `Review` should be
treated as the highest safe state for a candidate that has only review/CI approval but
no canonical acceptance grade.

### 4. Feature validation aggregates accepted work plus feature-level policy

A Feature may transition to `Validated` only when:
- all required WPs have authoritative accepted grades;
- feature-level acceptance criteria are satisfied where defined;
- governance/policy checks pass.

### 5. Ship must fail closed

Shipping requires an authoritative `Validated` state and must fail on:
- skipped validation;
- merge errors/conflicts;
- stale/invalid acceptance receipts where policy requires current evidence.

Diagnostic flags may produce reports but cannot promote lifecycle state.

### 6. AgilePlus remains standalone

Acceptance evaluation must not require a live Tracera runtime.

Tracera may supply or independently interpret product-level evidence through the
federation contract, but AgilePlus must be capable of evaluating its own development
assignment.

### 7. Product acceptance remains Tracera authority

An AgilePlus accepted grade means:

> this development assignment satisfied its accepted work criteria.

It does **not** mean:

> the product obligation is now satisfied.

Tracera independently reconciles resulting artifacts/evidence against product truth.

## Canonical future record

The agent-lab dossier generation already contains useful primitives that should be
promoted/reconciled rather than recreated:

```text
Assignment
SubjectSnapshot / Candidate
Criterion / CriterionInstance
EvaluationEpoch
Observation / Evidence
Grade
ClaimLease
Receipt
```

The mature runtime should bind these to DevelopmentId / AttemptId / WorkPackageId.

## Immediate implementation consequences

Already closed:
- `validate --force` and `--skip-policies` may no longer authorize state promotion;
- `ship --skip-validate` may no longer authorize shipping;
- merge transport errors fail shipping closed.

Still open:
- remove Review->Done self-award from implementation loop;
- persist canonical grade/evaluation receipts;
- bind WP state transition to accepted grade;
- aggregate WP grades at feature validation;
- include exact evidence refs in audit/event receipts;
- reconcile existing CoverageMatrix model with standalone AgilePlus grading.

## Supersession

This ADR refines ADR-0009 and ADR-0010.

It supersedes any interpretation that:
- review approval alone makes a WP Done;
- governance evidence presence alone is equivalent to criterion acceptance;
- a live Tracera-owned CoverageMatrix is required for AgilePlus to grade its own work.

## Invariant

**Policy permission is not proof of correctness, and proof of correctness is not policy permission. Terminal acceptance requires both.**
