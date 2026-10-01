-- UP
CREATE TABLE IF NOT EXISTS assignment_criteria (
    assignment_id TEXT NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    criterion_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL CHECK(ordinal >= 1),
    statement TEXT NOT NULL,
    source_ref TEXT,
    mandatory INTEGER NOT NULL CHECK(mandatory IN (0,1)),
    PRIMARY KEY (assignment_id, criterion_id),
    UNIQUE (assignment_id, ordinal)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_evaluations_id_assignment
    ON evaluations(id, assignment_id);

CREATE TABLE IF NOT EXISTS evaluation_criterion_results (
    evaluation_id TEXT NOT NULL,
    assignment_id TEXT NOT NULL,
    criterion_id TEXT NOT NULL,
    result TEXT NOT NULL CHECK(result IN (
        'satisfied','unsatisfied','inconclusive','unknown','not_configured','stale'
    )),
    evidence_refs TEXT NOT NULL DEFAULT '[]',
    rationale TEXT,
    PRIMARY KEY (evaluation_id, criterion_id),
    FOREIGN KEY (evaluation_id, assignment_id)
        REFERENCES evaluations(id, assignment_id) ON DELETE CASCADE,
    FOREIGN KEY (assignment_id, criterion_id)
        REFERENCES assignment_criteria(assignment_id, criterion_id)
);

CREATE INDEX IF NOT EXISTS idx_assignment_criteria_assignment
    ON assignment_criteria(assignment_id, ordinal);
CREATE INDEX IF NOT EXISTS idx_eval_criteria_evaluation
    ON evaluation_criterion_results(evaluation_id);

-- DOWN
DROP TABLE IF EXISTS evaluation_criterion_results;
DROP INDEX IF EXISTS idx_evaluations_id_assignment;
DROP TABLE IF EXISTS assignment_criteria;
