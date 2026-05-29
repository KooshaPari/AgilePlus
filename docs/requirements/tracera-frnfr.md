# Tracera FR/NFR Catalog — Backfilled Requirements

**Version:** 1.0.0
**Date:** 2026-05-29
**Status:** Active

> **Meta — Dog-fooding note:** Tracera IS the traceability tool. This catalog is designed to be ingested into Tracera's own Requirement / Artifact / TraceLink model, making it a self-referential demonstration of the platform's value. Every FR listed here corresponds to a `Requirement` node, every PR/test reference becomes an `Artifact`, and every (FR→PR) / (FR→test) pair becomes a `TraceLink` with `VERIFIES` or `IMPLEMENTS` semantics. This is the canonical seed dataset for Tracera's own dog-food project in AgilePlus.

---

## Functional Requirements

### FR-TRC-001 — Canonical TraceLink Domain Model

**Title:** Canonical Requirement / Artifact / TraceLink value objects with ISO-29148-aligned vocabulary

**Description:** The system shall expose a canonical domain layer (`tracertm.models.trace_link`) that defines `TraceLinkType` (SATISFIES, VERIFIES, IMPLEMENTS, DERIVES_FROM, REFINES, CONFLICTS_WITH, DUPLICATES), `ArtifactKind` (REQUIREMENT, DESIGN, CODE, TEST, EVIDENCE, RISK, RATIONALE), and lightweight Pydantic value objects (`TraceLink`, `Requirement`, `Artifact`) used at API, RAG, and pipeline boundaries.

**Acceptance Criteria:**
- `TraceLinkType` enum values match Neo4j relationship labels exactly (SCREAMING_SNAKE).
- `ArtifactKind` partitions Items into ISO 29148 / DO-178C traceability roles.
- Pydantic models are wire-format-clean (JSON-serialisable, no ORM dependencies).
- `Neo4jSchema` provides declarative Cypher constraints and index declarations.

**Traceability:**
- PR: #458 (feat(trace-link): canonical domain + Neo4j schema)
- Source: `src/tracertm/models/trace_link.py`
- Alembic: `062_add_trace_link_fields.py`

---

### FR-TRC-002 — Confidence-Scored Trace Links with Rationale

**Title:** Persist and query trace links with miner confidence score and natural-language rationale

**Description:** Every trace link shall carry a `confidence` float in [0.0, 1.0] (default 1.0 for human-curated links) and an optional `rationale` text field. The repository shall expose a `list_with_confidence` query supporting filter-by-confidence thresholds for downstream RAG and explainability layers.

**Acceptance Criteria:**
- `links.confidence` column has DB-level CHECK constraint `[0.0, 1.0]`.
- `links.rationale` is nullable TEXT.
- `LinkRepository.create()` rejects out-of-range confidence with `ValueError`.
- `list_with_confidence` query is exposed and tested.
- Indexes: `idx_links_confidence`, `idx_links_project_type_confidence`.

**Traceability:**
- PR: #460 (feat(link-repository): persist confidence + rationale + list_with_confidence)
- Source: `src/tracertm/repositories/link_repository.py`
- Tests: `tests/unit/repositories/test_link_repository.py`, `tests/unit/repositories/test_link_repository_comprehensive.py`
- Alembic: `062_add_trace_link_fields.py`

---

### FR-TRC-003 — Neo4j Trace-Graph Projection Writer

**Title:** Idempotent Neo4j writer for Requirement / Artifact / TraceLink graph projection

**Description:** The system shall provide `tracertm.storage.neo4j_trace_link_writer` with idempotent `MERGE`-based writers (`write_requirement`, `write_artifact`, `write_link`) and an `apply_schema` bootstrap that creates Neo4j constraints and indexes. Writers must be safe to call multiple times without creating duplicates.

