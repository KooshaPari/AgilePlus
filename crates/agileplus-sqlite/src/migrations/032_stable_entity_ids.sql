-- UP
-- Stable cross-device identities for the first canonical aggregate set.
-- Integer IDs remain local SQLite join keys. UUIDv7 values are immutable,
-- globally unique references used by exports, adapters, and other devices.
CREATE TABLE IF NOT EXISTS stable_entity_ids (
    entity_kind   TEXT    NOT NULL CHECK(entity_kind IN (
                         'project','feature','work_package')),
    entity_id     INTEGER NOT NULL CHECK(entity_id > 0),
    uid           TEXT    NOT NULL UNIQUE CHECK(length(uid) = 36),
    created_at    TEXT    NOT NULL,
    tombstoned_at TEXT,
    PRIMARY KEY (entity_kind, entity_id)
);

CREATE INDEX IF NOT EXISTS idx_stable_entity_ids_uid
    ON stable_entity_ids(uid);
CREATE INDEX IF NOT EXISTS idx_stable_entity_ids_live
    ON stable_entity_ids(entity_kind, tombstoned_at);

-- Preserve identity history when a local aggregate is deleted. Cascading
-- feature/work-package deletion fires the corresponding child triggers.
CREATE TRIGGER IF NOT EXISTS trg_stable_project_tombstone
AFTER DELETE ON projects
BEGIN
    UPDATE stable_entity_ids
       SET tombstoned_at = COALESCE(
           tombstoned_at,
           strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
       )
     WHERE entity_kind = 'project' AND entity_id = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_stable_feature_tombstone
AFTER DELETE ON features
BEGIN
    UPDATE stable_entity_ids
       SET tombstoned_at = COALESCE(
           tombstoned_at,
           strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
       )
     WHERE entity_kind = 'feature' AND entity_id = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_stable_work_package_tombstone
AFTER DELETE ON work_packages
BEGIN
    UPDATE stable_entity_ids
       SET tombstoned_at = COALESCE(
           tombstoned_at,
           strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
       )
     WHERE entity_kind = 'work_package' AND entity_id = OLD.id;
END;

-- DOWN
DROP TRIGGER IF EXISTS trg_stable_work_package_tombstone;
DROP TRIGGER IF EXISTS trg_stable_feature_tombstone;
DROP TRIGGER IF EXISTS trg_stable_project_tombstone;
DROP INDEX IF EXISTS idx_stable_entity_ids_live;
DROP INDEX IF EXISTS idx_stable_entity_ids_uid;
DROP TABLE IF EXISTS stable_entity_ids;
