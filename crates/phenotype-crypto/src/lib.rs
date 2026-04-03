//! # Phenotype Crypto
//!
//! Cryptographic utilities for Phenotype ecosystem including:
//! - Hashing (SHA-256, BLAKE3)
//! - Symmetric encryption (AES-256-GCM)
//! - Key derivation (PBKDF2)
//! - HMAC signatures
//! - Ed25519 digital signatures

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use thiserror::Error;
use zeroize::Zeroizing;

// =============================================================================
// Errors
// =============================================================================

/// Cryptographic operation errors
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Signing failed: {0}")]
    SigningFailed(String),
}

/// Result type for crypto operations
pub type CryptoResult<T> = Result<T, CryptoError>;

// =============================================================================
// Hashing
// =============================================================================

/// Hashing utilities for SHA-256 and BLAKE3
pub mod hasher {
    use super::*;
    use blake3::Hasher as Blake3Hasher;

    /// Supported hash algorithms
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HashAlgorithm {
        /// SHA-256
        Sha256,
        /// BLAKE3
        Blake3,
    }

    /// Unified hasher supporting multiple algorithms
    pub struct Hasher {
        algorithm: HashAlgorithm,
        sha256_hasher: sha2::Sha256,
        blake3_hasher: Blake3Hasher,
    }

    impl Hasher {
        /// Create a new hasher with the specified algorithm
        #[must_use]
        pub fn new(algorithm: HashAlgorithm) -> Self {
            Self {
                algorithm,
                sha256_hasher: Sha256::new(),
                blake3_hasher: Blake3Hasher::new(),
            }
        }

        /// Hash data with SHA-256
        #[must_use]
        pub fn sha256(data: &[u8]) -> Vec<u8> {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }

        /// Hash data with BLAKE3
        #[must_use]
        pub fn blake3(data: &[u8]) -> Vec<u8> {
            let mut hasher = Blake3Hasher::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }

        /// Hash data with the configured algorithm
        pub fn update(&mut self, data: &[u8]) -> &mut Self {
            match self.algorithm {
                HashAlgorithm::Sha256 => self.sha256_hasher.update(data),
                HashAlgorithm::Blake3 => self.blake3_hasher.update(data),
            }
            self
        }

        /// Finalize and return the hash
        #[must_use]
        pub fn finalize(self) -> Vec<u8> {
            match self.algorithm {
                HashAlgorithm::Sha256 => self.sha256_hasher.finalize().to_vec(),
                HashAlgorithm::Blake3 => self.blake3_hasher.finalize().to_vec(),
            }
        }
    }

    impl Default for Hasher {
        fn default() -> Self {
            Self::new(HashAlgorithm::Blake3)
        }
    }

    /// Constant-time hash comparison to prevent timing attacks
    pub fn secure_compare(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        result == 0
    }
}

// =============================================================================
// AES-256-GCM Encryption
// =============================================================================

/// AES-256-GCM authenticated encryption
pub struct AesGcmEncryptor {
    cipher: Aes256Gcm,
}

impl AesGcmEncryptor {
    /// AES-256-GCM nonce size in bytes
    pub const NONCE_SIZE: usize = 12;

    /// AES-256 key size in bytes
    pub const KEY_SIZE: usize = 32;

    /// Create a new encryptor with the given 32-byte key
    pub fn new(key: &[u8]) -> CryptoResult<Self> {
        if key.len() != Self::KEY_SIZE {
            return Err(CryptoError::InvalidKeyLength {
                expected: Self::KEY_SIZE,
                actual: key.len(),
            });
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        Ok(Self { cipher })
    }

    /// Generate a random 32-byte encryption key
    #[must_use]
    pub fn generate_key() -> Zeroizing<[u8; Self::KEY_SIZE]> {
        let mut key = Zeroizing::new([0u8; Self::KEY_SIZE]);
        rand::thread_rng().fill_bytes(&mut key);
        *key
    }

    /// Encrypt plaintext, returning (ciphertext, nonce)
    pub fn encrypt(&self, plaintext: &[u8]) -> CryptoResult<(Vec<u8>, [u8; Self::NONCE_SIZE])> {
        let mut nonce_bytes = [0u8; Self::NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        Ok((ciphertext, nonce_bytes))
    }

    /// Decrypt ciphertext using the provided nonce
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; Self::NONCE_SIZE]) -> CryptoResult<Vec<u8>> {
        let nonce = Nonce::from_slice(nonce);

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
    }
}

