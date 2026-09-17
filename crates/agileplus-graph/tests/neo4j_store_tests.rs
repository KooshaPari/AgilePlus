// SPDX-License-Identifier: MIT OR Apache-2.0
//! Feature-gated tests for `Neo4jGraphStore`.
//!
//! These tests run only under `--features neo4j`. They exercise construction
//! and the failure path without requiring a live Neo4j server, which keeps them
//! hermetic while still validating the store's error mapping.

#![cfg(feature = "neo4j")]

use std::time::Duration;

use agileplus_graph::{GraphError, GraphStore, Neo4jGraphStore};

/// `neo4rs::Graph::new` is lazy, so constructing the store must not dial the
/// server or reject an unreachable address.
#[tokio::test]
async fn connect_is_lazy_and_succeeds_without_server() {
    let outcome = tokio::time::timeout(
        Duration::from_secs(10),
        Neo4jGraphStore::connect("bolt://127.0.0.1:1", "neo4j", "password"),
    )
    .await;

    match outcome {
        Ok(Ok(_store)) => {}
        Ok(Err(err)) => panic!("lazy connect should not fail eagerly, got {err:?}"),
        Err(_) => panic!("connect attempt did not resolve within 10s"),
    }
}

/// A health check against a port with no listener must never be reported as
/// healthy. `neo4rs` retries aggressively, so the driver may still be pending
/// when the probe window elapses; either an error or an unfinished attempt is
/// acceptable, a successful return is not.
#[tokio::test]
async fn health_check_without_server_never_reports_healthy() {
    let store = Neo4jGraphStore::connect("bolt://127.0.0.1:1", "neo4j", "password")
        .await
        .expect("lazy connect should succeed");

    match tokio::time::timeout(Duration::from_secs(3), store.health_check()).await {
        Ok(Ok(())) => panic!("dead server must not be reported healthy"),
        Ok(Err(GraphError::QueryError(msg))) | Ok(Err(GraphError::ConnectionError(msg))) => {
            assert!(!msg.is_empty(), "error should carry a diagnostic message");
        }
        Ok(Err(other)) => panic!("unexpected error variant: {other:?}"),
        Err(_elapsed) => {
            // Driver is still retrying against the unreachable endpoint.
        }
    }
}

/// The store type must satisfy the same object-safety contract as the
/// in-memory implementation.
#[test]
fn neo4j_store_is_object_safe() {
    fn assert_graph_store<T: GraphStore>() {}
    assert_graph_store::<Neo4jGraphStore>();
}
