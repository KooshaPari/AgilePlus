pub mod types;
pub mod graph_store;

pub use types::{NodeType, RelType, Node, Relationship};
pub use graph_store::{GraphStore, InMemoryGraphStore, GraphError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Graph error: {0}")]
    Graph(#[from] GraphError),
    #[error("Config error: {0}")]
    Config(String),
}

