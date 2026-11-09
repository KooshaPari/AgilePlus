# AgilePlus Methodology Specification

**Version:** 1.0 | **Status:** Active | **Date:** 2026-03-31

AgilePlus is a local-first, spec-driven project management platform with event-sourced persistence and multi-VCS support. This document describes the AgilePlus methodology as implemented in the AgilePlus platform.

---

## Overview

AgilePlus transforms how teams manage software projects by treating specifications as the source of truth. Every feature, work package, and governance decision flows from a spec, creating a traceable path from customer requirement to implemented code.

### Core Philosophy

- **Spec as Contract:** Specifications define what will be built, not how it will be built
- **Event-Sourced Audit:** All state changes are recorded as immutable events with cryptographic integrity
- **Local-First:** Work continues offline; sync is a convenience, not a requirement
- **Hexagonal Architecture:** Domain logic isolated from infrastructure via well-defined ports
- **Multi-VCS Integration:** Git is the primary VCS; other systems can be adapted via plugins

---

## Core Concepts

### 1. Features

Features are the central planning unit in AgilePlus, representing a user-facing capability or technical improvement.

| Attribute | Description |
|-----------|-------------|
| `id` | Unique identifier (UUID) |
| `title` | Short description of the feature |
| `description` | Detailed specification in markdown |
| `state` | Current position in the feature lifecycle |
| `spec_hash` | SHA-256 hash of the specification document |
| `target_branch` | Git branch where this feature will be merged |
| `plane_id` | Optional Plane.so issue ID for sync |
| `module_id` | Parent module this feature belongs to |
| `priority` | `P1` (critical), `P2` (important), `P3` (nice-to-have) |
| `labels` | Categorization tags |

#### Feature State Machine

```
Created → Specified → Researched → Planned → Implementing → Validated → Shipped → Retrospected
     ↓           ↓           ↓         ↓            ↓             ↓         ↓
  [skipped]  [skipped]   [skipped] [skipped]    [blocked]   [failed]  [archived]
```

| State | Description |
|-------|-------------|
| `Created` | Feature idea recorded, not yet elaborated |
| `Specified` | Full spec written and hash computed |
| `Researched` | Technical approach validated |
| `Planned` | Work packages defined, dependencies mapped |
| `Implementing` | Active development in progress |
| `Validated` | Tests passing, governance satisfied |
| `Shipped` | Merged to target branch, released |
| `Retrospected` | Post-mortem complete, lessons learned |

State transitions are enforced by the domain model and logged to the event store.

### 2. Work Packages

Work Packages (WPs) are atomic units of implementation work scoped to a single concern.

| Attribute | Description |
|-----------|-------------|
| `id` | Unique identifier (UUID) |
| `feature_id` | Parent feature this WP belongs to |
| `title` | Short description of the work |
| `state` | Current position in the WP lifecycle |
| `assignee` | Agent or team responsible |
| `file_scope` | Files this WP is allowed to modify |
| `depends_on` | List of WP IDs that must complete first |
| `evidence_requirements` | Governance evidence types needed |

#### Work Package State Machine

```
Planned → Doing → Review → Done
   ↓       ↓       ↓      ↓
 Blocked  Blocked Blocked Skipped
```

| State | Description |
|-------|-------------|
| `Planned` | Defined but not started |
| `Doing` | Actively being implemented |
| `Review` | Under review or testing |
| `Done` | Completed and verified |
| `Blocked` | Waiting on external dependency |
| `Skipped` | Not needed after reconsideration |

### 3. Modules

Modules organize Features into coherent groups, typically corresponding to a subsystem or bounded context.

| Attribute | Description |
|-----------|-------------|
| `id` | Unique identifier |
| `name` | Module name (e.g., "sync", "api", "domain") |
| `description` | Module purpose and scope |
| `cycle_id` | Current development cycle |

### 4. Cycles

Cycles represent time-boxed development periods (similar to sprints or iterations).

