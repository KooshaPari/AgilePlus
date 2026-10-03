// SPDX-License-Identifier: MIT OR Apache-2.0
//! End-to-end behavior tests for the `trace` command.
//!
//! `trace` is not yet routed by the binary's top-level command enum, so these
//! tests drive the public `commands::trace` entry points directly against real
//! SQLite databases in real temp directories: `link` rows are read back
//! straight out of SQLite, and `list`/`show` stdout is captured by re-execing
//! this test binary (`__trace_stdout_probe`). A broken `trace` fails here; a
//! passing test does not depend on a double.

use std::path::{Path, PathBuf};

use tempfile::TempDir;

use agileplus_cli::commands::trace::{LinkArgs, ListArgs, ShowArgs, run_link, run_list, run_show};

mod support;
use support::trace::{
    capture_probe, count_links, db_path, fetch_links, find_file, init_git_repo, unique,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Create a temp dir for a database; returns the tempdir and a db path inside it.
fn tmp_db(name: &str) -> (TempDir, PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let db = db_path(tmp.path(), name);
    (tmp, db)
}

/// Insert one link through the production entry point, asserting it succeeds.
fn seed(db: &Path, from: &str, to: &str, link_type: &str) {
    let args = LinkArgs {
        from: from.to_string(),
        to: to.to_string(),
        link_type: link_type.to_string(),
        note: String::new(),
        by: Some("seeder".to_string()),
        db: Some(db.to_path_buf()),
    };
    run_link(&args).unwrap_or_else(|e| panic!("seeding {from} -> {to} failed: {e:#}"));
}

fn db_s(db: &Path) -> String {
    db.to_str().expect("utf-8 temp path").to_string()
}

/// The actor `link` defaults to when `--by` is omitted.
fn expected_default_actor() -> String {
    std::env::var("USER")
        .ok()
        .or_else(|| std::env::var("USERNAME").ok())
        .unwrap_or_else(|| "system".to_string())
}

/// Re-exec entry point used by [`capture_probe`].
///
/// When `TRACE_PROBE` is set the harness runs in THIS process so the parent can
/// capture the command's stdout. During a normal suite run it performs a real
/// in-process `list`, so the test is not vacuous.
#[test]
fn __trace_stdout_probe() {
    let env = |k: &str| std::env::var(k).ok();
    match env("TRACE_PROBE").as_deref() {
        Some("list") => {
            let db = PathBuf::from(std::env::var("TRACE_DB").expect("TRACE_DB for list probe"));
            let limit = env("TRACE_LIMIT")
                .and_then(|v| v.parse().ok())
                .unwrap_or(25);
            run_list(&ListArgs {
                limit,
                db: Some(db),
            })
            .expect("probe run_list");
        }
        Some("show") => {
            let db = PathBuf::from(std::env::var("TRACE_DB").expect("TRACE_DB for show probe"));
            let entity = std::env::var("TRACE_ENTITY").expect("TRACE_ENTITY for show probe");
            run_show(&ShowArgs {
                entity,
                db: Some(db),
            })
            .expect("probe run_show");
        }
        Some("link") => {
            let db = PathBuf::from(std::env::var("TRACE_DB").expect("TRACE_DB for link probe"));
            let args = LinkArgs {
                from: std::env::var("TRACE_FROM").expect("TRACE_FROM"),
                to: std::env::var("TRACE_TO").expect("TRACE_TO"),
                link_type: env("TRACE_LINK_TYPE").unwrap_or_else(|| "implements".to_string()),
                note: env("TRACE_NOTE").unwrap_or_default(),
                by: env("TRACE_BY"),
                db: Some(db),
            };
            run_link(&args).expect("probe run_link");
        }
        Some("link-default") => {
            // Runs with cwd = a temp git repo; db = None uses the repo-local path.
            let args = LinkArgs {
                from: "work_package:1".to_string(),
                to: "feature:1".to_string(),
                link_type: "implements".to_string(),
                note: "default db".to_string(),
                by: Some("probe".to_string()),
                db: None,
            };
            run_link(&args).expect("probe run_link with default db");
        }
        Some(other) => panic!("unknown TRACE_PROBE `{other}`"),
        None => {
            // Normal suite execution: exercise the same path for real.
            let (_tmp, db) = tmp_db("probe.db");
            seed(&db, "work_package:1", "feature:1", "implements");
            run_list(&ListArgs {
                limit: 25,
                db: Some(db),
            })
            .expect("probe run_list");
        }
    }
}

// ── `trace link`: persistence ───────────────────────────────────────────────

#[test]
fn link_persists_a_row_with_every_field() {
    let (_tmp, db) = tmp_db("trace.db");
    let args = LinkArgs {
        from: "work_package:42".to_string(),
        to: "feature:7".to_string(),
        link_type: "implements".to_string(),
        note: "WP-42 implements the dashboard".to_string(),
        by: Some("tester".to_string()),
        db: Some(db.to_path_buf()),
    };
    run_link(&args).expect("link succeeds");

    let rows = fetch_links(&db);
    assert_eq!(rows.len(), 1, "exactly one row persisted");
    let r = &rows[0];
    assert_eq!(r.from_kind, "work_package");
    assert_eq!(r.from_id, "42");
    assert_eq!(r.to_kind, "feature");
    assert_eq!(r.to_id, "7");
    assert_eq!(r.link_type, "implements");
    assert_eq!(r.note, "WP-42 implements the dashboard");
    assert_eq!(r.created_by, "tester");
    assert!(!r.created_at.is_empty(), "created_at must be stamped");
}

#[test]
fn link_prints_the_new_edge_and_reports_duplicates_as_already_existing() {
    let (_tmp, db) = tmp_db("idem.db");
    seed(&db, "work_package:1", "feature:1", "verifies");

    // First insert line comes from the seeded call; the re-run is the probe.
    let text = capture_probe(
        "link",
        &[
            ("TRACE_DB", &db_s(&db)),
            ("TRACE_FROM", "work_package:1"),
            ("TRACE_TO", "feature:1"),
            ("TRACE_LINK_TYPE", "verifies"),
            ("TRACE_BY", "bot"),
        ],
        None,
    );
    assert!(text.contains("trace link already exists"), "stdout: {text}");
    assert!(
        text.contains("work_package:1 --verifies--> feature:1"),
        "stdout: {text}"
    );
    assert_eq!(
        count_links(&db),
        1,
        "duplicate must not create a second row"
    );
}

#[test]
fn link_reports_a_fresh_insert_with_its_row_id_and_note() {
    let (_tmp, db) = tmp_db("fresh.db");
    let text = capture_probe(
        "link",
        &[
            ("TRACE_DB", &db_s(&db)),
            ("TRACE_FROM", "story:3"),
            ("TRACE_TO", "epic:1"),
            ("TRACE_LINK_TYPE", "parent_of"),
            ("TRACE_NOTE", "story rolls up"),
            ("TRACE_BY", "human"),
        ],
        None,
    );
    assert!(text.contains("trace link #"), "stdout: {text}");
    assert!(
        text.contains("story:3 --parent_of--> epic:1"),
        "stdout: {text}"
    );
    assert!(text.contains("note: story rolls up"), "stdout: {text}");
    assert!(text.contains("by:   human"), "stdout: {text}");

    let r = &fetch_links(&db)[0];
    assert_eq!(r.link_type, "parent_of");
    assert_eq!(r.note, "story rolls up");
    assert_eq!(r.created_by, "human");
}

#[test]
fn link_defaults_to_implements_and_falls_back_to_the_environment_actor() {
    let (_tmp, db) = tmp_db("defaults.db");
    let args = LinkArgs {
        from: "story:9".to_string(),
        to: "epic:2".to_string(),
        link_type: "implements".to_string(),
        note: String::new(),
        by: None,
        db: Some(db.to_path_buf()),
    };
    run_link(&args).expect("link succeeds");

    let r = &fetch_links(&db)[0];
    assert_eq!(r.link_type, "implements");
    assert_eq!(r.created_by, expected_default_actor());
    assert_eq!(r.note, "", "default note is empty");
}

#[test]
fn ids_containing_colons_are_preserved() {
    let (_tmp, db) = tmp_db("colon.db");
    seed(&db, "work_package:abc:def", "feature:7", "implements");

    let r = &fetch_links(&db)[0];
    assert_eq!(r.from_kind, "work_package");
    assert_eq!(r.from_id, "abc:def");

    let text = capture_probe("list", &[("TRACE_DB", &db_s(&db))], None);
    assert!(text.contains("work_package:abc:def"), "stdout: {text}");
}

// ── `trace link`: validation ────────────────────────────────────────────────

#[test]
fn link_refuses_self_links_and_unknown_kinds_and_types_without_writing() {
    let (_tmp, db) = tmp_db("rejects.db");

    let build = |from: &str, to: &str, link_type: &str| LinkArgs {
        from: from.to_string(),
        to: to.to_string(),
        link_type: link_type.to_string(),
        note: String::new(),
        by: Some("tester".to_string()),
        db: Some(db.to_path_buf()),
    };

    let err = run_link(&build("work_package:5", "work_package:5", "implements"))
        .expect_err("self-link must be refused");
    assert!(format!("{err:#}").contains("self-link"), "error: {err:#}");

    let err = run_link(&build("bogus:1", "feature:1", "implements"))
        .expect_err("unknown from-kind must be refused");
    assert!(
        format!("{err:#}").contains("invalid --from-kind"),
        "error: {err:#}"
    );

    let err = run_link(&build("work_package:1", "feature:1", "made_up"))
        .expect_err("unknown link type must be refused");
    assert!(
        format!("{err:#}").contains("invalid --link-type"),
        "error: {err:#}"
    );

    assert!(
        !db.exists(),
        "validation must fail before the database file is written"
    );
}

#[test]
fn link_rejects_refs_that_are_not_kind_colon_id() {
    let (_tmp, db) = tmp_db("malformed.db");

    for bad in ["wp42", ":42", "wp:"] {
        let args = LinkArgs {
            from: bad.to_string(),
            to: "feature:7".to_string(),
            link_type: "implements".to_string(),
            note: String::new(),
            by: Some("tester".to_string()),
            db: Some(db.to_path_buf()),
        };
        let err = run_link(&args).expect_err(&format!("input {bad:?} must fail"));
        assert!(
            format!("{err:#}").contains("must be in `<kind>:<id>` form"),
            "error for {bad:?}: {err:#}"
        );
    }
    assert!(
        !db.exists(),
        "no database should be created for malformed refs"
    );
}

#[test]
fn link_accepts_every_documented_kind_and_link_type() {
    let (_tmp, db) = tmp_db("allowed.db");
    let kinds = [
        "work_package",
        "feature",
        "story",
        "epic",
        "project",
        "cycle",
        "module",
        "requirement",
        "external",
    ];
    let link_types = [
        "parent_of",
        "child_of",
        "depends_on",
        "blocks",
        "implements",
        "verifies",
        "references",
        "duplicates",
    ];

    for (i, kind) in kinds.iter().enumerate() {
        let from = format!("{kind}:{i}");
        seed(&db, &from, "feature:0", link_types[i % link_types.len()]);
    }
    assert_eq!(count_links(&db), kinds.len() as i64);
    assert_eq!(fetch_links(&db).len(), kinds.len());
}

// ── Default repository-local database ───────────────────────────────────────

#[test]
fn link_without_db_writes_the_repository_local_database_on_disk() {
    let tmp = TempDir::new().expect("tempdir");
    let repo = tmp.path().join(unique("repo"));
    std::fs::create_dir_all(&repo).expect("creating fixture repo dir");
    init_git_repo(&repo);

    let text = capture_probe("link-default", &[], Some(&repo));
    assert!(
        text.contains("work_package:1 --implements--> feature:1"),
        "stdout: {text}"
    );

    let found = find_file(&repo, "agileplus.db")
        .expect("the default repository-local database must exist under the repo");
    assert_eq!(count_links(&found), 1);

    let list = capture_probe("list", &[("TRACE_DB", found.to_str().unwrap())], None);
    assert!(list.contains("work_package:1"), "stdout: {list}");
    assert!(list.contains("feature:1"), "stdout: {list}");
}

// ── `trace list` ────────────────────────────────────────────────────────────

#[test]
fn list_reports_emptiness_then_newest_first_and_honors_limit() {
    let (_tmp, db) = tmp_db("list.db");

    let empty = capture_probe("list", &[("TRACE_DB", &db_s(&db))], None);
    assert!(
        empty.contains("No trace links recorded."),
        "stdout: {empty}"
    );

    for id in 1..=3 {
        seed(
            &db,
            &format!("work_package:{id}"),
            "feature:1",
            "implements",
        );
    }

    let all = capture_probe("list", &[("TRACE_DB", &db_s(&db))], None);
    assert!(
        all.contains("3 trace link(s) shown (limit=25)."),
        "stdout: {all}"
    );
    // ORDER BY id DESC: the newest link (id 3) precedes the oldest (id 1).
    let newest = all.find("work_package:3").expect("row 3 present");
    let oldest = all.find("work_package:1").expect("row 1 present");
    assert!(newest < oldest, "rows must be newest-first:\n{all}");

    let limited = capture_probe(
        "list",
        &[("TRACE_DB", &db_s(&db)), ("TRACE_LIMIT", "2")],
        None,
    );
    assert!(
        limited.contains("2 trace link(s) shown (limit=2)."),
        "stdout: {limited}"
    );
    assert!(
        limited.contains("work_package:3") && limited.contains("work_package:2"),
        "stdout: {limited}"
    );
    assert!(
        !limited.contains("work_package:1"),
        "limit=2 must drop the oldest: {limited}"
    );
}

#[test]
fn list_clamps_a_zero_limit_up_to_one() {
    let (_tmp, db) = tmp_db("clamp.db");
    for id in 1..=2 {
        seed(
            &db,
            &format!("work_package:{id}"),
            "feature:1",
            "implements",
        );
    }

    let out = capture_probe(
        "list",
        &[("TRACE_DB", &db_s(&db)), ("TRACE_LIMIT", "0")],
        None,
    );
    assert!(
        out.contains("1 trace link(s) shown (limit=1)."),
        "stdout: {out}"
    );
    assert!(out.contains("work_package:2"), "newest row shown: {out}");
    assert!(!out.contains("work_package:1"), "only one row shown: {out}");
}

// ── `trace show` ────────────────────────────────────────────────────────────

#[test]
fn show_reports_both_outgoing_and_incoming_edges() {
    let (_tmp, db) = tmp_db("show.db");
    seed(&db, "work_package:5", "feature:3", "implements");
    seed(&db, "story:11", "work_package:5", "verifies");

    let out = capture_probe(
        "show",
        &[("TRACE_DB", &db_s(&db)), ("TRACE_ENTITY", "work_package:5")],
        None,
    );
    assert!(
        out.contains("Trace links touching `work_package:5` (2):"),
        "stdout: {out}"
    );
    assert!(
        out.contains("OUT work_package:5 --implements--> feature:3"),
        "stdout: {out}"
    );
    assert!(
        out.contains("IN  story:11 --verifies--> work_package:5"),
        "stdout: {out}"
    );
}

#[test]
fn show_reports_no_links_for_an_unknown_entity() {
    let (_tmp, db) = tmp_db("show-none.db");
    seed(&db, "work_package:1", "feature:2", "implements");

    let out = capture_probe(
        "show",
        &[("TRACE_DB", &db_s(&db)), ("TRACE_ENTITY", "feature:999")],
        None,
    );
    assert!(
        out.contains("No trace links touch `feature:999`."),
        "stdout: {out}"
    );
}

#[test]
fn show_includes_the_free_form_note_on_each_edge() {
    let (_tmp, db) = tmp_db("show-note.db");
    let args = LinkArgs {
        from: "work_package:5".to_string(),
        to: "feature:3".to_string(),
        link_type: "implements".to_string(),
        note: "outbound edge".to_string(),
        by: Some("tester".to_string()),
        db: Some(db.to_path_buf()),
    };
    run_link(&args).expect("link succeeds");

    let out = capture_probe(
        "show",
        &[("TRACE_DB", &db_s(&db)), ("TRACE_ENTITY", "work_package:5")],
        None,
    );
    assert!(out.contains("note: outbound edge"), "stdout: {out}");
}
