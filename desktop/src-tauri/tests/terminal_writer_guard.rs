// SPDX-License-Identifier: MIT OR Apache-2.0
//! TDD regression for audit defect A3: the desktop command must not be a
//! second terminal-state writer. This harness calls the extracted pure
//! mutation directly with a rusqlite connection, without Tauri State or IPC.

use app_lib::work_packages::update_work_package_state_in;
use agileplus_sqlite::migrations::MigrationRunner;
use rusqlite::Connection;

const FEATURE_ID: i64 = 42;
const WP_ID: i64 = 7;

fn migrated_db_with_wp_state(state: &str) -> Connection {
    let conn = Connection::open_in_memory().expect("in-memory connection");
    MigrationRunner::new(&conn).run_all().expect("migrations");
    conn.execute(
        "INSERT INTO features (id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at) \
         VALUES (?1, 'desktop-guard', 'Desktop Guard', 'created', X'01', 'main', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [FEATURE_ID],
    )
    .expect("seed feature");
    conn.execute(
        "INSERT INTO work_packages (id, feature_id, title, state, sequence, acceptance_criteria, created_at, updated_at) \
         VALUES (?1, ?2, 'Guarded WP', ?3, 1, 'criterion', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        rusqlite::params![WP_ID, FEATURE_ID, state],
    )
    .expect("seed work package");
    conn
}

fn migrated_db_with_planned_wp() -> Connection {
    migrated_db_with_wp_state("planned")
}

#[test]
fn update_work_package_state_rejects_direct_planned_to_done() {
    let conn = migrated_db_with_planned_wp();

    let result = update_work_package_state_in(&conn, WP_ID, "done", "2026-01-02T00:00:00Z");
    eprintln!("guarded update result: {result:?}");
    let err = result.expect_err("desktop direct Planned -> Done must be rejected");
    assert!(
        err.contains("terminal") || err.contains("not allowed"),
        "unexpected error: {err}"
    );

    let state: String = conn
        .query_row(
            "SELECT state FROM work_packages WHERE id = ?1",
            [WP_ID],
            |row| row.get(0),
        )
        .expect("work package still exists");
    assert_eq!(state, "planned", "rejected transition must not mutate state");

    let audit: i64 = conn
        .query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))
        .expect("count audit entries");
    assert_eq!(audit, 0, "rejected transition must not write audit entries");

    let receipts: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM feature_acceptance_receipts",
            [],
            |row| row.get(0),
        )
        .expect("count acceptance receipts");
    assert_eq!(
        receipts, 0,
        "rejected transition must not write acceptance receipts"
    );
}

#[test]
fn update_work_package_state_rejects_blocked_and_unknown_states() {
    for state in ["blocked", "DONE", "unknown"] {
        let conn = migrated_db_with_planned_wp();
        let err = update_work_package_state_in(&conn, WP_ID, state, "2026-01-02T00:00:00Z")
            .expect_err("blocked/terminal/unknown states must be rejected");
        assert!(
            err.contains("terminal or unsupported"),
            "unexpected error for {state}: {err}"
        );

        let current: String = conn
            .query_row(
                "SELECT state FROM work_packages WHERE id = ?1",
                [WP_ID],
                |row| row.get(0),
            )
            .expect("work package still exists");
        assert_eq!(current, "planned", "state must remain Planned for {state}");
    }
}

#[test]
fn update_work_package_state_allows_non_terminal_transition() {
    let conn = migrated_db_with_planned_wp();
    update_work_package_state_in(&conn, WP_ID, "doing", "2026-01-02T00:00:00Z")
        .expect("planned -> doing must remain allowed for the desktop UI");

    let state: String = conn
        .query_row(
            "SELECT state FROM work_packages WHERE id = ?1",
            [WP_ID],
            |row| row.get(0),
        )
        .expect("work package still exists");
    assert_eq!(state, "doing");
}

#[test]
fn update_work_package_state_rejects_mutation_of_done_wp() {
    for requested in ["planned", "review"] {
        let conn = migrated_db_with_wp_state("done");

        let err = update_work_package_state_in(
            &conn,
            WP_ID,
            requested,
            "2026-01-02T00:00:00Z",
        )
        .expect_err("an accepted Done work package must be immutable from desktop");
        assert!(
            err.contains("terminal"),
            "unexpected error for request {requested}: {err}"
        );

        let current: String = conn
            .query_row(
                "SELECT state FROM work_packages WHERE id = ?1",
                [WP_ID],
                |row| row.get(0),
            )
            .expect("work package still exists");
        assert_eq!(current, "done", "Done WP must remain Done");

        let audit: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))
            .expect("count audit entries");
        assert_eq!(audit, 0, "rejected mutation must not write audit entries");

        let receipts: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM feature_acceptance_receipts",
                [],
                |row| row.get(0),
            )
            .expect("count acceptance receipts");
        assert_eq!(
            receipts, 0,
            "rejected mutation must not write acceptance receipts"
        );
    }
}

#[test]
fn update_work_package_state_releases_a_blocked_work_package_for_retry() {
    // `implement` parks a work package in `blocked` when it exhausts its review
    // cycles. Only `done` carries an acceptance receipt, so `blocked` must stay
    // mutable: otherwise an operator can never retry the abandoned work.
    let conn = migrated_db_with_wp_state("blocked");

    update_work_package_state_in(&conn, WP_ID, "planned", "2026-01-02T00:00:00Z")
        .expect("blocked -> planned must remain available for operator retry");

    let state: String = conn
        .query_row(
            "SELECT state FROM work_packages WHERE id = ?1",
            [WP_ID],
            |row| row.get(0),
        )
        .expect("work package still exists");
    assert_eq!(state, "planned");
}
