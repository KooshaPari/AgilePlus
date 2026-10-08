//! Real Git + file SQLite crash boundaries for the canonical promotion saga.
use agileplus_cli::commands::ship::run_ship;
use agileplus_domain::{
    domain::state_machine::FeatureState,
    domain::work_package::WpState,
    ports::{
        StoragePort,
        promotion::{PromotionPort, PromotionVcsPort},
    },
};
use agileplus_git::GitVcsAdapter;
use agileplus_sqlite::SqliteStorageAdapter;
use std::{path::Path, process::Command};
mod support;
use support::ship::{args, block_on, seed_with_candidate};
fn git(path: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().into()
}
async fn setup(path: &Path, slug: &str) -> (SqliteStorageAdapter, GitVcsAdapter, i64, String) {
    std::fs::create_dir(path.join("repo")).unwrap();
    let repo = path.join("repo");
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["config", "user.email", "test@example.com"]);
    git(&repo, &["config", "user.name", "Test"]);
    std::fs::write(repo.join("base"), "base").unwrap();
    git(&repo, &["add", "base"]);
    git(&repo, &["commit", "-qm", "base"]);
    let branch = format!("feat/{slug}/WP01");
    git(&repo, &["checkout", "-qb", &branch]);
    std::fs::write(repo.join("candidate"), "accepted").unwrap();
    git(&repo, &["add", "candidate"]);
    git(&repo, &["commit", "-qm", "candidate"]);
    let candidate = git(&repo, &["rev-parse", "HEAD"]);
    let db = SqliteStorageAdapter::new(&path.join("state.sqlite")).unwrap();
    let id = seed_with_candidate(
        &db,
        slug,
        FeatureState::Validated,
        &[(1, WpState::Done)],
        Some(&candidate),
        slug != "missing",
    )
    .await;
    (db, GitVcsAdapter::new(repo), id, candidate)
}
#[allow(dead_code)]
fn state(db: &SqliteStorageAdapter, id: i64) -> FeatureState {
    block_on(async {
        StoragePort::get_feature_by_id(db, id)
            .await
            .unwrap()
            .unwrap()
            .state
    })
}
#[test]
fn committed_receipt_replays_after_reopen_without_another_merge_or_event() {
    block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let (db, vcs, id, candidate) = setup(dir.path(), "replay").await;
        run_ship(args("replay"), &db, &vcs).await.unwrap();
        let journal = db.get_promotion(id).await.unwrap().unwrap();
        let receipt = journal.receipt.unwrap();
        let target = vcs.promotion_target("main").await.unwrap();
        assert_eq!(receipt.resulting_commit, target);
        assert_eq!(git(vcs.repo_root(), &["rev-parse", "main^2"]), candidate);
        assert!(journal.cleanup_complete);
        drop(db);
        let db = SqliteStorageAdapter::new(&dir.path().join("state.sqlite")).unwrap();
        run_ship(args("replay"), &db, &vcs).await.unwrap();
        assert_eq!(
            db.get_promotion(id)
                .await
                .unwrap()
                .unwrap()
                .receipt
                .unwrap(),
            receipt
        );
        assert_eq!(vcs.promotion_target("main").await.unwrap(), target);
        assert_eq!(
            db.conn_for_bench()
                .unwrap()
                .query_row("SELECT COUNT(*) FROM events", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
    });
}
#[test]
fn restart_after_git_publication_before_confirmation_recovers_exact_intent() {
    block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let (db, vcs, id, _) = setup(dir.path(), "lost").await;
        db.conn_for_bench().unwrap().execute_batch("CREATE TRIGGER fail_confirm BEFORE UPDATE ON promotion_journals WHEN json_extract(NEW.journal_json,'$.steps[0].confirmed')=1 BEGIN SELECT RAISE(ABORT,'injected crash after ref publication'); END;").unwrap();
        assert!(run_ship(args("lost"), &db, &vcs).await.is_err());
        let j = db.get_promotion(id).await.unwrap().unwrap();
        assert!(!j.steps[0].confirmed);
        assert!(j.receipt.is_none());
        let target = vcs.promotion_target("main").await.unwrap();
        assert_eq!(target, j.steps[0].resulting_commit);
        assert_eq!(
            StoragePort::get_feature_by_id(&db, id)
                .await
                .unwrap()
                .unwrap()
                .state,
            FeatureState::Validated
        );
        drop(db);
        let db = SqliteStorageAdapter::new(&dir.path().join("state.sqlite")).unwrap();
        db.conn_for_bench()
            .unwrap()
            .execute_batch("DROP TRIGGER fail_confirm")
            .unwrap();
        run_ship(args("lost"), &db, &vcs).await.unwrap();
        assert_eq!(vcs.promotion_target("main").await.unwrap(), target);
        assert!(
            db.get_promotion(id)
                .await
                .unwrap()
                .unwrap()
                .receipt
                .is_some()
        );
    });
}
#[test]
fn late_receipt_failure_rolls_back_terminal_state_audit_and_event_then_resumes() {
    block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let (db, vcs, id, _) = setup(dir.path(), "rollback").await;
        db.conn_for_bench().unwrap().execute_batch("CREATE TRIGGER fail_receipt BEFORE UPDATE ON promotion_journals WHEN json_extract(NEW.journal_json,'$.receipt') IS NOT NULL BEGIN SELECT RAISE(ABORT,'late receipt failure'); END;").unwrap();
        assert!(run_ship(args("rollback"), &db, &vcs).await.is_err());
        assert_eq!(
            StoragePort::get_feature_by_id(&db, id)
                .await
                .unwrap()
                .unwrap()
                .state,
            FeatureState::Validated
        );
        assert!(
            StoragePort::get_audit_trail(&db, id)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            db.conn_for_bench()
                .unwrap()
                .query_row("SELECT COUNT(*) FROM events", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        let target = vcs.promotion_target("main").await.unwrap();
        drop(db);
        let db = SqliteStorageAdapter::new(&dir.path().join("state.sqlite")).unwrap();
        db.conn_for_bench()
            .unwrap()
            .execute_batch("DROP TRIGGER fail_receipt")
            .unwrap();
        run_ship(args("rollback"), &db, &vcs).await.unwrap();
        assert_eq!(vcs.promotion_target("main").await.unwrap(), target);
        assert_eq!(
            StoragePort::get_audit_trail(&db, id).await.unwrap().len(),
            1
        );
    });
}
#[test]
fn checked_out_target_preserves_files_and_resumes_after_target_is_freed() {
    block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let (db, vcs, id, _) = setup(dir.path(), "free").await;
        git(vcs.repo_root(), &["checkout", "-q", "main"]);
        let before = vcs.promotion_target("main").await.unwrap();
        let error = run_ship(args("free"), &db, &vcs).await.unwrap_err();
        assert!(error.to_string().contains("checked out"));
        assert_eq!(vcs.promotion_target("main").await.unwrap(), before);
        assert!(!vcs.repo_root().join("candidate").exists());
        assert!(!db.get_promotion(id).await.unwrap().unwrap().steps[0].confirmed);
        git(vcs.repo_root(), &["checkout", "-q", "feat/free/WP01"]);
        run_ship(args("free"), &db, &vcs).await.unwrap();
    });
}
#[test]
fn target_drift_after_intent_fails_closed_without_terminal_state() {
    block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let (db, vcs, id, _) = setup(dir.path(), "drift").await;
        git(vcs.repo_root(), &["checkout", "-q", "main"]);
        assert!(run_ship(args("drift"), &db, &vcs).await.is_err());
        std::fs::write(vcs.repo_root().join("outside"), "outside").unwrap();
        git(vcs.repo_root(), &["add", "outside"]);
        git(vcs.repo_root(), &["commit", "-qm", "outside"]);
        let drift = vcs.promotion_target("main").await.unwrap();
        git(vcs.repo_root(), &["checkout", "-q", "feat/drift/WP01"]);
        assert!(
            run_ship(args("drift"), &db, &vcs)
                .await
                .unwrap_err()
                .to_string()
                .contains("drifted")
        );
        assert_eq!(vcs.promotion_target("main").await.unwrap(), drift);
        assert_eq!(
            StoragePort::get_feature_by_id(&db, id)
                .await
                .unwrap()
                .unwrap()
                .state,
            FeatureState::Validated
        );
    });
}
#[test]
fn validated_label_without_immutable_acceptance_receipt_cannot_ship() {
    block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let (db, vcs, id, _) = setup(dir.path(), "missing").await;
        let before = vcs.promotion_target("main").await.unwrap();
        assert!(
            run_ship(args("missing"), &db, &vcs)
                .await
                .unwrap_err()
                .to_string()
                .contains("durable acceptance receipt")
        );
        assert_eq!(vcs.promotion_target("main").await.unwrap(), before);
        assert!(db.get_promotion(id).await.unwrap().is_none());
    });
}

