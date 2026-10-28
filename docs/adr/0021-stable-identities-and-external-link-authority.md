# ADR-0021: Stable AgilePlus Identities and Non-Authoritative External Links

## Status

**Authority boundary accepted by the operator on 2026-10-12.**

The implementation and migration details in this ADR remain **proposed until executable migration, export/import, adapter, and negative-control evidence passes**. This status distinction matters: the repository may implement the accepted authority split only through evidence-backed slices; this document alone does not advance any feature to Validated or Shipped.

## Refines

- [ADR-0018: Tracera and AgilePlus Are Interoperable Sibling Runtimes](0018-tracera-agileplus-sibling-runtimes.md)
- [ADR-0019: Acceptance Authority and Terminal-State Gates](0019-acceptance-authority-terminal-gates.md)
- Issue #1117, which is the current implementation tracker

This ADR does not supersede the sibling-runtime or acceptance invariants. It makes their identity, link, and projection consequences explicit.

## Human ruling

For code projects:

- **AgilePlus owns governed development work packages and terminal development acceptance.**
- **Tracera owns persistent product/system records and product-state interpretation.**
- **Plane or Paperclip, if adopted, owns organization/personal planning one level above code projects.**
- No Tracera, Plane, Paperclip, dashboard, tray client, import, or projection may fabricate an AgilePlus acceptance receipt or directly grant an AgilePlus terminal state.

## Context

The current implementation uses repository-local integer primary keys such as `WorkPackage.id`. Those are efficient local join keys but are not durable cross-system identities. The `WorkPackage` domain type also contains one optional `plane_sub_issue_id`, while `sync_mappings` is Plane-specific and assumes one mapping per local entity plus one local owner per Plane issue.

That shape cannot safely represent:

- a project database moved or rebuilt on another device;
- stable export/import references;
- one product or planning record linked to several work packages;
- one work package projected into several systems or views;
- two provider installations reusing the same opaque native ID;
- a pinned Tracera product-record revision;
- immutable historical links when an external record moves, renames, or is superseded;
- external-system unavailability without weakening local work or acceptance.

A provider-native ID is not a global identity. A synchronized status is not authority. A successful projection is not acceptance.

## Decision

### 1. Preserve local integer keys and add immutable stable UIDs

Existing SQLite integer primary keys remain for local joins, compatibility, and migration safety.

The canonical model gains immutable, globally unique, time-sortable 128-bit identities:

- `project_uid`
- `feature_uid`
- `wp_uid`

UUIDv7 is the initial representation. An imported valid UID is preserved; a new UID is minted only for a new logical entity. UIDs are never recycled after deletion, supersession, export, import, or database rebuild.

Human labels such as `WP01` remain sequence/display identifiers. They are not cross-project identity.

Canonical reference forms:

```text
agileplus://<project_uid>
agileplus://<project_uid>/features/<feature_uid>
agileplus://<project_uid>/work-packages/<wp_uid>
```

A parser must reject a URI whose embedded `project_uid` disagrees with the enclosing project context.

### 2. Provider identities are stored once as opaque external references

AgilePlus introduces a provider-neutral `ExternalRef` registry. Its logical fields are:

```text
external_ref_uid
provider
instance_uid
external_kind
external_id
external_revision
external_url
content_hash
first_observed_at
last_verified_at
superseded_by_ref_uid
```

Rules:

- `provider` identifies an adapter namespace, not authority.
- `instance_uid` identifies the account, installation, deployment, or tenant boundary.
- `external_id` is opaque. AgilePlus does not parse semantics from its shape.
- `(provider, instance_uid, external_kind, external_id, external_revision)` identifies one observed external revision.
- An absent native revision is represented explicitly, never silently replaced with `latest`.
- `external_url` is a mutable convenience projection, not identity.
- `content_hash` pins an observed payload when available.
- Supersession creates a new reference and link history; it does not overwrite accepted history.

The existing `trace_links` external endpoint may project these relationships, but it must not become a second provider-identity registry.

### 3. Local-to-external relationships are typed and many-to-many

A separate `ExternalLink` registry relates stable local UIDs to `ExternalRef` records.

Logical fields:

```text
link_uid
project_uid
local_kind
local_uid
external_ref_uid
relation
direction
authority_domain
conflict_state
created_at
superseded_at
```

Supported local kinds initially include:

- `project`
- `feature`
- `work_package`
- `spec_revision`
- `acceptance_receipt`

Supported relations initially include:

- `implements`
- `verifies`
- `references`
- `planned_by`
- `tracked_by`
- `supersedes`

The model permits one external record to link to several local entities and one local entity to link to several external records. Link uniqueness prevents duplicate active edges without forbidding historical supersession.

### 4. Authority is domain- and field-specific

| Domain | Canonical authority | External behavior |
|---|---|---|
| WP lifecycle, file scope, acceptance criteria | AgilePlus | Read-only projection elsewhere |
| SpecRevision, Assignment, Attempt, Evaluation | AgilePlus | May reference product/planning records |
| Evidence, audit chain, acceptance receipt | AgilePlus | Immutable observation projected outward |
| Product identity, capability graph, product-state interpretation | Tracera | Referenced by pinned revision |
| Organization/personal hierarchy, schedule, due date, planning status | Plane/Paperclip | Observed coordination metadata |
| Shared title/summary | Explicit per-field policy | Never silent last-write-wins |

No integration may infer authority from recency, a green color, a `done` string, a webhook source, or connectivity.

### 5. Tracera integration is revision-pinned and independently reconciled

