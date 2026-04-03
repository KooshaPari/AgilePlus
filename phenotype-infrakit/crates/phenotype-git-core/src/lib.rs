//! Git utilities

pub mod discover;
pub mod worktree;

pub use discover::{DiscoverError, GitDiscover};
pub use worktree::{Worktree, WorktreeSpec};