**Acceptance Criteria:**
- All writes use `MERGE` (idempotent) not `CREATE`.
- `apply_schema` creates uniqueness constraints and indexes declared in `Neo4jSchema`.
- Integration tests pass against a real Neo4j 5 instance via testcontainers.
- Module is skipped gracefully when `neo4j` driver or testcontainers is absent.

**Traceability:**
- PR: #461 (feat(neo4j): idempotent TraceLink projection writer)
- Source: `src/tracertm/storage/neo4j_trace_link_writer.py` (inferred)
- Tests: `tests/integration/storage/test_neo4j_trace_link_writer.py`

---

### FR-TRC-004 — Forward and Reverse Impact Analysis via Cypher

**Title:** REST endpoints for Cypher-powered forward/reverse artifact impact traversal

**Description:** The API shall expose two endpoints (`GET /impact/forward/{artifact_id}` and `GET /impact/reverse/{artifact_id}`) that traverse the Neo4j trace-link graph using multi-hop Cypher queries. Forward impact returns all downstream artifacts affected by a change; reverse impact returns all upstream artifacts (requirements, specs) that led to a given artifact.

**Acceptance Criteria:**
- Forward traversal: `MATCH (src)-[l*1..]->(affected)` collects all reachable downstream nodes.
- Reverse traversal: `MATCH (affected)-[l*1..]->(src)` collects upstream chain.
- Response includes `id`, `project_id`, `kind`, `title`, `external_id`, `link_types`.
- Endpoints are wired into FastAPI router with proper async Neo4j driver dependency injection.
- Handler correctly closes driver after each request.

**Traceability:**
- PR: #462 (feat(impact-api): Cypher forward/reverse impact analysis endpoints)
- Source: `src/tracertm/api/handlers/impact.py`
- Tests: `tests/integration/graph/test_cypher_impact_api.py`

---

### FR-TRC-005 — Authentication with DB Account Lookup

**Title:** Auth handler resolves authenticated identities against the accounts database table

**Description:** Upon successful OAuth/device-code authentication, the system shall look up the authenticated user's account record in the PostgreSQL `accounts` table via `AccountRepository`, rather than relying solely on the identity provider's token claims. System admin privileges are determined by matching the email against the `TRACERTM_SYSTEM_ADMIN_EMAILS` environment variable (comma-separated).

**Acceptance Criteria:**
- `AccountRepository` is called post-token-verification to resolve the local account record.
- Missing account records result in a well-formed 401/403 (not a 500).
- System admin cache (`_admin_emails_cache`) is populated from env var at startup.
- Device Authorization Flow (RFC 8628) is supported alongside standard OAuth.

**Traceability:**
- PR: #463 (feat(auth): implement DB account lookup, closes #223)
- Source: `src/tracertm/api/handlers/auth.py`, `src/tracertm/repositories/account_repository.py`
- Tests: `tests/unit/repositories/test_account_repository.py`, `tests/unit/api/test_authentication.py`

---

### FR-TRC-006 — Spatial GiST Index for Graph Viewport Queries

**Title:** PostgreSQL GIST index on item (position_x, position_y) for O(log n) viewport range queries

**Description:** Graph items with positional coordinates shall be indexed using a PostgreSQL GIST index (`idx_items_spatial`) over a `box(point(position_x, position_y), ...)` expression, enabling sub-linear rectangular range queries for graph viewport/frustum culling without full-table scans.

**Acceptance Criteria:**
- `items.position_x` and `items.position_y` columns exist (NUMERIC 10,2, default 0).
- GIST index `idx_items_spatial` is created with a `WHERE deleted_at IS NULL` partial filter.
- Migration is idempotent (uses `CREATE INDEX IF NOT EXISTS` and `ADD COLUMN IF NOT EXISTS`).
- Index is used by the query planner for bbox queries (verified via `EXPLAIN ANALYZE`).

**Traceability:**
- PR: #464 (perf: spatial index for edge midpoint distance queries)
- Source: `alembic/versions/054_add_spatial_gist_index.py`

---

