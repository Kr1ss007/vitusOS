//! The 7-Layer Canonical Rendering Pipeline (Part 6.3 of Architecture Specification).
//!
//! Enforces exact compositing order:
//! - Layer 0: Wallpaper (full screen texture quad)
//! - Layer 1: Window shadows (spring-lagged SDF)
//! - Layer 2: Window glass backgrounds (Kawase blur + luminosity + tint)
//! - Layer 3: Window content (client surface pixels)
//! - Layer 4: Shell surfaces (Top Panel, Floating Dock)
//! - Layer 5: Boot crossfade overlay (Space Orange fade)
//! - Layer 6: Floating overlays (Pathfinder, Control Center, Notifications)

use crate::altitude::SurfaceAltitude;
use crate::color::Oklab;
use crate::framebuffer::ScanoutFramebuffer;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderWindow {
    pub id: u64,
    pub title: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub shadow_x: f32,
    pub shadow_y: f32,
    pub corner_radius: f32,
    pub altitude: SurfaceAltitude,
    pub is_visible: bool,
    pub is_focused: bool,
}

pub struct RenderPipeline {
    pub framebuffer: ScanoutFramebuffer,
    pub wallpaper_tint: Oklab,
    pub boot_crossfade_opacity: f32,
}

impl RenderPipeline {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            framebuffer: ScanoutFramebuffer::new(width, height),
            wallpaper_tint: Oklab::new(0.55, 0.12, 0.08), // Space Orange tint
            boot_crossfade_opacity: 0.0,
        }
    }


    /// Executes the 7-layer compositing pass for the active frame.
    pub fn render_frame(
        &mut self,
        windows: &[RenderWindow],
        dock_item_count: usize,
        is_control_center_open: bool,
        is_pathfinder_open: bool,
        active_app_title: &str,
    ) {
        // ── Layer 0: Wallpaper ───────────────────────────────────────────────
        self.framebuffer.clear(0xFF141416); // Deep space background

        // ── Layer 1: Window Shadows (Spring-lagged) ─────────────────────────
        for win in windows.iter().filter(|w| w.is_visible) {
            self.framebuffer.draw_window_shadow(
                win.shadow_x,
                win.shadow_y,
                win.width,
                win.height,
                win.corner_radius,
            );
        }

        // ── Layer 2: Window Glass Backgrounds ───────────────────────────────
        for win in windows.iter().filter(|w| w.is_visible) {
            // Apply Kawase glass blur over background
            self.framebuffer.apply_kawase_glass_blur(
                win.x as i32,
                win.y as i32,
                win.width as u32,
                win.height as u32,
                win.altitude,
                self.wallpaper_tint,
            );

            // Draw window squircle shell
            let fill = if win.is_focused { 0xD8242428 } else { 0xB01C1C20 };
            let border = if win.is_focused { 0x50FFFFFF } else { 0x20FFFFFF };
            self.framebuffer.draw_squircle_surface(
                win.x,
                win.y,
                win.width,
                win.height,
                win.corner_radius,
                fill,
                border,
                1.0,
            );
        }

        // ── Layer 3: Window Content ─────────────────────────────────────────
        // (Handled via scanout blit / xdg client buffer commitments)

        // ── Layer 4: Shell Surfaces (Panel & Dock) ──────────────────────────
        self.render_top_panel(active_app_title);
        self.render_floating_dock(dock_item_count);

        // ── Layer 5: Boot Crossfade (Space Orange Overlay) ───────────────────
        if self.boot_crossfade_opacity > 0.001 {
            let alpha = ((self.boot_crossfade_opacity * 255.0) as u32).min(255);
            let orange_splash = (alpha << 24) | 0x00E85D00;
            self.framebuffer.draw_squircle_surface(
                0.0,
                0.0,
                self.framebuffer.width as f32,
                self.framebuffer.height as f32,
                0.0,
                orange_splash,
                0x0,
                0.0,
            );
        }

        // ── Layer 6: Floating Overlays ──────────────────────────────────────
        if is_pathfinder_open {
            self.render_pathfinder_overlay();
        }

        if is_control_center_open {
            self.render_control_center_overlay();
        }
    }

    fn render_top_panel(&mut self, _active_app: &str) {
        let panel_height = 32.0;
        let w = self.framebuffer.width as f32;

        // Frosted glass panel with 1px bottom border
        self.framebuffer.draw_squircle_surface(
            0.0,
            0.0,
            w,
            panel_height,
            0.0,
            0xCC18181A,
            0x25FFFFFF,
            1.0,
        );
    }

    fn render_floating_dock(&mut self, item_count: usize) {
        let dock_h = 64.0;
        let item_w = 56.0;
        let dock_w = (item_count as f32 * item_w + 32.0).max(180.0);
        let screen_w = self.framebuffer.width as f32;
        let screen_h = self.framebuffer.height as f32;

        let dock_x = (screen_w - dock_w) * 0.5;
        let dock_y = screen_h - dock_h - 16.0; // 16px floating bottom margin

        // Dock warm shadow
        self.framebuffer.draw_window_shadow(dock_x, dock_y + 4.0, dock_w, dock_h, 16.0);

        // Apply Kawase glass blur on dock background
        self.framebuffer.apply_kawase_glass_blur(
            dock_x as i32,
            dock_y as i32,
            dock_w as u32,
            dock_h as u32,
            SurfaceAltitude::High,
            self.wallpaper_tint,
        );

        // Dock frosted capsule container
        self.framebuffer.draw_squircle_surface(
            dock_x,
            dock_y,
            dock_w,
            dock_h,
            16.0,
            0xD01E1E22,
            0x40FFFFFF,
            1.0,
        );
    }

    fn render_pathfinder_overlay(&mut self) {
        let w = 680.0;
        let h = 420.0;
        let x = (self.framebuffer.width as f32 - w) * 0.5;
        let y = (self.framebuffer.height as f32 - h) * 0.35;

        // Shadow & Blur
        self.framebuffer.draw_window_shadow(x, y + 8.0, w, h, 20.0);
        self.framebuffer.apply_kawase_glass_blur(
            x as i32,
            y as i32,
            w as u32,
            h as u32,
            SurfaceAltitude::Floating,
            self.wallpaper_tint,
        );

        // Pathfinder glass capsule
        self.framebuffer.draw_squircle_surface(
            x,
            y,
            w,
            h,
            20.0,
            0xEB1E1E24,
            0x60FFFFFF,
            1.5,
        );
    }

    fn render_control_center_overlay(&mut self) {
        let w = 340.0;
        let h = 480.0;
        let x = self.framebuffer.width as f32 - w - 16.0;
        let y = 40.0; // below top panel

        self.framebuffer.draw_window_shadow(x, y + 6.0, w, h, 18.0);
        self.framebuffer.apply_kawase_glass_blur(
            x as i32,
            y as i32,
            w as u32,
            h as u32,
            SurfaceAltitude::High,
            self.wallpaper_tint,
        );

        self.framebuffer.draw_squircle_surface(
            x,
            y,
            w,
            h,
            18.0,
            0xE01C1C20,
            0x40FFFFFF,
            1.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_pipeline_7_layers() {
        let mut pipeline = RenderPipeline::new(1920, 1080);
        let windows = vec![RenderWindow {
            id: 1,
            title: "Filer".to_string(),
            x: 100.0,
            y: 100.0,
            width: 800.0,
            height: 600.0,
            shadow_x: 100.0,
            shadow_y: 108.0,
            corner_radius: 14.0,
            altitude: SurfaceAltitude::Mid,
            is_visible: true,
            is_focused: true,
        }];

        pipeline.render_frame(&windows, 5, true, true, "Filer");
        assert_ne!(pipeline.framebuffer.get_pixel(150, 150), 0x0);
    }
}
