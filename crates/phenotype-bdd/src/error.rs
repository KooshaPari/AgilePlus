//! BDD error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BddError {
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("execution error: {0}")]
    ExecutionError(String),
    #[error("step not found: {0}")]
    StepNotFound(String),
}

pub type Result<T> = std::result::Result<T, BddError>;
