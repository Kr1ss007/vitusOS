use std::path::Path;
use std::sync::Arc;
use animus_core::EventBus;
use animus_physics::{SpringProfile, SpringSolver};
use animus_render::{FontManager, FontMetadata};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPreviewState {
    pub current_point_size: f32,
    pub sample_text: String,
    pub is_installing: bool,
    pub is_installed: bool,
    pub selected_face_index: usize,
}

pub struct FontBookSheet {
    pub metadata_list: Vec<FontMetadata>,
    pub state: FontPreviewState,
    pub font_manager: Arc<FontManager>,
    pub event_bus: Arc<EventBus>,
    sheet_spring: SpringSolver,
}

impl FontBookSheet {
    pub fn new(
        font_path: &Path,
        font_manager: Arc<FontManager>,
        event_bus: Arc<EventBus>,
    ) -> Result<Self, String> {
        let metadata_list = FontManager::inspect_font_file(font_path)
            .map_err(|e| format!("Failed to inspect font file: {}", e))?;

        if metadata_list.is_empty() {
            return Err("No valid font definitions found in file".to_string());
        }

        let mut sheet_spring = SpringSolver::new(0.0, SpringProfile::Sheet);
        sheet_spring.set_target(1.0);

        Ok(Self {
            metadata_list,
            state: FontPreviewState {
                current_point_size: 24.0,
                sample_text: String::from("The quick brown fox jumps over the lazy dog."),
                is_installing: false,
                is_installed: false,
                selected_face_index: 0,
            },
            font_manager,
            event_bus,
            sheet_spring,
        })
    }

    pub fn primary_metadata(&self) -> &FontMetadata {
        &self.metadata_list[self.state.selected_face_index]
    }

    pub fn set_point_size(&mut self, pt: f32) {
        self.state.current_point_size = pt.clamp(10.0, 96.0);
    }

    pub fn set_sample_text(&mut self, text: &str) {
        self.state.sample_text = text.to_string();
    }

    pub fn select_face(&mut self, index: usize) {
        if index < self.metadata_list.len() {
            self.state.selected_face_index = index;
        }
    }

    /// Triggers single-click atomic font installation into user font catalog.
    pub fn install_current_font(&mut self) -> Result<Vec<FontMetadata>, String> {
        self.state.is_installing = true;
        let source_path = &self.primary_metadata().file_path;

        let result = self.font_manager.install_font(source_path);
        self.state.is_installing = false;

        match result {
            Ok(metas) => {
                self.state.is_installed = true;
                info!("FontBook: Successfully installed font family '{}'", metas[0].family);
                Ok(metas)
            }
            Err(e) => Err(format!("Installation error: {}", e)),
        }
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        self.sheet_spring.update(dt)
    }

    pub fn is_settled(&self) -> bool {
        self.sheet_spring.is_settled()
    }
}
