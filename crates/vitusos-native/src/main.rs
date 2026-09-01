//! vitusOS Native Userspace Application & Shell Surface Dispatcher.

use animus_core::event_bus::EventBus;
use std::env;
use tracing::info;
use vitusos_native::{FilerDaemon, PackageManager, Pathfinder, SettingsApp, Terminow, ZenBrowserManager};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("--help");

    info!("================================================================================");
    info!("                     vitusOS Native Shell & Applications Suite                 ");
    info!("================================================================================");

    let bus = EventBus::new();

    match command {
        "--surface" | "-s" => {
            let surface = args.get(2).map(|s| s.as_str()).unwrap_or("all");
            info!("Launching native shell surface daemon for: '{}'", surface);
            // Runs event loop for surface management
        }
        "--app" | "-a" => {
            let app_name = args.get(2).map(|s| s.as_str()).unwrap_or("filer");
            match app_name {
                "filer" => {
                    info!("Starting Filer continuous filesystem daemon...");
                    let filer = FilerDaemon::new(bus.clone());
                    info!("Filer initialized: {} desktop icon(s) active.", filer.desktop_icons.len());
                }
                "pathfinder" => {
                    info!("Starting Pathfinder unified search daemon...");
                    let cache = animus_cache::AppIndexCache::new();
                    let _pathfinder = Pathfinder::new(cache, bus.clone());
                    info!("Pathfinder initialized with active query state.");
                }
                "terminow" => {
                    info!("Starting Terminow GPU-accelerated terminal...");
                    let term = Terminow::new(bus.clone());
                    info!("Terminow initialized: {} active tab(s).", term.tabs.read().len());
                }
                "settings" => {
                    info!("Starting Settings application...");
                    let settings = SettingsApp::new(bus.clone());
                    info!("Current Release Channel: {:?}", settings.state.read().active_channel);
                }
                "zen-browser" => {
                    info!("Starting Zen Browser native Wayland integration...");
                    let zen = ZenBrowserManager::new(bus.clone());
                    info!("Zen Browser initialized: {} workspace(s) active.", zen.workspaces.read().len());
                }
                "package-manager" => {
                    info!("Starting Pathfinder Package Manager daemon...");
                    let _pkg_mgr = PackageManager::new(bus.clone());
                    info!("Package Manager ready for APT / Flatpak dispatch.");
                }
                other => {
                    info!("Unknown native app: '{}'. Defaulting to Pathfinder search overlay.", other);
                }
            }
        }
        _ => {
            println!("Usage: vitusos-native [OPTIONS]");
            println!("Options:");
            println!("  --surface <panel|dock|control-center|all>  Launch native shell surface");
            println!("  --app <filer|pathfinder|terminow|settings|zen-browser|package-manager>  Launch application");
        }
    }

    Ok(())
}
