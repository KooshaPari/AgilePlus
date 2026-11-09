# Tracera Handoff: AgilePlus Stable Identity and Acceptance Observation Contract

## Purpose

This is a **copy/paste handoff for the already-running Tracera forensic recovery**. It does not authorize AgilePlus work to edit, merge, rename, migrate, or deploy the Tracera repository.

The Tracera recovery should evaluate this contract against Tracera’s recovered product model and return a compatibility decision with exact evidence. AgilePlus must not guess Tracera’s canonical entity, revision, or reconciliation shapes.

## Accepted ecosystem boundary

- Tracera owns persistent product/system records, product identity, product/capability graph, and product-state interpretation.
- AgilePlus owns governed code-project work packages, SpecRevision/Assignment/Attempt/Evaluation, evidence, development acceptance, audit, and acceptance receipts.
- Neither runtime is a mandatory dependency of the other.
- AgilePlus development acceptance does not mark a Tracera product obligation satisfied.
- Tracera product satisfaction does not create an AgilePlus acceptance receipt.

See:

- `docs/adr/0018-tracera-agileplus-sibling-runtimes.md`
- `docs/adr/0019-acceptance-authority-terminal-gates.md`
- `docs/adr/0021-stable-identities-and-external-link-authority.md`
- `spec/product/external-link-contract.v1.schema.json`
- `verification/EXTERNAL_LINK_AUTHORITY_NEGATIVE_CONTROLS.md`

## Requested Tracera recovery output

Please return a source-backed handoff covering the following. Preserve Tracera’s recovered terminology rather than mechanically adopting AgilePlus names.

### 1. Product-record identity

Identify Tracera’s canonical identifiers for:

- product/system;
- capability/requirement/obligation;
- product change or dissatisfaction record;
- immutable product revision/baseline/configuration;
- evidence observation;
- reconciliation/assessment result.

For each identifier, state:

- owning record/type;
- stability across rename, move, export/import, and database relocation;
- whether revisions have distinct immutable IDs;
- whether IDs are globally unique or instance-qualified;
- current implementation evidence versus proposed target behavior.

### 2. Revision pinning

Define the smallest immutable Tracera reference that an AgilePlus `SpecRevision` can pin.

The answer must distinguish:

- mutable product-record identity;
- immutable product revision/baseline identity;
- optional content hash;
- observed time;
- Tracera instance/deployment identity.

Do not return `latest` as an immutable reference. If Tracera currently lacks immutable revision identity, report that as a gap rather than inventing one.

### 3. AgilePlus acceptance observation

Evaluate this proposed outbound observation shape:

```text
observation_uid
idempotency_key
tracera_instance_uid
tracera_product_ref
tracera_revision_ref
agileplus_project_uid
agileplus_feature_uid
agileplus_wp_uid(s)
agileplus_spec_revision_uid
agileplus_acceptance_receipt_uid
candidate_ref
evidence_refs + hashes
audit/event references
accepted_at
source_contract_version
```

Required behavior:

- ingest is idempotent;
- duplicate delivery cannot duplicate product evidence;
- Tracera independently reconciles product satisfaction;
- ingestion failure cannot roll back AgilePlus acceptance;
- a later Tracera revision cannot rewrite the historical AgilePlus receipt;
- disputed or stale observations remain explicit rather than silently overwritten.

Return which fields Tracera can currently consume, which require adapter work, and which conflict with Tracera’s recovered model.

### 4. Product-to-development request

Define the proposed Tracera-to-AgilePlus reference used to initiate or relate development work:

```text
tracera_instance_uid
product_record_ref
product_revision_ref
change_or_dissatisfaction_ref
requested_relation
content_hash (when available)
observed_at
```

This request may inform an AgilePlus `SpecRevision` or Assignment. It must not create accepted development intent without AgilePlus’s local specification/governance process.

### 5. Reconciliation states

Map Tracera’s recovered reconciliation vocabulary to these minimum outcomes without collapsing distinctions:

- observed, not assessed;
- evidence accepted for consideration;
- product obligation satisfied;
- product obligation unsatisfied;
- disputed;
- stale/superseded;
- unavailable/unknown.

Do not map an AgilePlus `Done`/`Validated` state directly to product satisfaction.

### 6. Availability and retry contract

Confirm how the integration behaves when Tracera is offline:

- AgilePlus continues local creation/execution/acceptance;
- observations remain in an AgilePlus outbox;
- reconnect/retry is idempotent;
- Tracera can request replay from a stable cursor or observation UID;
- no live Tracera call is inside the AgilePlus acceptance transaction.

### 7. Security and tenancy

Specify:

- Tracera instance/tenant qualification;
- authentication boundary;
- least-privilege write scope for AgilePlus observations;
- rejection of references to another tenant/instance;
- provenance required before an observation is considered verified;
- redaction rules for private evidence/artifact locations.

## Required contradiction checks

The Tracera recovery must explicitly test these alternatives:

1. Tracera has one persistent product identity but no immutable revision identity.
2. Current graph-node IDs are projection IDs rather than canonical record IDs.
3. Historical “Tracera acceptance” terminology actually refers to product interpretation, not development acceptance.
4. Existing integration code assumes AgilePlus embedding or mandatory availability and is superseded by ADR-0018.
5. A prior shared ID type exists but does not preserve authority or instance qualification.

For each, state evidence, confidence, and what would overturn the conclusion.

## Required response format

Return a handoff containing:

1. exact Tracera revision(s) inspected;
2. identity/revision capability table;
3. compatible contract fields;
4. incompatible or missing fields;
5. adapter and migration work packages;
6. negative tests;
7. authority conflicts;
8. unresolved decisions requiring Koosha;
9. an explicit statement that no AgilePlus acceptance authority moved to Tracera.

## Non-goals

- No direct Tracera changes from the AgilePlus issue.
- No universal graph ontology.
- No requirement that both runtimes be deployed together.
- No assumption that the current Tracera branch or latest document is canonical without its forensic recovery.
- No acceptance of product green merely because code work completed.
