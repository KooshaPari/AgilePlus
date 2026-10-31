# PM work-item schema-first slice

Additive migration 032 introduces `work_item` and `audit_event`; it does not change existing table definitions. `WorkItemPort` is the atomic persistence boundary implemented by `SqliteStorageAdapter`. API embedders opt in with `AppState::with_work_items()`; the shipped API binary does so.

Authenticated routes (existing Bearer / X-API-Key middleware):

- `POST /api/v1/work-items`: `{ "kind": "task", "title": "JOURNEY-example", "cycle_id": null, "feature_id": null }` returns 201 with the created item in `intake`.
- `POST /api/v1/work-items/{id}/transition`: `{ "state": "triaged" }` returns 200 with the updated item.
- `GET /api/v1/work-items/{id}/audit-events`: returns ordered, verified events, or 404 for an unknown item.

Kinds: task, bug, story, feature. Workflow edges: intake -> triaged -> planned -> in_progress -> in_review; in_review -> in_progress supports rework. There is no terminal acceptance state. Existing AcceptedWork/feature acceptance remains a separate governed path. Invalid edges return 409, invalid bodies/titles/references return 422, and missing credentials return 401.

The actor is assigned by the server as `api-key`, representing the shared operator credential. This auth layer has no individual user identity; clients cannot supply or forge actor fields. Domain callers explicitly pass an actor.

Creation and transitions each write the item and event in one immediate SQLite transaction. Audit insertion failure rolls back the item mutation. Each item has its own SHA-256 chain with an empty genesis prev_hash. The hash input is a versioned JSON tuple of event id, item id, actor, action, exact payload_json text, previous hash, and UTC timestamp. SQL triggers reject UPDATE and DELETE; a unique item/prev_hash index prevents chain forks. Reads and transitions verify the entire chain and reject corruption. This detects tampering; it is not an externally signed log protecting against a database administrator rewriting the schema and entire history.

An existing item without its creation event is reported as corruption. Transitions reuse the verified chain head when appending. Migration 032 is intentionally irreversible: rollback returns an error before changing either schema or the migration marker, preserving audit history and allowing normal startup to reopen the database.

Targeted witnesses:

```text
cargo test -p agileplus-domain --lib work_item
cargo test -p agileplus-sqlite --lib work_item
cargo test -p agileplus-api --test work_items_http
cargo check -p agileplus-domain -p agileplus-sqlite -p agileplus-api
```

The HTTP witness uses a temporary file-backed SQLite database and an in-process Axum server. It covers auth, input validation, forbidden acceptance, actor spoofing, legal/illegal transitions, chain continuity, 404, and reopening persistence. SQLite witnesses cover append-only triggers, rollback after forced audit failure, invalid input/references, and corruption detection.
