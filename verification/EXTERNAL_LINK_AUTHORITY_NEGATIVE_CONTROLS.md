# External-Link Authority Negative Controls

## Status

Executable verification plan for ADR-0021 and issue #1117.

The cases below are **not evidence that the implementation exists or passes**. They define the minimum same-fixture oracles required before stable identity migration, provider adapters, or multi-device rollout can be called complete.

## Authority oracle

For every case, capture the complete before/after state of:

- Feature and WorkPackage lifecycle state;
- SpecRevision, Assignment, Attempt, and Evaluation identities;
- evidence rows and artifact digests;
- audit/event chain heads;
- acceptance receipts;
- external references and links;
- projection outbox/delivery records;
- exported stable-identity envelope.

A test fails if an external input changes any AgilePlus terminal state or acceptance record without the canonical local acceptance use case producing a valid receipt in the same transaction.

## Fixture identities

Use deterministic fixture UIDs so CLI, HTTP, gRPC/MCP, export/import, and adapter tests compare identical values.

```text
project_uid: 0199f2a5-4a64-7f90-8000-000000000001
feature_uid: 0199f2a5-4a64-7f90-8000-000000000002
wp_uid_a:    0199f2a5-4a64-7f90-8000-000000000003
wp_uid_b:    0199f2a5-4a64-7f90-8000-000000000004
tracera_instance_uid: 0199f2a5-4a64-7f90-8000-000000000010
plane_instance_uid:   0199f2a5-4a64-7f90-8000-000000000011
paperclip_instance_uid: 0199f2a5-4a64-7f90-8000-000000000012
```

Provider-native IDs deliberately collide in several fixtures:

```text
external_id = "TASK-42"
```

The collision must remain harmless because provider plus instance plus kind plus revision state qualify the identity.

## NC-EXT-001 — planning completion cannot grant acceptance

**Arrange**

- WP A is in `Review` with no accepted Evaluation and no acceptance receipt.
- A Plane or Paperclip `tracked_by` link exists.

**Act**

- Ingest a signed/valid provider event marking the planning record `done` or `closed`.

**Require**

- WP A remains `Review`.
- Feature state is unchanged.
- No acceptance receipt, accepted Evaluation, audit transition, or terminal event appears.
- The provider status is stored only in its projection/observation record.
- CLI, API, MCP, and desktop/web projections report the same result.

## NC-EXT-002 — product satisfaction cannot fabricate development proof

**Arrange**

- A pinned Tracera product revision links to the feature and WP A.
- WP A has no accepted candidate.

**Act**

- Ingest a Tracera observation that the product record is satisfied.

**Require**

- No AgilePlus acceptance record is created.
- The Tracera observation remains product-domain evidence only.
- `validate` and terminal acceptance still fail for the missing local assignment/evaluation.

## NC-EXT-003 — provider and instance collisions remain distinct

Create references with identical `external_id = "TASK-42"` for:

1. Plane instance A;
2. Plane instance B;
3. Paperclip instance A;
4. Tracera instance A, kind `change`.

**Require**

- Four distinct `external_ref_uid` records.
- Lookup by unqualified `external_id` fails as ambiguous.
- Qualified lookup returns exactly one record.
- Export/import round-trip preserves all four.

## NC-EXT-004 — many-to-many links are not collapsed

**Arrange**

- One planning task links to WP A and WP B.
- WP A links to one Tracera change and two planning views.

**Require**

- Every active typed edge survives storage, API serialization, export/import, and re-open.
- Removing one edge does not remove the reference or sibling edges.
- Duplicate creation of the same active edge is idempotent or rejected deterministically.

## NC-EXT-005 — Tracera revision change creates new local intent

**Arrange**

- SpecRevision S1 is pinned to Tracera revision T1.
- WP A is accepted against S1 and candidate C1.

**Act**

- Observe Tracera revision T2 for the same product record.

**Require**

- S1, its Assignment, Evaluation, evidence, candidate, and receipt remain byte-for-byte immutable.
- T2 creates or requests a new SpecRevision/Assignment path.
- T2 cannot relabel the S1 receipt as satisfying T2.
- The relationship between T1 and T2 is explicit supersession, not row overwrite.

## NC-EXT-006 — outbound retry is idempotent

**Arrange**

