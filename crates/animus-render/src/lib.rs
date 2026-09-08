pub mod altitude;
pub mod appkit;
pub mod color;
pub mod font_manager;
pub mod framebuffer;
pub mod glyph_atlas;
pub mod gpu;
pub mod pipeline;
pub mod shadow;
pub mod squircle;
pub mod text_renderer;
pub mod typography;
pub mod vulkan_context;
pub mod wallpaper_sampler;

#[cfg(target_os = "linux")]
pub mod smithay_renderer;

pub use altitude::{GlassProperties, SurfaceAltitude};
pub use appkit::{AEButton, AESegmentedControl, AETextField, AETrafficLights, ButtonVariant};
pub use color::{Oklab, SystemColors, TextColor};
pub use font_manager::{FontError, FontFormat, FontManager, FontMetadata};
pub use framebuffer::{Rect, ScanoutFramebuffer};
pub use glyph_atlas::{AtlasError, GlyphAtlas, GlyphEntry};
pub use gpu::AnimusVulkanRenderer;
pub use pipeline::{RenderPipeline, RenderWindow};
pub use shadow::ShadowParams;
pub use squircle::SquircleParams;
pub use typography::{FontFamily, FontRoleInfo, TextRole};
pub use text_renderer::{TextRenderer, ShapedText, PositionedGlyph, TextAlign, TextTruncation, TextLayoutParams};
pub use vulkan_context::{DmaBufAttributes, ImportedBuffer, VulkanContext};
pub use wallpaper_sampler::{WallpaperMetrics, WallpaperTintSampler};



