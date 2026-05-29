//! agileplus-api — axum HTTP server library.

pub mod api_key;
pub mod error;
pub mod middleware;
pub mod openapi;
pub mod responses;
pub mod router;
pub mod routes;
pub mod state;

pub use router::create_router;
pub use state::AppState;