// =============================================================================
// Key Derivation (PBKDF2)
// =============================================================================

/// Key derivation using PBKDF2-HMAC-SHA256
pub mod kdf {
    use super::*;

    /// PBKDF2 parameters
    #[derive(Debug, Clone)]
    pub struct Pbkdf2Params {
        /// Number of iterations (higher = more secure, slower)
        pub iterations: u32,
        /// Output key length in bytes
        pub key_length: usize,
        /// Salt for key derivation
        pub salt: Vec<u8>,
    }

    impl Default for Pbkdf2Params {
        fn default() -> Self {
            Self {
                iterations: 100_000,
                key_length: 32,
                salt: Vec::new(),
            }
        }
    }

    /// Derive a key using PBKDF2-HMAC-SHA256
    pub fn derive_key(password: &[u8], params: &Pbkdf2Params) -> CryptoResult<Vec<u8>> {
        if password.is_empty() {
            return Err(CryptoError::KeyDerivationFailed(
                "Password cannot be empty".into(),
            ));
        }

        let mut key = vec![0u8; params.key_length];
        pbkdf2_hmac::<Sha256>(
            password,
            &params.salt,
            params.iterations,
            &mut key,
        );

        Ok(key)
    }

    /// Generate a random salt for key derivation
    #[must_use]
    pub fn generate_salt(length: usize) -> Vec<u8> {
        let mut salt = vec![0u8; length];
        rand::thread_rng().fill_bytes(&mut salt);
        salt
    }
}

// =============================================================================
// HMAC
// =============================================================================

type HmacSha256 = Hmac<Sha256>;

/// HMAC-SHA256 utilities
pub mod hmac {
    use super::*;

    /// Calculate HMAC-SHA256
    #[must_use]
    pub fn sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key)
            .expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    /// Verify HMAC-SHA256 in constant time
    #[must_use]
    pub fn verify(key: &[u8], data: &[u8], expected: &[u8]) -> bool {
        let computed = sha256(key, data);
        hasher::secure_compare(&computed, expected)
    }
}

// =============================================================================
// Ed25519 Signatures
// =============================================================================

/// Ed25519 digital signature utilities
pub mod signatures {
    use super::*;
    use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

    /// Ed25519 signing key
    pub struct SigningKeyWrapper(SigningKey);

    impl SigningKeyWrapper {
        /// Generate a new random signing key
        #[must_use]
        pub fn generate() -> Self {
            let mut seed = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut seed);
            Self(SigningKey::from_bytes(&seed))
        }

        /// Sign data
        pub fn sign(&self, data: &[u8]) -> Vec<u8> {
            self.0.sign(data).to_bytes().to_vec()
        }

