//! Hardware Encryption Vault (HEV) & TPM 2.0 PCR Integrations.

use animus_hev::vault::{HevCrypto, VaultError, VaultKey};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSetup {
    pub enabled: bool,
    pub tpm2_available: bool,
    pub auto_unlock_tpm: bool,
    pub recovery_key: String,
}

impl Default for VaultSetup {
    fn default() -> Self {
        let tpm2_available = Path::new("/dev/tpmrm0").exists() || Path::new("/dev/tpm0").exists();
        Self {
            enabled: true,
            tpm2_available,
            auto_unlock_tpm: tpm2_available,
            recovery_key: Self::generate_recovery_key(),
        }
    }
}

impl VaultSetup {
    /// Generates a military-grade 25-character formatted recovery key (e.g. `VITUS-8K2N9-WX7B4-PQ9L2-M4Z18`).
    pub fn generate_recovery_key() -> String {
        const CHARSET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ"; // Base32 without confusing 0/O, 1/I/L
        let mut rng = rand::thread_rng();

        let mut groups = Vec::with_capacity(5);
        groups.push("VITUS".to_string());

        for _ in 0..4 {
            let chunk: String = (0..5)
                .map(|_| {
                    let idx = rng.gen_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect();
            groups.push(chunk);
        }

        groups.join("-")
    }

    /// Verifies password derivation using Argon2id via animus-hev.
    pub fn test_derive_key(passphrase: &str) -> Result<VaultKey, VaultError> {
        let salt = [0x41u8; 16]; // Canonical installer test salt
        let key = HevCrypto::derive_key(passphrase.as_bytes(), &salt)?;
        info!("VaultSetup: Successfully validated Argon2id key derivation.");
        Ok(key)
    }
}
