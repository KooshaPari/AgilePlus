pub mod graph_store;
pub mod types;

pub use graph_store::{GraphError, GraphStore, InMemoryGraphStore};
pub use types::{Node, NodeType, RelType, Relationship};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Graph error: {0}")]
    Graph(#[from] GraphError),
    #[error("Config error: {0}")]
    Config(String),
}
