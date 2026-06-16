//! agileplus-cache — caching layer (stub; full implementation pending)

/// Placeholder cache error type
#[derive(Debug)]
pub struct CacheError(pub String);

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CacheError: {}", self.0)
    }
}

impl std::error::Error for CacheError {}
