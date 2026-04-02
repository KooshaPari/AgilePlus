//! AES-GCM cipher implementation

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use rand::RngCore;

use super::error::{CipherError, CipherResult};

/// Cipher trait for symmetric encryption
pub trait Cipher: Send + Sync {
    fn encrypt(&self, plaintext: &[u8]) -> CipherResult<Vec<u8>>;
    fn decrypt(&self, ciphertext: &[u8]) -> CipherResult<Vec<u8>>;
}

/// AES-256-GCM cipher implementation
#[derive(Clone)]
pub struct AesGcmCipher {
    key: [u8; 32],
}

impl AesGcmCipher {
    /// Create from a 32-byte key
    pub fn from_key(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Generate a new random key
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Create from a password using key derivation
    pub fn from_password(password: &str, salt: &[u8]) -> CipherResult<Self> {
        let key = super::key_derivation::derive_key_with_salt(password, salt, b"AgilePlusCipher")?;
        Ok(Self { key })
    }
}

impl Cipher for AesGcmCipher {
    fn encrypt(&self, plaintext: &[u8]) -> CipherResult<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| CipherError::EncryptionFailed(e.to_string()))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CipherError::EncryptionFailed(e.to_string()))?;

        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> CipherResult<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(CipherError::InvalidCiphertext);
        }

        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| CipherError::DecryptionFailed(e.to_string()))?;

        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let ciphertext = &ciphertext[12..];

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CipherError::DecryptionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = AesGcmCipher::generate_key();
        let cipher = AesGcmCipher::from_key(key);

        let plaintext = b"secret API key: sk-12345";
        let ciphertext = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn encrypt_produces_different_output() {
        let key = AesGcmCipher::generate_key();
        let cipher = AesGcmCipher::from_key(key);

        let plaintext = b"same message";
        let c1 = cipher.encrypt(plaintext).unwrap();
        let c2 = cipher.encrypt(plaintext).unwrap();

        assert_ne!(c1, c2); // Different nonces
    }

    #[test]
    fn invalid_ciphertext_too_short() {
        let key = AesGcmCipher::generate_key();
        let cipher = AesGcmCipher::from_key(key);
        let res = cipher.decrypt(&[0u8; 5]);
        assert!(matches!(res, Err(CipherError::InvalidCiphertext)));
    }

    #[test]
    fn decrypt_with_wrong_key() {
        let c1 = AesGcmCipher::from_key([1u8; 32]);
        let c2 = AesGcmCipher::from_key([2u8; 32]);
        let plaintext = b"secret";
        let ciphertext = c1.encrypt(plaintext).unwrap();
        let res = c2.decrypt(&ciphertext);
        assert!(res.is_err());
    }
}
