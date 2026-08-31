//! Military-Grade HEV Vault Cryptography: Argon2id + AES-256-GCM with Zero-on-Drop.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),
    #[error("Encryption failed")]
    Encryption,
    #[error("Decryption failed / authentication tag mismatch")]
    Decryption,
    #[error("Vault is locked")]
    Locked,
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

/// Sensitive master key buffer that automatically zeroes itself on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct VaultKey(pub [u8; 32]);

pub struct HevCrypto;

impl HevCrypto {
    /// Derives a 256-bit key from a passphrase and 16-byte salt using Argon2id.
    /// Memory: 64MB, Iterations: 3, Parallelism: 4 threads.
    pub fn derive_key(passphrase: &[u8], salt: &[u8; 16]) -> Result<VaultKey, VaultError> {
        let params = Params::new(64 * 1024, 3, 4, Some(32))
            .map_err(|e| VaultError::KeyDerivation(e.to_string()))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut key_bytes = [0u8; 32];
        argon2
            .hash_password_into(passphrase, salt, &mut key_bytes)
            .map_err(|e| VaultError::KeyDerivation(e.to_string()))?;

        Ok(VaultKey(key_bytes))
    }

    /// Encrypts plaintext using AES-256-GCM with a 96-bit random nonce.
    /// Output format: [12-byte Nonce] + [Ciphertext + 16-byte Auth Tag].
    pub fn encrypt(key: &VaultKey, plaintext: &[u8]) -> Result<Vec<u8>, VaultError> {
        let cipher_key = Key::<Aes256Gcm>::from_slice(&key.0);
        let cipher = Aes256Gcm::new(cipher_key);

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| VaultError::Encryption)?;

        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypts AES-256-GCM payload with authentication tag validation.
    pub fn decrypt(key: &VaultKey, payload: &[u8]) -> Result<Vec<u8>, VaultError> {
        if payload.len() < 12 + 16 {
            return Err(VaultError::Decryption);
        }

        let cipher_key = Key::<Aes256Gcm>::from_slice(&key.0);
        let cipher = Aes256Gcm::new(cipher_key);

        let nonce = Nonce::from_slice(&payload[..12]);
        let ciphertext = &payload[12..];

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| VaultError::Decryption)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_encryption_decryption() {
        let salt = [0x42u8; 16];
        let passphrase = b"correct_horse_battery_staple";
        let key = HevCrypto::derive_key(passphrase, &salt).unwrap();

        let secret = b"super_secret_wifi_password_12345";
        let encrypted = HevCrypto::encrypt(&key, secret).unwrap();
        let decrypted = HevCrypto::decrypt(&key, &encrypted).unwrap();

        assert_eq!(decrypted, secret);

        // Verify invalid key fails decryption
        let wrong_key = HevCrypto::derive_key(b"wrong_pass", &salt).unwrap();
        assert!(HevCrypto::decrypt(&wrong_key, &encrypted).is_err());
    }
}
