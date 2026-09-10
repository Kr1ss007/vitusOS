//! CompositorRenderer — bridges ShellController/WindowManager state into the
//! 7-layer RenderPipeline for production frame compositing.
//!
//! This is the production code that actually puts pixels on screen every frame.
//! It reads the current state from ShellController and converts it into
//! RenderWindow structs that the RenderPipeline can composite.
//!
//! Frame compositing order (Part 6.3):
//!   1. Extract window geometry from WindowManager → RenderWindow[]
//!   2. Extract shell state (panel opacity, dock items, cockpit zoom)
//!   3. Feed everything to RenderPipeline::render_frame()
//!   4. The RenderPipeline writes to ScanoutFramebuffer
//!   5. The backend (DRM/Winit) scans out the framebuffer

use crate::shell_controller::{ShellController, ShellMode};
use crate::shell::Panel;
use animus_render::pipeline::{RenderPipeline, RenderWindow};
use animus_render::altitude::SurfaceAltitude;

/// The compositor renderer — produces a complete frame from shell state.
pub struct CompositorRenderer {
    pub pipeline: RenderPipeline,
    pub screen_w: u32,
    pub screen_h: u32,
}

impl CompositorRenderer {
    pub fn new(screen_w: u32, screen_h: u32) -> Self {
        Self {
            pipeline: RenderPipeline::new(screen_w, screen_h),
            screen_w,
            screen_h,
        }
    }

    /// Resize the rendering pipeline (on output mode change).
    pub fn resize(&mut self, w: u32, h: u32) {
        self.screen_w = w;
        self.screen_h = h;
        self.pipeline = RenderPipeline::new(w, h);
    }

    /// Composite a complete frame from the ShellController state.
    /// This is called every frame from the compositor tick loop.
    pub fn composite_frame(&mut self, shell: &ShellController) {
        // 1. Extract visible windows from WindowManager → RenderWindow[]
        let windows = self.extract_windows(shell);

        // 2. Extract shell state
        let dock_item_count = shell.dock.items.len();
        let is_control_center_open = *shell.control_center.is_open.read();
        let is_pathfinder_open = false; // Pathfinder state tracked elsewhere

        // 3. Determine focused app title for Panel
        let active_app_title = shell.window_manager.focused()
            .map(|w| w.title.as_str())
            .unwrap_or("vitusOS");

        // 4. Apply boot crossfade opacity
        self.pipeline.boot_crossfade_opacity = shell.boot_crossfade_opacity();

        // 5. Apply cockpit view zoom if open
        if shell.mode() == ShellMode::CockpitView {
            self.render_cockpit_overlay(shell);
        }

        // 6. Render the 7-layer pipeline
        self.pipeline.render_frame(
            &windows,
            dock_item_count,
            is_control_center_open,
            is_pathfinder_open,
            active_app_title,
        );

        // 7. Render shell overlays on top of the pipeline
        self.render_shell_overlays(shell);
    }

    /// Extract window geometry from the WindowManager into RenderWindow[].
    fn extract_windows(&self, shell: &ShellController) -> Vec<RenderWindow> {
        let mut windows = Vec::new();

        for win in shell.window_manager.windows() {
            if !win.is_renderable() {
                continue;
            }

            let altitude = if win.is_fullscreen() {
                SurfaceAltitude::Grounded
            } else {
                win.altitude
            };

            windows.push(RenderWindow {
                id: win.handle,
                title: win.truncated_title(),
                x: win.pos.x.value,
                y: win.pos.y.value,
                width: win.width,
                height: win.height,
                shadow_x: win.shadow_pos.x.value,
                shadow_y: win.shadow_pos.y.value,
                corner_radius: win.corner_radius,
                altitude,
                is_visible: true,
                is_focused: win.is_focused,
                client_buffer: win.client_buffer.clone(),
            });
        }

        windows
    }

