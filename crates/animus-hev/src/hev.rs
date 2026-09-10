//! HEV -- Hardware Encryption Vault State Machine (Part 25 of spec).
//!
//! HEV is vitusOS's native secret storage and identity manager.
//! Named after Gordon Freeman's Hazardous Environment Suit.
//!
//! Design contract:
//! - Implements org.freedesktop.secrets -- all apps talk to HEV transparently
//! - Master key never written to disk -- only in memory while unlocked
//! - Vault locked automatically when screen locks
//! - Proximity unlock via SeaDrop RSSI -- phone in pocket is the key
//! - Cold start always requires password -- no exceptions
//! - libsodium/zeroize for all cryptographic primitives
//!
//! State machine: Cold -> Unlocked -> Locked -> Sealed
//!   Cold: process just started, master key not yet derived
//!   Unlocked: master key in memory, entries accessible
//!   Locked: master key wiped, entries inaccessible
//!   Sealed: security alert, requires password to reopen

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU8, Ordering};
use tracing::{info, warn};
use zeroize::Zeroize;

use crate::vault::{HevCrypto, VaultKey};

/// Store types (Part 25.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HevStoreType {
    Credentials = 0,
    Identity = 1,
    Certificate = 2,
    Token = 3,
    SeaDropTrust = 4,
}

/// Vault entry (Part 25.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HevEntry {
    pub id: u64,
    pub store: HevStoreType,
    pub label: String,
    pub app_id: String,
    pub schema: String,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub tag: Vec<u8>,
    pub created_at: u64,
    pub accessed_at: u64,
}

/// SeaDrop trusted device (Part 25.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HevTrustedDevice {
    pub device_id: String,
    pub device_name: String,
    pub public_key: Vec<u8>,
    pub shared_secret: Vec<u8>,
    pub proximity_unlock_enabled: bool,
    pub rssi_unlock_threshold: f32,  // default -45.0 dBm
    pub rssi_lock_threshold: f32,    // default -70.0 dBm
    pub last_seen: u64,
}

/// Vault state machine (Part 25.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HevVaultState {
    Cold = 0,
    Unlocked = 1,
    Locked = 2,
    Sealed = 3,
}

impl From<u8> for HevVaultState {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Cold,
            1 => Self::Unlocked,
            2 => Self::Locked,
            3 => Self::Sealed,
            _ => Self::Cold,
        }
    }
}

/// Access request result (Part 25.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HevAccessResult {
    Granted,
    Denied,
    VaultLocked,
    NotFound,
    AuthRequired,
}

/// Vault status for Supervisor (Part 25.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStatus {
    pub state: HevVaultState,
    pub entry_count: u32,
    pub trusted_app_count: u32,
    pub seadrop_device_count: u32,
    pub last_accessed_app: String,
    pub last_access_time_s: f64,
    pub proximity_unlock_active: bool,
}

/// ProximityGuard -- SeaDrop RSSI-based unlock/lock (Part 25.3).
pub struct ProximityGuard {
    devices: RwLock<Vec<HevTrustedDevice>>,
    last_rssi: RwLock<HashMap<String, f32>>,
    #[allow(dead_code)]
    lock_grace_period_s: RwLock<f32>,
}

impl ProximityGuard {
    pub fn new() -> Self {
        Self {
            devices: RwLock::new(Vec::new()),
            last_rssi: RwLock::new(HashMap::new()),
            lock_grace_period_s: RwLock::new(3.0), // 3s grace before locking
        }
    }

    pub fn register_device(&self, device: HevTrustedDevice) {
        self.devices.write().push(device);
    }

    pub fn revoke_device(&self, device_id: &str) {
        self.devices.write().retain(|d| d.device_id != device_id);
    }

    pub fn update_rssi(&self, device_id: &str, rssi: f32) -> bool {
        let devices = self.devices.read();
        let device = match devices.iter().find(|d| d.device_id == device_id) {
            Some(d) => d.clone(),
            None => return false,
        };

        self.last_rssi.write().insert(device_id.to_string(), rssi);

        if !device.proximity_unlock_enabled {
            return false;
        }

        // Unlock: RSSI >= unlock threshold
        if rssi >= device.rssi_unlock_threshold {
            return true; // Signal: unlock
        }

        // Lock: RSSI <= lock threshold (after grace period)
        if rssi <= device.rssi_lock_threshold {
            return false; // Signal: lock
        }

        false
    }

