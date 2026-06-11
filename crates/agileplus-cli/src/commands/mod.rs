// NOTE: Modules marked with `// STUB` have unresolved dependencies (missing
// crate additions or missing domain types) and are temporarily excluded from
// compilation until those upstream gaps are filled.  They are kept in the
// source tree for reference.

pub mod list;
pub mod list_epics;
pub mod list_projects;
pub mod list_stories;
pub mod list_tests;
pub mod seed_requirements;
pub mod worklog;

// ── stub modules (excluded until upstream deps are resolved) ──────────────────
// pub mod branch;          // STUB: agileplus_events + missing VCS types
// pub mod cycle;           // STUB: agileplus_plane dep missing
// pub mod governance;      // STUB: incomplete
// pub mod implement;       // STUB: agileplus_domain::ports::agent fields mismatch
// pub mod module;          // STUB: agileplus_plane dep missing
// pub mod plan;            // STUB: incomplete
// pub mod pr_builder;      // STUB: incomplete
// pub mod queue;           // STUB: agileplus_triage dep missing
// pub mod research;        // STUB: incomplete
// pub mod retrospective;   // STUB: agileplus_events dep missing
// pub mod review_loop;     // STUB: agent port field mismatch
// pub mod scheduler;       // STUB: incomplete
// pub mod scope;           // STUB: incomplete
// pub mod ship;            // STUB: agileplus_events dep missing
// pub mod specify;         // STUB: similar dep missing
// pub mod triage;          // STUB: agileplus_triage dep missing
// pub mod validate;        // STUB: agileplus_events dep missing