#[test]
fn primary_worktree_target_is_protected_when_shipping_from_linked_worktree() {
    block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let (db, vcs, id, _) = setup(dir.path(), "linked").await;
        git(vcs.repo_root(), &["checkout", "-q", "main"]);
        let linked = dir.path().join("linked-worktree");
        git(
            vcs.repo_root(),
            &[
                "worktree",
                "add",
                "-q",
                linked.to_str().unwrap(),
                "feat/linked/WP01",
            ],
        );
        let caller = GitVcsAdapter::new(linked);
        let before = caller.promotion_target("main").await.unwrap();
        assert!(
            run_ship(args("linked"), &db, &caller)
                .await
                .unwrap_err()
                .to_string()
                .contains("checked out in another worktree")
        );
        assert_eq!(caller.promotion_target("main").await.unwrap(), before);
        assert_eq!(
            StoragePort::get_feature_by_id(&db, id)
                .await
                .unwrap()
                .unwrap()
                .state,
            FeatureState::Validated
        );
    });
}
#[test]
fn conflicting_object_preparation_never_changes_git_target_or_terminal_history() {
    block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let (db, vcs, id, _) = setup(dir.path(), "conflict").await;
        git(vcs.repo_root(), &["checkout", "-q", "main"]);
        std::fs::write(vcs.repo_root().join("candidate"), "different content").unwrap();
        git(vcs.repo_root(), &["add", "candidate"]);
        git(vcs.repo_root(), &["commit", "-qm", "conflicting target"]);
        let before = vcs.promotion_target("main").await.unwrap();
        git(vcs.repo_root(), &["checkout", "-q", "feat/conflict/WP01"]);
        assert!(
            run_ship(args("conflict"), &db, &vcs)
                .await
                .unwrap_err()
                .to_string()
                .contains("Merge conflict")
        );
        assert_eq!(vcs.promotion_target("main").await.unwrap(), before);
        assert!(
            db.get_promotion(id)
                .await
                .unwrap()
                .unwrap()
                .steps
                .is_empty()
        );
        assert!(
            StoragePort::get_audit_trail(&db, id)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(!vcs.repo_root().join(".git/MERGE_HEAD").exists());
    });
}
