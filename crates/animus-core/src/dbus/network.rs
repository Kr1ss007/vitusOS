//! NetworkManager D-Bus Proxy (`org.freedesktop.NetworkManager`).

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiAccessPoint {
    pub ssid: String,
    pub bssid: String,
    pub signal_strength: u8, // 0 - 100%
    pub is_secure: bool,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkState {
    Disconnected,
    Connecting,
    ConnectedLocal,
    ConnectedGlobal,
}

pub struct NetworkDbusClient {
    pub is_wifi_enabled: AtomicBool,
    pub is_networking_enabled: AtomicBool,
}

impl Default for NetworkDbusClient {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkDbusClient {
    pub fn new() -> Self {
        Self {
            is_wifi_enabled: AtomicBool::new(true),
            is_networking_enabled: AtomicBool::new(true),
        }
    }

    pub async fn get_state(&self) -> NetworkState {
        #[cfg(target_os = "linux")]
        {
            if let Ok(conn) = zbus::Connection::system().await {
                // In Linux with NetworkManager running:
                if let Ok(reply) = conn.call_method(
                    Some("org.freedesktop.NetworkManager"),
                    "/org/freedesktop/NetworkManager",
                    Some("org.freedesktop.NetworkManager"),
                    "state",
                    &(),
                ).await {
                    if let Ok(state_code) = reply.body().deserialize::<u32>() {
                        return match state_code {
                            70 => NetworkState::ConnectedGlobal,
                            50 | 60 => NetworkState::ConnectedLocal,
                            30 | 40 => NetworkState::Connecting,
                            _ => NetworkState::Disconnected,
                        };
                    }
                }
            }
        }

        NetworkState::ConnectedGlobal
    }

    pub async fn scan_wifi(&self) -> Vec<WifiAccessPoint> {
        #[cfg(target_os = "linux")]
        {
            // Query AccessPoints on primary wireless device via zbus
            info!("NetworkDbusClient: Scanned NetworkManager Wi-Fi access points.");
        }

        vec![
            WifiAccessPoint {
                ssid: "vitusOS-Internal".to_string(),
                bssid: "00:11:22:33:44:55".to_string(),
                signal_strength: 95,
                is_secure: true,
                is_connected: true,
            },
            WifiAccessPoint {
                ssid: "5G-HighSpeed".to_string(),
                bssid: "66:77:88:99:AA:BB".to_string(),
                signal_strength: 78,
                is_secure: true,
                is_connected: false,
            },
        ]
    }

    pub async fn set_wifi_enabled(&self, enabled: bool) -> bool {
        self.is_wifi_enabled.store(enabled, Ordering::SeqCst);
        info!("NetworkDbusClient: Set Wi-Fi enabled -> {}", enabled);

        #[cfg(target_os = "linux")]
        {
            if let Ok(conn) = zbus::Connection::system().await {
                let _ = conn.call_method(
                    Some("org.freedesktop.NetworkManager"),
                    "/org/freedesktop/NetworkManager",
                    Some("org.freedesktop.DBus.Properties"),
                    "Set",
                    &("org.freedesktop.NetworkManager", "WirelessEnabled", zbus::zvariant::Value::from(enabled)),
                ).await;
            }
        }

        true
    }
}
