//! D-Bus System Integration Layer for vitusOS.
//!
//! Provides direct D-Bus communication with system daemons:
//! - NetworkManager (Wi-Fi, Ethernet, Signal strength)
//! - BlueZ (Bluetooth adapter, device pairing)
//! - systemd-logind (Power management, Shutdown, Reboot, Lock)
//! - PipeWire / PulseAudio (Audio sink, Volume, Mute)

pub mod audio;
pub mod bluetooth;
pub mod login;
pub mod network;

pub use audio::AudioDbusClient;
pub use bluetooth::BluetoothDbusClient;
pub use login::LogindDbusClient;
pub use network::NetworkDbusClient;

use tracing::info;

pub struct SystemDbusManager {
    pub network: NetworkDbusClient,
    pub bluetooth: BluetoothDbusClient,
    pub login: LogindDbusClient,
    pub audio: AudioDbusClient,
}

impl Default for SystemDbusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemDbusManager {
    pub fn new() -> Self {
        info!("SystemDbusManager: Initialized native D-Bus integration proxies.");
        Self {
            network: NetworkDbusClient::new(),
            bluetooth: BluetoothDbusClient::new(),
            login: LogindDbusClient::new(),
            audio: AudioDbusClient::new(),
        }
    }
}
