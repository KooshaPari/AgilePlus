//! T120 – Graph query performance benchmark.
//!
//! Benchmarks the in-memory `InMemoryGraphStore` to measure the overhead of:
//! - Node creation (Feature, WorkPackage, Agent)
//! - Node lookup by ID
//! - Dependency / blocking-path traversal queries
//! - Bulk node creation (seeding 100 features)
//!
//! Note: The in-memory backend doesn't parse Cypher; it pattern-matches
//! query strings.  These benchmarks therefore measure the overhead of the
//! Rust-side dispatch layer rather than a real graph database.
//! A Neo4j-backed benchmark would be added in CI using the `neo4j` feature.

use agileplus_graph::{GraphStore, InMemoryGraphStore, Node, NodeType, Relationship, RelType};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_store() -> InMemoryGraphStore {
    InMemoryGraphStore::new()
}

fn make_feature_node(i: u32) -> Node {
    Node::feature(
        format!("feature-{}", i),
        "Created".to_string(),
        format!("Feature {}", i),
    )
}

fn make_workpackage_node(i: u32) -> Node {
    Node::workpackage(
        format!("WP-{}", i),
        "todo".to_string(),
        i as i32,
    )
}

/// Seed N feature nodes into the store and return their UUIDs.
async fn seed_features(store: &InMemoryGraphStore, n: u32) -> Vec<Uuid> {
    let mut ids = Vec::with_capacity(n as usize);
    for i in 1..=n {
        let node = make_feature_node(i);
        ids.push(node.id);
        store.upsert_node(&node).await.expect("create feature node");
    }
    ids
}

// ---------------------------------------------------------------------------
// Benchmark: create a single feature node
// ---------------------------------------------------------------------------

fn bench_create_feature_node(c: &mut Criterion) {
    c.bench_function("graph_create_feature_node", |b| {
        b.iter(|| {
            let store = make_store();
            let node = Node::feature(
                black_box("feat-bench".to_string()),
                black_box("Created".to_string()),
                black_box("Bench Feature".to_string()),
            );
            let store = &store;
            let node = &node;
            futures::executor::block_on(async {
                store.upsert_node(node).await.expect("create");
            });
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: get feature node by ID
// ---------------------------------------------------------------------------

fn bench_get_feature_node(c: &mut Criterion) {
    let store = make_store();
    let feature_ids = futures::executor::block_on(seed_features(&store, 100));
    let target_id = feature_ids[49]; // 50th feature (0-indexed)

    c.bench_function("graph_get_feature_node", |b| {
        b.iter(|| {
            let store = &store;
            let target_id = target_id;
            futures::executor::block_on(async {
                let f = store.get_node(black_box(target_id)).await.expect("get");
                black_box(f.map(|v| v.id))
            })
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: seed N nodes (measures bulk-creation throughput)
// ---------------------------------------------------------------------------

fn bench_seed_n_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_seed_features");

    for count in [10_u32, 50, 100] {
        group.bench_with_input(BenchmarkId::new("seed_features", count), &count, |b, &n| {
            b.iter(|| {
                let store = make_store();
                futures::executor::block_on(seed_features(black_box(&store), black_box(n)));
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: relationship creation
// ---------------------------------------------------------------------------

fn bench_create_relationships(c: &mut Criterion) {
    let store = make_store();
    futures::executor::block_on(async {
        for i in 1..=10_u32 {
            let feature = make_feature_node(i);
            let wp = make_workpackage_node(i);
            store.upsert_node(&feature).await.unwrap();
            store.upsert_node(&wp).await.unwrap();
        }
    });

    c.bench_function("graph_create_owns_relationship", |b| {
        b.iter(|| {
            let store = &store;
            let feature_ids = futures::executor::block_on(seed_features(&make_store(), 10));
            let wp_ids = futures::executor::block_on(async {
                let mut ids = Vec::new();
                for i in 1..=10_u32 {
                    let wp = make_workpackage_node(i);
                    ids.push(wp.id);
                    store.upsert_node(&wp).await.unwrap();
                }
                ids
            });
            let rel = Relationship::new(
                black_box(feature_ids[0]),
                black_box(wp_ids[0]),
                RelType::Owns,
            );
            futures::executor::block_on(async {
                store.create_relationship(&rel).await.expect("owns");
            });
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark: dependency-chain traversal query
// ---------------------------------------------------------------------------

fn bench_dependency_chain_query(c: &mut Criterion) {
    let store = make_store();
    futures::executor::block_on(seed_features(&store, 100));
    let target_id = {
        let nodes = futures::executor::block_on(store.get_nodes_by_type(NodeType::Feature)).unwrap();
        nodes[0].id
    };

    c.bench_function("graph_dependency_chain_query", |b| {
        b.iter(|| {
            let store = &store;
            let target_id = target_id;
            futures::executor::block_on(async {
                let chain = store.get_dependencies(black_box(target_id)).await.expect("query");
                black_box(chain.len())
            })
        });
    });
}

criterion_group!(
    benches,
    bench_create_feature_node,
    bench_get_feature_node,
    bench_seed_n_features,
    bench_create_relationships,
    bench_dependency_chain_query,
);
criterion_main!(benches);

// ---------------------------------------------------------------------------
// Smoke tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::{make_store, make_feature_node, seed_features};
    use agileplus_graph::{GraphStore, InMemoryGraphStore, Node, NodeType, Relationship, RelType};

    #[tokio::test]
    async fn create_and_get_feature_smoke() {
        let store = make_store();
        let node = Node::feature("smoke".into(), "Created".into(), "Smoke".into());
        store.upsert_node(&node).await.unwrap();
        let f = store.get_node(node.id).await.unwrap();
        assert!(f.is_some());
    }

    #[tokio::test]
    async fn seed_100_features_smoke() {
        let store = make_store();
        let ids = seed_features(&store, 100).await;
        assert_eq!(ids.len(), 100);
        let f = store.get_node(ids[49]).await.unwrap();
        assert!(f.is_some());
    }

    #[tokio::test]
    async fn dependency_chain_empty_smoke() {
        let store = make_store();
        seed_features(&store, 10).await;
        let nodes = store.get_nodes_by_type(NodeType::Feature).await.unwrap();
        let chain = store.get_dependencies(nodes[0].id).await.unwrap();
        assert!(chain.is_empty());
    }

    #[tokio::test]
    async fn relationship_create_smoke() {
        let store = make_store();
        let feature = Node::feature("f1".into(), "Created".into(), "F1".into());
        let wp = Node::workpackage("WP1".into(), "todo".into(), 1);
        store.upsert_node(&feature).await.unwrap();
        store.upsert_node(&wp).await.unwrap();
        let rel = Relationship::new(feature.id, wp.id, RelType::Owns);
        store.create_relationship(&rel).await.unwrap();
    }
}
