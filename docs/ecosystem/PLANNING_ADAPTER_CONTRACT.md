# Planning Adapter Contract: Plane, Paperclip, and Future Providers

## Status

Proposed provider-neutral adapter contract for ADR-0021 and issue #1117.

This document does not choose Plane versus Paperclip, enable either adapter, or claim that the current Plane integration conforms. It defines the boundary every organization/personal planning adapter must satisfy before it can participate in AgilePlus rollout.

## Purpose

A planning provider may coordinate work one level above code projects. It may carry:

- objective/initiative hierarchy;
- planning task/work-item identity;
- priority and schedule;
- due date;
- human/team coordination assignment;
- discussion and convenience links;
- an observed planning status.

It does not own:

- AgilePlus WorkPackage identity or lifecycle;
- file scope or acceptance criteria;
- SpecRevision, Assignment, Attempt, or Evaluation;
- evidence provenance or audit chain;
- terminal development acceptance;
- acceptance receipts;
- Tracera product-state interpretation.

## Adapter identity

Every adapter instance has a stable `instance_uid` representing one account, tenant, installation, or deployment boundary.

A native work item is identified by:

```text
provider
instance_uid
external_kind
external_id
revision_state
external_revision (when pinned)
```

`external_id` is opaque. The adapter may use it in provider API calls but AgilePlus does not infer workspace, hierarchy, status, or authority from its syntax.

Two installations using `TASK-42` are different identities. Plane `TASK-42` and Paperclip `TASK-42` are different identities.

## Inbound observation

The normalized inbound planning observation is conceptually:

```text
observation_uid
provider
instance_uid
external_ref_uid
provider_event_id
provider_revision_or_etag
observed_at
status
priority
due_at
assignee_refs
title
summary
content_hash
raw_payload_ref
```

Rules:

- `provider_event_id` or an equivalent idempotency key is required when the provider exposes one.
- Provider status is stored as an observation/projection, never written directly into AgilePlus lifecycle state.
- Mutable title/summary/schedule fields follow explicit per-field projection policy.
- Missing fields mean unknown/not supplied, not deletion, unless the provider event explicitly represents deletion.
- Payloads are bounded, validated, and retained or referenced according to privacy policy.
- Webhook receipt acknowledgement does not imply semantic application; invalid events fail closed and remain diagnosable.

## Status mapping

Adapters may map provider-native statuses into a minimal planning vocabulary:

```text
planned
active
blocked
completed
cancelled
unknown
```

This vocabulary is a convenience projection only.

Forbidden mappings:

```text
planning completed -> WorkPackage Done
planning completed -> Feature Validated
planning completed -> acceptance receipt
planning cancelled -> deletion of AgilePlus evidence/history
```

An operator or policy may request local work changes after observing planning status, but those changes must enter the canonical AgilePlus command path and satisfy normal lifecycle/governance rules.

## Outbound projection

AgilePlus may emit planning updates after local events such as:

- feature/work-package creation;
- lifecycle progress suitable for coordination display;
- blocking reason;
- accepted completion;
- supersession/cancellation.

Every outbound request contains:

```text
outbox_uid
idempotency_key
project_uid
source_event_or_receipt_uid
external_ref_uid
projection_kind
expected_external_revision (when supported)
payload
attempt_count
next_attempt_at
```

Rules:

1. Local state commits before network delivery.
2. Delivery is retryable and idempotent.
3. Optimistic concurrency uses provider revision/etag when available.
4. A conflict becomes `pending_reconciliation` or `disputed`; it is not silently overwritten.
5. Permanent provider failure does not roll back local acceptance.
6. An outbound accepted-completion projection may close a planning item only when explicitly configured for that link.
7. Deleting/unlinking a planning record does not delete local work, evidence, audit, or receipts.

## Link policy

Recommended relation use:

| Local entity | Relation | Planning meaning |
|---|---|---|
| Project | `tracked_by` | Provider project/initiative displays project coordination |
| Feature | `planned_by` | Provider work item schedules/coordinates the feature |
| WorkPackage | `tracked_by` | Provider item displays implementation progress |
| SpecRevision | `references` | Planning item points to the accepted local intent revision |
| AcceptanceReceipt | `verifies` | Immutable completion receipt is projected as evidence/link |

A single planning item may link to multiple AgilePlus WPs. One WP may link to multiple planning views. Adapters must not collapse those edges into a single owner field.

## Read and write capabilities

Each adapter declares capabilities rather than relying on provider name:

```text
read_items
read_revisions
read_events
create_items
update_fields
update_status
idempotency_keys
optimistic_concurrency
webhooks
polling_cursor
attachment_links
```

AgilePlus must degrade explicitly when a capability is absent. For example, a provider without revisions uses `provider_has_no_revision`; it must not fabricate one.

## Security and privacy

- Credentials live in the AgilePlus credential boundary, not in links, URLs, exported envelopes, or browser storage.
- Provider URLs are convenience data and must reject embedded credentials.
- Adapter permissions are least privilege and instance-scoped.
- Inbound references to another configured instance/tenant are rejected.
- External descriptions/comments are untrusted text.
- Private artifact paths, secrets, local usernames, and unreviewed evidence are not projected.
- Logging records reference IDs and bounded diagnostics, not full credential-bearing payloads.

## Reconciliation

A reconciliation operation compares:

- canonical AgilePlus local projection state;
- last successfully delivered outbound revision;
- current provider revision;
- pending inbound observations;
- active link policy.

Possible outcomes:

```text
in_sync
local_ahead
provider_ahead
conflict
provider_missing
local_link_superseded
unavailable
```

No reconciliation outcome grants acceptance. A conflict can block further automatic projection while local development continues.

## Required adapter tests

Every provider adapter uses the same contract suite with a provider stub/recording server:

1. Identical native IDs across instances do not collide.
2. Duplicate webhook/event delivery is idempotent.
3. Out-of-order provider revisions do not overwrite newer observations.
4. Provider `completed` cannot change AgilePlus terminal state.
5. Accepted local completion projects once despite lost acknowledgement and retry.
6. Provider 401/403/404/409/429/500 and timeout preserve local state and produce typed retry/reconciliation outcomes.
7. Unlink preserves local history and other many-to-many links.
8. Offline provider leaves the complete local work/acceptance journey operational.
9. Invalid/oversized/untrusted payloads fail closed without acknowledgement-as-applied.
10. Export/import preserves provider and instance qualification.
11. CLI, API, MCP, web, and tray expose identical link semantics.
12. Credentials and secret-like fields never enter exported link envelopes.

## Plane compatibility migration

The existing `plane_sub_issue_id` and `sync_mappings` data are migration inputs, not canonical target fields.

For each record:

1. resolve the configured Plane `instance_uid`;
2. preserve the native issue ID as opaque `external_id`;
3. create one `ExternalRef` with the best evidenced kind/revision state;
4. create typed local links preserving the original entity mapping;
5. record source table/row and migration revision in a mapping receipt;
6. compare legacy and new read projections;
7. do not delete or overwrite the legacy source during initial migration.

Records without an unambiguous Plane instance are blocked for operator reconciliation rather than assigned a default global instance.

## Paperclip onboarding

Paperclip receives no special authority. Before enabling it:

- identify its actual native item, tenant, revision, event, and idempotency semantics from primary/source evidence;
- map those into this capability contract;
- keep native IDs opaque;
- run the shared negative controls;
- document unsupported capabilities and fallback polling/reconciliation;
- prove that `done` remains a planning observation only.

## Invariant

**Planning adapters coordinate and project AgilePlus work. They do not define whether the work is accepted.**
