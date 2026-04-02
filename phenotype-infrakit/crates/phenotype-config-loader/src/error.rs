//! Configuration loader errors

use thiserror::Error;
use phenotype_config_core::ConfigError;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("core config error: {0}")]
    Core(#[from] ConfigError),
    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, LoaderError>;
