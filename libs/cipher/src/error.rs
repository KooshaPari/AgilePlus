//! Error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CipherError {
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("invalid ciphertext")]
    InvalidCiphertext,

    #[error("key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("hash verification failed")]
    HashMismatch,
}

pub type CipherResult<T> = Result<T, CipherError>;