- Local acceptance commits receipt R1 and outbox item O1.
- The external provider applies the update but the client loses the acknowledgement.

**Act**

- Retry O1 repeatedly, including after daemon restart.

**Require**

- The provider observes one logical effect.
- O1 retains one stable idempotency key.
- AgilePlus retains one acceptance receipt and one source event.
- Delivery attempts are append-only observations; they do not duplicate authority records.

## NC-EXT-007 — projection failure cannot roll back acceptance

Inject timeout, authentication failure, provider 500, malformed provider response, and permanent provider deletion after local acceptance.

**Require**

- The local acceptance transaction remains committed and verifiable.
- Outbox state becomes retryable or terminally failed with evidence.
- No compensation changes WP/Feature terminal state.
- Operator replay can resume after credentials/connectivity return.

## NC-EXT-008 — duplicate import cannot create competing accepted intent

**Arrange**

- Import an envelope that links one external reference to an existing accepted WP/spec revision.

**Act**

- Re-import the same envelope.
- Import a modified envelope using the same UIDs but incompatible local/external identity.

**Require**

- Identical replay is idempotent.
- Incompatible reuse fails before any mutation.
- No second active SpecRevision, Assignment, accepted Evaluation, or receipt appears.
- A collision receipt identifies the exact conflicting records.

## NC-EXT-009 — external systems may be absent

Run the complete local journey with Tracera, Plane, and Paperclip endpoints unavailable:

```text
create feature
→ create governed WP
→ bind SpecRevision/Assignment
→ execute and record Attempt/Evaluation/evidence
→ validate exact candidate
→ commit acceptance receipt
→ restart core
→ verify audit/event/receipt
```

**Require**

- Every local operation succeeds or fails solely on local contract/evidence.
- No external timeout is on the local acceptance critical path.
- Pending outbound projections remain inspectable and replayable.

## NC-EXT-010 — relocation and export/import preserve identity

**Arrange**

- Export a project with stable UIDs, references, links, receipts, and supersession history.
- Move the repository/database to a different filesystem path and device fixture.

**Require**

- Integer database keys may differ after a supported rebuild/import.
- `project_uid`, `feature_uid`, `wp_uid`, external reference UIDs, and receipt UIDs remain stable.
- Canonical AgilePlus URIs resolve to the same logical entities.
- No absolute old-device path is treated as identity.

## NC-EXT-011 — client surfaces cannot diverge

Perform link, unlink/supersede, list, and qualified lookup through each supported surface.

**Require**

- CLI, HTTP API, gRPC/MCP, web, and tray use one application contract.
- No surface writes provider tables directly.
- Unsupported operations return typed errors rather than silently dropping rows.
- An external status update yields the same non-authoritative observation on every surface.

## NC-EXT-012 — acceptance receipt projection is one-way

**Arrange**

- Receipt R1 is linked outward to Tracera and a planning task.

**Act**

- Modify or delete the external projections.

**Require**

- R1, its hash/evidence bindings, audit event, and candidate identity remain unchanged.
- Reconciliation reports missing/divergent projections.
- Pull synchronization cannot import an altered copy as a replacement receipt.

## Migration gates

Before enabling dual-write:

- all stable UID columns exist and are unique/non-null for migrated entities;
- deterministic/backfill mapping receipt is complete;
- provider references are instance-qualified;
- legacy Plane rows map one-for-one into provider-neutral records;
- migration is rerunnable without new UIDs or duplicate links.

Before switching reads:

- old/new projection comparison is clean on a production-shaped fixture;
- CLI/API/MCP return identical stable identities;
- export/import and relocation controls pass;
- offline local acceptance passes.

Before stopping legacy writes:

- rollback views reproduce the legacy interface from canonical new records;
- adapter delivery and idempotency evidence exists;
- collision and supersession operations are operator-visible.

Before removing legacy fields:

- separate approval and migration;
- backup and restoration rehearsal;
- retained mapping receipts and compatibility window;
- no supported client consumes the physical legacy fields.

## Evidence contract

Each executed case records:

- immutable source revision and schema version;
- fixture/export hashes;
- exact command/request payloads;
- before/after database and event-chain summaries;
- provider stub observations;
- restart/relocation environment;
- exit status and logs;
- pass/fail oracle with no self-generated expected values;
- unresolved platform/provider limitations.
