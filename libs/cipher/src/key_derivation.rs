//! Key derivation functions

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use super::error::{CipherError, CipherResult};

/// Configuration for Argon2 key derivation
#[derive(Debug, Clone)]
pub struct Argon2Config {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

impl Default for Argon2Config {
    fn default() -> Self {
        Self {
            memory_kib: 65536, // 64 MiB
            iterations: 3,
            parallelism: 4,
        }
    }
}

/// Derive a key from a password using Argon2id
pub fn derive_key(password: &str, config: Argon2Config) -> CipherResult<[u8; 32]> {
    derive_key_with_salt_impl(password, None, config)
}

/// Derive a key with a specific salt
pub fn derive_key_with_salt(password: &str, salt: &[u8], domain: &[u8]) -> CipherResult<[u8; 32]> {
    let config = Argon2Config {
        memory_kib: 32768, // 32 MiB for faster derivation
        ..Default::default()
    };
    derive_key_with_salt_impl(password, Some((salt, domain)), config)
}

fn derive_key_with_salt_impl(
    password: &str,
    salt_info: Option<(&[u8], &[u8])>,
    config: Argon2Config,
) -> CipherResult<[u8; 32]> {
    let salt = match salt_info {
        Some((salt, _)) => SaltString::encode_b64(salt)
            .map_err(|e| CipherError::KeyDerivationFailed(e.to_string()))?,
        None => SaltString::generate(&mut OsRng),
    };

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(
            config.memory_kib,
            config.iterations,
            config.parallelism,
            Some(32),
        )
        .map_err(|e| CipherError::KeyDerivationFailed(e.to_string()))?,
    );

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| CipherError::KeyDerivationFailed(e.to_string()))?;

    let hash_str = hash
        .hash
        .ok_or_else(|| CipherError::KeyDerivationFailed("no hash output".into()))?;
    let hash_bytes = hash_str.as_bytes();

    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes[..32]);
    Ok(key)
}

/// Hash a password for storage (use for password verification, not key derivation)
pub fn hash_password(password: &str) -> CipherResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| CipherError::KeyDerivationFailed(e.to_string()))?;

    Ok(hash.to_string())
}

/// Verify a password against a stored hash
pub fn verify_password(password: &str, hash: &str) -> CipherResult<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| CipherError::KeyDerivationFailed(e.to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_key_produces_32_bytes() {
        let key = derive_key("password123", Argon2Config::default()).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn derive_key_with_salt_deterministic() {
        let salt = b"fixedsalt12345678901234567890";
        let domain = b"test";
        let key1 = derive_key_with_salt("password", salt, domain).unwrap();
        let key2 = derive_key_with_salt("password", salt, domain).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn password_hash_verify() {
        let hash = hash_password("mysecretpassword").unwrap();
        assert!(verify_password("mysecretpassword", &hash).unwrap());
        assert!(!verify_password("wrongpassword", &hash).unwrap());
    }
}
