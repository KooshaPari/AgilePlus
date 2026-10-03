# AgilePlus Interface and Profile Contracts

**Status:** accepted mature specification layer  
**Date:** 2026-10-01

## Universal interface rule

CLI, HTTP, MCP, optional gRPC and human UI are projections over the same application semantics. No transport owns an alternate lifecycle, evaluator, identity model or acceptance rule.

Every mutating operation must expose or internally bind:
- Development/WorkPackage/Assignment identity as applicable;
- accepted SpecRevision;
- actor/authority;
- idempotency/replay key where retries are plausible;
- expected version/precondition where concurrent mutation is plausible.

Every read of evaluation/acceptance state must expose enough canonical identity to prevent a caller from confusing one candidate/revision with another.

## CLI profile

CLI is a first-class local human/agent interface.

Mature operation groups:
- inspect/recover;
- intent/spec revision;
- research/clarify;
- plan/decompose;
- assignment/context;
- attempt/claim;
- evaluate;
- governance;
- promote/release;
- audit/trace;
- import/harmonize;
- federation/status.

Exact command spelling is versioned UX, not ontology.

Machine-readable output must be available for automation-critical operations. Human-friendly output must not omit failure state in a way that appears successful.

## HTTP profile

If HTTP is mounted:
- versioned resource/action contracts;
- authenticated/authorized mutation;
- stable machine-readable errors;
- pagination/continuation;
- idempotency for replay-prone creates/actions;
- optimistic concurrency/version preconditions for accepted-state mutation;
- no endpoint may bypass the canonical evaluator/governance boundary.

## MCP profile

MCP tools/resources/prompts:
- tools are bounded application operations, not arbitrary authority;
- resources expose canonical revision-bound state;
- prompts are convenience/context artifacts and cannot grant mutation/acceptance authority;
- tool results include exact identities/statuses;
- sampling/model output remains untrusted proposal/work content until accepted/evaluated.

## gRPC profile

gRPC is optional. If enabled, it preserves the same semantics and uses explicit protocol versioning/compatibility and appropriate transport security. Generated stubs are derivatives of the protocol contract, not independent authority.

## Human dashboard/UI profile

Human UI may inspect, author proposals/specs, dispatch work and review evidence according to granted authority. Visual status must match canonical work/evaluation state. A green badge cannot exist where the canonical result is Unknown/Unsatisfied/NotConfigured/Stale.

## Git/GitHub execution profile

For software work:
- base candidate is explicit;
- worktree/branch is Attempt resource;
- dirty/base drift is detected before candidate/evaluation/promotion;
- commit/tree/artifact candidate identity is exact;
- PR/review state is context/evidence, not acceptance;
- merge conflicts/errors fail promotion closed.

## Triage/intake profile

Incoming bugs/features/ideas/tasks may be classified/routed/prioritized automatically, but:
- source/provenance retained;
- classification confidence/method retained where model-derived;
- human/authorized override supported;
- intake item does not become accepted SpecRevision without acceptance.

## Scheduling/concurrency profile

Ready work selection respects:
- dependency closure;
- claim/resource availability;
- configured priority/fairness;
- bounded concurrency/resource pressure;
- no double-authorized mutation of exclusive resources.

Scheduling optimization never changes accepted criteria or authority.

## Review/HITL profile

Review comments/approval are evidence/input to convergence. HITL is required where policy/authority says so. Review approval alone does not award AcceptedWork.

Escalation triggers include unresolved high-severity findings, repeated evaluator failure, authority ambiguity, destructive/irreversible operations and policy-defined human gates.

## Release/promotion profile

Promotion profiles may distinguish canary/nightly/pre-stable/stable/LTS or other release classes.

The semantic invariant is:
- exact accepted candidate;
- profile-specific governance;
- explicit release identity/receipt;
- stable/official human gate where policy requires;
- rollback/supersession history;
- no promotion after candidate drift without re-evaluation.

## Optional infrastructure profiles

Plane, NATS, Neo4j, MinIO, P2P and similar integrations are optional profiles unless a future accepted contract elevates one.

Optional profile failure cannot erase or redefine core local AgilePlus truth.
