//! In-memory adapter implementations

pub mod spec_repository;
pub mod task_repository;

pub use spec_repository::InMemorySpecRepository;
pub use task_repository::InMemoryTaskRepository;