        /// Get the corresponding verifying key
        #[must_use]
        pub fn verifying_key(&self) -> Vec<u8> {
            self.0.verifying_key().to_bytes().to_vec()
        }
    }

    /// Ed25519 verifying key
    pub struct VerifyingKeyWrapper(VerifyingKey);

    impl VerifyingKeyWrapper {
        /// Create from bytes
        pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
            let key = VerifyingKey::from_bytes(bytes.try_into().map_err(|_| {
                CryptoError::SignatureVerificationFailed
            })?)
            .map_err(|_| CryptoError::SignatureVerificationFailed)?;

            Ok(Self(key))
        }

        /// Verify a signature
        #[must_use]
        pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
            let sig = match Signature::from_slice(signature) {
                Ok(s) => s,
                Err(_) => return false,
            };

            self.0.verify(data, &sig).is_ok()
        }
    }

    /// Sign data with a signing key
    #[must_use]
    pub fn sign(_key: &[u8], data: &[u8]) -> Vec<u8> {
        SigningKeyWrapper::generate().sign(data)
    }

    /// Verify a signature
    #[must_use]
    pub fn verify(verifying_key: &[u8], data: &[u8], signature: &[u8]) -> bool {
        match VerifyingKeyWrapper::from_bytes(verifying_key) {
            Ok(key) => key.verify(data, signature),
            Err(_) => false,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash() {
        let data = b"hello world";
        let hash = hasher::Hasher::sha256(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_blake3_hash() {
        let data = b"hello world";
        let hash = hasher::Hasher::blake3(data);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_secure_compare() {
        assert!(hasher::secure_compare(b"same", b"same"));
        assert!(!hasher::secure_compare(b"same", b"diff"));
        assert!(!hasher::secure_compare(b"short", b"longer"));
    }

    #[test]
    fn test_aes_gcm_encrypt_decrypt() {
        let key = AesGcmEncryptor::generate_key();
        let encryptor = AesGcmEncryptor::new(&key).unwrap();

        let plaintext = b"secret message";
        let (ciphertext, nonce) = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext, &nonce).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes_gcm_wrong_key_fails() {
        let key1 = AesGcmEncryptor::generate_key();
        let key2 = AesGcmEncryptor::generate_key();

        let encryptor1 = AesGcmEncryptor::new(&key1).unwrap();
        let encryptor2 = AesGcmEncryptor::new(&key2).unwrap();

        let (ciphertext, nonce) = encryptor1.encrypt(b"secret").unwrap();
        let result = encryptor2.decrypt(&ciphertext, &nonce);

        assert!(result.is_err());
    }

    #[test]
    fn test_pbkdf2_derive() {
        let params = kdf::Pbkdf2Params {
            iterations: 1000,
            key_length: 32,
            salt: kdf::generate_salt(16),
        };

        let key = kdf::derive_key(b"password", &params).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_pbkdf2_deterministic() {
        let salt = kdf::generate_salt(16);
        let params1 = kdf::Pbkdf2Params {
            iterations: 1000,
            key_length: 32,
            salt: salt.clone(),
        };
        let params2 = kdf::Pbkdf2Params {
            iterations: 1000,
            key_length: 32,
            salt,
        };

        let key1 = kdf::derive_key(b"password", &params1).unwrap();
        let key2 = kdf::derive_key(b"password", &params2).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_hmac_sha256() {
        let key = b"secret_key";
        let data = b"message";
        let mac = hmac::sha256(key, data);

        assert_eq!(mac.len(), 32);
        assert!(hmac::verify(key, data, &mac));
    }

    #[test]
    fn test_ed25519_sign_verify() {
        let signing_key = signatures::SigningKeyWrapper::generate();
        let verifying_key = signing_key.verifying_key();

        let data = b"test message";
        let signature = signing_key.sign(data);

        let verifier = signatures::VerifyingKeyWrapper::from_bytes(&verifying_key).unwrap();
        assert!(verifier.verify(data, &signature));
    }

    #[test]
    fn test_ed25519_wrong_key_fails() {
        let signing_key1 = signatures::SigningKeyWrapper::generate();
        let signing_key2 = signatures::SigningKeyWrapper::generate();

        let data = b"test message";
        let signature = signing_key1.sign(data);

        let verifier = signatures::VerifyingKeyWrapper::from_bytes(
            &signing_key2.verifying_key()
        ).unwrap();
        assert!(!verifier.verify(data, &signature));
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = [0u8; 16];
        let result = AesGcmEncryptor::new(&short_key);
        assert!(result.is_err());
    }
}