### FR-TRC-007 — UICodeTracePanel Live API Integration

**Title:** UICodeTracePanel React component wired to live traceability chain API

**Description:** The `UICodeTracePanel` component shall fetch live traceability chain data from the backend API (not mock/stub data), displaying the full UI→code→requirement linkage for any selected artifact. The panel shall show loading states, support `CanonicalConcept`, `CanonicalProjection`, `CodeReference`, and `EquivalenceStrategy` typed data from `@tracertm/types`.

**Acceptance Criteria:**
- Component imports from `@tracertm/types` (not local stubs).
- API fetch uses the project's shared API client (no raw `fetch` with hardcoded URLs).
- Loading, error, and empty states are handled.
- Integration test (`UICodeTracePanel.integration.tsx`) exercises the live API path.

**Traceability:**
- PR: #465 (feat: wire UICodeTracePanel to live API, closes #226)
- Source: `frontend/apps/web/src/components/graph/UICodeTracePanel.tsx`
- Tests: `frontend/apps/web/src/components/graph/UICodeTracePanel.integration.tsx`

---

### FR-TRC-008 — MCP Auth / Config / DB Contract Tests

**Title:** FastMCP server auth, config, and database tool functions verified by contract tests

**Description:** The MCP tool layer (`tracertm.mcp.tools.auth_config_db`) shall be covered by contract tests verifying `auth_status`, `auth_logout`, `config_get`, `config_set`, `config_list`, `config_unset`, and `db_init` functions in isolation, using a lightweight stub for `tracertm.mcp.core` to avoid triggering the full MCP boot sequence during test collection.

**Acceptance Criteria:**
- All seven tool functions have at least one test assertion.
- Core MCP bootstrap is fully stubbed (`@mcp.tool()` becomes a pass-through decorator).
- Tests are importable and runnable in unit-test context (no network/DB required).
- Tests are collected under `tests/mcp/test_auth_config_db.py`.

**Traceability:**
- PR: #466 (fix(mcp): implement auth/config/db contract tests, closes #232)
- Source: `src/tracertm/mcp/tools/auth_config_db.py` (inferred), `src/tracertm/mcp/auth.py`
- Tests: `tests/mcp/test_auth_config_db.py`

---

### FR-TRC-009 — Live Comment Submission in CommentsTab

**Title:** CommentsTab UI component submits comments to the live backend API

**Description:** The `CommentsTab` component shall POST new comments to the backend `item_comments` REST endpoint and optimistically update the UI on success. The `item_comments` table shall persist `item_id`, `author_id`, `author_name`, `content`, `edited`, `created_at`, and `updated_at`.

**Acceptance Criteria:**
- POST to `/api/items/{item_id}/comments` creates a persisted record.
- Comment appears in the thread without a full page reload.
- `item_comments` table created by alembic revision `063_add_item_comments`.
- Author attribution uses the authenticated session's identity.

**Traceability:**
- PR: #469 (feat(comments): live comment submission in CommentsTab, closes #225)
- Source: `frontend/apps/web/src/` (CommentsTab component), `src/tracertm/api/routers/comments.py`
- Alembic: `063_add_item_comments.py`

---

### FR-TRC-010 — E2E Project Lifecycle Test Against Real Backend

**Title:** End-to-end project lifecycle test exercises real FastAPI app via HTTPX ASGI transport

**Description:** The E2E test suite shall include a `test_project_lifecycle_roundtrip` test that exercises a full create→list→item→update→link→delete→404 cycle against a real in-process FastAPI application wired to the test database engine via HTTPX ASGI transport (no mocks for business logic).

**Acceptance Criteria:**
- Test uses `test_db_engine` fixture with real SQLAlchemy repositories.
- HTTP transport is HTTPX ASGI (real HTTP semantics, no mock client).
- Auth guard is a fixed-identity stub (not real WorkOS).
- All lifecycle steps assert correct HTTP status codes and response bodies.
- Test is marked `@pytest.mark.integration @pytest.mark.slow`.
- Passes in under 30 seconds on local hardware.

