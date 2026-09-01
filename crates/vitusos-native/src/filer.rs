//! Filer: The Glass-Native Spatial File Manager & Persistent Desktop Daemon.
//!
//! Filer is always running (like macOS Finder), managing the desktop surface layer,
//! volume events, real filesystem navigation, and global file operations.
//! Filer's toolbar searchbar is physically unified with Pathfinder.

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver, SpringSolver2D};
use animus_render::altitude::SurfaceAltitude;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilerViewMode {
    Icon,
    List,
    Columns,
    Gallery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarItem {
    pub label: String,
    pub icon_name: String,
    pub path: Option<String>,
    pub is_section_header: bool,
    pub badge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub date_modified: String,
    pub size_bytes: u64,
    pub formatted_size: String,
    pub kind: String,
    pub icon_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopIcon {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub grid_x: u32,
    pub grid_y: u32,
    pub is_selected: bool,
}

pub enum FileOp {
    Copy { src: PathBuf, dst: PathBuf },
    Move { src: PathBuf, dst: PathBuf },
    Trash { path: PathBuf },
    Delete { path: PathBuf },
    CreateDirectory { path: PathBuf },
}

pub struct FileOperationDaemon {
    tx: Sender<FileOp>,
}

impl FileOperationDaemon {
    pub fn new(bus: EventBus) -> Self {
        let (tx, rx): (Sender<FileOp>, Receiver<FileOp>) = channel();

        thread::spawn(move || {
            while let Ok(op) = rx.recv() {
                match op {
                    FileOp::Copy { src, dst } => {
                        info!("FileOpDaemon: Copying {:?} -> {:?}", src, dst);
                        if let Err(e) = fs::copy(&src, &dst) {
                            warn!("FileOpDaemon: Copy error: {}", e);
                        }
                    }
                    FileOp::Move { src, dst } => {
                        info!("FileOpDaemon: Moving {:?} -> {:?}", src, dst);
                        if let Err(e) = fs::rename(&src, &dst) {
                            warn!("FileOpDaemon: Move error: {}", e);
                        }
                    }
                    FileOp::Trash { path } => {
                        info!("FileOpDaemon: Moving to trash: {:?}", path);
                        // In Linux/vitusOS, moves to ~/.local/share/Trash/files
                        let trash_dir = dirs::data_local_dir()
                            .unwrap_or_else(|| PathBuf::from("/tmp"))
                            .join("Trash/files");
                        let _ = fs::create_dir_all(&trash_dir);
                        if let Some(file_name) = path.file_name() {
                            let target = trash_dir.join(file_name);
                            let _ = fs::rename(&path, target);
                        }
                    }
                    FileOp::Delete { path } => {
                        info!("FileOpDaemon: Permanently deleting {:?}", path);
                        if path.is_dir() {
                            let _ = fs::remove_dir_all(&path);
                        } else {
                            let _ = fs::remove_file(&path);
                        }
                    }
                    FileOp::CreateDirectory { path } => {
                        info!("FileOpDaemon: Creating directory {:?}", path);
                        let _ = fs::create_dir_all(&path);
                    }
                }
                bus.publish(AEEvent::DirectoryChanged {
                    path: "/".to_string(),
                });
            }
        });

        Self { tx }
    }

    pub fn dispatch(&self, op: FileOp) {
        let _ = self.tx.send(op);
    }
}

/// Persistent background daemon (macOS Finder equivalent) that never terminates.
pub struct FilerDaemon {
    pub is_running: bool,
    pub desktop_icons: Vec<DesktopIcon>,
    pub open_windows: Vec<FilerWindow>,
    pub op_daemon: FileOperationDaemon,
    bus: EventBus,
}

impl FilerDaemon {
    pub fn new(bus: EventBus) -> Self {
        let op_daemon = FileOperationDaemon::new(bus.clone());
        let mut default_desktop_icons = Vec::new();
        
        default_desktop_icons.push(DesktopIcon {
            name: "vitusOS Root".to_string(),
            path: "/".to_string(),
            is_directory: true,
            grid_x: 0,
            grid_y: 0,
            is_selected: false,
        });

        Self {
            is_running: true,
            desktop_icons: default_desktop_icons,
            open_windows: Vec::new(),
            op_daemon,
            bus,
        }
    }

    pub fn spawn_window(&mut self, initial_dir: impl Into<PathBuf>) -> &mut FilerWindow {
        let mut window = FilerWindow::new(self.bus.clone());
        window.navigate_to(initial_dir.into());
        self.open_windows.push(window);
        self.open_windows.last_mut().unwrap()
    }

    pub fn update(&mut self, dt: f32) {
        for window in &mut self.open_windows {
            window.update(dt);
        }
    }
}

pub struct FilerWindow {
    pub sidebar_altitude: SurfaceAltitude, // Mid (20px Kawase Blur, 82% Opacity)
    pub toolbar_altitude: SurfaceAltitude, // Low (8px Kawase Blur, 94% Opacity)
    pub content_altitude: SurfaceAltitude, // Grounded (100% Opaque Canvas)
    pub current_directory: PathBuf,
    pub view_mode: FilerViewMode,
    pub sidebar_items: Vec<SidebarItem>,
    pub files: Vec<FileEntry>,
    pub selected_sidebar_idx: usize,
    pub selected_file_indices: Vec<usize>,
    pub selection_pill_y: SpringSolver,    // SPRING_SELECTION (400, 28)
    pub search_bar_width: SpringSolver,    // SPRING_HOVER (600, 40): 188 -> 260px
    pub drag_ghost_pos: SpringSolver2D,    // SPRING_WINDOW_DRAG (800, 35)
    pub is_dragging_file: bool,
    pub is_zebra_striped: bool,
    bus: EventBus,
}

impl FilerWindow {
    pub fn new(bus: EventBus) -> Self {
        let mut sidebar_items = Vec::new();
        
        sidebar_items.push(SidebarItem {
            label: "FAVORITES".to_string(),
            icon_name: "".to_string(),
            path: None,
            is_section_header: true,
            badge: None,
        });
        sidebar_items.push(SidebarItem {
            label: "SeaDrop".to_string(),
            icon_name: "seadrop".to_string(),
            path: Some("/var/run/vitusos/seadrop".to_string()),
            is_section_header: false,
            badge: None,
        });
        sidebar_items.push(SidebarItem {
            label: "Desktop".to_string(),
            icon_name: "desktop".to_string(),
            path: dirs::desktop_dir().map(|p| p.to_string_lossy().to_string()),
            is_section_header: false,
            badge: None,
        });
        sidebar_items.push(SidebarItem {
            label: "Documents".to_string(),
            icon_name: "documents".to_string(),
            path: dirs::document_dir().map(|p| p.to_string_lossy().to_string()),
            is_section_header: false,
            badge: None,
        });
        sidebar_items.push(SidebarItem {
            label: "Downloads".to_string(),
            icon_name: "downloads".to_string(),
            path: dirs::download_dir().map(|p| p.to_string_lossy().to_string()),
            is_section_header: false,
            badge: None,
        });
        sidebar_items.push(SidebarItem {
            label: "Applications".to_string(),
            icon_name: "apps".to_string(),
            path: Some("/usr/share/applications".to_string()),
            is_section_header: false,
            badge: None,
        });

        sidebar_items.push(SidebarItem {
            label: "LOCATIONS".to_string(),
            icon_name: "".to_string(),
            path: None,
            is_section_header: true,
            badge: None,
        });
        sidebar_items.push(SidebarItem {
            label: "vitusOS Root".to_string(),
            icon_name: "drive".to_string(),
            path: Some("/".to_string()),
            is_section_header: false,
            badge: None,
        });
        sidebar_items.push(SidebarItem {
            label: "Hardware Vault".to_string(),
            icon_name: "vault".to_string(),
            path: Some("/run/media/vault".to_string()),
            is_section_header: false,
            badge: Some("HEV".to_string()),
        });

        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        let mut win = Self {
            sidebar_altitude: SurfaceAltitude::Mid,
            toolbar_altitude: SurfaceAltitude::Low,
            content_altitude: SurfaceAltitude::Grounded,
            current_directory: home_dir.clone(),
            view_mode: FilerViewMode::Columns,
            sidebar_items,
            files: Vec::new(),
            selected_sidebar_idx: 2, // Desktop by default
            selected_file_indices: Vec::new(),
            selection_pill_y: SpringSolver::new(72.0, SpringProfile::Selection),
            search_bar_width: SpringSolver::new(188.0, SpringProfile::Hover),
            drag_ghost_pos: SpringSolver2D::new(0.0, 0.0, SpringProfile::WindowDrag),
            is_dragging_file: false,
            is_zebra_striped: true,
            bus,
        };

        win.navigate_to(home_dir);
        win
    }

    /// Reads directory contents from filesystem and populates file entries.
    pub fn navigate_to(&mut self, path: PathBuf) {
        self.current_directory = path.clone();
        self.files.clear();
        self.selected_file_indices.clear();

        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                let is_dir = file_path.is_dir();
                let file_name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files unless specified
                if file_name.starts_with('.') {
                    continue;
                }

                let meta = entry.metadata().ok();
                let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let formatted_size = if is_dir {
                    "--".to_string()
                } else {
                    format_bytes(size_bytes)
                };

                let kind = if is_dir {
                    "Folder".to_string()
                } else {
                    file_path
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|ext| ext.to_uppercase() + " Document")
                        .unwrap_or_else(|| "Document".to_string())
                };

                let icon_path = if is_dir {
                    "assets/icons/sidebar/folder.svg".to_string()
                } else {
                    "assets/icons/sidebar/folder-documents.svg".to_string()
                };

                self.files.push(FileEntry {
                    name: file_name,
                    path: file_path.to_string_lossy().to_string(),
                    is_directory: is_dir,
                    date_modified: "Today".to_string(),
                    size_bytes,
                    formatted_size,
                    kind,
                    icon_path,
                });
            }
        }

        // Sort folders first, then alphabetical
        self.files.sort_by(|a, b| {
            b.is_directory
                .cmp(&a.is_directory)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        self.bus.publish(AEEvent::DirectoryLoaded {
            path: path.to_string_lossy().to_string(),
            count: self.files.len(),
        });
    }

    pub fn select_sidebar_item(&mut self, idx: usize) {
        if idx < self.sidebar_items.len() && !self.sidebar_items[idx].is_section_header {
            self.selected_sidebar_idx = idx;
            let target_y = idx as f32 * 36.0;
            self.selection_pill_y.set_target(target_y);

            if let Some(ref path_str) = self.sidebar_items[idx].path {
                self.navigate_to(PathBuf::from(path_str));
            }
        }
    }

    /// Filer's searchbar IS Pathfinder: focusing/clicking search directly opens Pathfinder!
    pub fn activate_search(&mut self) {
        self.search_bar_width.set_target(260.0);
        self.bus.publish(AEEvent::PathfinderOpened);
    }

    pub fn focus_search(&mut self, focused: bool) {
        if focused {
            self.activate_search();
        } else {
            self.search_bar_width.set_target(188.0);
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.selection_pill_y.update(dt);
        self.search_bar_width.update(dt);
        self.drag_ghost_pos.update(dt);
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filer_daemon_and_searchbar_unification() {
        let bus = EventBus::new();
        let mut daemon = FilerDaemon::new(bus.clone());
        assert!(daemon.is_running);
        assert!(!daemon.desktop_icons.is_empty());

        let window = daemon.spawn_window(std::env::current_dir().unwrap());
        assert_eq!(window.sidebar_altitude, SurfaceAltitude::Mid);
        assert_eq!(window.content_altitude, SurfaceAltitude::Grounded);

        // Filer searchbar activation sends AEEvent::PathfinderOpened
        window.activate_search();
        assert_eq!(window.search_bar_width.target, 260.0);
    }
}
