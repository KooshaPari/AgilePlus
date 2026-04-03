# ADR-002: Git Integration Strategy

**Date**: 2026-04-02  
**Status**: Accepted  
**Deciders**: Agent  

## Context

AgilePlus needs tight Git integration to correlate work items with code changes. This provides traceability from feature specifications to implementation commits.

## Decision Drivers

- **Non-intrusive**: Must not disrupt normal git workflow
- **Reliable**: Work even with messy commit messages
- **Performance**: Fast correlation without scanning entire history
- **Portable**: Work across GitHub, GitLab, Bitbucket, etc.
- **Offline**: No dependency on GitHub API

## Options Considered

### Option A: Commit Message Parsing

Pattern: "AGILE-123: Fix login bug"

**Pros**:
- Simple
- Works everywhere
- Human-readable

**Cons**:
- Requires discipline
- Regex matching can be brittle

**Decision**: Primary mechanism

### Option B: Git Notes

```bash
git notes add -m "Feature: AGILE-123" <commit-sha>
```

**Pros**:
- Non-intrusive to commit messages
- Can add metadata without changing history

**Cons**:
- Obscure feature
- Requires `git notes fetch` to sync
- Not visible in most UIs

**Decision**: Supplementary metadata storage

### Option C: Branch Naming

Pattern: `feature/AGILE-123-login-fix`

**Pros**:
- Visual in GitHub/GitLab
- Easy to filter

**Cons**:
- Branch noise
- Single feature per branch (not always true)
- Requires rebase discipline

**Decision**: Optional correlation

### Option D: Pre-commit Hooks

```bash
# .git/hooks/pre-commit
if ! grep -qE "AGILE-[0-9]+" "$1"; then
    echo "Warning: No feature ID in commit message"
fi
```

**Pros**:
- Automatic
- Enforces discipline

**Cons**:
- Setup friction
- Can be bypassed
- Annoying for quick commits

**Decision**: Optional enforcement

## Decision

**Multi-layer Git integration**:

1. **Primary**: Commit message parsing (flexible regex)
2. **Secondary**: Git notes for additional metadata
3. **Tertiary**: Branch naming conventions
4. **Optional**: Pre-commit hooks for enforcement

### Implementation

```rust
use regex::Regex;
use git2::{Repository, Commit};
use std::collections::HashMap;

pub struct GitIntegration {
    repo: Repository,
    feature_pattern: Regex,
    work_package_pattern: Regex,
}

impl GitIntegration {
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repo = Repository::open(repo_path)?;
        
        // Pattern matches: AGILE-123, #123, [AGILE-123], etc.
        let feature_pattern = Regex::new(
            r"(?:^|\s)(?:AGILE|agile|#)-?(\d+)(?:\s|$|:)"
        )?;
        
        let work_package_pattern = Regex::new(
            r"(?:^|\s)(?:WP|wp|work-?package)-?(\d+)(?:\s|$|:)"
        )?;
        
        Ok(Self {
            repo,
            feature_pattern,
            work_package_pattern,
        })
    }
    
    pub fn scan_commits(&self, since: Option<Time>) -> Result<Vec<CommitCorrelation>> {
        let mut walk = self.repo.revwalk()?;
        walk.push_head()?;
        
        if let Some(time) = since {
            // Filter by time
        }
        
        let mut correlations = Vec::new();
        
        for oid in walk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            let message = commit.message().unwrap_or("");
            
            // Extract feature IDs
            for cap in self.feature_pattern.captures_iter(message) {
                let feature_id = format!("AGILE-{}", &cap[1]);
                correlations.push(CommitCorrelation {
                    commit_hash: oid.to_string(),
                    feature_id,
                    work_package_id: None,
                    commit_message: message.to_string(),
                    commit_author: commit.author().name().unwrap_or("").to_string(),
                    commit_date: commit.time().seconds(),
                });
            }
        }
        
        Ok(correlations)
    }
    
    pub fn add_note(&self, commit_sha: &str, note: &str) -> Result<()> {
        let oid = self.repo.find_commit(
            &Oid::from_str(commit_sha)?
        )?.id();
        
        let sig = self.repo.signature()?;
        self.repo.note(
            &sig, &sig, None,
            oid, note, false
        )?;
        
        Ok(())
    }
    
    pub fn get_correlated_commits(&self, feature_id: &str) -> Result<Vec<Commit>> {
        // Query database for cached correlations
        // Return commits
        todo!()
    }
}
```

### CLI Commands

```bash
# Scan and correlate
agileplus git scan
agileplus git scan --since=2024-01-01

# Show correlations
agileplus git show AGILE-123

# Add manual correlation
agileplus git link AGILE-123 abc123def

# Sync notes
agileplus git sync-notes
```

## Consequences

### Positive
- **Flexible**: Multiple correlation methods
- **Non-intrusive**: Works with existing workflows
- **Fast**: Local git operations, no API calls
- **Portable**: Works with any git host

### Negative
- **Discipline required**: Best results with consistent conventions
- **Messy history**: Merge commits, revert commits can confuse
- **Sync**: Notes require explicit sync

## Configuration

```toml
[git]
# Patterns for feature ID extraction
feature_patterns = [
    "AGILE-{id}",
    "#{id}",
    "[AGILE-{id}]"
]

# Enable pre-commit hook installation
install_hooks = true

# Correlation strictness
strictness = "warn"  # "warn", "error", or "off"

# Auto-scan on sync
auto_scan = true
```

## References

- git2 crate: https://github.com/rust-lang/git2-rs
- Git notes: https://git-scm.com/docs/git-notes
- AgilePlus SOTA Research: `docs/research/AGILE_TOOLS_SOTA.md`

---

*This ADR will be updated as implementation progresses*
