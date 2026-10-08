//! Object preparation precedes durable journal recording; ref publication is CAS.
use crate::GitVcsAdapter;
use agileplus_domain::{error::DomainError, ports::promotion::PromotionVcsPort};
use git2::{BranchType, Oid};
fn storage(e: impl std::fmt::Display) -> DomainError {
    DomainError::Storage(e.to_string())
}
fn oid(s: &str) -> Result<Oid, DomainError> {
    Oid::from_str(s).map_err(storage)
}
#[async_trait::async_trait]
impl PromotionVcsPort for GitVcsAdapter {
    async fn promotion_target(&self, target: &str) -> Result<String, DomainError> {
        let repo = self.open()?;
        let branch = repo
            .find_branch(target, BranchType::Local)
            .map_err(storage)?;
        Ok(branch
            .get()
            .peel_to_commit()
            .map_err(storage)?
            .id()
            .to_string())
    }
    async fn prepare_promotion_merge(
        &self,
        source: &str,
        expected: &str,
    ) -> Result<String, DomainError> {
        let repo = self.open()?;
        let source = repo.find_commit(oid(source)?).map_err(storage)?;
        let target = repo.find_commit(oid(expected)?).map_err(storage)?;
        if source.id() == target.id()
            || repo
                .graph_descendant_of(target.id(), source.id())
                .map_err(storage)?
        {
            return Ok(expected.into());
        }
        let mut index = repo
            .merge_commits(&target, &source, None)
            .map_err(storage)?;
        if index.has_conflicts() {
            return Err(DomainError::Validation("Merge conflict while preparing exact-candidate promotion; no target or worktree changed".into()));
        }
        let tree_id = index.write_tree_to(&repo).map_err(storage)?;
        let tree = repo.find_tree(tree_id).map_err(storage)?;
        let signature = repo.signature().map_err(storage)?;
        let commit = repo
            .commit(
                None,
                &signature,
                &signature,
                &format!("AgilePlus promotion of exact candidate {}", source.id()),
                &tree,
                &[&target, &source],
            )
            .map_err(storage)?;
        Ok(commit.to_string())
    }
    async fn publish_promotion_merge(
        &self,
        target: &str,
        expected: &str,
        result: &str,
    ) -> Result<(), DomainError> {
        let repo = self.open()?;
        let branch = repo
            .find_branch(target, BranchType::Local)
            .map_err(storage)?;
        let current = branch.get().peel_to_commit().map_err(storage)?.id();
        if current == oid(result)? {
            return Ok(());
        }
        if current != oid(expected)? {
            return Err(DomainError::Conflict(
                "promotion target drifted from the durable merge intent".into(),
            ));
        }
        // Updating a checked-out ref would leave its index/worktree stale. Require
        // the operator to free the target rather than overwrite local resources.
        if branch.is_head() {
            return Err(DomainError::Conflict("promotion target is checked out; switch that worktree to another branch before shipping".into()));
        }
        let branch_ref = branch.get().name().map_err(storage)?;
        if self
            .run_git(&["worktree", "list", "--porcelain"])?
            .lines()
            .any(|line| line.strip_prefix("branch ") == Some(branch_ref))
        {
            return Err(DomainError::Conflict(
                "promotion target is checked out in another worktree; free it before shipping"
                    .into(),
            ));
        }
        let result_commit = repo.find_commit(oid(result)?).map_err(storage)?;
        if result_commit.parent_id(0).map_err(storage)? != current {
            return Err(DomainError::Conflict(
                "prepared merge does not extend the expected target".into(),
            ));
        }
        let name = branch.get().name().map_err(storage)?;
        repo.reference_matching(
            name,
            result_commit.id(),
            true,
            current,
            "AgilePlus durable promotion",
        )
        .map_err(storage)?;
        Ok(())
    }
}
