# Known issues and limits

- Rust workspace checks are unavailable in this sparse worktree because the root `Cargo.toml`
  is skip-worktree and absent. Running against another checkout would violate isolation.
- No cargo/rustc/clippy process was active during inspection; free space was above the 7 GiB stop threshold.
- Parent must perform spec review, code review, and the required Airlock snapshot.
- Hosted CI remains the authoritative proof that `protoc` installation resolves the Rust
  coverage build; local validation is limited to workflow syntax and diff hygiene.
