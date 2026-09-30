# Execution Records v1 — Compatibility Contract

**Date:** 2026-09-30  
**Implementation:** SQLite migration 027.

## Purpose

Add the minimum immutable records needed to prove worker replacement and exact evaluation without replacing Feature/WorkPackage.

## Compatibility mapping

For the first witness:
- Feature acts as the durable development root;
- WorkPackage remains the durable DAG unit;
- SpecRevision freezes the accepted spec basis;
- Assignment freezes WP + SpecRevision execution basis;
- Attempt records each worker episode;
- Evaluation binds exact candidate/evaluator/result.

A generalized DevelopmentId remains a mature-model requirement, but is intentionally deferred until this slice proves the need and migration shape.

## Invariants

1. prior SpecRevision is never mutated to represent new instructions;
2. Assignment supersession is explicit;
3. every Attempt has exactly one Assignment;
4. replacement worker gets a new AttemptId;
5. worktree/job id are properties of Attempt, not WP identity;
6. Evaluation binds an immutable candidate reference;
7. zero configured criteria cannot be represented as Satisfied by default evaluation policy;
8. evidence refs belong to Evaluation receipt;
9. WP Done should eventually require an admitted completion Evaluation, not review-loop approval alone;
10. Tracera independently decides product evidence applicability.

## Next code

- domain record structs;
- repository methods;
- create SpecRevision from current Feature.spec_hash;
- create Assignment before dispatch;
- create/update Attempt around dispatch job;
- create Evaluation from centralized evaluator;
- completion audit references Evaluation/Evidence;
- replacement witness.
