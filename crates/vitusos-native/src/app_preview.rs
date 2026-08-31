//! App Info & GUI Preview Sheet powered by Zen Browser Gecko Toolkit.
//!
//! Renders rich product pages, screenshots, permission audits, and live web previews
//! using Zen Browser's Gecko/Wayland embeddable webview stack.

use animus_cache::app_index::{AppEntry, InstallState};
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};
use animus_render::appkit::{AEButton, AESegmentedControl, ButtonVariant};

pub struct AppPreviewSheet {
    pub app: Option<AppEntry>,
    pub sheet_scale: SpringSolver, // SPRING_SHEET (420, 30): 0.92 -> 1.0
    pub opacity: SpringSolver,     // SPRING_SELECTION (400, 28): 0.0 -> 1.0
    pub is_open: bool,
    pub format_selector: Option<AESegmentedControl>,
    pub get_button: AEButton,
    pub zen_webview_active: bool,
    pub zen_webview_url: String,
    pub current_screenshot_idx: usize,
    bus: EventBus,
}

impl AppPreviewSheet {
    pub fn new(bus: EventBus) -> Self {
        Self {
            app: None,
            sheet_scale: SpringSolver::new(0.92, SpringProfile::Sheet),
            opacity: SpringSolver::new(0.0, SpringProfile::Selection),
            is_open: false,
            format_selector: None,
            get_button: AEButton::new("Get", ButtonVariant::Primary),
            zen_webview_active: false,
            zen_webview_url: String::new(),
            current_screenshot_idx: 0,
            bus,
        }
    }

    /// Opens the rich preview sheet for a specific app entry.
    pub fn open(&mut self, app: AppEntry) {
        let segment_labels: Vec<String> = app
            .available_formats
            .iter()
            .map(|f| f.label().to_string())
            .collect();

        self.format_selector = Some(AESegmentedControl::new(segment_labels, 110.0));
        
        let button_label = match app.install_state {
            InstallState::Installed => "Open",
            InstallState::Installing => "Installing...",
            InstallState::Available | InstallState::Failed => "Get",
        };
        self.get_button = AEButton::new(button_label, ButtonVariant::Primary);

        if let Some(ref url) = app.webview_url {
            self.zen_webview_active = true;
            self.zen_webview_url = url.clone();
        } else {
            self.zen_webview_active = false;
            self.zen_webview_url.clear();
        }

        self.app = Some(app);
        self.is_open = true;
        self.sheet_scale.set_target(1.0);
        self.opacity.set_target(1.0);
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.sheet_scale.set_target(0.92);
        self.opacity.set_target(0.0);
    }

    /// Selects package format (.deb / Flatpak / Snap)
    pub fn select_format(&mut self, index: usize) {
        if let Some(ref mut selector) = self.format_selector {
            selector.select(index);
        }
        if let Some(ref mut app) = self.app {
            if index < app.available_formats.len() {
                app.selected_format = app.available_formats[index];
            }
        }
    }

    /// Triggers "Click and Go" instant download via official repository backend
    pub fn click_get(&mut self) {
        if let Some(ref app) = self.app {
            if app.install_state == InstallState::Installed {
                // Launch application
                return;
            }

            self.get_button.label = "Installing...".to_string();
            self.bus.publish(AEEvent::InstallProgress {
                app_id: app.app_id.clone(),
                progress: 0.05,
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.sheet_scale.update(dt);
        self.opacity.update(dt);
        self.get_button.update(dt);
        if let Some(ref mut selector) = self.format_selector {
            selector.update(dt);
        }
    }
}