**Traceability:**
- PR: #470 (feat(e2e): real backend fixture in project lifecycle test, closes #230)
- Source: `tests/e2e/test_project_lifecycle.py`

---

## Non-Functional Requirements

### NFR-TRC-001 — Spatial Index Query Performance

**Title:** Graph viewport queries execute in O(log n) via GIST index

**Description:** Rectangular range queries over item positions (graph viewport panning/zooming) shall use the `idx_items_spatial` GIST index and must not degrade to sequential scans on datasets of 10,000+ items.

**Evidence:**
- GIST index created in `alembic/versions/054_add_spatial_gist_index.py`.
- `WHERE deleted_at IS NULL` partial filter reduces index size by excluding soft-deleted items.
- PR #464 description: "spatial index for edge midpoint distance queries".

---

### NFR-TRC-002 — Auth Contract Correctness (Fail-Safe Defaults)

**Title:** Authentication layer has no silent failure modes; missing accounts produce explicit errors

**Description:** Auth failures (missing account record, invalid token, revoked device code) shall produce deterministic HTTP error responses (401/403), never 500 errors or silent pass-throughs.

**Evidence:**
- `WORKOS_HANDLER_ERRORS` and `TOKEN_EXTRACTION_ERRORS` tuples in `auth.py` enumerate all swallowed exception types explicitly.
- Contract tests in `tests/mcp/test_auth_config_db.py` verify auth tool behavior under error conditions.
- Unit tests: `tests/unit/api/test_authentication.py`, `tests/unit/api/test_authorization_scenarios.py`.

---

### NFR-TRC-003 — E2E Tests Exercise Real Backend (No Mock Bypass)

**Title:** Integration-marked E2E tests must use real repositories, not mock client stubs

**Description:** Tests marked `@pytest.mark.integration` shall exercise the real SQLAlchemy repository layer and real HTTP semantics. MSW / mock-client patterns are acceptable only in frontend unit tests.

