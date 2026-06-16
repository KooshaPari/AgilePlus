-- UP
ALTER TABLE features ADD COLUMN labels TEXT NOT NULL DEFAULT '[]';

-- DOWN
-- ALTER TABLE features DROP COLUMN labels;