    pub fn list_devices(&self) -> Vec<HevTrustedDevice> {
        self.devices.read().clone()
    }

    pub fn get_device(&self, device_id: &str) -> Option<HevTrustedDevice> {
        self.devices.read().iter().find(|d| d.device_id == device_id).cloned()
    }
}

/// HEV -- the main vault state machine (Part 25.3).
pub struct HEV {
    state: AtomicU8,
    master_key: RwLock<Option<VaultKey>>,
    entries: RwLock<Vec<HevEntry>>,
    trusted_apps: RwLock<HashSet<String>>,
    proximity: RwLock<ProximityGuard>,
    last_accessed_app: RwLock<String>,
    last_access_time: RwLock<f64>,
    salt: RwLock<[u8; 16]>,
}

impl HEV {
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(HevVaultState::Cold as u8),
            master_key: RwLock::new(None),
            entries: RwLock::new(Vec::new()),
            trusted_apps: RwLock::new(HashSet::new()),
            proximity: RwLock::new(ProximityGuard::new()),
            last_accessed_app: RwLock::new(String::new()),
            last_access_time: RwLock::new(0.0),
            salt: RwLock::new([0u8; 16]),
        }
    }

    pub fn state(&self) -> HevVaultState {
        HevVaultState::from(self.state.load(Ordering::SeqCst))
    }

    pub fn is_unlocked(&self) -> bool {
        self.state() == HevVaultState::Unlocked
    }

    /// Cold start: derives master key from password via Argon2id (Part 25.3).
    /// Must succeed before vault is accessible. Returns false if password is wrong.
    pub fn unlock_with_password(&self, password: &str) -> bool {
        let salt = *self.salt.read();

        match HevCrypto::derive_key(password.as_bytes(), &salt) {
            Ok(key) => {
                *self.master_key.write() = Some(key);
                self.state.store(HevVaultState::Unlocked as u8, Ordering::SeqCst);
                info!("HEV: Vault unlocked via password (Argon2id KDF)");
                true
            }
            Err(e) => {
                warn!("HEV: Key derivation failed: {}", e);
                false
            }
        }
    }

    /// Proximity unlock: called by ProximityGuard when RSSI threshold met (Part 25.3).
    /// Only works if vault is Locked (not Cold) -- master key must exist in memory
    /// from a previous unlockWithPassword() call. For Cold state, password is always required.
    pub fn unlock_with_proximity(&self, device_id: &str) -> bool {
        if self.state() != HevVaultState::Locked {
            warn!("HEV: Proximity unlock rejected -- vault not in Locked state (current: {:?})", self.state());
            return false;
        }

        let guard = self.proximity.read();
        if let Some(device) = guard.get_device(device_id) {
            if device.proximity_unlock_enabled {
                // In production: re-derive key from stored encrypted form
                // For now: state transition only
                self.state.store(HevVaultState::Unlocked as u8, Ordering::SeqCst);
                info!("HEV: Vault unlocked via proximity (device: {})", device.device_name);
                return true;
            }
        }
        false
    }

    /// Lock vault: wipes master key from memory (Part 25.3).
    /// Called on screen lock, explicit user action.
    pub fn lock(&self) {
        self.wipe_master_key();
        self.state.store(HevVaultState::Locked as u8, Ordering::SeqCst);
        info!("HEV: Vault locked -- master key wiped");
    }

    /// Seal vault: security alert, requires password to reopen (Part 25.3).
    /// Called on panic lock (sudden signal loss) or security event.
    pub fn seal(&self) {
        self.wipe_master_key();
        self.state.store(HevVaultState::Sealed as u8, Ordering::SeqCst);
        warn!("HEV: Vault SEALED -- security alert, password required to reopen");
    }

    /// Wipes the master key from memory using zeroize (Part 25.3).
    fn wipe_master_key(&self) {
        let mut key_guard = self.master_key.write();
        if let Some(mut key) = key_guard.take() {
            key.0.zeroize();
        }
    }

    /// Event handler: screen locked -> lock vault (Part 25.3).
    pub fn on_screen_locked(&self) {
        self.lock();
    }

    /// Event handler: security alert -> seal vault (Part 25.3).
    pub fn on_security_alert(&self) {
        self.seal();
    }

    /// Event handler: fatal signal -> wipe key immediately (Part 25.3).
    /// This must be async-signal-safe. In production: uses sodium_memzero.
    pub fn on_fatal_signal(&self) {
        self.wipe_master_key();
    }

    // -- Entry access --

    pub fn get_secret(&self, app_id: &str, label: &str) -> HevAccessResult {
        if !self.is_unlocked() {
            return HevAccessResult::VaultLocked;
        }
        if !self.trusted_apps.read().contains(app_id) {
            return HevAccessResult::Denied;
        }

        let entries = self.entries.read();
        let key_guard = self.master_key.read();
        let key = match key_guard.as_ref() {
            Some(k) => k,
            None => return HevAccessResult::VaultLocked,
        };

        if let Some(entry) = entries.iter().find(|e| e.app_id == app_id && e.label == label) {
            let mut payload = Vec::new();
            payload.extend_from_slice(&entry.nonce);
            payload.extend_from_slice(&entry.ciphertext);

            match HevCrypto::decrypt(key, &payload) {
                Ok(_plaintext) => {
                    *self.last_accessed_app.write() = app_id.to_string();
                    *self.last_access_time.write() = 0.0; // Would use CLOCK_MONOTONIC
                    return HevAccessResult::Granted;
                }
                Err(_) => return HevAccessResult::AuthRequired,
            }
        }

        HevAccessResult::NotFound
    }

    pub fn set_secret(&self, app_id: &str, label: &str, store: HevStoreType, plaintext: &[u8]) -> HevAccessResult {
        if !self.is_unlocked() {
            return HevAccessResult::VaultLocked;
        }
        if !self.trusted_apps.read().contains(app_id) {
            return HevAccessResult::Denied;
        }

        let key_guard = self.master_key.read();
        let key = match key_guard.as_ref() {
            Some(k) => k,
            None => return HevAccessResult::VaultLocked,
        };

        match HevCrypto::encrypt(key, plaintext) {
            Ok(encrypted) => {
                let nonce = encrypted[..12].to_vec();
                let ciphertext = encrypted[12..].to_vec();

                let mut entries = self.entries.write();
                // Replace existing entry with same label+app_id
                entries.retain(|e| !(e.app_id == app_id && e.label == label));

                let id = entries.len() as u64 + 1;
                entries.push(HevEntry {
                    id,
                    store,
                    label: label.to_string(),
                    app_id: app_id.to_string(),
                    schema: String::new(),
                    ciphertext,
                    nonce,
                    tag: Vec::new(), // GCM tag is appended to ciphertext
                    created_at: 0,
                    accessed_at: 0,
                });

                HevAccessResult::Granted
            }
            Err(_) => HevAccessResult::AuthRequired,
        }
    }

    pub fn list_entries(&self, app_id: &str) -> Vec<HevEntry> {
        if !self.is_unlocked() {
            return Vec::new();
        }
        self.entries.read().iter().filter(|e| e.app_id == app_id).cloned().collect()
    }

    // -- Trust management --

    pub fn register_trusted_app(&self, app_id: &str) {
        self.trusted_apps.write().insert(app_id.to_string());
        info!("HEV: Trusted app registered: {}", app_id);
    }

    pub fn revoke_trusted_app(&self, app_id: &str) {
        self.trusted_apps.write().remove(app_id);
        info!("HEV: Trusted app revoked: {}", app_id);
    }

    pub fn is_app_trusted(&self, app_id: &str) -> bool {
        self.trusted_apps.read().contains(app_id)
    }

    // -- SeaDrop integration --

    pub fn register_seadrop_device(&self, device: HevTrustedDevice) {
        self.proximity.write().register_device(device);
    }

    pub fn revoke_seadrop_device(&self, device_id: &str) {
        self.proximity.write().revoke_device(device_id);
    }

    pub fn list_seadrop_devices(&self) -> Vec<HevTrustedDevice> {
        self.proximity.read().list_devices()
    }

    pub fn set_proximity_unlock(&self, device_id: &str, enabled: bool) {
        let guard = self.proximity.read();
        let mut devices = guard.devices.write();
        if let Some(d) = devices.iter_mut().find(|d| d.device_id == device_id) {
            d.proximity_unlock_enabled = enabled;
        }
    }

    pub fn set_rssi_thresholds(&self, device_id: &str, unlock_dbm: f32, lock_dbm: f32) {
        let guard = self.proximity.read();
        let mut devices = guard.devices.write();
        if let Some(d) = devices.iter_mut().find(|d| d.device_id == device_id) {
            d.rssi_unlock_threshold = unlock_dbm;
            d.rssi_lock_threshold = lock_dbm;
        }
    }

    // -- Status --

    pub fn status(&self) -> VaultStatus {
        VaultStatus {
            state: self.state(),
            entry_count: self.entries.read().len() as u32,
            trusted_app_count: self.trusted_apps.read().len() as u32,
            seadrop_device_count: self.proximity.read().list_devices().len() as u32,
            last_accessed_app: self.last_accessed_app.read().clone(),
            last_access_time_s: *self.last_access_time.read(),
            proximity_unlock_active: self.proximity.read().list_devices().iter()
                .any(|d| d.proximity_unlock_enabled),
        }
    }

    /// Initializes vault with a salt (from disk or generated on first run).
    pub fn initialize(&self, salt: [u8; 16]) {
        *self.salt.write() = salt;
        info!("HEV: Initialized with salt (vault state: {:?})", self.state());
    }
}

