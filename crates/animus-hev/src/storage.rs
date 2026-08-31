//! SQLite Persistent Storage for Encrypted Secrets.

use crate::vault::{HevCrypto, VaultError, VaultKey};
use parking_lot::RwLock;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;

pub struct HevStorage {
    conn: Arc<RwLock<Connection>>,
    active_key: Arc<RwLock<Option<VaultKey>>>,
}

impl HevStorage {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, VaultError> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS hev_secrets (
                key TEXT PRIMARY KEY,
                ciphertext BLOB NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(RwLock::new(conn)),
            active_key: Arc::new(RwLock::new(None)),
        })
    }

    pub fn unlock(&self, key: VaultKey) {
        *self.active_key.write() = Some(key);
    }

    pub fn lock(&self) {
        *self.active_key.write() = None;
    }

    pub fn is_unlocked(&self) -> bool {
        self.active_key.read().is_some()
    }

    pub fn store(&self, secret_key: &str, secret_value: &[u8]) -> Result<(), VaultError> {
        let key_guard = self.active_key.read();
        let key = key_guard.as_ref().ok_or(VaultError::Locked)?;

        let ciphertext = HevCrypto::encrypt(key, secret_value)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let conn = self.conn.write();
        conn.execute(
            "INSERT OR REPLACE INTO hev_secrets (key, ciphertext, updated_at) VALUES (?1, ?2, ?3)",
            params![secret_key, ciphertext, now],
        )?;

        Ok(())
    }

    pub fn retrieve(&self, secret_key: &str) -> Result<Vec<u8>, VaultError> {
        let key_guard = self.active_key.read();
        let key = key_guard.as_ref().ok_or(VaultError::Locked)?;

        let conn = self.conn.read();
        let mut stmt = conn.prepare("SELECT ciphertext FROM hev_secrets WHERE key = ?1")?;
        let ciphertext: Vec<u8> = stmt.query_row(params![secret_key], |row| row.get(0))?;

        HevCrypto::decrypt(key, &ciphertext)
    }

    pub fn delete(&self, secret_key: &str) -> Result<bool, VaultError> {
        let conn = self.conn.write();
        let rows = conn.execute("DELETE FROM hev_secrets WHERE key = ?1", params![secret_key])?;
        Ok(rows > 0)
    }
}
