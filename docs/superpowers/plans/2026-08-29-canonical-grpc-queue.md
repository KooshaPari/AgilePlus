# Canonical gRPC Queue Implementation Plan

> **SUPERSEDED (2026-08-29):** The core-runtime repair established
> `crates/agileplus-proto` as the active canonical generated package and removed
> the duplicate `rust/` workspace member. Do not execute this plan as written;
> its proto-ownership assumptions would reverse the repaired workspace boundary.
> Any remaining queue work must be replanned against `crates/agileplus-proto`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development or executing-plans task-by-task. Steps use checkbox syntax.

**Goal:** Serve canonical queue CreateBacklogItem, ListBacklog, and PromoteBacklogItem RPCs from one buildable AgilePlus gRPC core.

**Architecture:** rust/ remains the only agileplus-proto package and generates agileplus.v1 from proto/agileplus/v1/\*.proto. The legacy gRPC server must use those generated types; it must not revive obsolete RPCs or fields. The first proof is a real tonic client plus in-memory SQLite storage, before any supervised process is added.

**Tech Stack:** Rust, tonic 0.14, prost, Tokio, StoragePort, SqliteStorageAdapter, Python MCP.

---

## Fixed contract

| Canonical input                         | Domain mapping                                                     |
| --------------------------------------- | ------------------------------------------------------------------ |
| type, title, body, priority, triaged_by | Intent, BacklogItem::from_triage, optional BacklogPriority, source |
| feature_id, wp_id                       | not persisted until a schema contract is approved                  |
| type_filter, state_filter, feature_slug | BacklogFilters                                                     |
| backlog_item_id, target_type            | existing item lookup and promotion acknowledgement                 |

### Task 1: Select one generated proto package

**Files:**

- Modify: Cargo.toml
- Modify: crates/agileplus-grpc/Cargo.toml
- Do not modify: crates/agileplus-proto/\*\* or proto/agileplus/v1/integrations.proto

- [ ] **Step 1: Verify the current red boundary**

Run:

```
cargo check --manifest-path crates/agileplus-grpc/Cargo.toml --locked
```

Expected: it fails because the legacy server is outside the active workspace and uses the obsolete proto package.

- [ ] **Step 2: Wire the server to the canonical generator**

Add crates/agileplus-grpc to root workspace members. In crates/agileplus-grpc/Cargo.toml replace:

```
agileplus-proto = { path = "../agileplus-proto" }
```

with:

```
agileplus-proto = { path = "../../rust" }
```

Update the root agileplus-proto workspace dependency to the same rust path. Do not add crates/agileplus-proto to workspace members because it duplicates the canonical package name.

- [ ] **Step 3: Verify the intended next red state**

Run:

```
cargo check -p agileplus-grpc --locked
```

Expected: only removed request fields and obsolete generated RPC symbols fail in server/integrations.rs.

### Task 2: Implement the three canonical queue RPCs

**Files:**

- Modify: crates/agileplus-grpc/src/server/integrations.rs
- Modify: crates/agileplus-grpc/src/conversions.rs
- Create: crates/agileplus-grpc/tests/backlog_contract.rs

- [ ] **Step 1: Write the failing real-client test**

Use SqliteStorageAdapter::in_memory and a tonic IntegrationsServiceClient. Create a task item with body "persist this", list it by type, then promote it. Assert the listed item has the same body and the promotion returns success.

Run:

```
cargo test -p agileplus-grpc --test backlog_contract canonical_queue_round_trip -- --nocapture
```

Expected: FAIL because IntegrationsService is not implemented against canonical generated types.

- [ ] **Step 2: Remove obsolete queue surface**

Delete imports and handlers for GetBacklogItem, ImportBacklog, PopBacklog, and UpdateBacklogStatus. Keep ClassifyInput and router generation unchanged. Import integrations_service_server::IntegrationsService and only the canonical queue messages.

- [ ] **Step 3: Convert request and response values**

Create a domain item with:

```
let mut item = BacklogItem::from_triage(
    request.title,
    request.body,
    parse_intent(&request.r#type)?,
    if request.triaged_by.is_empty() { "grpc".to_string() } else { request.triaged_by },
);
if !request.priority.is_empty() {
    item.priority = parse_priority(&request.priority)?;
}
```

Map domain intent, description, priority, status, source, and created_at into the canonical BacklogItem. Do not invent storage for feature_id or wp_id.

- [ ] **Step 4: Implement the generated service trait**

Implement IntegrationsService for AgilePlusCoreServer with exactly create_backlog_item, list_backlog, and promote_backlog_item. Each accepts tonic::Request, calls the existing helper with into_inner, and returns that result. Reject an empty target_type with Status::invalid_argument.

- [ ] **Step 5: Verify green**

Run:

```
cargo test -p agileplus-grpc --test backlog_contract canonical_queue_round_trip -- --nocapture
cargo test -p agileplus-grpc --locked
cargo fmt --check
```

Expected: PASS using generated client calls and real in-memory SQLite storage.

### Task 3: Add a supervised loopback process after contract green

**Files:**

- Create: crates/agileplus-grpc/src/bin/agileplus-grpc.rs
- Modify: process-compose.yml
- Test: python/tests/test_grpc_backlog_contract.py

- [ ] **Step 1: Write the failing external-client test**

Start the binary with a temporary SQLite path and loopback port. Use the Python client to create, list, and promote an item. The test must fail before the binary exists.

- [ ] **Step 2: Construct only production adapters**

Build SqliteStorageAdapter from the supplied database path, construct production ports, and call bootstrap::start_server on loopback. Do not use test doubles or bind externally.

- [ ] **Step 3: Add a real readiness probe**

Add one agileplus-grpc process with readiness:

```
grpcurl -plaintext 127.0.0.1:<port> list agileplus.v1.IntegrationsService
```

Do not change HTTP MCP transport or start MinIO in this task.

- [ ] **Step 4: Verify acceptance**

Run:

```
cargo check --workspace --locked
cargo test -p agileplus-proto -p agileplus-grpc --locked
uv run pytest -q python/tests/test_grpc_backlog_contract.py
process-compose -p 8081 process list
```

Expected: Python MCP queue operations reach the real canonical core.

## Plan self-review

- One generated proto package and three canonical queue methods only.
- Explicitly excluded: old queue RPCs, unresolved pop semantics, feature/WP metadata schema changes, and public network binding.
- Every implementation task has a red proof, green proof, and narrow command.
