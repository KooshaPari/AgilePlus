//! Hash functions

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Hash trait
pub trait Hash: Send + Sync {
    fn hash(&self, data: &[u8]) -> Vec<u8>;
}

/// SHA-256 hash wrapper
#[derive(Debug, Clone, Default)]
pub struct Sha256Hash;

impl Sha256Hash {
    pub fn new() -> Self {
        Self
    }
}

impl Hash for Sha256Hash {
    fn hash(&self, data: &[u8]) -> Vec<u8> {
        sha256_hash(data)
    }
}

/// Compute SHA-256 hash
pub fn sha256_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute SHA-256 hash as hex string
pub fn sha256_hex(data: &[u8]) -> String {
    hex::encode(sha256_hash(data))
}

/// Compute HMAC-SHA256
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC takes key size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// Hash output structure for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashOutput {
    pub algorithm: String,
    pub hash: String,
    pub salt: Option<String>,
}

impl HashOutput {
    pub fn new_sha256(data: &[u8], salt: Option<String>) -> Self {
        let mut hasher = Sha256::new();
        if let Some(ref s) = salt {
            hasher.update(s.as_bytes());
        }
        hasher.update(data);
        let hash = hex::encode(hasher.finalize());
        Self {
            algorithm: "SHA-256".to_string(),
            hash,
            salt,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hash_produces_32_bytes() {
        let hash = sha256_hash(b"test");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn sha256_hex_produces_64_chars() {
        let hex = sha256_hex(b"test");
        assert_eq!(hex.len(), 64);
    }

    #[test]
    fn hmac_sha256_produces_32_bytes() {
        let hmac = hmac_sha256(b"key", b"message");
        assert_eq!(hmac.len(), 32);
    }

    #[test]
    fn hash_output_new() {
        let output = HashOutput::new_sha256(b"password", Some("salt".to_string()));
        assert_eq!(output.algorithm, "SHA-256");
        assert_eq!(output.hash.len(), 64);
        assert_eq!(output.salt, Some("salt".to_string()));
    }
}
