//! Git observer - watches .git/ for ref changes and emits typed events.

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::broadcast;
use tracing::{debug, info};

#[derive(Debug, Clone, serde::Serialize)]
pub enum GitEvent {
    RefChanged {
        ref_name: String,
        old_oid: Option<String>,
        new_oid: String,
    },
    Checkout {
        branch: String,
    },
    Merge {
        source: String,
        target: String,
    },
    Rebase {
        branch: String,
    },
    WorktreeAdded {
        path: PathBuf,
    },
    WorktreeRemoved {
        path: PathBuf,
    },
}

pub struct GitObserver {
    repo_root: PathBuf,
    tx: broadcast::Sender<GitEvent>,
    _watcher: RecommendedWatcher,
}

impl GitObserver {
    pub fn new(repo_root: PathBuf) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, _) = broadcast::channel(256);
        let tx_clone = tx.clone();
        let root = repo_root.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                Self::handle_fs_event(&root, &tx_clone, event);
            }
        })?;

        // Watch .git directory for ref changes
        let git_dir = repo_root.join(".git");
        if git_dir.exists() {
            watcher.watch(&git_dir.join("refs"), RecursiveMode::Recursive)?;
            watcher.watch(&git_dir.join("HEAD"), RecursiveMode::NonRecursive)?;
        }

        info!("Git observer started for {}", repo_root.display());

        Ok(Self {
            repo_root,
            tx,
            _watcher: watcher,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GitEvent> {
        self.tx.subscribe()
    }

    pub fn repo_root(&self) -> &std::path::Path {
        &self.repo_root
    }

    fn handle_fs_event(
        repo_root: &std::path::Path,
        tx: &broadcast::Sender<GitEvent>,
        event: Event,
    ) {
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                for path in &event.paths {
                    if let Some(git_event) = Self::classify_change(repo_root, path) {
                        debug!("Git event: {:?}", git_event);
                        let _ = tx.send(git_event);
                    }
                }
            }
            _ => {}
        }
    }

    fn classify_change(repo_root: &std::path::Path, path: &std::path::Path) -> Option<GitEvent> {
        let rel = path.strip_prefix(repo_root.join(".git")).ok()?;
        let rel_str = rel.to_string_lossy();

        if rel_str == "HEAD" {
            let head_content = std::fs::read_to_string(path).ok()?;
            if head_content.starts_with("ref: refs/heads/") {
                let branch = head_content
                    .trim()
                    .strip_prefix("ref: refs/heads/")?
                    .to_string();
                return Some(GitEvent::Checkout { branch });
            }
        }

        if rel_str.starts_with("refs/heads/") {
            let ref_name = rel_str.to_string();
            let new_oid = std::fs::read_to_string(path).ok()?.trim().to_string();
            return Some(GitEvent::RefChanged {
                ref_name,
                old_oid: None,
                new_oid,
            });
        }

        if rel_str.starts_with("refs/stash") || rel_str.contains("MERGE_HEAD") {
            return Some(GitEvent::Merge {
                source: "unknown".to_string(),
                target: "HEAD".to_string(),
            });
        }

        if rel_str.contains("rebase-merge") || rel_str.contains("rebase-apply") {
            return Some(GitEvent::Rebase {
                branch: "HEAD".to_string(),
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn new_without_git_dir_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let obs = GitObserver::new(dir.path().to_path_buf()).unwrap();
        assert_eq!(obs.repo_root(), dir.path());
    }

    #[test]
    fn subscribe_returns_live_receiver() {
        let dir = tempfile::tempdir().unwrap();
        let obs = GitObserver::new(dir.path().to_path_buf()).unwrap();
        let mut rx = obs.subscribe();
        // No events yet.
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn classify_head_ref_is_checkout() {
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join(".git");
        std::fs::create_dir_all(&git).unwrap();
        std::fs::write(git.join("HEAD"), "ref: refs/heads/feature/x\n").unwrap();
        let ev = GitObserver::classify_change(dir.path(), &git.join("HEAD"));
        match ev {
            Some(GitEvent::Checkout { branch }) => assert_eq!(branch, "feature/x"),
            other => panic!("expected Checkout, got {other:?}"),
        }
    }

    #[test]
    fn classify_detached_head_is_none() {
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join(".git");
        std::fs::create_dir_all(&git).unwrap();
        std::fs::write(git.join("HEAD"), "0123456789abcdef0123456789abcdef01234567\n").unwrap();
        assert!(GitObserver::classify_change(dir.path(), &git.join("HEAD")).is_none());
    }

    #[test]
    fn classify_branch_ref_is_ref_changed() {
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join(".git");
        let refs = git.join("refs/heads");
        std::fs::create_dir_all(&refs).unwrap();
        let p = refs.join("main");
        std::fs::write(&p, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n").unwrap();
        match GitObserver::classify_change(dir.path(), &p) {
            Some(GitEvent::RefChanged { ref_name, new_oid, old_oid }) => {
                assert_eq!(ref_name, "refs/heads/main");
                assert_eq!(new_oid, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
                assert!(old_oid.is_none());
            }
            other => panic!("expected RefChanged, got {other:?}"),
        }
    }

    #[test]
    fn classify_merge_head_is_merge() {
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join(".git");
        std::fs::create_dir_all(&git).unwrap();
        let p = git.join("MERGE_HEAD");
        std::fs::write(&p, "abc\n").unwrap();
        assert!(matches!(
            GitObserver::classify_change(dir.path(), &p),
            Some(GitEvent::Merge { .. })
        ));
    }

    #[test]
    fn classify_rebase_merge_is_rebase() {
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join(".git");
        let p = git.join("rebase-merge");
        std::fs::create_dir_all(&p).unwrap();
        let f = p.join("head-name");
        std::fs::write(&f, "refs/heads/x\n").unwrap();
        assert!(matches!(
            GitObserver::classify_change(dir.path(), &f),
            Some(GitEvent::Rebase { .. })
        ));
    }

    #[test]
    fn classify_stash_ref_is_merge() {
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join(".git");
        let p = git.join("refs/stash");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "abc\n").unwrap();
        assert!(matches!(
            GitObserver::classify_change(dir.path(), &p),
            Some(GitEvent::Merge { .. })
        ));
    }

    #[test]
    fn classify_unrelated_path_is_none() {
        let dir = tempfile::tempdir().unwrap();
        let git = dir.path().join(".git");
        std::fs::create_dir_all(&git).unwrap();
        let p = git.join("some-other-file");
        std::fs::write(&p, "x").unwrap();
        assert!(GitObserver::classify_change(dir.path(), &p).is_none());
    }

    #[test]
    fn classify_path_outside_git_dir_is_none() {
        let dir = tempfile::tempdir().unwrap();
        let outside = dir.path().join("README.md");
        std::fs::write(&outside, "hi").unwrap();
        assert!(GitObserver::classify_change(dir.path(), Path::new(&outside)).is_none());
    }

    #[test]
    fn git_event_serializes_with_variant_name() {
        let json = serde_json::to_string(&GitEvent::Checkout {
            branch: "main".into(),
        })
        .unwrap();
        assert!(json.contains("Checkout"));
        assert!(json.contains("main"));
    }

    #[test]
    fn git_event_clone_is_independent() {
        let e = GitEvent::RefChanged {
            ref_name: "refs/heads/x".into(),
            old_oid: None,
            new_oid: "abc".into(),
        };
        let c = e.clone();
        match c {
            GitEvent::RefChanged { new_oid, .. } => assert_eq!(new_oid, "abc"),
            other => panic!("unexpected {other:?}"),
        }
    }
}
