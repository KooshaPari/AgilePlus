---
work_package_id: WP15
title: 'heliosApp: Aggregated Audit Trail View'
lane: planned
dependencies: [WP11]
subtasks: [T039]
history:
- date: '2026-03-03'
  event: created
  by: spec-kitty.tasks
---

# WP15: heliosApp — Aggregated Audit Trail View

**Implementation command**: `spec-kitty implement WP15 --base WP12`

## Objective

Implement an aggregated audit trail view in heliosApp that consumes structured audit events from all 6 repos, displaying end-to-end execution traces.

## Context

- @helios/audit-core extracted in WP11, now built on top of **Emmett** event store (see WP11 T034)
- All repos emit AuditEvent per WP12
- Events have correlation_id for cross-repo tracing
- This is the read-side of the CQRS pattern
- **With Emmett adoption**: the audit trail view should consume Emmett's projection and replay APIs directly rather than reimplementing custom read-model logic

## Subtasks

### T039: Aggregated audit trail view

> **Emmett integration**: Use Emmett's projection engine and `readStream` / `readAll` APIs instead of custom ingestion and replay code. Emmett projections automatically maintain read models as events are appended, eliminating the need for manual SQLite read-side updates.

**Steps**:
1. Create audit trail ingestion:
   - Consume events from all repos (file-based, API, or event bus — start with file-based for MVP)
   - Append to Emmett event store via @helios/audit-core (which wraps Emmett)
   - Define Emmett projections for: timeline read-model, per-repo aggregation, correlation trace
2. Create UI component:
   - Timeline view showing events chronologically (reads from Emmett projection)
   - Filter by repo, event type, correlation_id
   - Expand event details (payload, actor, timestamps)
   - Trace view: follow a correlation_id across repos to see full execution path
3. Implement replay:
   - Use Emmett's `readStream` with time-range to replay event sequences
   - Use Emmett inline snapshots for efficient state reconstruction at any point in time
4. Wire into heliosApp navigation (new "Audit" tab/panel)

**Files**:
- `apps/desktop/src/components/audit/AuditTrailView.tsx` (~200 lines)
- `apps/desktop/src/components/audit/EventTimeline.tsx` (~150 lines)
- `apps/desktop/src/components/audit/TraceView.tsx` (~150 lines)
- `apps/runtime/src/services/audit-ingestion.ts` (~100 lines)

**Validation**:
- Events from multiple repos display in timeline
- Filtering works correctly
- Correlation ID trace shows cross-repo flow
- Replay reconstructs state accurately

## Definition of Done

- [ ] Audit events from all repos viewable in heliosApp
- [ ] Filter by repo, type, correlation_id
- [ ] Trace view follows execution across repos
- [ ] Replay engine integrated
- [ ] UI accessible from main navigation

## Risks

- **Event ingestion at scale**: Start with file-based; plan for event bus (NATS, Redis Streams) later.
- **UI performance**: Large event volumes need virtualized lists and pagination.

## Reviewer Guidance

- Verify cross-repo trace correctly follows correlation_ids
- Check replay accuracy against original event sequence
- Ensure UI is responsive with large event volumes (test with 10K+ events)
