//! Real Production Framebuffer & Direct Scanout Rasterizer.
//!
//! Provides bare-metal 32-bit (XRGB8888 / RGBA8888) scanout buffers,
//! multi-pass Kawase blur filtering, superellipse squircle clipping,
//! OKLab luminosity adjustments, and dual SDF shadow rasterization.

use crate::altitude::SurfaceAltitude;
use crate::color::Oklab;
use crate::squircle::SquircleParams;
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width as i32
            && self.x + self.width as i32 > other.x
            && self.y < other.y + other.height as i32
            && self.y + self.height as i32 > other.y
    }
}

#[derive(Debug, Clone)]
pub struct ScanoutFramebuffer {
    pub width: u32,
    pub height: u32,
    pub stride: usize, // in pixels (pitch / 4)
    pub pixels: Vec<u32>, // 0xAARRGGBB format (matches DRM_FORMAT_XRGB8888 / ARGB8888)
    pub damage_rects: Vec<Rect>,
}

impl ScanoutFramebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let stride = width as usize;
        let total_pixels = stride * height as usize;
        Self {
            width,
            height,
            stride,
            pixels: vec![0xFF000000; total_pixels],
            damage_rects: vec![Rect::new(0, 0, width, height)],
        }
    }

    #[inline(always)]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width as usize && y < self.height as usize {
            self.pixels[y * self.stride + x] = color;
        }
    }

    #[inline(always)]
    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        if x < self.width as usize && y < self.height as usize {
            self.pixels[y * self.stride + x]
        } else {
            0
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
        self.damage_rects.push(Rect::new(0, 0, self.width, self.height));
    }

    /// Blends a source pixel with alpha over destination using pre-multiplied alpha math
    #[inline(always)]
    pub fn blend_pixel(&mut self, x: usize, y: usize, src_color: u32) {
        if x >= self.width as usize || y >= self.height as usize {
            return;
        }
        let sa = ((src_color >> 24) & 0xFF) as u32;
        if sa == 0 {
            return;
        }
        if sa == 255 {
            self.pixels[y * self.stride + x] = src_color;
            return;
        }

        let idx = y * self.stride + x;
        let dst = self.pixels[idx];

        let sr = (src_color >> 16) & 0xFF;
        let sg = (src_color >> 8) & 0xFF;
        let sb = src_color & 0xFF;

        let dr = (dst >> 16) & 0xFF;
        let dg = (dst >> 8) & 0xFF;
        let db = dst & 0xFF;
        let da = (dst >> 24) & 0xFF;

        let inv_a = 255 - sa;
        let out_r = ((sr * sa + dr * inv_a) / 255).min(255);
        let out_g = ((sg * sa + dg * inv_a) / 255).min(255);
        let out_b = ((sb * sa + db * inv_a) / 255).min(255);
        let out_a = (sa + (da * inv_a) / 255).min(255);

        self.pixels[idx] = (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
    }

    /// Draws an anti-aliased G2 continuous superellipse (squircle) surface
    pub fn draw_squircle_surface(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        fill_color: u32,
        border_color: u32,
        border_width: f32,
    ) {
        let mut squircle = SquircleParams::for_window(width, height);
        squircle.corner_radius = radius;

        let min_x = (x.floor() as i32).max(0) as usize;
        let max_x = ((x + width).ceil() as i32).min(self.width as i32) as usize;
        let min_y = (y.floor() as i32).max(0) as usize;
        let max_y = ((y + height).ceil() as i32).min(self.height as i32) as usize;

        let half_w = width * 0.5;
        let half_h = height * 0.5;
        let center_x = x + half_w;
        let center_y = y + half_h;

        let fa = ((fill_color >> 24) & 0xFF) as f32 / 255.0;
        let fr = ((fill_color >> 16) & 0xFF) as f32;
        let fg = ((fill_color >> 8) & 0xFF) as f32;
        let fb = (fill_color & 0xFF) as f32;

        let br = ((border_color >> 16) & 0xFF) as f32;
        let bg = ((border_color >> 8) & 0xFF) as f32;
        let bb = (border_color & 0xFF) as f32;

        for py in min_y..max_y {
            let ly = py as f32 - center_y;
            for px in min_x..max_x {
                let lx = px as f32 - center_x;
                let d = squircle.signed_distance(Vec2::new(lx, ly));

                if d > 1.0 {
                    continue; // fully outside
                }

                // Anti-aliased outer edge coverage (smoothstep -0.5 to 0.5)
                let outer_alpha = (1.0 - (d + 0.5).clamp(0.0, 1.0)) * fa;

                if outer_alpha <= 0.001 {
                    continue;
                }

                let mut r = fr;
                let mut g = fg;
                let mut b = fb;

                if border_width > 0.0 {
                    let inner_d = d + border_width;
                    let inner_factor: f32 = (1.0 - (inner_d + 0.5).clamp(0.0, 1.0)).clamp(0.0, 1.0);
                    r = br * (1.0 - inner_factor) + fr * inner_factor;
                    g = bg * (1.0 - inner_factor) + fg * inner_factor;
                    b = bb * (1.0 - inner_factor) + fb * inner_factor;
                }

                // Frosted top highlight catch (1px at 8% white)
                let rel_y = py as f32 - y;
                if rel_y >= 0.0 && rel_y <= 1.5 {
                    let top_hi = (1.0 - (rel_y / 1.5)).clamp(0.0, 1.0) * 20.0;
                    r = (r + top_hi).min(255.0);
                    g = (g + top_hi).min(255.0);
                    b = (b + top_hi).min(255.0);
                }

                let pixel_color = (((outer_alpha * 255.0) as u32) << 24)
                    | ((r as u32) << 16)
                    | ((g as u32) << 8)
                    | (b as u32);

                self.blend_pixel(px, py, pixel_color);
            }
        }
    }

    /// Renders warm dual SDF window shadow (#1A1208) with spring-lagged position
    pub fn draw_window_shadow(
        &mut self,
        shadow_x: f32,
        shadow_y: f32,
        width: f32,
        height: f32,
        radius: f32,
    ) {
        let margin = 60.0; // shadow spread margin
        let min_x = ((shadow_x - margin).floor() as i32).max(0) as usize;
        let max_x = ((shadow_x + width + margin).ceil() as i32).min(self.width as i32) as usize;
        let min_y = ((shadow_y - margin).floor() as i32).max(0) as usize;
        let max_y = ((shadow_y + height + margin).ceil() as i32).min(self.height as i32) as usize;

        let mut squircle = SquircleParams::for_window(width, height);
        squircle.corner_radius = radius;

        let center_x = shadow_x + width * 0.5;
        let center_y = shadow_y + height * 0.5;

        // Warm dark: #1A1208 -> rgb(26, 18, 8)
        let sr = 26u32;
        let sg = 18u32;
        let sb = 8u32;

        for py in min_y..max_y {
            let ly = py as f32 - center_y;
            for px in min_x..max_x {
                let lx = px as f32 - center_x;
                let d = squircle.signed_distance(Vec2::new(lx, ly));

                if d <= 0.0 {
                    continue; // inside window rect
                }

                // Ambient shadow: spread 60px, soft blur 40px, 18% peak
                let ambient = (-d / 40.0).exp() * 0.18;
                // Contact shadow: tight 8px, grounds bottom edge 12% peak
                let contact = (-d / 8.0).exp() * 0.12;
                let shadow_intensity = (ambient + contact).clamp(0.0, 1.0);

                if shadow_intensity <= 0.002 {
                    continue;
                }

                let alpha = (shadow_intensity * 255.0) as u32;
                let shadow_pixel = (alpha << 24) | (sr << 16) | (sg << 8) | sb;
                self.blend_pixel(px, py, shadow_pixel);
            }
        }
    }

    /// Performs multi-pass Kawase blur over a designated region with OKLab luminosity boost
    pub fn apply_kawase_glass_blur(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        altitude: SurfaceAltitude,
        tint: Oklab,
    ) {

        let (passes, radius) = match altitude {
            SurfaceAltitude::Grounded => return,
            SurfaceAltitude::Low => (4, 8.0f32),
            SurfaceAltitude::Mid => (4, 20.0f32),
            SurfaceAltitude::High => (8, 32.0f32),
            SurfaceAltitude::Floating => (8, 48.0f32),
        };

        let min_x = x.max(0) as usize;
        let max_x = (x + width as i32).min(self.width as i32) as usize;
        let min_y = y.max(0) as usize;
        let max_y = (y + height as i32).min(self.height as i32) as usize;

        if min_x >= max_x || min_y >= max_y {
            return;
        }

        // Extract sub-region
        let mut buf_a = Vec::with_capacity((max_x - min_x) * (max_y - min_y));
        for py in min_y..max_y {
            for px in min_x..max_x {
                buf_a.push(self.get_pixel(px, py));
            }
        }

        let sub_w = max_x - min_x;
        let sub_h = max_y - min_y;
        let mut buf_b = vec![0u32; sub_w * sub_h];

        // 4 or 8 Kawase blur iterations (0.5, 1.5, 2.5, 3.5 ...)
        for p in 0..passes {
            let iter_offset = (p as f32 + 0.5) * (radius / 16.0);
            let off = (iter_offset.round() as i32).max(1) as usize;

            for row in 0..sub_h {
                for col in 0..sub_w {
                    let y0 = if row >= off { row - off } else { 0 };
                    let y1 = (row + off).min(sub_h - 1);
                    let x0 = if col >= off { col - off } else { 0 };
                    let x1 = (col + off).min(sub_w - 1);

                    let c0 = buf_a[y0 * sub_w + x0];
                    let c1 = buf_a[y0 * sub_w + x1];
                    let c2 = buf_a[y1 * sub_w + x0];
                    let c3 = buf_a[y1 * sub_w + x1];

                    let r = (((c0 >> 16) & 0xFF) + ((c1 >> 16) & 0xFF) + ((c2 >> 16) & 0xFF) + ((c3 >> 16) & 0xFF)) >> 2;
                    let g = (((c0 >> 8) & 0xFF) + ((c1 >> 8) & 0xFF) + ((c2 >> 8) & 0xFF) + ((c3 >> 8) & 0xFF)) >> 2;
                    let b = ((c0 & 0xFF) + (c1 & 0xFF) + (c2 & 0xFF) + (c3 & 0xFF)) >> 2;

                    buf_b[row * sub_w + col] = (0xFF << 24) | (r << 16) | (g << 8) | b;
                }
            }
            std::mem::swap(&mut buf_a, &mut buf_b);
        }

        // Apply OKLab luminosity boost & tint back to framebuffer
        let (tint_r, tint_g, tint_b) = tint.to_srgb();
        let tint_strength = 0.15f32;

        for (i, &blurred_pix) in buf_a.iter().enumerate() {
            let row = min_y + i / sub_w;
            let col = min_x + i % sub_w;

            let br = ((blurred_pix >> 16) & 0xFF) as f32;
            let bg = ((blurred_pix >> 8) & 0xFF) as f32;
            let bb = (blurred_pix & 0xFF) as f32;

            // Boost luminosity slightly + mix with ambient tint
            let out_r = ((br * 1.06).min(255.0) * (1.0 - tint_strength) + (tint_r as f32 * 255.0) * tint_strength).min(255.0) as u32;
            let out_g = ((bg * 1.06).min(255.0) * (1.0 - tint_strength) + (tint_g as f32 * 255.0) * tint_strength).min(255.0) as u32;
            let out_b = ((bb * 1.06).min(255.0) * (1.0 - tint_strength) + (tint_b as f32 * 255.0) * tint_strength).min(255.0) as u32;

            self.set_pixel(col, row, (0xFF << 24) | (out_r << 16) | (out_g << 8) | out_b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanout_framebuffer_lifecycle() {
        let mut fb = ScanoutFramebuffer::new(640, 480);
        assert_eq!(fb.pixels.len(), 640 * 480);

        fb.clear(0xFF141416);
        assert_eq!(fb.get_pixel(0, 0), 0xFF141416);

        fb.draw_squircle_surface(10.0, 10.0, 100.0, 100.0, 16.0, 0xFF222226, 0xFFFFFFFF, 1.0);
        assert_ne!(fb.get_pixel(50, 50), 0xFF141416);
    }
}
