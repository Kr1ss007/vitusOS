//! Pathfinder Universal Search & App Orchestrator.

use animus_cache::app_index::{AppEntry, AppIndexCache, InstallState, PackageFormat};
use animus_core::context::AnimusContext;
use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};

use crate::app_preview::AppPreviewSheet;
use crate::package_manager::PackageManager;

pub struct Pathfinder {
    pub is_open: bool,
    pub query: String,
    pub scale: SpringSolver,          // SPRING_SELECTION (400, 28): 0.92 -> 1.0
    pub opacity: SpringSolver,        // SPRING_SELECTION (400, 28): 0.0 -> 1.0
    pub spinner_active: bool,
    pub results: Vec<AppEntry>,
    pub cache: AppIndexCache,
    pub has_opened_before: bool,
    pub preview_sheet: AppPreviewSheet,
    pub package_manager: PackageManager,
    bus: EventBus,
}

impl Pathfinder {
    pub fn new(cache: AppIndexCache, bus: EventBus) -> Self {
        let preview_sheet = AppPreviewSheet::new(bus.clone());
        let package_manager = PackageManager::new(bus.clone());

        Self {
            is_open: false,
            query: String::new(),
            scale: SpringSolver::new(0.92, SpringProfile::Selection),
            opacity: SpringSolver::new(0.0, SpringProfile::Selection),
            spinner_active: false,
            results: Vec::new(),
            cache,
            has_opened_before: false,
            preview_sheet,
            package_manager,
            bus,
        }
    }

    pub fn placeholder(&self) -> &'static str {
        if self.has_opened_before {
            "Search victusOS"
        } else {
            "what are you looking for?"
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.scale.set_target(1.0);
        self.opacity.set_target(1.0);
        self.bus.publish(AEEvent::PathfinderOpened);
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.scale.set_target(0.92);
        self.opacity.set_target(0.0);
        self.has_opened_before = true;
        self.preview_sheet.close();
        self.bus.publish(AEEvent::PathfinderClosed);
    }

    pub fn on_query_changed(&mut self, query: impl Into<String>) {
        let q = query.into();
        self.query = q.clone();
        self.results = self.cache.search(&self.query);
        self.bus.publish(AEEvent::PathfinderQueryChanged { query: q });
        self.bus.publish(AEEvent::PathfinderResultsReady {
            count: self.results.len(),
        });
    }

    /// Opens the rich Zen Browser Gecko-powered preview sheet for an app result.
    pub fn open_app_preview(&mut self, app_id: &str) {
        if let Some(entry) = self.cache.get(app_id) {
            self.preview_sheet.open(entry);
        }
    }

    /// Triggers "Click and Go" instant download via official repository backend (.deb, Flatpak, or Snap)
    pub fn install_app(&mut self, app_id: &str) {
        let format = self
            .cache
            .get(app_id)
            .map(|e| e.selected_format)
            .unwrap_or(PackageFormat::Deb);

        self.cache.update_install_state(app_id, InstallState::Installing, 0.15, None);
        self.package_manager.install_package(app_id, format);
    }

    /// Selects package format (.deb / Flatpak / Snap) for an app
    pub fn select_package_format(&mut self, app_id: &str, format: PackageFormat) {
        self.cache.set_package_format(app_id, format);
        if let Some(entry) = self.cache.get(app_id) {
            if self.preview_sheet.is_open {
                self.preview_sheet.open(entry);
            }
        }
    }

    /// Spawns an AnimusContext origin for window launch.
    pub fn launch_context(&self, card_x: f32, card_y: f32, card_w: f32, card_h: f32) -> AnimusContext {
        AnimusContext::from_pathfinder_result(card_x, card_y, card_w, card_h)
    }

    pub fn update(&mut self, dt: f32) {
        self.scale.update(dt);
        self.opacity.update(dt);
        self.preview_sheet.update(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pathfinder_search_and_preview_and_install() {
        let bus = EventBus::new();
        let cache = AppIndexCache::new();
        let mut pathfinder = Pathfinder::new(cache, bus);

        // 1. Search for zen browser
        pathfinder.on_query_changed("zen");
        assert!(!pathfinder.results.is_empty());
        assert_eq!(pathfinder.results[0].app_id, "zen-browser");

        // 2. Open rich preview sheet with Zen Browser webview URL
        pathfinder.open_app_preview("zen-browser");
        assert!(pathfinder.preview_sheet.is_open);
        assert!(pathfinder.preview_sheet.zen_webview_active);
        assert_eq!(pathfinder.preview_sheet.zen_webview_url, "https://zen-browser.app");

        // 3. Select package format
        pathfinder.select_package_format("vlc", PackageFormat::Deb);
        let vlc = pathfinder.cache.get("vlc").unwrap();
        assert_eq!(vlc.selected_format, PackageFormat::Deb);

        // 4. Trigger Click and Go install
        pathfinder.install_app("vlc");
        assert_eq!(pathfinder.cache.get("vlc").unwrap().install_state, InstallState::Installing);
    }
}
