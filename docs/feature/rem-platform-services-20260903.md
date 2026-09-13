# Feature: Remove Platform Services from AgilePlus Core

## Metadata

- **ID:** 1
- **Slug:** `rem-platform-services-20260903`
- **State:** specified
- **Created:** 2026-09-03T14:50:00Z
- **Updated:** 2026-09-03T14:50:00Z
- **Priority:** P0 (blocks MCP, desktop, full feature parity)
- **Type:** refactor
- **Source:** user directive

---

## Problem Statement

AgilePlus is architected as a client-side, VSCode-like spec-driven development engine with state in `.agileplus/agileplus.db` (SQLite). However, the current implementation includes platform services (NATS, Dragonfly/Neo4j/MinIO/API) as hard dependencies in the gRPC core, breaking MCP functionality and contradicting the local-first architecture.

**Evidence:**
- CLI works standalone with SQLite only: `./target/release/agileplus --db .agileplus/agileplus.db dashboard`
- MCP server fails: gRPC core timeouts waiting for platform services
- User confirmed: "AgilePlus is a CLI, a semantic process, an app, a tray app — NOT a platform. State lives in repo and device. Tracera is the platform."
- Research confirms: No competing spec-driven tool uses external services as core dependencies

---

## Requirements

### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-001 | `./target/release/agileplus --db .agileplus/agileplus.db dashboard` must work without any platform services running | P0 |
| FR-002 | `agileplus platform` subcommand must be removed (returns "unrecognized subcommand") | P0 |
| FR-003 | `process-compose.yml` must be deleted from repo root | P0 |
| FR-004 | MCP server health check must pass without platform services | P0 |
| FR-005 | MCP tools (`get_feature`, `list_features`, etc.) must work via SQLite directly | P0 |
| FR-006 | Desktop client (Electrobun) must read/write `.agileplus/agileplus.db` directly | P0 |
| FR-007 | `agileplus validate` must work without platform services | P1 |
| FR-008 | `agileplus cockpit` must work without platform services | P1 |
| FR-009 | `agileplus dag` must work without platform services | P1 |

### Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-001 | No new external dependencies (Docker, nats-server, redis-server, minio, neo4j) | P0 |
| NFR-002 | Build time must not increase significantly | P1 |
| NFR-003 | All existing tests must pass without platform services | P0 |
| NFR-004 | No breaking changes to CLI interface (except removal of `platform` subcommand) | P1 |

---

## Acceptance Criteria

- [ ] `./target/release/agileplus --db .agileplus/agileplus.db dashboard` returns feature data without platform services
- [ ] `agileplus platform status` returns "unrecognized subcommand"
- [ ] `process-compose.yml` deleted from repo
- [ ] MCP health check passes: `{"result":{"healthy":true,...}}`
- [ ] MCP `get_feature` returns feature data via SQLite
- [ ] All existing tests pass: `cargo test --workspace`
- [ ] Desktop app launches and reads `.agileplus/agileplus.db`
- [ ] No Docker, nats-server, redis-server, minio, neo4j in Cargo.toml

---

## Dependencies

- **Blocked by:** None
- **Blocks:** MCP completion, desktop client completion, full feature parity

---

## Traceability

- **ADRs:** ADR-018 (Remove Platform Services), ADR-019 (Local-First SQLite State Only), ADR-020 (Rust Core + Electrobun Desktop as Primary Interface)
- **Research:** `/docs/research/agent-spec-alternatives-2026-09-03.md`, `/docs/research/sdlc-methodology-comparison-2026-09-03.md`
- **Plan:** `/docs/PLAN-remove-platform-services-2026-09-03.md`

---

## Work Packages

| WP ID | Title | State | Owner |
|-------|-------|-------|-------|
| WP-01 | Remove process-compose.yml and platform subcommand | planned | [assignee] |
| WP-02 | Update MCP to use SQLite directly | planned | [assignee] |
| WP-03 | Remove gRPC service dependencies | planned | [assignee] |
| WP-04 | Verify all interfaces work without platform services | planned | [assignee] |
| WP-05 | Update documentation | planned | [assignee] |

---

## Audit Trail

| Timestamp | Event | Actor | Details |
|-----------|-------|-------|---------|
| 2026-09-03T14:50:00Z | feature_specified | user | Created feature spec for platform service removal |
| 2026-09-03T14:50:00Z | research_completed | agent | Research on agent-based spec-driven alternatives |
| 2026-09-03T14:50:00Z | plan_created | agent | Created implementation plan |
| 2026-09-03T14:50:00Z | adr_proposed | agent | Proposed ADR-018, ADR-019, ADR-020 |