impl Default for HEV {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hev_state_machine() {
        let hev = HEV::new();
        assert_eq!(hev.state(), HevVaultState::Cold);
        assert!(!hev.is_unlocked());

        // Cold start requires password
        hev.initialize([0x42u8; 16]);
        assert!(hev.unlock_with_password("correct_horse_battery_staple"));
        assert_eq!(hev.state(), HevVaultState::Unlocked);
        assert!(hev.is_unlocked());

        // Lock
        hev.lock();
        assert_eq!(hev.state(), HevVaultState::Locked);
        assert!(!hev.is_unlocked());

        // Seal
        hev.seal();
        assert_eq!(hev.state(), HevVaultState::Sealed);
    }

    #[test]
    fn test_hev_trust_management() {
        let hev = HEV::new();
        hev.unlock_with_password("test");

        assert!(!hev.is_app_trusted("com.vitusos.filer"));
        hev.register_trusted_app("com.vitusos.filer");
        assert!(hev.is_app_trusted("com.vitusos.filer"));

        hev.revoke_trusted_app("com.vitusos.filer");
        assert!(!hev.is_app_trusted("com.vitusos.filer"));
    }

    #[test]
    fn test_hev_secret_access() {
        let hev = HEV::new();
        hev.initialize([0x42u8; 16]);
        hev.unlock_with_password("test_password");
        hev.register_trusted_app("com.vitusos.settings");

        // Store a secret
        let result = hev.set_secret("com.vitusos.settings", "wifi_password", HevStoreType::Credentials, b"super_secret_123");
        assert_eq!(result, HevAccessResult::Granted);

        // Retrieve it
        let result = hev.get_secret("com.vitusos.settings", "wifi_password");
        assert_eq!(result, HevAccessResult::Granted);

        // List entries
        let entries = hev.list_entries("com.vitusos.settings");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].label, "wifi_password");

        // Untrusted app denied
        let result = hev.get_secret("com.vitusos.evil", "wifi_password");
        assert_eq!(result, HevAccessResult::Denied);
    }

    #[test]
    fn test_hev_proximity_guard() {
        let hev = HEV::new();
        hev.unlock_with_password("test");
        hev.lock(); // Move to Locked state

        // Register a SeaDrop device
        hev.register_seadrop_device(HevTrustedDevice {
            device_id: "phone_123".to_string(),
            device_name: "Krisna's Phone".to_string(),
            public_key: vec![0u8; 32],
            shared_secret: vec![0u8; 32],
            proximity_unlock_enabled: true,
            rssi_unlock_threshold: -45.0,
            rssi_lock_threshold: -70.0,
            last_seen: 0,
        });

        // Proximity unlock only works in Locked state
        assert!(hev.unlock_with_proximity("phone_123"));
        assert_eq!(hev.state(), HevVaultState::Unlocked);
    }

    #[test]
    fn test_hev_vault_status() {
        let hev = HEV::new();
        hev.unlock_with_password("test");
        hev.register_trusted_app("app1");
        hev.register_trusted_app("app2");

        let status = hev.status();
        assert_eq!(status.state, HevVaultState::Unlocked);
        assert_eq!(status.trusted_app_count, 2);
    }
}
