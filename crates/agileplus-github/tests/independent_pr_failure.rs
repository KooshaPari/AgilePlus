use agileplus_github::map::{GhIssue, GhPullRequest};
use agileplus_github::sync::{GhDataSource, sync_repository};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};

struct PrFailure {
    issues: AtomicUsize,
    prs: AtomicUsize,
}

#[async_trait]
impl GhDataSource for PrFailure {
    async fn list_issues(&self) -> anyhow::Result<Vec<GhIssue>> {
        self.issues.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    }

    async fn list_prs(&self) -> anyhow::Result<Vec<GhPullRequest>> {
        self.prs.fetch_add(1, Ordering::SeqCst);
        anyhow::bail!("independent PR listing failure")
    }
}

#[tokio::test]
async fn pr_listing_failure_propagates_after_successful_issue_listing_on_retry() {
    let source = PrFailure {
        issues: AtomicUsize::new(0),
        prs: AtomicUsize::new(0),
    };
    for _ in 0..2 {
        let error = sync_repository(&source, 1, 2)
            .await
            .expect_err("PR failure must abort sync");
        assert!(format!("{error:#}").contains("independent PR listing failure"));
    }
    assert_eq!(source.issues.load(Ordering::SeqCst), 2);
    assert_eq!(source.prs.load(Ordering::SeqCst), 2);
}
