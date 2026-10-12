-- UP
CREATE TABLE promotion_journals (
    feature_id INTEGER PRIMARY KEY REFERENCES features(id),
    journal_json TEXT NOT NULL
);
CREATE TRIGGER promotion_receipt_immutable BEFORE UPDATE ON promotion_journals
WHEN json_extract(OLD.journal_json,'$.receipt') IS NOT NULL
AND json_extract(NEW.journal_json,'$.receipt') IS NOT json_extract(OLD.journal_json,'$.receipt')
BEGIN SELECT RAISE(ABORT,'promotion receipts are immutable'); END;
CREATE TRIGGER promotion_history_immutable BEFORE DELETE ON promotion_journals
BEGIN SELECT RAISE(ABORT,'promotion history is immutable'); END;
-- DOWN
DROP TRIGGER promotion_receipt_immutable;
DROP TRIGGER promotion_history_immutable;
DROP TABLE promotion_journals;
