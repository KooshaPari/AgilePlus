-- UP
-- Enforce the mature execution invariant that a WorkPackage has at most one
-- active Assignment. Historical superseded/cancelled Assignments remain durable.
CREATE UNIQUE INDEX IF NOT EXISTS idx_assignments_one_active_per_wp
    ON assignments(wp_id)
    WHERE status = 'active';

-- DOWN
DROP INDEX IF EXISTS idx_assignments_one_active_per_wp;
