# Known Issues

| Issue | Impact | Disposition |
|---|---|---|
| Case-colliding `.github` pull-request templates | Only one spelling is materialized on macOS | Preserve in Git; resolve in a reviewed follow-up |
| Dirty source has divergent local main and many worktrees | High risk of accidental loss during cleanup | Evidence archived; source remains read-only |
| Unreachable-object inventory is large | Manual review is required before any future GC | Full `.git` archive preserved with checksums |
| Recovery docs make the new branch differ from upstream | Expected recovery-only commit | Keep isolated and do not push without review |