| Attribute | Description |
|-----------|-------------|
| `id` | Unique identifier |
| `name` | Cycle name (e.g., "2026-Q1-S1") |
| `start_date` | Cycle start |
| `end_date` | Cycle end |
| `status` | `planning`, `active`, `closing`, `closed` |

---

## Event Sourcing

AgilePlus uses event sourcing for all state mutations, providing a complete audit trail and the ability to rebuild state at any point.

### Event Structure

```rust
struct Event {
    id: Uuid,
    entity_type: EntityType,      // Feature, WorkPackage, Cycle, etc.
    entity_id: Uuid,
    event_type: EventType,          // Created, Transitioned, Updated, etc.
    payload: JsonValue,             // Event-specific data
    timestamp: DateTime,
    actor: String,                 // Who/what triggered this event
    prev_hash: Option<Sha256Hash>,  // Hash of previous event in chain
    hash: Sha256Hash,               // SHA-256(entity_id + event_type + payload + timestamp + prev_hash)
}
```

### Hash Chain Integrity

Each event contains a SHA-256 hash linking to the previous event, forming an tamper-evident chain:

```
Event(N) = SHA256(Event(N-1).hash + entity_id + event_type + payload + timestamp)
```

This allows detection of any tampering with historical events.

### Snapshots

To avoid replaying thousands of events for frequently-accessed entities:

- Snapshots are taken every 100 events or 5 minutes (configurable)
- Current state is served from the latest snapshot
- Snapshot integrity is verified on startup

---

## Governance & Evidence

AgilePlus enforces governance contracts that require evidence before features can transition to validated state.

### Governance Contracts

```rust
struct GovernanceContract {
    id: Uuid,
    feature_id: Uuid,
    version: u32,
    rules: Vec<PolicyRule>,
    enforce: bool,
}
```

### Policy Rules

```rust
struct PolicyRule {
    id: Uuid,
    domain: Domain,           // Test, Security, Review, etc.
    severity: Severity,       // Info, Warning, Error, Critical
    evidence_type: EvidenceType,
    description: String,
    auto_enforce: bool,
}
```

### Evidence Types

| Type | Description |
|------|-------------|
| `TestResults` | Output from test suite execution |
| `CiOutput` | CI/CD pipeline logs |
| `SecurityScan` | Vulnerability scan results |
| `ReviewApproval` | Human reviewer sign-off |
| `LintResults` | Linter/formatter check output |
| `ManualAttestation` | Human-declared compliance |

### Validation Flow

```
Feature reaches "Implementing" state
         ↓
All policy rules evaluated
         ↓
Evidence collected for each rule
         ↓
[If auto_enforce=true and all evidence present]
         ↓
Feature transitions to "Validated"
[If manual enforcement required]
         ↓
Dashboard shows pending approvals
         ↓
Human reviewer approves/rejects
         ↓
Feature transitions based on decision
```

---

## Spec-Driven Development

### The Specification Document

Each Feature SHOULD have a corresponding spec document (e.g., `spec.md` in the feature directory):

```markdown
# Feature: Bidirectional Plane.so Sync

**Status:** Draft | **Created:** 2026-03-15

## User Story

As a developer, I want features created in AgilePlus to automatically
appear in Plane.so so that my team can track progress in their
preferred tool.

## Acceptance Criteria

1. Feature created in AgilePlus → Plane.so issue created within 5s
2. Plane.so state change → AgilePlus state updated within 3s
3. Conflict detected when both sides edit same field

## Technical Approach

- Use Plane.so REST API for CRUD operations
- Webhooks for real-time Plane.so updates
- Local event store as source of truth
- Conflict resolution: last-write-wins with full audit

## Dependencies

- agileplus-plane crate
- Plane.so API key
- NATS for event distribution
```

### Spec Hash Computation

The spec document content is hashed with SHA-256:

```rust
spec_hash = SHA256(trim_trailing_whitespace(read("spec.md")))
```