- A Tracera reference identifies a product record/change/capability and, when available, a specific product revision.
- An immutable AgilePlus `SpecRevision` records which Tracera revision informed a development assignment.
- AgilePlus evaluates the locally pinned specification and exact code candidate.
- AgilePlus acceptance emits an immutable observation containing the acceptance receipt, evidence references, candidate identity, and pinned product reference.
- Tracera independently decides whether product truth is satisfied.
- A newer Tracera revision creates a new local spec revision or assignment. It cannot rewrite an accepted receipt.
- AgilePlus creation, execution, validation, and acceptance continue when Tracera is unavailable.

### 6. Plane/Paperclip integration is planning projection, not acceptance

- Planning systems may own objective hierarchy, scheduling, priority, due date, and coordination assignment.
- External `done`, `closed`, or equivalent state is recorded as an observed planning status only.
- It cannot set an AgilePlus WP to `Done`, set a Feature to `Validated`, or create an acceptance receipt.
- AgilePlus acceptance may enqueue an idempotent one-way update to a linked planning record.
- Projection delivery failure never rolls back committed local acceptance.
- AgilePlus continues operating when the planning system is unavailable.
- This contract does not choose Plane versus Paperclip.

### 7. Outbound effects use a durable outbox and idempotency identity

Acceptance and external projection are separate transactions.

The local acceptance transaction commits first. It may atomically append an outbound projection request to a local outbox, but a network call is never part of terminal acceptance.

Each outbound request carries a stable idempotency identity derived from:

```text
project_uid + source_event_or_receipt_uid + external_ref_uid + projection_kind
```

Retries may repeat transport but must not create duplicate external effects. Permanent delivery failure remains visible and replayable without altering the accepted local receipt.

### 8. Import/export preserves identity and rejects ambiguity

Export includes stable local UIDs, provider/instance-qualified external references, link history, hashes, and explicit supersession.

Import rules:

- preserve an identical logical entity UID;
- reject one UID mapped to two incompatible entities;
- reject one canonical project root claiming two project UIDs without explicit reconciliation;
- reject external IDs lacking `instance_uid`;
- reject active duplicate links that disagree on authority or relation;
- never convert an imported planning/product state into accepted development state;
- emit a migration/import receipt that records mappings, collisions, skips, and operator resolutions.

### 9. Compatibility fields become projections, then are retired

`plane_sub_issue_id` and Plane-specific `sync_mappings` remain readable during migration but cease to be canonical.

Migration order:

1. Add stable UIDs and provider-neutral tables without deleting old fields.
2. Deterministically backfill or record one-time generated UIDs.
3. Convert each Plane-specific mapping into an `ExternalRef` plus typed `ExternalLink`.
4. Compare old and new projections bidirectionally.
5. Switch read surfaces to the provider-neutral registries.
6. Stop writing legacy fields.
7. Retain compatibility views and mapping receipts for rollback/audit.
8. Remove legacy physical fields only in a later, separately approved migration.

## Required negative controls

1. Plane/Paperclip `done` cannot change AgilePlus terminal state.
2. Tracera `satisfied` cannot fabricate an AgilePlus acceptance receipt.
3. Identical opaque IDs from two providers or instances do not collide.
4. Many-to-many local/external links survive round-trip serialization.
5. A newer Tracera revision creates a new local revision/assignment and preserves prior receipts.
6. Retried outbound projection is idempotent.
7. Projection failure after acceptance does not roll back local state.
8. Importing an already-linked reference cannot create competing accepted intent.
9. All local work and acceptance operations function with external systems unavailable.
10. Database relocation and export/import preserve stable references.

## Consequences

### Positive

- Stable cross-device and cross-runtime references.
- External products remain optional.
- Acceptance authority cannot drift into planning or product dashboards.
- One provider adapter can be replaced without changing local identity.
- Link and supersession history remains auditable.
- Multi-device migration can detect collisions rather than silently merging them.

### Costs

- New schema, ports, serialization, migration, and compatibility logic.
- Existing Plane-specific code must be adapted rather than merely renamed.
- Eventual consistency and explicit conflict states are unavoidable.
- External adapters require durable retry/outbox behavior.
- UIDs and integer keys coexist permanently or for a long migration horizon.

## Rejected alternatives

### Use local integer IDs externally

Rejected because they collide across databases, devices, exports, and rebuilds.

### Make `trace_links` the provider registry

Rejected because graph edges do not own provider identity, revision, instance qualification, verification timestamp, or supersession semantics.

### Use external system IDs as AgilePlus identity

Rejected because it makes local operation dependent on an optional provider and transfers identity authority outside AgilePlus.

### Mirror external terminal status directly into AgilePlus state

Rejected because it violates ADR-0019 and allows planning/product systems to bypass evidence-bound acceptance.

### Perform external updates inside the acceptance transaction

Rejected because network availability would control or roll back local acceptance and would make retries unsafe.

## Rollback and abort conditions

Abort migration if any fixture demonstrates UID instability, ambiguous external identity, receipt mutation, acceptance rollback after projection failure, or incompatible CLI/API/MCP semantics.

Rollback means:

- keep stable UID columns and mapping receipts;
- restore reads from legacy fields/views;
- stop provider-neutral writes;
- never delete accepted receipts or external-link history;
- reconcile forward through a new migration rather than rewriting applied history.

## Invariants

**External linkage may explain, coordinate, or project AgilePlus work. It cannot grant AgilePlus acceptance.**

**Development truth, product truth, and planning truth remain connected but separately authoritative.**
