// NOTE: Modules marked with `// STUB` have unresolved dependencies (missing
// crate additions or missing domain types) and are temporarily excluded from
// compilation until those upstream gaps are filled.  They are kept in the
// source tree for reference.

pub mod dag;
pub mod dashboard;
pub mod gate_add;
pub mod gate_run;
pub mod import_dagctl;
pub mod list;
pub mod list_epics;
pub mod list_projects;
pub mod list_stories;
pub mod list_tests;
pub mod run_record;
pub mod scope_status;
pub mod seed_requirements;
pub mod trace;
pub mod triage;
pub mod version;
pub mod worklog;

// ── stub modules (excluded until upstream deps are resolved) ──────────────────
pub mod branch;          // OK: VcsPort only
pub mod cycle;           // STUB: agileplus_plane dep missing
pub mod governance;      // OK: VcsPort read_artifact only
#[cfg(feature = "full-deps")]
pub mod implement;       // STUB: agileplus_domain::ports::agent fields mismatch
// pub mod module;        // STUB: agileplus_plane dep missing
#[cfg(all(feature = "events", feature = "plane", feature = "triage"))]
pub mod plan;            // STUB: incomplete
#[cfg(all(feature = "events", feature = "plane", feature = "triage"))]
pub mod pr_builder;      // STUB: incomplete
// pub mod queue;         // STUB: agileplus_triage dep missing
#[cfg(all(feature = "events", feature = "plane", feature = "triage"))]
pub mod research;        // STUB: incomplete
// pub mod retrospective; // STUB: agileplus_events dep missing
#[cfg(all(feature = "events", feature = "plane", feature = "triage"))]
pub mod review_loop;     // STUB: agent port field mismatch
#[cfg(all(feature = "events", feature = "plane", feature = "triage"))]
pub mod scheduler;       // STUB: incomplete
#[cfg(all(feature = "events", feature = "plane", feature = "triage"))]
pub mod scope;           // STUB: incomplete
#[cfg(all(feature = "events", feature = "plane", feature = "triage"))]
pub mod ship;            // STUB: agileplus_events dep missing
#[cfg(all(feature = "events", feature = "plane", feature = "triage"))]
pub mod specify;         // STUB: similar dep missing
#[cfg(all(feature = "events", feature = "plane", feature = "triage"))]
pub mod validate;        // STUB: agileplus_events dep missing
