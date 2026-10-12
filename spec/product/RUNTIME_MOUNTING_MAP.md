# AgilePlus Runtime Mounting Map — Mature-First Reconciliation

**Date:** 2026-09-30  
**Status:** source-backed first pass from current main.

## Feature

Runtime: `agileplus-domain::Feature`.
Persistence: `StoragePort` + SQLite adapter.
Identity: auto-assigned `i64 id` + unique slug.
Current spec binding: one `[u8;32] spec_hash`.
Lifecycle: rigid Created→Specified→Researched→Planned→Implementing→Validated→Shipped→Retrospected.

### Mature finding

Feature is a real durable aggregate, but:
- it is feature-shaped rather than a general Development identity;
- `spec_hash` is current-state binding, not an explicit historical SpecRevision record;
- lifecycle is strictly linear and conflicts with proposed adaptive/fluid methodology.

Disposition: KEEP aggregate for compatibility; introduce general Development/SpecRevision semantics additively rather than deleting Feature.

## WorkPackage

Runtime: `agileplus-domain::WorkPackage`.
Persisted fields include:
- feature_id;
- state;
- sequence;
- file scope;
- acceptance criteria;
- agent_id;
- PR state/url;
- worktree_path;
- base/head commits.

### Mature finding

WP already carries much of an assignment/candidate envelope, but mutably on the durable WP row.

This conflates:
- durable planned work;
- current assigned worker;
- current execution location;
- current candidate.

A replacement worker/second attempt overwrites or mutates fields unless separate history/audit reconstructs them.

Disposition: KEEP WP as durable plan/DAG object; move attempt-specific worker/worktree/candidate bindings into explicit Attempt records over time.

## GovernanceContract

Runtime: versioned contract bound to Feature.
Contains transition-oriented GovernanceRules:
- transition;
- required evidence labels;
- policy refs.

Evidence is attached to WP and FR.

### Mature finding

Strong reusable base, but transition/evidence-label orientation is narrower than a generic Assignment/Criterion/Evaluator model.

Disposition: ADAPT as policy projection. Do not build a second evaluator before mapping GovernanceEvaluator behavior.

## Evidence

Current fields:
- id;
- wp_id;
- fr_id;
- EvidenceType;
- artifact_path;
- metadata;
- created_at.

### Mature finding

Useful work evidence receipt but lacks explicit:
- candidate identity;
- evaluator/verifier identity+version;
- assignment/spec revision;
- configuration/environment basis;
- result semantics beyond type/metadata.

Do not mutate old Evidence schema destructively. Add evaluation/receipt binding around it.

## Audit

Hash chain is real and tested.

Important integrity-boundary finding:
`hash_entry` covers:
- feature_id;
- wp_id;
- timestamp;
- actor;
- transition;
- prev_hash.

It does **not** cover:
- id;
- evidence_refs;
- event_id;
- archived_to.

Existing test `hash_ignores_id_event_id_and_archived_to` also demonstrates evidence_refs do not change the hash.

Documentation comment saying the hash "covers all mutable fields" is inaccurate.

### Decision required

Either:
A. intentionally define the tamper-evident envelope as transition core only and document evidence/event/archive metadata as externally verifiable references; or
B. version the audit hash schema and include those fields for future entries.

Do not retroactively change historical hash computation; that would invalidate the ledger.

## StoragePort

Real broad hexagonal port exists with SQLite primary implementation.

It covers Feature/WP/audit/evidence/governance/etc.

A separate triage application path also defines a narrower `WpRepository` and hardcodes an in-memory ClaimStore.

### Mature finding

There are overlapping application/storage generations:
- canonical domain `StoragePort`;
- triage `WpRepository`;
- ClaimStoreTrait;
- concrete in-memory ClaimStore mounted in AppState.

Need consolidation at application composition, not another storage abstraction.

## Smallest additive identities for worker-replacement witness

Add/derive only:
- DevelopmentId — initially may reference Feature id for feature work;
- SpecRevisionId — immutable spec hash + accepted metadata;
- AssignmentId — exact WP/spec/criterion binding handed to worker;
- AttemptId — one worker execution episode;
- EvaluationId — exact grader run;
- CandidateRef — commit/tree/artifact identity.

Reuse:
- WorkPackage;
- Claim/lease;
- worktree;
- GovernanceContract;
- Evidence;
- Audit.

## Worker-replacement witness target

1. Feature/Development exists.
2. WP exists.
3. SpecRevision frozen.
4. Assignment A created.
5. Attempt A claims worktree, produces partial verified result, dies/expires.
6. Attempt B starts under same durable Assignment or explicit revised Assignment.
7. B receives new claim; A history remains.
8. Candidate committed.
9. Evaluation binds exact candidate + assignment/spec revision + evaluator.
10. AgilePlus marks work complete.
11. Evidence/receipt is handed to Tracera; Tracera independently decides product applicability/acceptance.