This hash is stored on the Feature and verified before state transitions to ensure spec hasn't changed mid-development.

---

## Hexagonal Architecture

AgilePlus follows hexagonal (ports-and-adapters) architecture to keep domain logic pure and infrastructure swappable.

### Layer Structure

```
┌─────────────────────────────────────────────────────────────┐
│                      Adapters                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │  REST API   │  │    CLI      │  │   gRPC      │        │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
│         │                │                │                 │
│  ┌──────┴────────────────┴────────────────┴──────┐         │
│  │              Application Layer               │         │
│  │         Commands, Queries, Handlers           │         │
│  └──────────────────────┬───────────────────────┘         │
│                         │                                   │
│  ┌──────────────────────┴───────────────────────┐         │
│  │                 Domain Layer                   │         │
│  │   Entities, Value Objects, Domain Services    │         │
│  │   Ports (traits), Domain Events                │         │
│  └──────────────────────┬───────────────────────┘         │
│                         │                                   │
│  ┌──────────────────────┴───────────────────────┐         │
│  │            Infrastructure Adapters             │         │
│  │  SQLite Storage, Git VCS, NATS Messaging      │         │
│  │  Plane.so Integration, Cache Layer             │         │
│  └───────────────────────────────────────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

### Ports (Traits)

```rust
// Primary (driving) ports
trait FeatureQuery {
    fn get_feature(&self, id: Uuid) -> Result<Feature>;
    fn list_features(&self, filter: FeatureFilter) -> Result<Vec<Feature>>;
}

trait FeatureCommand {
    fn create_feature(&self, cmd: CreateFeature) -> Result<Feature>;
    fn transition_feature(&self, cmd: TransitionFeature) -> Result<Event>;
    fn update_feature(&self, cmd: UpdateFeature) -> Result<Event>;
}

// Secondary (driven) ports
trait StoragePort {
    fn append_event(&self, event: Event) -> Result<()>;
    fn get_events(&self, entity_id: Uuid) -> Result<Vec<Event>>;
    fn get_snapshot(&self, entity_id: Uuid) -> Result<Option<Snapshot>>;
}

trait VcsPort {
    fn create_branch(&self, name: &str) -> Result<()>;
    fn merge(&self, branch: &str, into: &str) -> Result<MergeResult>;
}
```

---

## CLI Commands

### Feature Management

```bash
# Create a new feature
agileplus feature create --title "Plane.so Sync" --module sync

# List features
agileplus feature list --state implementing --module sync

# Transition feature state
agileplus feature transition f123456 --to validating

# Show feature details
agileplus feature show f123456
```

### Work Package Management

```bash
# Create a work package
agileplus wp create --feature f123456 --title "Implement sync engine"

# Assign to agent
agileplus wp assign wp789 --assignee agent-1

# Complete a work package
agileplus wp complete wp789

# Show dependency graph
agileplus wp dag --feature f123456
```

### Event Store

```bash
# Query events for a feature
agileplus events --feature f123456 --since 24h

# Verify event chain integrity
agileplus events verify --entity f123456

# Export events to JSON
agileplus events export --since 2026-01-01 > events.json
```

### Platform Services

```bash
# Start all services
agileplus platform up

# Check health status
agileplus platform status

# Stop all services
agileplus platform down

# View logs
agileplus platform logs --service nats
```

### Sync

```bash
# Push local changes to Plane.so
agileplus sync push

# Pull remote changes from Plane.so
agileplus sync pull

# Enable auto-sync
agileplus sync auto --enable