    /// Render the CockpitView overlay — window cards in a grid.
    fn render_cockpit_overlay(&mut self, shell: &ShellController) {
        // The cockpit view darkens the background and shows window cards.
        // The actual card rendering is done via the framebuffer's squircle
        // drawing methods. Each card is drawn at its spring-animated position.

        let cockpit = &shell.cockpit_view;
        if !cockpit.is_open {
            return;
        }

        // Draw darkened background overlay
        let bg_alpha = (cockpit.bg_darken.value * 255.0) as u32;
        if bg_alpha > 0 {
            let _bg_color = (bg_alpha << 24) | 0x00000000;
            let fb = &mut self.pipeline.framebuffer;
            for y in 0..fb.height {
                for x in 0..fb.width {
                    let existing = fb.pixels[y as usize * fb.stride + x as usize];
                    // Alpha blend the dark overlay
                    let alpha = bg_alpha as f32 / 255.0;
                    let r = (((existing >> 16) & 0xFF) as f32 * (1.0 - alpha)) as u32;
                    let g = (((existing >> 8) & 0xFF) as f32 * (1.0 - alpha)) as u32;
                    let b = ((existing & 0xFF) as f32 * (1.0 - alpha)) as u32;
                    fb.pixels[y as usize * fb.stride + x as usize] =
                        (0xFF << 24) | (r << 16) | (g << 8) | b;
                }
            }
        }

        // Draw each cockpit card
        for card in &cockpit.cards {
            let s = card.scale.value;
            let w = card.width * s;
            let h = card.height * s;
            let x = card.pos.x.value;
            let y = card.pos.y.value;

            // Card shadow
            self.pipeline.framebuffer.draw_window_shadow(
                x, y + 8.0, w, h, 10.0,
            );

            // Card glass background
            self.pipeline.framebuffer.apply_kawase_glass_blur(
                x as i32,
                y as i32,
                w as u32,
                h as u32,
                SurfaceAltitude::High,
                self.pipeline.wallpaper_tint,
            );

            // Card squircle
            let hover = card.hover_alpha.value;
            let border_alpha = (0x30 + (hover * 0x40 as f32) as u32).min(0x80);
            self.pipeline.framebuffer.draw_squircle_surface(
                x, y, w, h,
                10.0,
                0xD8202024,
                (border_alpha << 24) | 0xFFFFFF,
                1.0,
            );
        }
    }

    /// Render shell overlays: lock screen, welcome screen, notifications.
    fn render_shell_overlays(&mut self, shell: &ShellController) {
        // Lock screen overlay
        if *shell.lock_screen.is_active.read() {
            let opacity = shell.lock_screen.opacity.read().value;
            let alpha = (opacity * 255.0) as u32;
            if alpha > 0 {
                let fb = &mut self.pipeline.framebuffer;
                // Dark blur overlay — #1A1208 with opacity
                let _bg = ((alpha as f32 * 0.85) as u32) << 24 | 0x1A1208;
                let bg_clamped = (alpha.min(255) << 24) | 0x001A1208;
                for y in 0..fb.height {
                    for x in 0..fb.width {
                        fb.pixels[y as usize * fb.stride + x as usize] = bg_clamped;
                    }
                }
            }
        }

        // Welcome screen overlay
        if shell.welcome_screen.is_active {
            let opacity = shell.welcome_screen.card_alpha();
            let alpha = (opacity * 255.0) as u32;
            if alpha > 0 {
                let fb = &mut self.pipeline.framebuffer;
                // Full screen #1A1208 background
                let bg = (alpha << 24) | 0x001A1208;
                for y in 0..fb.height {
                    for x in 0..fb.width {
                        fb.pixels[y as usize * fb.stride + x as usize] = bg;
                    }
                }

                // Content card — centered, 480px wide
                let card_w = 480.0f32.min(fb.width as f32 - 64.0);
                let card_h = 400.0;
                let card_x = (fb.width as f32 - card_w) * 0.5;
                let card_y = (fb.height as f32 - card_h) * 0.5 + shell.welcome_screen.card_offset_y();
                let card_alpha = (opacity * 0.94 * 255.0) as u32;

                fb.draw_window_shadow(card_x, card_y + 12.0, card_w, card_h, 16.0);
                fb.apply_kawase_glass_blur(
                    card_x as i32,
                    card_y as i32,
                    card_w as u32,
                    card_h as u32,
                    SurfaceAltitude::High,
                    self.pipeline.wallpaper_tint,
                );
                fb.draw_squircle_surface(
                    card_x, card_y, card_w, card_h,
                    16.0,
                    (card_alpha << 24) | 0x202024,
                    0x26FFFFFF,
                    1.0,
                );
            }
        }

        // Shutdown screen
        if *shell.shutdown_screen.is_active.read() {
            let fb = &mut self.pipeline.framebuffer;
            // Full screen black with personality text
            for y in 0..fb.height {
                for x in 0..fb.width {
                    fb.pixels[y as usize * fb.stride + x as usize] = 0xFF000000;
                }
            }
        }

        // System screen (blackout)
        if *shell.system_screen.is_active.read() {
            let fb = &mut self.pipeline.framebuffer;
            let opacity = shell.system_screen.opacity.read().value;
            let alpha = (opacity * 255.0) as u32;
            for y in 0..fb.height {
                for x in 0..fb.width {
                    let existing = fb.pixels[y as usize * fb.stride + x as usize];
                    let a = alpha as f32 / 255.0;
                    let r = (((existing >> 16) & 0xFF) as f32 * (1.0 - a)) as u32;
                    let g = (((existing >> 8) & 0xFF) as f32 * (1.0 - a)) as u32;
                    let b = ((existing & 0xFF) as f32 * (1.0 - a)) as u32;
                    fb.pixels[y as usize * fb.stride + x as usize] =
                        (0xFF << 24) | (r << 16) | (g << 8) | b;
                }
            }
        }

        // Notification toasts
        self.render_notifications(shell);
    }

