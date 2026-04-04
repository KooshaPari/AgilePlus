//! T120 – Graph query performance benchmark.
//!
//! Benchmarks the in-memory `GraphBackend` to measure the overhead of:
//! - Node creation (Feature, WorkPackage, Agent)
//! - Node lookup by ID
//! - Dependency / blocking-path traversal queries
//! - Bulk node creation (seeding 100 features)
//!
//! Note: The in-memory backend doesn't parse Cypher; it pattern-matches
//! query strings.  These benchmarks therefore measure the overhead of the
//! Rust-side dispatch layer rather than a real graph database.
//! A Neo4j-backed benchmark would be added in CI using the `neo4j` feature.

use agileplus_benchmarks::helpers::make_feature;
use agileplus_graph::{GraphStore, InMemoryGraphStore, Node, NodeType, Relationship, RelType};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use uuid::Uuid;

fn make_store() -> InMemoryGraphStore {
    InMemoryGraphStore::new()
}

async fn seed_features(store: &InMemoryGraphStore, n: u64) {
    for i in 1..=n {
        let f = make_feature(i as i64);
        let node = Node::new(
            NodeType::Feature,
            serde_json::json!({
                "id": f.id,
                "slug": f.slug,
                "state": format!("{:?}", f.state),
                "friendly_name": f.friendly_name,
            }),
        );
        store.upsert_node(&node).await.expect("create feature node");
    }
}

fn bench_create_feature_node(c: &mut Criterion) {
    c.bench_function("graph_create_feature_node", |b| {
        b.iter(|| {
            let store = make_store();
            let node = Node::new(
                NodeType::Feature,
                serde_json::json!({
                    "slug": "feat-bench",
                    "state": "Created",
                    "friendly_name": "Bench Feature",
                }),
            );
            let store = &store;
            let node = &node;
            futures::executor::block_on(async {
                store.upsert_node(node).await.expect("create");
            });
        });
    });
}

fn bench_get_feature_node(c: &mut Criterion) {
    let store = make_store();
    let feature_node = Node::new(
        NodeType::Feature,
        serde_json::json!({
            "id": 50_i64,
            "slug": "feat-50",
            "state": "Created",
            "friendly_name": "Feature 50",
        }),
    );
    let feature_id = feature_node.id;
    futures::executor::block_on(async {
        store.upsert_node(&feature_node).await.expect("seed");
    });

    c.bench_function("graph_get_feature_node", |b| {
        b.iter(|| {
            let store = &store;
            let feature_id = black_box(feature_id);
            futures::executor::block_on(async {
                let node = store.get_node(feature_id);
                black_box(node.map(|v| v.id));
            })
        });
    });
}

fn bench_seed_n_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_seed_features");

    for count in [10_u64, 50, 100] {
        group.bench_with_input(BenchmarkId::new("seed_features", count), &count, |b, &n| {
            b.iter(|| {
                let store = make_store();
                futures::executor::block_on(seed_features(&store, black_box(n)));
            });
        });
    }

    group.finish();
}

fn bench_create_relationships(c: &mut Criterion) {
    let store = make_store();
    futures::executor::block_on(async {
        for i in 1..=10_u64 {
            let feature_node = Node::new(
                NodeType::Feature,
                serde_json::json!({"id": i, "slug": format!("f-{i}"), "state": "Created", "friendly_name": format!("F{i}")}),
            );
            let wp_node = Node::new(
                NodeType::WorkPackage,
                serde_json::json!({"id": i, "title": format!("WP-{i}"), "state": "todo", "ordinal": i as i32}),
            );
            store.upsert_node(&feature_node).await.unwrap();
            store.upsert_node(&wp_node).await.unwrap();
        }
    });

    c.bench_function("graph_create_owns_relationship", |b| {
        b.iter(|| {
            let store = &store;
            let nodes = futures::executor::block_on(async {
                store.get_nodes_by_type(NodeType::Feature)
            });
            if let Some(feature) = nodes.first() {
                let wps = futures::executor::block_on(async {
                    store.get_nodes_by_type(NodeType::WorkPackage)
                });
                if let Some(wp) = wps.first() {
                    let rel = Relationship::new(feature.id, wp.id, RelType::Owns);
                    futures::executor::block_on(async {
                        store.create_relationship(&rel).await.expect("owns");
                    });
                }
            }
        });
    });
}

fn bench_dependency_chain_query(c: &mut Criterion) {
    let store = make_store();
    futures::executor::block_on(seed_features(&store, 100));

    c.bench_function("graph_dependency_chain_query", |b| {
        b.iter(|| {
            let store = &store;
            let nodes = futures::executor::block_on(async {
                store.get_nodes_by_type(NodeType::Feature)
            });
            if let Some(first_feature) = nodes.first() {
                let id = black_box(first_feature.id);
                futures::executor::block_on(async {
                    let chain = store.get_dependencies(id).await.expect("query");
                    black_box(chain.len())
                })
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_and_get_feature_smoke() {
        let store = make_store();
        let node = Node::new(
            NodeType::Feature,
            serde_json::json!({"slug": "smoke", "state": "Created", "friendly_name": "Smoke"}),
        );
        store.upsert_node(&node).await.unwrap();
        let retrieved = store.get_node(node.id);
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn seed_100_features_smoke() {
        let store = make_store();
        seed_features(&store, 100).await;
        let nodes = store.get_nodes_by_type(NodeType::Feature);
        assert_eq!(nodes.len(), 100);
    }

    #[tokio::test]
    async fn dependency_chain_empty_smoke() {
        let store = make_store();
        seed_features(&store, 10).await;
        let nodes = store.get_nodes_by_type(NodeType::Feature);
        if let Some(first) = nodes.first() {
            let chain = store.get_dependencies(first.id).await.unwrap();
            assert!(chain.is_empty());
        }
    }

    #[tokio::test]
    async fn relationship_create_smoke() {
        let store = make_store();
        let feature = Node::new(
            NodeType::Feature,
            serde_json::json!({"id": 1, "slug": "f1", "state": "Created", "friendly_name": "F1"}),
        );
        let wp = Node::new(
            NodeType::WorkPackage,
            serde_json::json!({"id": 1, "title": "WP1", "state": "todo", "ordinal": 1}),
        );
        store.upsert_node(&feature).await.unwrap();
        store.upsert_node(&wp).await.unwrap();
        let rel = Relationship::new(feature.id, wp.id, RelType::Owns);
        store.create_relationship(&rel).await.unwrap();
        let deps = store.get_dependencies(feature.id).await.unwrap();
        assert_eq!(deps.len(), 1);
    }
}