# Show sync status
agileplus sync status
```

---

## Integration with Plane.so

AgilePlus maintains a bidirectional sync with Plane.so, treating AgilePlus as the source of truth for spec-driven development.

### Sync State

```rust
struct SyncState {
    agileplus_id: Uuid,
    plane_id: String,
    entity_type: EntityType,
    last_sync: DateTime,
    content_hash: Sha256Hash,
    conflict_history: Vec<ConflictRecord>,
}
```

### Conflict Resolution

When both AgilePlus and Plane.so modify the same entity since last sync:

1. **Detect:** Compare content hashes to identify conflicts
2. **Present:** Show both versions to the user via dashboard or CLI
3. **Resolve:** User chooses which version to keep or manually merges
4. **Record:** Conflict and resolution logged to audit trail

---

## File Naming Conventions

### Feature Directories

```
features/
├── 001-plane-so-sync/
│   ├── spec.md           # Feature specification
│   ├── plan.md           # Implementation plan
│   ├── research.md       # Technical research
│   ├── spec.json        # Structured spec data
│   └── evidence/        # Collected evidence
├── 002-event-sourcing/
│   └── ...
```

### Work Package References

In code and commits, WPs are referenced by their ID:

```
feat(sync): Implement Plane.so webhook handler

WP01 from feature/001-plane-so-sync

- Add webhook endpoint POST /webhooks/plane
- Verify webhook signature
- Parse issue state change events
```

---

## Best Practices

### Writing Specifications

1. **User Story First:** Start with who wants what and why
2. **Acceptance Criteria:** Define done-ness clearly
3. **Technical Approach:** Document constraints and choices
4. **Dependencies:** List what this feature needs
5. **Avoid Ambiguity:** Use concrete examples

### Work Package Creation

1. **Atomic Scope:** Each WP does exactly one thing
2. **File Scope:** Limit files a WP can modify
3. **Clear Dependencies:** List what must complete first
4. **Evidence Requirements:** Specify what governance needs

### Commit Messages

```
<type>(<scope>): <description>

[Optional body with context]

Refs: WP<id> from feature/<slug>
```

Types: `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `ci`

### State Transitions

1. **Verify Preconditions:** Ensure all prerequisites met
2. **Collect Evidence:** Gather required governance evidence
3. **Log Transition:** Every transition creates an event
4. **Update Related Entities:** Propagate changes to dependents

---

## Architecture Patterns

### xDD Methodologies

AgilePlus supports multiple development approaches:

| Methodology | Application |
|-------------|-------------|
| **TDD** | Red/green/refactor for domain logic |
| **BDD** | Gherkin scenarios as acceptance criteria |
| **SDD** | Specs define what to build |
| **DDD** | Bounded contexts, aggregates, domain events |
| **CQRS** | Separate read and write models |
| **Event Sourcing** | All state changes as immutable events |

### Event-Driven Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Feature    │────▶│    NATS     │────▶│  Subscriber  │
│  Transition  │     │   Event     │     │              │
└──────────────┘     │    Bus       │     └──────────────┘
                    └──────────────┘
                          │
         ┌────────────────┼────────────────┐
         ▼                ▼                ▼
  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
  │   Audit     │  │    Sync     │  │   Graph     │
  │   Logger    │  │   Engine    │  │  Update     │
  └─────────────┘  └─────────────┘  └─────────────┘
```

---

## Success Metrics

### Development Velocity

- **Cycle Time:** Feature from Specified to Shipped in target days
- **WP Throughput:** Work packages completed per cycle
- **Block Rate:** Percentage of WPs that become blocked

### Quality

- **Test Coverage:** Percentage of domain logic covered
- **Governance Compliance:** Features passing all policy rules
- **Event Chain Integrity:** Percentage of valid hash chains

### Integration

- **Sync Latency:** Time between AgilePlus and Plane.so state divergence
- **Conflict Rate:** Percentage of syncs requiring manual resolution
- **Offline Capability:** Operations available without network

---

## References

- [Feature Specifications](../specs/)
- [Architecture Decision Records](../adr/)
- [Functional Requirements](../../FUNCTIONAL_REQUIREMENTS.md)
- [Implementation Plan](../../PLAN.md)
- Plane.so API Documentation
- Hexagonal Architecture (Ports and Adapters)

---

**End of Specification**
