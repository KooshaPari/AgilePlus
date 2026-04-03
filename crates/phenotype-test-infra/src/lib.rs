//! # Phenotype Test Infrastructure
//!
//! Testing utilities including BDD, fixtures, and assertions.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod bdd;
pub mod fixtures;
pub mod assertions;

// Re-export commonly used types
pub use bdd::TestContext;
pub use fixtures::Fixture;
pub use assertions::Assertion;
