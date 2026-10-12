# AgilePlus Specification Acceptance Contract

**Status:** canonical non-runtime acceptance contract  
**Date:** 2026-10-01

This grades AgilePlus's specification, not its implementation.

## Canonical semantic basis

The grader MUST use `spec/product/CANONICAL_SEMANTIC_DECISIONS.md`.

Older PRD/spec/ADR generations remain provenance. A conflicting older statement is not current authority unless explicitly retained by the canonical decisions.

## Required mature-spec domains

The accepted specification must decompose, where applicable:
- development identity and accepted spec revisions;
- exploration/brownfield understanding;
- research/clarification;
- requirements/scenarios;
- architecture/design/ADRs/contracts;
- planning/decomposition/DAG;
- WorkPackage and context bundle;
- Assignment and Criterion binding;
- Attempt/worker/model/tool dispatch;
- Claim/lease/worktree/sandbox authority;
- candidate identity;
- independent evaluation/autograding;
- retry/revision/replacement;
- review/HITL escalation;
- governance/policy;
- evidence/receipts/audit;
- traceability;
- promotion/release handoff;
- recovery/resumption/history;
- adaptive workflow profiles;
- CLI/API/MCP/human semantic parity;
- Tracera federation.

No domain is complete merely because a corresponding runtime type exists.

## Assignment oracle readiness

Every executable Assignment specification must define:
- accepted SpecRevision;
- exact WorkPackage/development scope;
- context bundle and provenance;
- allowed/forbidden work surfaces;
- prerequisites/dependencies;
- mandatory Criteria;
- evaluator/oracle identity requirements;
- expected evidence;
- authority/claim needs;
- retry/revision rules;
- terminal acceptance rule.

## Mandatory adversarial families

Where applicable:
- zero criteria;
- missing/stale/wrong-candidate evidence;
- evaluator unavailable;
- worker self-awards acceptance;
- review/CI green but criterion fails;
- policy passes but correctness fails;
- correctness passes but policy denies transition;
- worker dies/claim expires and replacement resumes;
- replacement inherits unauthorized authority;
- instructions change without SpecRevision/Assignment revision;
- candidate changes after evaluation;
- claim conflict/TTL/reap/transfer;
- worktree dirty/base drift;
- merge conflict/error;
- transport disagreement;
- audit covered/non-covered field mutation;
- v1→v2 audit chain boundary;
- Tracera unavailable;
- Tracera rejects otherwise accepted AgilePlus evidence;
- framework import produces conflicting duplicate intent;
- adaptive profile omits required acceptance prerequisite.

## Specification statuses

`AcceptedSpec / UnsatisfiedSpec / InconclusiveSpec / IncompleteSpec / NotApplicable`.

No empty catalogue, empty journey set or empty criterion set can be accepted.

## Final gate

AgilePlus non-code specification is not final until:
- mature semantic decisions are canonical;
- source generations are dispositioned;
- mature requirements are decomposed;
- journeys are accepted and bound;
- verification cases are concrete;
- trace graph expectations are complete;
- all remaining runtime mismatches are explicitly implementation debt rather than unresolved product meaning.
