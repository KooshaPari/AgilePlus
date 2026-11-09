---
spec_id: eco-007-schema-evolution
state: IN_PROGRESS
plan_status: REQUIRED
last_audit: 2026-05-02
---

# Specification: Schema Versioning and API Evolution
**Slug**: eco-007-schema-evolution | **Date**: 2026-05-02 | **State**: in_progress

## Problem Statement

AgilePlus uses a multi-layer data stack — SQLite for persistence, gRPC for inter-service communication, Protocol Buffers for serialization, and event sourcing for audit trails. As the system evolves, schema changes in any layer can break:
- **Stored events**: Event-sourced records with old schemas become unreadable after schema changes
- **gRPC clients**: Breaking proto changes silently break all existing clients
- **SQLite plugins**: Ad-hoc migrations cause data loss or corruption
- **Plugin adapters**: Third-party storage plugins expect stable, versioned schemas

Without a formal schema versioning and API evolution discipline, each release risks silent data loss, client breakage, or plugin incompatibility.

## Target Users

- Core platform engineers maintaining the event store and SQLite adapter
- Plugin authors building new storage or VCS adapters
- External API consumers integrating via gRPC
- Agent workflows that depend on stable event schemas

## Functional Requirements

| ID | Requirement |
|----|-------------|
| FR-SEV-01 | All SQLite tables carry an explicit schema version number in a `schema_version` table |
| FR-SEV-02 | All proto files carry a package version suffix (e.g., `agileplus.v2`) to allow parallel API versions |
| FR-SEV-03 | Every event in the event store includes a `schema_version` field in its payload header |
| FR-SEV-04 | A migration runner executes sequential versioned migrations on startup and on-demand |
| FR-SEV-05 | Proto breaking-change detection is enforced in CI via `buf breaking` |
| FR-SEV-06 | gRPC clients can negotiate API version via metadata header or URI prefix |
| FR-SEV-07 | All schema changes require a corresponding migration file in `migrations/` |
| FR-SEV-08 | Plugin adapters declare their compatible schema version range |
| FR-SEV-09 | Rolling back a migration is supported via `agileplus migrate rollback <version>` |
| FR-SEV-10 | A schema diff report is generated on every migration file addition |

## Non-Functional Requirements

- **Backward compatibility**: Additive changes (new fields, new enum values) never break existing clients
- **Tamper evidence**: Event schema version is part of the event hash chain
- **Performance**: Migration runner handles 10,000+ events without OOM
- **Safety**: Rollback is idempotent and does not corrupt event history
- **Auditability**: Every migration has a corresponding GitHub issue or PR reference

## Constraints & Dependencies

- SQLite adapter lives in `libs/plugin-sqlite/`
- gRPC definitions live in `proto/`
- Event types live in domain crates under `libs/`
- Migration files stored in `migrations/` at repo root
- Relies on `buf` CLI for proto linting and breaking-change detection
- Relies on `rusqlite` with versioned connection setup

## Acceptance Criteria

| AC | Criterion |
|----|-----------|
| AC01 | `schema_version` table exists in SQLite and is populated on first run |
| AC02 | All proto packages carry a versioned suffix; `buf breaking` passes on additive-only changes |
| AC03 | Events serialized after this spec carry `schema_version` in their header |
| AC04 | `agileplus migrate status` reports current schema version and pending migrations |
| AC05 | `agileplus migrate apply` executes exactly one pending migration atomically |
| AC06 | Plugin adapters fail fast if their compatible version range does not include current schema version |
| AC07 | `agileplus migrate rollback` reverts exactly one migration and updates schema_version |
| AC08 | CI workflow runs `buf breaking --against .bufbreaking.json` and fails on breaking changes |
