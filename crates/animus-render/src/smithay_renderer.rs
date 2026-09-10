#![cfg(target_os = "linux")]

use smithay::backend::renderer::glow::GlowRenderer;
use smithay::reexports::glow::{self, HasContext};

/// Our real Wayland renderer utilizing Smithay's GlowRenderer backend.
/// Maps directly to the 7-Layer Canonical Pipeline.
pub struct AnimusGlowRenderer {
    pub renderer: GlowRenderer,
}

impl AnimusGlowRenderer {
    pub fn new(renderer: GlowRenderer) -> Self {
        Self { renderer }
    }

    /// The 7-layer compositing pass.
    /// Clears with vitusOS canonical warm black (#1A1208) and configures GL pipeline state.
    pub fn render_frame(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.renderer.with_context(|gl| {
            unsafe {
                gl.viewport(0, 0, width as i32, height as i32);
                // #1A1208 warm black: R=26/255=0.102, G=18/255=0.071, B=8/255=0.031, A=1.0
                gl.clear_color(0.102, 0.071, 0.031, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

                // Enable alpha blending for translucent windows & glass panels
                gl.enable(glow::BLEND);
                gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            }
        })?;

        Ok(())
    }
}
