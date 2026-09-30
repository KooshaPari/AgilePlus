# AgilePlus Methodology Generations — Reconciliation Ledger

**Date:** 2026-09-30  
**Status:** source-backed first reconciliation; incomplete until full source ledger closes.

## Rule

Do not invent another AgilePlus methodology until existing generations are classified.

Classification:
- KEEP — current concept/implementation should remain;
- ADAPT — sound concept needs semantic/interface correction;
- MERGE — overlapping generations should become one authority;
- SUPERSEDE — newer accepted model should replace older current meaning while preserving history;
- RETIRE — no longer current; retain historical provenance;
- EXPERIMENT — useful but not accepted product contract;
- UNRESOLVED — authority/conflict still needs adjudication.

## Generation A — March active PRD / original spec-driven engine

Sources:
- `PRD.md` v2.1, Status Active, 2026-03-27;
- `docs/specs/001-spec-driven-development-engine/`.

Important concepts:
- Feature FSM;
- WP decomposition/DAG;
- governance/evidence;
- immutable/hash-chained audit;
- CLI/API/MCP;
- isolated worktrees;
- agent dispatch/review loop;
- local-first SQLite;
- broader service/integration architecture.

Disposition:
- Feature/WP durable work identity: **KEEP/ADAPT**.
- Work DAG: **KEEP**.
- Governance/evidence concept: **KEEP/ADAPT** to MACE exact-evidence model.
- Hash-chained audit: **KEEP**, but Git/event/external evidence authority must remain distinct.
- Isolated worktrees: **KEEP**.
- Agent dispatch/review loop: **ADAPT** to pluggable workers and independent graders.
- Exact March infrastructure stack: **UNRESOLVED**; do not treat Neo4j/NATS/MinIO/etc. as mature defaults without current need/SOTA evidence.
- Feature FSM as universal lifecycle: **UNRESOLVED** because later harmonization proposes a different unified spine.

## Generation B — Framework harmonization

Sources:
- `docs/harmonization/FRAMEWORK_ANALYSIS.md`;
- ADR-0012 unified artifact set;
- ADR-0013 lifecycle/gates;
- ADR-0014 traceability;
- ADR-0016 framework-layer stack.

Status warning: these ADRs are **Proposed**, not accepted. They are design candidates, not authority merely because they are newer.

### ADR-0012 canonical kitty-specs root
Disposition: **ADAPT / likely SUPERSEDE**.

Good:
- one canonical intent authority;
- ingress/harmonization rather than duplicate hand-maintained roots;
- explicit derivatives.

Problem:
- hard-coding `kitty-specs/` as permanent product ontology risks coupling AgilePlus to one imported framework's filesystem convention.
- mature model should define canonical artifact identities/roles and allow a repo profile to choose a materialized path.

### ADR-0013 eight-stage lifecycle
Disposition: **ADAPT**.

Good:
- one durable lifecycle authority;
- framework stages map into one spine;
- quick-mode/adaptive depth;
- Tracera observes product state rather than mutating work lifecycle.

Problem:
- a single linear eight-stage spine may be too rigid for OpenSpec-like fluid work, experiments, maintenance, incident response and nested loops.
- likely need stable semantic milestones + partial-order/sub-lifecycles rather than every change physically traversing eight states.

### ADR-0014 unified traceability
Disposition: **MERGE/ADAPT**.

Good:
- single logical IDs;
- source intent vs derivative/evidence separation;
- bidirectional trace;
- coverage-based acceptance.

Problem:
- RequirementId-only canonical identity is too narrow for the mature product/development ontology.
- Tracera now owns product-level evidence interpretation; AgilePlus should not duplicate a second product acceptance graph.
- trace.json/IntentGraph/TraceLink need explicit authority boundaries.

### ADR-0016 five-layer framework stack
Disposition: **KEEP as explanatory projection, not runtime ontology**.

L0 Evidence → L4 Outcomes is useful for mapping frameworks and navigation.
Do not force every durable object into exactly one layer or create runtime coupling to the diagram.

## Generation C — Runtime work/claim system

Sources:
- `agileplus-triage::claim`;
- SQLite claim store;
- `agileplus-application::use_cases::triage`;
- worktree routes/commands.

Disposition:
- Claim/lease primitive: **KEEP/ADAPT**.
- TTL/heartbeat/reap: **KEEP**.
- structured claim reason: **KEEP**.
- resource kinds: **ADAPT** as real workloads reveal scope.
- claim transfer: **ADAPT**; replacement workers must not inherit authority merely because a lease transfers.
- in-memory application claim store: **SUPERSEDE for durable/multi-process operation**.

Concrete mounting gap:
`AppState<W>` currently embeds `Mutex<ClaimStore>` even though a SQLite-backed implementation exists. The source comment says production wiring is added later, but this use-case surface still hardcodes the in-memory store.

The mature application boundary should depend on a claim/lease port, with in-memory and SQLite implementations behind it.

## Generation D — Agent-lab / assignment-grader control model

Recovered concepts include:
- subject snapshot;
- intent;
- assignment;
- criterion;
- criterion instance;
- epoch;
- claim lease;
- evaluator/grader;
- evidence/receipt.

Disposition: **MERGE with MACE / likely canonical seed**.

This generation is closer to the independently derived ZyBooks/MACE model than the older Feature/WP-only workflow.

Need to determine:
- whether these records are production contract, handbook proposal or assessment-specific schema;
- how Assignment maps to Feature/WP;
- whether Epoch is evaluation basis, development spec revision or both;
- how evaluator identity/version binds;
- which fields are already executable/validated.

Do not duplicate these records in a new schema until this is resolved.

## Generation E — Mature-first overlay

Sources on `spec/mature-product-contract-v1`:
- RECOVERED_PRODUCT_INTENT;
- MACE_AUTOGRADER_DOCTRINE;
- mature-contract;
- journeys;
- SOTA gate;
- durable-memory governance.

Disposition: **KEEP as governing recovery program**, but it is not allowed to silently overwrite current-main implementation semantics.

It must reconcile A–D and then promote accepted decisions explicitly.

## Cross-generation decisions now safe

1. AgilePlus owns durable development/work state, not Tracera product truth.
2. Worker attempts are replaceable and distinct from durable development identity.
3. Claims/leases are execution authority, not product/development identity.
4. Worktree isolation is an implementation mechanism, not the durable work object.
5. Assignment/grader semantics should be reconciled with existing agent-lab schemas before new schema work.
6. Framework-specific file roots are ingress/materialization details, not ideal mature ontology.
7. Existing harmonization ADRs remain proposals until explicitly accepted/superseded.
8. Work completion/evidence collection in AgilePlus cannot directly award Tracera product acceptance.

## Highest-risk contradictions / gaps

- Active PRD lifecycle vs Proposed eight-stage lifecycle.
- Feature/WP model vs Assignment/Epoch/Criterion model.
- kitty-specs filesystem authority vs framework-neutral mature ontology.
- AgilePlus evidence acceptance vs Tracera product-level assessment ownership.
- in-memory ClaimStore mounted in application use case vs existing SQLite durability.
- March infrastructure assumptions vs current portfolio modular-monolith/tooling doctrine.
- multiple trace artifacts and graph models with unclear authority.

## Next

1. fetch and classify agent-lab schema/docs in full;
2. map Feature/WP/Assignment/Epoch/Criterion/Attempt/Claim identities;
3. propose one authority table and migration, not another independent model;
4. inspect actual persistence/API/CLI mounting for those identities;
5. update ADR statuses only after evidence-backed adjudication.
