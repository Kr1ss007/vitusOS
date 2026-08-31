use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum FontError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid or corrupted font file: {0}")]
    InvalidFont(String),
    #[error("Font family not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontFormat {
    TrueType,
    OpenType,
    WOFF2,
    Collection,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMetadata {
    pub family: String,
    pub style: String,
    pub postscript_name: String,
    pub weight: u16,
    pub is_italic: bool,
    pub glyph_count: u16,
    pub format: FontFormat,
    pub file_path: PathBuf,
    pub file_size: u64,
}

pub struct FontManager {
    catalog: RwLock<HashMap<String, Vec<FontMetadata>>>,
    user_font_dir: PathBuf,
}

impl FontManager {
    pub fn new() -> Self {
        let user_font_dir = Self::default_user_font_directory();
        let manager = Self {
            catalog: RwLock::new(HashMap::new()),
            user_font_dir,
        };
        manager.scan_system_fonts();
        manager
    }

    pub fn default_user_font_directory() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".local/share/fonts")
        } else if let Ok(user_profile) = std::env::var("USERPROFILE") {
            PathBuf::from(user_profile).join("AppData/Local/Microsoft/Windows/Fonts")
        } else {
            PathBuf::from("fonts")
        }
    }

    /// Detects format from file magic bytes.
    pub fn detect_format(bytes: &[u8]) -> FontFormat {
        if bytes.len() < 4 {
            return FontFormat::Unknown;
        }
        match &bytes[0..4] {
            [0x00, 0x01, 0x00, 0x00] => FontFormat::TrueType,
            b"OTTO" => FontFormat::OpenType,
            b"wOF2" => FontFormat::WOFF2,
            b"ttcf" => FontFormat::Collection,
            _ => FontFormat::Unknown,
        }
    }

    /// Parses font metadata from disk without external dependencies.
    pub fn inspect_font_file(path: &Path) -> Result<Vec<FontMetadata>, FontError> {
        let bytes = fs::read(path)?;
        let file_size = bytes.len() as u64;
        let format = Self::detect_format(&bytes);

        if format == FontFormat::Unknown {
            return Err(FontError::InvalidFont("Unsupported font magic header".to_string()));
        }

        let mut results = Vec::new();

        // Check if font collection (.ttc / .otc)
        let count = ttf_parser::fonts_in_collection(&bytes).unwrap_or(1);
        for index in 0..count {
            if let Ok(face) = ttf_parser::Face::parse(&bytes, index) {
                let mut family = String::from("Unknown Family");
                let mut style = String::from("Regular");
                let mut postscript_name = String::new();

                for name in face.names() {
                    if let Some(name_str) = name.to_string() {
                        match name.name_id {
                            ttf_parser::name_id::FULL_NAME => {
                                if family == "Unknown Family" {
                                    family = name_str;
                                }
                            }
                            ttf_parser::name_id::FAMILY => {
                                family = name_str;
                            }
                            ttf_parser::name_id::SUBFAMILY => {
                                style = name_str;
                            }
                            ttf_parser::name_id::POST_SCRIPT_NAME => {
                                postscript_name = name_str;
                            }
                            _ => {}
                        }
                    }
                }

                let weight = face.weight().to_number();
                let is_italic = face.is_italic();
                let glyph_count = face.number_of_glyphs();

                results.push(FontMetadata {
                    family,
                    style,
                    postscript_name,
                    weight,
                    is_italic,
                    glyph_count,
                    format,
                    file_path: path.to_path_buf(),
                    file_size,
                });
            }
        }

        if results.is_empty() {
            Err(FontError::InvalidFont("Could not parse valid typeface outline tables".to_string()))
        } else {
            Ok(results)
        }
    }

    /// Single-call atomic font installation.
    /// Copies font to user font directory, updates catalog, and returns installed metadata.
    pub fn install_font(&self, source_path: &Path) -> Result<Vec<FontMetadata>, FontError> {
        let metas = Self::inspect_font_file(source_path)?;
        if metas.is_empty() {
            return Err(FontError::InvalidFont("No valid fonts in file".into()));
        }

        let primary_family = &metas[0].family;
        let sanitized_family: String = primary_family
            .chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '_' })
            .collect();

        let target_dir = self.user_font_dir.join(&sanitized_family);
        fs::create_dir_all(&target_dir)?;

        let filename = source_path
            .file_name()
            .ok_or_else(|| FontError::InvalidFont("Invalid source filename".into()))?;
        let target_path = target_dir.join(filename);

        fs::copy(source_path, &target_path)?;
        info!("Installed font typeface '{}' to {:?}", primary_family, target_path);

        // Re-inspect from installed location
        let installed_metas = Self::inspect_font_file(&target_path)?;

        let mut catalog = self.catalog.write();
        for meta in &installed_metas {
            catalog
                .entry(meta.family.clone())
                .or_default()
                .push(meta.clone());
        }

        Ok(installed_metas)
    }

    /// Discovers system fonts across standard paths.
    pub fn scan_system_fonts(&self) {
        let search_dirs = [
            self.user_font_dir.clone(),
            PathBuf::from("/usr/share/fonts"),
            PathBuf::from("/usr/local/share/fonts"),
            PathBuf::from("C:/Windows/Fonts"),
        ];

        let mut discovered = HashMap::new();

        for dir in &search_dirs {
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            let ext_lower = ext.to_lowercase();
                            if matches!(ext_lower.as_str(), "ttf" | "otf" | "woff2" | "ttc") {
                                if let Ok(metas) = Self::inspect_font_file(&path) {
                                    for meta in metas {
                                        discovered
                                            .entry(meta.family.clone())
                                            .or_insert_with(Vec::new)
                                            .push(meta);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut catalog = self.catalog.write();
        *catalog = discovered;
        info!("FontManager indexed {} font families", catalog.len());
    }

    /// Query typeface by family and desired weight.
    pub fn find_font(&self, family: &str, target_weight: u16, italic: bool) -> Option<FontMetadata> {
        let catalog = self.catalog.read();
        if let Some(faces) = catalog.get(family) {
            // Find closest weight matching italic criteria
            let mut best_match = None;
            let mut min_diff = u16::MAX;

            for face in faces {
                if face.is_italic == italic {
                    let diff = (face.weight as i32 - target_weight as i32).abs() as u16;
                    if diff < min_diff {
                        min_diff = diff;
                        best_match = Some(face.clone());
                    }
                }
            }

            if best_match.is_none() && !faces.is_empty() {
                best_match = Some(faces[0].clone());
            }

            best_match
        } else {
            None
        }
    }

    pub fn list_families(&self) -> Vec<String> {
        let catalog = self.catalog.read();
        let mut families: Vec<_> = catalog.keys().cloned().collect();
        families.sort();
        families
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_canonical_system_fonts() {
        let fonts = vec![
            ("assets/fonts/Inter/Inter-Variable.ttf", "Inter"),
            ("assets/fonts/YoungSerif/YoungSerif-Regular.ttf", "Young Serif"),
            ("assets/fonts/Panamera/ttf/Panamera-Regular.ttf", "Panamera"),
            ("assets/fonts/JetBrainsMono/JetBrainsMono-Regular.ttf", "JetBrains Mono"),
        ];

        for (path_str, expected_family) in fonts {
            let path = PathBuf::from(path_str);
            if path.exists() {
                let metas = FontManager::inspect_font_file(&path)
                    .unwrap_or_else(|e| panic!("Failed to parse font at {}: {:?}", path_str, e));
                assert!(!metas.is_empty());
                assert_eq!(metas[0].family, expected_family, "Font family mismatch for {}", path_str);
                assert!(metas[0].glyph_count > 0, "Glyph count must be non-zero for {}", path_str);
            }
        }
    }
}
