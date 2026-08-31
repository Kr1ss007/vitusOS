//! In-Memory App Index Cache for Sub-16ms Pathfinder Search.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageFormat {
    /// Official Ubuntu repository (.deb / APT)
    Deb,
    /// Flathub sandboxed bundle
    Flatpak,
    /// Canonical Snap Store package
    Snap,
}

impl PackageFormat {
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Deb => "Ubuntu .deb",
            Self::Flatpak => "Flatpak",
            Self::Snap => "Snap",
        }
    }

    pub const fn source_name(&self) -> &'static str {
        match self {
            Self::Deb => "Ubuntu Official Repo (APT)",
            Self::Flatpak => "Flathub",
            Self::Snap => "Snapcraft",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallState {
    Installed,
    Available,
    Installing,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub app_id: String,
    pub display_name: String,
    pub description: String,
    pub publisher: String,
    pub version: String,
    pub icon_path: String,
    pub exec_path: String,
    pub keywords: Vec<String>,
    pub available_formats: Vec<PackageFormat>,
    pub selected_format: PackageFormat,
    pub install_state: InstallState,
    pub install_progress: f32,
    pub install_error: Option<String>,
    pub webview_url: Option<String>,
    pub screenshot_urls: Vec<String>,
}

#[derive(Clone)]
pub struct AppIndexCache {
    entries: Arc<RwLock<HashMap<String, AppEntry>>>,
}

impl Default for AppIndexCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AppIndexCache {
    pub fn new() -> Self {
        let cache = Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        };
        cache.populate_default_system_apps();
        cache
    }

    pub fn populate_default_system_apps(&self) {
        let default_apps = vec![
            AppEntry {
                app_id: "zen-browser".to_string(),
                display_name: "Zen Browser".to_string(),
                description: "Native privacy-focused spatial web browser with vertical tabs".to_string(),
                publisher: "Zen Development Team".to_string(),
                version: "1.0.2-a.1".to_string(),
                icon_path: "/usr/share/icons/vitusos/zen-browser.png".to_string(),
                exec_path: "/usr/bin/zen-browser".to_string(),
                keywords: vec!["browser".into(), "web".into(), "internet".into(), "zen".into(), "firefox".into()],
                available_formats: vec![PackageFormat::Flatpak, PackageFormat::Deb, PackageFormat::Snap],
                selected_format: PackageFormat::Flatpak,
                install_state: InstallState::Installed,
                install_progress: 1.0,
                install_error: None,
                webview_url: Some("https://zen-browser.app".to_string()),
                screenshot_urls: vec!["https://zen-browser.app/assets/screenshot-1.png".to_string()],
            },
            AppEntry {
                app_id: "filer".to_string(),
                display_name: "Files".to_string(),
                description: "Spatial 3-zone glass file manager".to_string(),
                publisher: "vitusOS Core Team".to_string(),
                version: "1.0.0".to_string(),
                icon_path: "/usr/share/icons/vitusos/filer.png".to_string(),
                exec_path: "/usr/bin/vitusos-filer".to_string(),
                keywords: vec!["file".into(), "folder".into(), "directory".into(), "storage".into(), "documents".into()],
                available_formats: vec![PackageFormat::Deb],
                selected_format: PackageFormat::Deb,
                install_state: InstallState::Installed,
                install_progress: 1.0,
                install_error: None,
                webview_url: None,
                screenshot_urls: Vec::new(),
            },
            AppEntry {
                app_id: "pathfinder".to_string(),
                display_name: "Pathfinder".to_string(),
                description: "Universal search and instant app installer".to_string(),
                publisher: "vitusOS Core Team".to_string(),
                version: "1.0.0".to_string(),
                icon_path: "/usr/share/icons/vitusos/pathfinder.png".to_string(),
                exec_path: "/usr/bin/vitusos-pathfinder".to_string(),
                keywords: vec!["search".into(), "find".into(), "apps".into(), "install".into()],
                available_formats: vec![PackageFormat::Deb],
                selected_format: PackageFormat::Deb,
                install_state: InstallState::Installed,
                install_progress: 1.0,
                install_error: None,
                webview_url: None,
                screenshot_urls: Vec::new(),
            },
            AppEntry {
                app_id: "terminow".to_string(),
                display_name: "Terminow".to_string(),
                description: "GPU-accelerated spatial terminal emulator".to_string(),
                publisher: "vitusOS Core Team".to_string(),
                version: "1.0.0".to_string(),
                icon_path: "/usr/share/icons/vitusos/terminow.png".to_string(),
                exec_path: "/usr/bin/vitusos-terminow".to_string(),
                keywords: vec!["terminal".into(), "console".into(), "shell".into(), "bash".into(), "zsh".into()],
                available_formats: vec![PackageFormat::Deb],
                selected_format: PackageFormat::Deb,
                install_state: InstallState::Installed,
                install_progress: 1.0,
                install_error: None,
                webview_url: None,
                screenshot_urls: Vec::new(),
            },
            AppEntry {
                app_id: "settings".to_string(),
                display_name: "Settings".to_string(),
                description: "System preferences, display, wallpaper, and accounts".to_string(),
                publisher: "vitusOS Core Team".to_string(),
                version: "1.0.0".to_string(),
                icon_path: "/usr/share/icons/vitusos/settings.png".to_string(),
                exec_path: "/usr/bin/vitusos-settings".to_string(),
                keywords: vec!["settings".into(), "preferences".into(), "display".into(), "wallpaper".into(), "wifi".into()],
                available_formats: vec![PackageFormat::Deb],
                selected_format: PackageFormat::Deb,
                install_state: InstallState::Installed,
                install_progress: 1.0,
                install_error: None,
                webview_url: None,
                screenshot_urls: Vec::new(),
            },
            AppEntry {
                app_id: "vlc".to_string(),
                display_name: "VLC Media Player".to_string(),
                description: "Universal multi-format media player and streaming engine".to_string(),
                publisher: "VideoLAN Organization".to_string(),
                version: "3.0.20".to_string(),
                icon_path: "/usr/share/icons/vitusos/vlc.png".to_string(),
                exec_path: "/usr/bin/vlc".to_string(),
                keywords: vec!["vlc".into(), "video".into(), "music".into(), "player".into(), "media".into()],
                available_formats: vec![PackageFormat::Deb, PackageFormat::Flatpak, PackageFormat::Snap],
                selected_format: PackageFormat::Deb,
                install_state: InstallState::Available,
                install_progress: 0.0,
                install_error: None,
                webview_url: Some("https://www.videolan.org/vlc/".to_string()),
                screenshot_urls: vec!["https://www.videolan.org/vlc/screenshots/vlc.png".to_string()],
            },
            AppEntry {
                app_id: "obs-studio".to_string(),
                display_name: "OBS Studio".to_string(),
                description: "High performance live broadcasting and screen recording".to_string(),
                publisher: "OBS Project".to_string(),
                version: "30.1.2".to_string(),
                icon_path: "/usr/share/icons/vitusos/obs.png".to_string(),
                exec_path: "/usr/bin/obs".to_string(),
                keywords: vec!["obs".into(), "record".into(), "stream".into(), "video".into(), "broadcast".into()],
                available_formats: vec![PackageFormat::Flatpak, PackageFormat::Deb, PackageFormat::Snap],
                selected_format: PackageFormat::Flatpak,
                install_state: InstallState::Available,
                install_progress: 0.0,
                install_error: None,
                webview_url: Some("https://obsproject.com".to_string()),
                screenshot_urls: vec!["https://obsproject.com/assets/images/obs-screenshot.png".to_string()],
            },
        ];

        let mut map = self.entries.write();
        for app in default_apps {
            map.insert(app.app_id.clone(), app);
        }
    }

    pub fn insert(&self, entry: AppEntry) {
        let mut map = self.entries.write();
        map.insert(entry.app_id.clone(), entry);
    }

    pub fn get(&self, app_id: &str) -> Option<AppEntry> {
        self.entries.read().get(app_id).cloned()
    }

    pub fn set_package_format(&self, app_id: &str, format: PackageFormat) {
        let mut map = self.entries.write();
        if let Some(entry) = map.get_mut(app_id) {
            if entry.available_formats.contains(&format) {
                entry.selected_format = format;
            }
        }
    }

    pub fn update_install_state(&self, app_id: &str, state: InstallState, progress: f32, error: Option<String>) {
        let mut map = self.entries.write();
        if let Some(entry) = map.get_mut(app_id) {
            entry.install_state = state;
            entry.install_progress = progress;
            entry.install_error = error;
        }
    }

    /// Sub-16ms in-memory app search with multi-keyword scoring.
    pub fn search(&self, query: &str) -> Vec<AppEntry> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let map = self.entries.read();
        let mut scored_results: Vec<(u32, AppEntry)> = Vec::new();

        for entry in map.values() {
            let mut score = 0;

            if entry.display_name.to_lowercase() == q {
                score += 100;
            } else if entry.display_name.to_lowercase().starts_with(&q) {
                score += 60;
            } else if entry.display_name.to_lowercase().contains(&q) {
                score += 40;
            }

            for kw in &entry.keywords {
                if kw.to_lowercase() == q {
                    score += 50;
                } else if kw.to_lowercase().contains(&q) {
                    score += 20;
                }
            }

            if score > 0 {
                scored_results.push((score, entry.clone()));
            }
        }

        // Sort descending by score
        scored_results.sort_by(|a, b| b.0.cmp(&a.0));
        scored_results.into_iter().map(|(_, e)| e).collect()
    }
}
