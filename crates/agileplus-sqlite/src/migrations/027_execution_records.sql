-- UP
CREATE TABLE IF NOT EXISTS spec_revisions (
    id TEXT PRIMARY KEY,
    feature_id INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
    content_hash TEXT NOT NULL,
    parent_revision_id TEXT REFERENCES spec_revisions(id),
    accepted_at TEXT NOT NULL,
    authority TEXT NOT NULL,
    UNIQUE(feature_id, content_hash)
);

CREATE TABLE IF NOT EXISTS assignments (
    id TEXT PRIMARY KEY,
    wp_id INTEGER NOT NULL REFERENCES work_packages(id) ON DELETE CASCADE,
    spec_revision_id TEXT NOT NULL REFERENCES spec_revisions(id),
    created_at TEXT NOT NULL,
    supersedes_assignment_id TEXT REFERENCES assignments(id),
    status TEXT NOT NULL CHECK(status IN ('active','superseded','cancelled'))
);

CREATE TABLE IF NOT EXISTS attempts (
    id TEXT PRIMARY KEY,
    assignment_id TEXT NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    worker_id TEXT NOT NULL,
    backend TEXT NOT NULL,
    job_id TEXT,
    worktree_path TEXT,
    base_candidate_ref TEXT,
    result_candidate_ref TEXT,
    status TEXT NOT NULL CHECK(status IN ('pending','running','failed','cancelled','completed','expired')),
    failure_class TEXT,
    started_at TEXT NOT NULL,
    ended_at TEXT
);

CREATE TABLE IF NOT EXISTS evaluations (
    id TEXT PRIMARY KEY,
    assignment_id TEXT NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    attempt_id TEXT REFERENCES attempts(id),
    candidate_ref TEXT NOT NULL,
    evaluator_id TEXT NOT NULL,
    evaluator_version TEXT NOT NULL,
    result TEXT NOT NULL CHECK(result IN ('satisfied','unsatisfied','inconclusive','unknown','not_configured','stale')),
    evidence_refs TEXT NOT NULL DEFAULT '[]',
    started_at TEXT NOT NULL,
    finished_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_spec_revisions_feature ON spec_revisions(feature_id, accepted_at);
CREATE INDEX IF NOT EXISTS idx_assignments_wp ON assignments(wp_id, created_at);
CREATE INDEX IF NOT EXISTS idx_attempts_assignment ON attempts(assignment_id, started_at);
CREATE INDEX IF NOT EXISTS idx_evaluations_assignment ON evaluations(assignment_id, finished_at);

-- DOWN
DROP TABLE IF EXISTS evaluations;
DROP TABLE IF EXISTS attempts;
DROP TABLE IF EXISTS assignments;
DROP TABLE IF EXISTS spec_revisions;
