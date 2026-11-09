-- UP
CREATE TABLE work_item (
 id INTEGER PRIMARY KEY AUTOINCREMENT,
 kind TEXT NOT NULL CHECK(kind IN ('task','bug','story','feature')),
 title TEXT NOT NULL CHECK(length(trim(title)) > 0),
 state TEXT NOT NULL CHECK(state IN ('intake','triaged','planned','in_progress','in_review')),
 cycle_id INTEGER REFERENCES cycles(id) ON DELETE RESTRICT,
 feature_id INTEGER REFERENCES features(id) ON DELETE RESTRICT,
 created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE INDEX idx_work_item_cycle ON work_item(cycle_id);
CREATE INDEX idx_work_item_feature ON work_item(feature_id);
CREATE TABLE audit_event (
 id INTEGER PRIMARY KEY,
 work_item_id INTEGER NOT NULL REFERENCES work_item(id) ON DELETE RESTRICT,
 actor TEXT NOT NULL CHECK(length(trim(actor)) > 0), action TEXT NOT NULL,
 payload_json TEXT NOT NULL CHECK(json_valid(payload_json)),
 prev_hash TEXT NOT NULL, hash TEXT NOT NULL CHECK(length(hash) = 64), created_at TEXT NOT NULL
);
CREATE INDEX idx_audit_event_item ON audit_event(work_item_id, id);
CREATE UNIQUE INDEX idx_audit_event_chain ON audit_event(work_item_id, prev_hash);
CREATE TRIGGER audit_event_no_update BEFORE UPDATE ON audit_event BEGIN
 SELECT RAISE(ABORT, 'audit_event is append-only'); END;
CREATE TRIGGER audit_event_no_delete BEFORE DELETE ON audit_event BEGIN
 SELECT RAISE(ABORT, 'audit_event is append-only'); END;
