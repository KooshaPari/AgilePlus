# AgilePlus Historical Requirement Disposition

**Status:** canonical reconciliation of `FUNCTIONAL_REQUIREMENTS.md` v2.2 into the mature semantic contract  
**Date:** 2026-10-01

The historical FR catalogue is retained as provenance and implementation lineage. It is not accepted wholesale as the mature product contract because many requirements were derived from a particular March 2026 implementation architecture.

## Disposition vocabulary

- **KEEP** — product obligation remains semantically current.
- **ADAPT** — intent remains but mature semantics differ.
- **SUPERSEDE** — current mature decision replaces the old normative meaning; historical ID remains traceable.
- **OPTIONAL-PROFILE** — valid adapter/deployment/profile capability, not universal core.
- **RETIRE** — no longer mature product intent.
- **REVIEW-ATOMIC** — family is relevant but must be decomposed/reconciled item by item.

## FR-DOMAIN

**Disposition: ADAPT / REVIEW-ATOMIC**

Keep durable work/decomposition/project/module/cycle/snapshot/metric concepts where useful.

Supersede:
- `FeatureState` as universal ordered lifecycle;
- forward-only ordinal lifecycle as mature authority;
- worker/worktree/PR fields as durable WP identity.

Mature authority is `CANONICAL_SEMANTIC_DECISIONS.md`: Development, SpecRevision, WP, Assignment, Attempt, ClaimLease, Candidate, Evaluation.

## FR-AUDIT

**Disposition: ADAPT**

Keep append-only audit and chain verification.

Correct the normative envelope:
- v1 hashes transition-core fields only;
- evidence_refs/event_id/archived_to are not retroactively claimed covered;
- v2 stronger receipt semantics are mature intent;
- MinIO archival is OPTIONAL-PROFILE, not required core architecture.

## FR-CLI

**Disposition: ADAPT / REVIEW-ATOMIC**

CLI remains a primary human/agent surface. Command names are not themselves the product ontology.

Keep semantic operations for specify/research/plan/decompose/implement/evaluate/promote/recover/inspect where applicable.

Supersede any command behavior that:
- treats fixed lifecycle stages as universal;
- allows review/CI/policy alone to award accepted work;
- couples canonical artifact authority to kitty-specs path.

## FR-API

**Disposition: ADAPT**

HTTP surface remains valid where supported, but exact routes and API-key-only assumptions are implementation/profile details. Mature requirement is semantic parity, authorization, truthful errors, idempotency/versioning and exact identity binding.

## FR-GRPC

**Disposition: OPTIONAL-PROFILE / REVIEW-ATOMIC**

gRPC is not a universal mature requirement. If supported, it must preserve the same canonical semantics and security boundary.

## FR-STORAGE

**Disposition: KEEP core local-first persistence / ADAPT implementation details**

SQLite local-first durable operation remains core. WAL/synchronous settings, exact migration library and cache strategy are implementation decisions unless required by measured behavior.

Alternative storage is replaceable/profile-driven.

## FR-EVENTS

**Disposition: ADAPT / OPTIONAL-PROFILE**

Durable event/audit semantics may be product obligations. NATS JetStream and exact subject strings are not universal mature requirements.

## FR-GRAPH

**Disposition: ADAPT**

Work dependency/DAG queries remain useful. Mandatory Neo4j is superseded. Graph capability must be available through a replaceable boundary or local implementation where required.

## FR-IMPORT

**Disposition: KEEP/ADAPT**

Framework/source import, validation, idempotency, conflict reporting and provenance remain mature intent. Exact manifest shape is versioned contract detail.

## FR-TRIAGE

**Disposition: KEEP/ADAPT**

Intake/classification/routing/prioritization are valid work-control capabilities, but model/heuristic choice is replaceable and automated classifications must retain provenance/override semantics.

## FR-PLANE

**Disposition: OPTIONAL-PROFILE**

Plane integration is an adapter, not canonical work authority. It cannot override AgilePlus durable identity/spec/evaluation semantics.

## FR-GIT

**Disposition: KEEP/ADAPT**

Git candidate/worktree/branch integration remains central for software work. Worktree/branch/PR are execution resources/references, not durable work identities.

## FR-GOVERN

**Disposition: ADAPT**

Governance remains required but is distinct from correctness grading. ADR-0019 controls.

Evidence presence alone cannot establish criterion acceptance.

## Historical infrastructure epics

Mandatory Neo4j, NATS, MinIO, Plane and similar service assumptions are superseded as universal architecture. They may remain optional profiles/adapters if justified by current implementation/SOTA evidence.

## Out-of-scope statements

Historical "single-user", "no web authoring", "no collaboration" and similar March exclusions are not automatically mature exclusions. They require explicit mature-domain disposition rather than inheriting an implementation-era constraint.

## Rule for atomic migration

Every historical FR ID eventually receives one of:
```text
UNCHANGED
MAPPED_TO:<new-id>
SPLIT_INTO:<new-ids>
SUPERSEDED_BY:<decision/new-id>
OPTIONAL_PROFILE:<profile>
RETIRED:<rationale>
```

Until that mapping is complete, historical catalogue completeness must not be reported as mature-contract completeness.
