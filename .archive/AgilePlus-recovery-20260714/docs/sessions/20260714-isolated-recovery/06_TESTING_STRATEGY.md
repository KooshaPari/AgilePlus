# Testing Strategy

## Recovery Gates

1. `git fsck --full --strict`
2. `git remote get-url origin` matches the canonical SSH remote.
3. Recovery documentation is the only delta from the live baseline.
4. Recovery branch descends from the captured live `origin/main` commit.
5. Worktree status is clean after committing the recovery documentation.
6. Evidence archive checksums verify with `shasum -a 256 -c SHA256SUMS`.

Product builds and tests are intentionally out of scope for the boundary-establishment step.
