//! Cryptography utilities for AgilePlus
//!
//! Provides secure encryption, key derivation, and hashing for:
//! - Securing credentials (database passwords, API keys)
//! - Token generation and validation
//! - Data encryption at rest

pub mod cipher;
pub mod error;
pub mod hash;
pub mod key_derivation;

pub use cipher::{AesGcmCipher, Cipher};
pub use error::{CipherError, CipherResult};
pub use hash::{Hash, Sha256Hash, sha256_hash};
pub use key_derivation::{Argon2Config, derive_key, derive_key_with_salt};
