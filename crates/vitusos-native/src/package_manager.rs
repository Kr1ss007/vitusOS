//! "Click and Go" Native Package Manager for Ubuntu .deb, Flatpak, and Snap.

#[cfg(target_os = "linux")]
use std::process::Command;
use animus_cache::app_index::PackageFormat;
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use tracing::{info, warn};

pub struct PackageManager {
    bus: EventBus,
}

impl PackageManager {
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }

    /// Triggers non-blocking "Click and Go" package installation from the official repository.
    pub fn install_package(&self, app_id: &str, format: PackageFormat) {
        let app_id_owned = app_id.to_string();
        let bus_clone = self.bus.clone();

        info!(
            "PackageManager: Initiating 'Click and Go' installation for '{}' from source '{}'",
            app_id,
            format.source_name()
        );

        // Immediate responsive progress broadcast
        bus_clone.publish(AEEvent::InstallProgress {
            app_id: app_id_owned.clone(),
            progress: 0.15,
        });

        // Spawn asynchronous installation worker
        std::thread::spawn(move || {
            let success = match format {
                PackageFormat::Deb => Self::install_deb(&app_id_owned, &bus_clone),
                PackageFormat::Flatpak => Self::install_flatpak(&app_id_owned, &bus_clone),
                PackageFormat::Snap => Self::install_snap(&app_id_owned, &bus_clone),
            };

            if success {
                bus_clone.publish(AEEvent::InstallProgress {
                    app_id: app_id_owned.clone(),
                    progress: 1.0,
                });
                bus_clone.publish(AEEvent::InstallComplete {
                    app_id: app_id_owned.clone(),
                });
                info!("PackageManager: Installation of '{}' succeeded!", app_id_owned);
            } else {
                bus_clone.publish(AEEvent::InstallFailed {
                    app_id: app_id_owned.clone(),
                    error: format!("Package installation failed for format {:?}", format),
                });
                warn!("PackageManager: Installation of '{}' failed.", app_id_owned);
            }
        });
    }

    fn install_deb(package: &str, bus: &EventBus) -> bool {
        info!("APT: Calling official Ubuntu repository for '{}'", package);
        bus.publish(AEEvent::InstallProgress {
            app_id: package.to_string(),
            progress: 0.45,
        });

        // In Linux/Ubuntu environment, invokes apt-get; on dev host simulates progress
        #[cfg(target_os = "linux")]
        {
            let status = Command::new("apt-get")
                .args(["install", "-y", package])
                .status();
            status.map(|s| s.success()).unwrap_or(false)
        }
        #[cfg(not(target_os = "linux"))]
        {
            std::thread::sleep(std::time::Duration::from_millis(50));
            true
        }
    }

    fn install_flatpak(package: &str, bus: &EventBus) -> bool {
        info!("Flatpak: Calling Flathub for '{}'", package);
        bus.publish(AEEvent::InstallProgress {
            app_id: package.to_string(),
            progress: 0.50,
        });

        #[cfg(target_os = "linux")]
        {
            let status = Command::new("flatpak")
                .args(["install", "-y", "flathub", package])
                .status();
            status.map(|s| s.success()).unwrap_or(false)
        }
        #[cfg(not(target_os = "linux"))]
        {
            std::thread::sleep(std::time::Duration::from_millis(50));
            true
        }
    }

    fn install_snap(package: &str, bus: &EventBus) -> bool {
        info!("Snap: Calling Canonical Snap Store for '{}'", package);
        bus.publish(AEEvent::InstallProgress {
            app_id: package.to_string(),
            progress: 0.50,
        });

        #[cfg(target_os = "linux")]
        {
            let status = Command::new("snap")
                .args(["install", package])
                .status();
            status.map(|s| s.success()).unwrap_or(false)
        }
        #[cfg(not(target_os = "linux"))]
        {
            std::thread::sleep(std::time::Duration::from_millis(50));
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_manager_dispatch() {
        let bus = EventBus::new();
        let pm = PackageManager::new(bus.clone());
        pm.install_package("vlc", PackageFormat::Deb);
    }
}
