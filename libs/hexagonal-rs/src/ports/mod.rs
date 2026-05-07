//! Port traits for hexagonal architecture

pub mod input;
pub mod output;

pub use input::InputPort;
pub use output::{OutputPort, SpecRepository, TaskRepository};
