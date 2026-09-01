//! systemd-logind D-Bus Proxy (`org.freedesktop.login1`).

use tracing::info;

pub struct LogindDbusClient;

impl Default for LogindDbusClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LogindDbusClient {
    pub fn new() -> Self {
        Self
    }

    /// Dispatches shutdown to systemd-logind (clean zero-flicker poweroff)
    pub async fn power_off(&self) -> bool {
        info!("LogindDbusClient: Requesting system power off via org.freedesktop.login1");

        #[cfg(target_os = "linux")]
        {
            if let Ok(conn) = zbus::Connection::system().await {
                let _ = conn.call_method(
                    Some("org.freedesktop.login1"),
                    "/org/freedesktop/login1",
                    Some("org.freedesktop.login1.Manager"),
                    "PowerOff",
                    &(true), // interactive auth
                ).await;
                return true;
            }
        }

        true
    }

    /// Dispatches system restart
    pub async fn reboot(&self) -> bool {
        info!("LogindDbusClient: Requesting system reboot via org.freedesktop.login1");

        #[cfg(target_os = "linux")]
        {
            if let Ok(conn) = zbus::Connection::system().await {
                let _ = conn.call_method(
                    Some("org.freedesktop.login1"),
                    "/org/freedesktop/login1",
                    Some("org.freedesktop.login1.Manager"),
                    "Reboot",
                    &(true),
                ).await;
                return true;
            }
        }

        true
    }

    /// Locks current session
    pub async fn lock_session(&self) -> bool {
        info!("LogindDbusClient: Requesting session lock via org.freedesktop.login1");

        #[cfg(target_os = "linux")]
        {
            if let Ok(conn) = zbus::Connection::system().await {
                let _ = conn.call_method(
                    Some("org.freedesktop.login1"),
                    "/org/freedesktop/login1/session/auto",
                    Some("org.freedesktop.login1.Session"),
                    "Lock",
                    &(),
                ).await;
                return true;
            }
        }

        true
    }
}
