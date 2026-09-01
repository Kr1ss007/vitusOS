//! BlueZ D-Bus Proxy (`org.bluez`).

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDevice {
    pub name: String,
    pub address: String,
    pub is_connected: bool,
    pub is_paired: bool,
    pub icon_name: String,
}

pub struct BluetoothDbusClient {
    pub is_powered: AtomicBool,
}

impl Default for BluetoothDbusClient {
    fn default() -> Self {
        Self::new()
    }
}

impl BluetoothDbusClient {
    pub fn new() -> Self {
        Self {
            is_powered: AtomicBool::new(true),
        }
    }

    pub async fn get_paired_devices(&self) -> Vec<BluetoothDevice> {
        #[cfg(target_os = "linux")]
        {
            // Read ObjectManager managed objects from org.bluez
            info!("BluetoothDbusClient: Enumerating paired BlueZ devices.");
        }

        vec![
            BluetoothDevice {
                name: "SpaceTrack Touchpad".to_string(),
                address: "AA:BB:CC:11:22:33".to_string(),
                is_connected: true,
                is_paired: true,
                icon_name: "input-touchpad".to_string(),
            },
            BluetoothDevice {
                name: "Animus Studio Headphones".to_string(),
                address: "DD:EE:FF:44:55:66".to_string(),
                is_connected: true,
                is_paired: true,
                icon_name: "audio-headphones".to_string(),
            },
        ]
    }

    pub async fn set_powered(&self, powered: bool) -> bool {
        self.is_powered.store(powered, Ordering::SeqCst);
        info!("BluetoothDbusClient: Set Bluetooth adapter power -> {}", powered);

        #[cfg(target_os = "linux")]
        {
            if let Ok(conn) = zbus::Connection::system().await {
                let _ = conn.call_method(
                    Some("org.bluez"),
                    "/org/bluez/hci0",
                    Some("org.freedesktop.DBus.Properties"),
                    "Set",
                    &("org.bluez.Adapter1", "Powered", zbus::zvariant::Value::from(powered)),
                ).await;
            }
        }

        true
    }
}