**Evidence:**
- `tests/e2e/test_project_lifecycle.py` uses HTTPX ASGI transport with `test_db_engine` (PR #470).
- `tests/integration/storage/test_neo4j_trace_link_writer.py` uses testcontainers-neo4j for real Neo4j (PR #461).
- `tests/integration/graph/test_cypher_impact_api.py` is marked `integration + slow` with live Neo4j requirement.

---

### NFR-TRC-004 — Neo4j Projection Idempotency

**Title:** All Neo4j writes are idempotent (MERGE semantics)

**Description:** The Neo4j trace-link writer shall use `MERGE` for all node and relationship creation, ensuring that replaying the ingestion pipeline does not create duplicate nodes or relationships.

**Evidence:**
- `write_requirement`, `write_artifact`, `write_link` in `tracertm.storage.neo4j_trace_link_writer` use `MERGE` (confirmed in `tests/integration/storage/test_neo4j_trace_link_writer.py`).
- `apply_schema` uses `CREATE CONSTRAINT IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS`.

---

### NFR-TRC-005 — Confidence Column DB Integrity Constraint

**Title:** links.confidence enforced at DB level; application-layer validation is redundant defence

**Description:** The `confidence` column on the `links` table shall have a `CHECK (confidence >= 0.0 AND confidence <= 1.0)` constraint so out-of-range values are rejected even if application code is bypassed.

**Evidence:**
- Alembic: `ALTER TABLE links ADD CONSTRAINT links_confidence_range CHECK (...)` in `062_add_trace_link_fields.py`.
- `LinkRepository.create()` also validates before INSERT (defence-in-depth).

---

### NFR-TRC-006 — MCP Server Boot Isolation in Unit Tests

**Title:** MCP tool unit tests must not trigger the full FastMCP server boot sequence

**Description:** Contract tests for MCP tool functions shall stub `tracertm.mcp.core` before any import of tool modules, ensuring test collection completes in under 5 seconds without network access.

**Evidence:**
- `_stub_mcp_core()` pattern in `tests/mcp/test_auth_config_db.py` inserts a `ModuleType` stub into `sys.modules` before tool module import.

---

### NFR-TRC-007 — Traceability Model Compliance (ISO 29148 / DO-178C Alignment)

**Title:** TraceLinkType vocabulary is aligned with ISO 29148 §5.2.6 and DO-178C Table A-3

**Description:** The canonical link-type vocabulary shall be documented with its ISO/DO-178C semantic mapping, enabling future compliance audits against aerospace and safety-critical standards.

**Evidence:**
- Docstring in `TraceLinkType` class (`src/tracertm/models/trace_link.py`) provides per-value ISO 29148 §5.2.6 + DO-178C Table A-3 mapping.
- `ArtifactKind` docstring references ISO 29148 / DO-178C / IEC 62304 style traces.

---

## Gap Analysis / PLANNED Forward Requirements

| ID | Title | Status | Notes |
|----|-------|--------|-------|
| FR-TRC-011 | RAG-layer traceability query (LLM-assisted link suggestion) | PLANNED | Referenced in `trace_link.py` docstring as "later PRs"; miner posterior = `confidence` field |
| FR-TRC-012 | Automated duplicate / conflict detection via TraceLink miner | PLANNED | `DUPLICATES` and `CONFLICTS_WITH` link types defined but no miner service wired |
| FR-TRC-013 | Bulk TraceLink ingestion from external sources (Jira, GitHub Issues) | PLANNED | `github_import_service.py`, `jira_import_service.py` exist but TraceLink confidence mapping TBD |
| FR-TRC-014 | Traceability coverage matrix export (CSV/JSON/PDF) | PLANNED | `traceability_matrix_service.py` and `frontend/apps/web/e2e/traceability-matrix.spec.ts` exist; FR/NFR ingestion path not wired |
| FR-TRC-015 | Graph-level impact blast-radius scoring (risk-weighted path analysis) | PLANNED | `impact_analysis_service.py`, `critical_path_service.py` exist; no confidence-weighted scoring yet |
| FR-TRC-016 | AgilePlus integration — push Requirements / TraceLinks to AgilePlus project | PLANNED | Dog-food use case; AgilePlus at `C:/Users/koosh/Dev/AgilePlus`; API contract TBD |
| FR-TRC-017 | Requirement quality scoring (completeness, ambiguity detection) | PARTIAL | `requirement_quality_service.py` and `requirement_quality_repository.py` exist; needs FR spec |
| NFR-TRC-008 | Link confidence index selectivity target ≥ 90% for miner-generated links | PLANNED | Baseline TBD once miner ships |
| NFR-TRC-009 | Neo4j projection sync latency < 500 ms p99 for single-link writes | PLANNED | No SLA defined yet |

---

## Requirement → PR Traceability Map (Summary)

| Requirement | Shipped PRs | Key Tests |
|-------------|-------------|-----------|
| FR-TRC-001 | #458 | `test_neo4j_trace_link_writer.py` |
| FR-TRC-002 | #460 | `test_link_repository*.py` |
| FR-TRC-003 | #461 | `test_neo4j_trace_link_writer.py` |
| FR-TRC-004 | #462 | `test_cypher_impact_api.py` |
| FR-TRC-005 | #463 | `test_account_repository.py`, `test_authentication.py` |
| FR-TRC-006 | #464 | alembic `054` |
| FR-TRC-007 | #465 | `UICodeTracePanel.integration.tsx` |
| FR-TRC-008 | #466 | `test_auth_config_db.py` |
| FR-TRC-009 | #469 | alembic `063` |
| FR-TRC-010 | #470 | `test_project_lifecycle.py` |
| NFR-TRC-001..007 | #458–#470 | (see individual evidences above) |
