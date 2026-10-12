-- UP
CREATE TABLE IF NOT EXISTS feature_acceptance_receipts (
    request_id TEXT PRIMARY KEY NOT NULL,
    feature_id INTEGER NOT NULL REFERENCES features(id),
    command_json TEXT NOT NULL,
    receipt_json TEXT NOT NULL,
    committed_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_feature_acceptance_receipts_feature ON feature_acceptance_receipts(feature_id);
CREATE TRIGGER IF NOT EXISTS acceptance_receipts_no_update BEFORE UPDATE ON feature_acceptance_receipts
BEGIN SELECT RAISE(ABORT, 'acceptance receipts are immutable'); END;
CREATE TRIGGER IF NOT EXISTS acceptance_receipts_no_delete BEFORE DELETE ON feature_acceptance_receipts
BEGIN SELECT RAISE(ABORT, 'acceptance receipts are append-only'); END;
-- DOWN
DROP TRIGGER IF EXISTS acceptance_receipts_no_update;
DROP TRIGGER IF EXISTS acceptance_receipts_no_delete;
DROP TABLE IF EXISTS feature_acceptance_receipts;
