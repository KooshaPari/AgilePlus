// SPDX-License-Identifier: MIT OR Apache-2.0
//! Use-case tests for the application layer.
//!
//! One module per use case (creation and lifecycle of the same aggregate are
//! kept together), plus [`wp_graph`] for the topology primitives that back
//! the triage `topology` use case.
//!
//! All modules drive the in-memory doubles in [`crate::test_mocks`].

mod epic;
mod feature;
mod feature_lifecycle;
mod persist_synced_stories;
mod story;
mod story_lifecycle;
mod triage;
mod wp_graph;
