use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use tracing::debug;

use super::CrossCryptError;

pub const DEFAULT_SALT_SIZE: usize = 32;
pub const DEFAULT_KEY_SIZE: usize = 64; // 512 bits for XTS

pub struct KdfEngine;

impl KdfEngine {
    /// Derive key using Argon2id
    pub fn argon2id(
        password: &[u8],
        salt: &[u8],
        iterations: u32,
        memory_kb: u32,
        parallelism: u32,
    ) -> Result<Vec<u8>, CrossCryptError> {
        debug!(
            "Deriving key with Argon2id (t={}, m={}, p={})",
            iterations, memory_kb, parallelism
        );

        let params = Params::new(
            memory_kb,
            iterations,
            parallelism,
            Some(DEFAULT_KEY_SIZE),
        )
        .map_err(|e| CrossCryptError::Crypto(format!("Argon2id params failed: {:?}", e)))?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut key = vec![0u8; DEFAULT_KEY_SIZE];
        argon2
            .hash_password_into(password, salt, &mut key)
            .map_err(|e| CrossCryptError::Crypto(format!("Argon2id failed: {:?}", e)))?;

        Ok(key)
    }

    /// Generate a random salt
    pub fn generate_salt() -> Vec<u8> {
        let mut salt = vec![0u8; DEFAULT_SALT_SIZE];
        rand::thread_rng().fill_bytes(&mut salt);
        salt
    }
}

pub struct Argon2idParams {
    pub iterations: u32,
    pub memory_kb: u32,
    pub parallelism: u32,
}

impl Default for Argon2idParams {
    fn default() -> Self {
        Self {
            iterations: 3,
            memory_kb: 64 * 1024, // 64 MB
            parallelism: 4,
        }
    }
}

impl Argon2idParams {
    /// Conservative parameters for maximum security
    pub fn conservative() -> Self {
        Self {
            iterations: 4,
            memory_kb: 256 * 1024, // 256 MB
            parallelism: 4,
        }
    }

    /// Fast parameters for testing
    pub fn fast() -> Self {
        Self {
            iterations: 1,
            memory_kb: 8 * 1024, // 8 MB
            parallelism: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2id() {
        let salt = KdfEngine::generate_salt();
        let password = b"test_password";

        let key1 = KdfEngine::argon2id(password, &salt, 1, 8192, 1).unwrap();
        let key2 = KdfEngine::argon2id(password, &salt, 1, 8192, 1).unwrap();

        assert_eq!(key1, key2); // Same input should produce same output
        assert_eq!(key1.len(), DEFAULT_KEY_SIZE);
    }

    #[test]
    fn test_different_salts() {
        let salt1 = KdfEngine::generate_salt();
        let salt2 = KdfEngine::generate_salt();
        let password = b"test_password";

        let key1 = KdfEngine::argon2id(password, &salt1, 1, 8192, 1).unwrap();
        let key2 = KdfEngine::argon2id(password, &salt2, 1, 8192, 1).unwrap();

        assert_ne!(key1, key2); // Different salts should produce different keys
    }
}