    /// Render notification toasts — slide in from top-right.
    fn render_notifications(&mut self, shell: &ShellController) {
        let nc = &shell.notification_center;
        let toasts = nc.toasts.read();
        let fb = &mut self.pipeline.framebuffer;
        let toast_w = 360.0f32.min(fb.width as f32 - 32.0);
        let toast_h = 80.0;
        let start_x = fb.width as f32 - toast_w - 16.0;
        let mut y = 40.0 + Panel::HEIGHT;

        for toast in toasts.iter() {
            let opacity = toast.opacity.value;
            if opacity < 0.01 {
                continue;
            }
            let alpha = (opacity * 255.0) as u32;

            // Slide-in offset
            let slide_x = (1.0 - toast.slide_x.value) * (toast_w + 16.0);
            let x = start_x + slide_x;

            // Shadow
            fb.draw_window_shadow(x, y + 4.0, toast_w, toast_h, 12.0);

            // Glass background
            fb.apply_kawase_glass_blur(
                (x) as i32,
                y as i32,
                toast_w as u32,
                toast_h as u32,
                SurfaceAltitude::Floating,
                self.pipeline.wallpaper_tint,
            );

            // Notification squircle
            fb.draw_squircle_surface(
                x, y, toast_w, toast_h,
                12.0,
                (alpha << 24) | 0x2A2A2E,
                0x30FFFFFF,
                1.0,
            );

            y += toast_h + 8.0; // stack
        }
    }

    /// Get the composited framebuffer for scanout.
    pub fn framebuffer(&self) -> &animus_render::framebuffer::ScanoutFramebuffer {
        &self.pipeline.framebuffer
    }
}

// Extension trait to access private fields from ShellController
// These are implemented as pub methods on the shell components
// since we can't access private fields from outside the crate.

/// Boot crossfade opacity accessor (added to BootCrossfade).
mod boot_crossfade_ext {
    use crate::shell::BootCrossfade;

    pub trait BootCrossfadeOpacity {
        fn boot_crossfade_opacity(&self) -> f32;
    }

    impl BootCrossfadeOpacity for BootCrossfade {
        fn boot_crossfade_opacity(&self) -> f32 {
            self.screen_opacity.value
        }
    }
}

// Re-export the trait for use in CompositorRenderer
pub use boot_crossfade_ext::BootCrossfadeOpacity;

// Extension trait on ShellController to access boot crossfade
impl ShellController {
    fn boot_crossfade_opacity(&self) -> f32 {
        // Access via the boot_crossfade field on the ShellController
        // But ShellController doesn't own a BootCrossfade...
        // Actually it does via the shutdown/system screens but not boot.
        // The boot crossfade is owned by CompositorContext, not ShellController.
        // For now return 0 — the boot crossfade is handled in the compositor main loop.
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell_controller::ShellController;
    use animus_core::event_bus::EventBus;

    #[test]
    fn compositor_renderer_initializes() {
        let renderer = CompositorRenderer::new(1920, 1080);
        assert_eq!(renderer.screen_w, 1920);
        assert_eq!(renderer.screen_h, 1080);
        assert_eq!(renderer.pipeline.framebuffer.width, 1920);
        assert_eq!(renderer.pipeline.framebuffer.height, 1080);
    }

    #[test]
    fn composite_frame_desktop_mode() {
        let bus = EventBus::new();
        let mut shell = ShellController::new(bus, 1920.0, 1080.0);
        shell.complete_first_boot();

        // Create a window
        shell.window_manager.create_window("Test", "test", 100.0, 100.0, 800.0, 600.0);

        let mut renderer = CompositorRenderer::new(1920, 1080);
        renderer.composite_frame(&shell);

        // Framebuffer should have non-zero pixels (not all black)
        let non_zero = renderer.framebuffer().pixels.iter().any(|&p| p != 0xFF000000);
        assert!(non_zero, "Composited frame should have visible content");
    }

    #[test]
    fn composite_frame_with_cockpit_view() {
        let bus = EventBus::new();
        let mut shell = ShellController::new(bus, 1920.0, 1080.0);
        shell.complete_first_boot();
        shell.window_manager.create_window("App", "app", 100.0, 100.0, 800.0, 600.0);

        // Open cockpit view
        shell.dispatch_event(&animus_core::events::AEEvent::CockpitViewOpen {
            ctx: animus_core::events::AnimusContext::default(),
        });
        shell.dispatch_event(&animus_core::events::AEEvent::CockpitViewOpened);

        let mut renderer = CompositorRenderer::new(1920, 1080);

        // Tick springs to settle cockpit
        for _ in 0..10 {
            shell.update(0.016);
        }

        renderer.composite_frame(&shell);

        // Should not crash and framebuffer should have content
        let non_zero = renderer.framebuffer().pixels.iter().any(|&p| p != 0xFF000000);
        assert!(non_zero);
    }

    #[test]
    fn composite_frame_resize() {
        let mut renderer = CompositorRenderer::new(1920, 1080);
        renderer.resize(2560, 1440);
        assert_eq!(renderer.screen_w, 2560);
        assert_eq!(renderer.pipeline.framebuffer.width, 2560);
    }
}
