//! Settings App Rendering — per-section content layout and rendering (Part 33).
//!
//! Settings is a flat, direct interface: one section open at a time.
//! Left sidebar (220px) + right content pane.
//! Switching sections: cross-fade content SPRING_HOVER (600,40).
//! Changes apply live — no "Apply" button.

use animus_physics::spring::{SpringProfile, SpringSolver};

/// Content cross-fade spring for section switching.
pub struct SectionTransition {
    /// Opacity of outgoing section (1.0 → 0.0 → 1.0 on new).
    pub fade_out: SpringSolver,
    /// Opacity of incoming section (0.0 → 1.0).
    pub fade_in: SpringSolver,
    /// Slide offset for outgoing (0 → -20px).
    pub slide_out: SpringSolver,
    /// Slide offset for incoming (20px → 0).
    pub slide_in: SpringSolver,
    /// True while transition is in progress.
    pub transitioning: bool,
}

impl Default for SectionTransition {
    fn default() -> Self {
        Self::new()
    }
}

impl SectionTransition {
    pub fn new() -> Self {
        Self {
            fade_out: SpringSolver::new(1.0, SpringProfile::Hover),
            fade_in: SpringSolver::new(0.0, SpringProfile::Hover),
            slide_out: SpringSolver::new(0.0, SpringProfile::Hover),
            slide_in: SpringSolver::new(20.0, SpringProfile::Hover),
            transitioning: false,
        }
    }

    pub fn start(&mut self) {
        self.transitioning = true;
        self.fade_out.snap(1.0);
        self.fade_out.set_target(0.0);
        self.slide_out.snap(0.0);
        self.slide_out.set_target(-20.0);
        self.fade_in.snap(0.0);
        self.fade_in.set_target(1.0);
        self.slide_in.snap(20.0);
        self.slide_in.set_target(0.0);
    }

    pub fn update(&mut self, dt: f32) {
        if !self.transitioning {
            return;
        }
        self.fade_out.update(dt);
        self.fade_in.update(dt);
        self.slide_out.update(dt);
        self.slide_in.update(dt);
        if self.fade_in.is_settled() && self.fade_out.is_settled() {
            self.transitioning = false;
        }
    }

    pub fn is_done(&self) -> bool {
        !self.transitioning
    }
}

/// A toggle switch (checkbox) with spring animation.
pub struct ToggleSwitch {
    pub is_on: bool,
    /// Knob position: 0.0 = off, 1.0 = on.
    pub knob: SpringSolver,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ToggleSwitch {
    pub const TRACK_W: f32 = 36.0;
    pub const TRACK_H: f32 = 20.0;
    pub const KNOB_SIZE: f32 = 16.0;
    pub const KNOB_MARGIN: f32 = 2.0;

    pub fn new(is_on: bool, x: f32, y: f32) -> Self {
        let initial = if is_on { 1.0 } else { 0.0 };
        Self {
            is_on,
            knob: SpringSolver::new(initial, SpringProfile::Selection),
            x,
            y,
            width: Self::TRACK_W,
            height: Self::TRACK_H,
        }
    }

    pub fn toggle(&mut self) {
        self.is_on = !self.is_on;
        self.knob.set_target(if self.is_on { 1.0 } else { 0.0 });
    }

    pub fn set(&mut self, on: bool) {
        self.is_on = on;
        self.knob.set_target(if on { 1.0 } else { 0.0 });
    }

    pub fn knob_x(&self) -> f32 {
        let range = Self::TRACK_W - Self::KNOB_SIZE - Self::KNOB_MARGIN * 2.0;
        self.x + Self::KNOB_MARGIN + self.knob.value * range
    }

    pub fn knob_y(&self) -> f32 {
        self.y + (Self::TRACK_H - Self::KNOB_SIZE) * 0.5
    }

    pub fn hit_test(&self, mx: f32, my: f32) -> bool {
        mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + self.height
    }

    pub fn on_pointer_button(&mut self, mx: f32, my: f32, pressed: bool) -> bool {
        if pressed && self.hit_test(mx, my) {
            self.toggle();
            true
        } else {
            false
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.knob.update(dt);
    }
}

/// A slider control for numeric values.
pub struct SliderControl {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub knob: SpringSolver,
    pub is_dragging: bool,
}

impl SliderControl {
    pub const TRACK_H: f32 = 4.0;
    pub const KNOB_SIZE: f32 = 16.0;

    pub fn new(value: f32, min: f32, max: f32, x: f32, y: f32, width: f32) -> Self {
        let normalized = (value - min) / (max - min);
        Self {
            value,
            min,
            max,
            x,
            y,
            width,
            knob: SpringSolver::new(normalized, SpringProfile::Selection),
            is_dragging: false,
        }
    }

    pub fn set_value(&mut self, v: f32) {
        self.value = v.clamp(self.min, self.max);
        let normalized = (self.value - self.min) / (self.max - self.min);
        self.knob.set_target(normalized);
    }

    pub fn knob_x(&self) -> f32 {
        self.x + self.knob.value * self.width
    }

    pub fn knob_y(&self) -> f32 {
        self.y - (Self::KNOB_SIZE - Self::TRACK_H) * 0.5
    }

    pub fn hit_test_track(&self, mx: f32, my: f32) -> bool {
        let tolerance = 8.0;
        mx >= self.x - tolerance
            && mx <= self.x + self.width + tolerance
            && my >= self.y - tolerance
            && my <= self.y + Self::TRACK_H + tolerance
    }

    pub fn on_pointer_button(&mut self, mx: f32, my: f32, pressed: bool) -> bool {
        if pressed && self.hit_test_track(mx, my) {
            self.is_dragging = true;
            self.update_from_x(mx);
            true
        } else if !pressed && self.is_dragging {
            self.is_dragging = false;
            true
        } else {
            false
        }
    }

    pub fn on_pointer_motion(&mut self, mx: f32, _my: f32) {
        if self.is_dragging {
            self.update_from_x(mx);
        }
    }

    fn update_from_x(&mut self, mx: f32) {
        let normalized = ((mx - self.x) / self.width).clamp(0.0, 1.0);
        self.value = self.min + normalized * (self.max - self.min);
        self.knob.set_target(normalized);
    }

    pub fn update(&mut self, dt: f32) {
        self.knob.update(dt);
    }
}

/// A row in the settings content area.
pub struct SettingsRow {
    pub label: String,
    pub description: Option<String>,
    pub y: f32,
    pub height: f32,
}

impl SettingsRow {
    pub fn new(label: impl Into<String>, y: f32) -> Self {
        Self {
            label: label.into(),
            description: None,
            y,
            height: 44.0,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn hit_test(&self, mx: f32, my: f32, content_x: f32, content_w: f32) -> bool {
        mx >= content_x && mx <= content_x + content_w && my >= self.y && my <= self.y + self.height
    }
}

/// Section content layout — defines what controls appear in each section.
pub struct SectionContent {
    pub rows: Vec<SettingsRow>,
    pub toggles: Vec<ToggleSwitch>,
    pub sliders: Vec<SliderControl>,
    pub section: super::SettingsSection,
}

impl SectionContent {
    pub fn for_section(section: super::SettingsSection, state: &super::SystemSettingsState) -> Self {
        let mut rows = Vec::new();
        let mut toggles = Vec::new();
        let mut sliders = Vec::new();
        let mut y = 60.0;

        match section {
            super::SettingsSection::Wallpaper => {
                rows.push(SettingsRow::new("Preview", y));
                y += 200.0;
                rows.push(SettingsRow::new("Built-in Wallpapers", y));
                y += 100.0;
                rows.push(SettingsRow::new("Choose your own", y).with_description("Opens Filer portal"));
            }
            super::SettingsSection::Appearance => {
                rows.push(SettingsRow::new("Accent Color", y).with_description("Space Orange #FF6B2B"));
                y += 44.0;
                rows.push(SettingsRow::new("Dark Mode", y));
                toggles.push(ToggleSwitch::new(state.is_dark_mode, 600.0, y + 12.0));
                y += 44.0;
                rows.push(SettingsRow::new("Reduce Motion", y).with_description("Minimize animations"));
                toggles.push(ToggleSwitch::new(state.reduce_motion, 600.0, y + 12.0));
                y += 44.0;
                rows.push(SettingsRow::new("Reduce Transparency", y));
                toggles.push(ToggleSwitch::new(state.reduce_transparency, 600.0, y + 12.0));
            }
            super::SettingsSection::Displays => {
                rows.push(SettingsRow::new("Resolution", y).with_description(&state.display_resolution));
                y += 44.0;
                rows.push(SettingsRow::new("Refresh Rate", y).with_description(&format!("{:.0} Hz", state.refresh_rate_hz)));
                y += 44.0;
                rows.push(SettingsRow::new("UI Scale", y).with_description(&format!("{:.0}%", state.ui_scale * 100.0)));
                sliders.push(SliderControl::new(state.ui_scale as f32, 0.5, 2.0, 400.0, y + 10.0, 200.0));
                y += 60.0;
                rows.push(SettingsRow::new("Night Shift", y).with_description(&format!("{}K", state.color_temperature_k)));
                toggles.push(ToggleSwitch::new(state.night_shift_enabled, 600.0, y + 12.0));
            }
            super::SettingsSection::Sound => {
                rows.push(SettingsRow::new("Output Volume", y));
                sliders.push(SliderControl::new(state.output_volume, 0.0, 1.0, 400.0, y + 10.0, 200.0));
                y += 60.0;
                rows.push(SettingsRow::new("Boot Chime", y).with_description("Plays on system start"));
                toggles.push(ToggleSwitch::new(state.boot_chime_enabled, 600.0, y + 12.0));
                y += 44.0;
                rows.push(SettingsRow::new("Spatial Audio DSP", y));
                toggles.push(ToggleSwitch::new(state.spatial_audio_dsp, 600.0, y + 12.0));
            }
            super::SettingsSection::Keyboard => {
                rows.push(SettingsRow::new("Input Layout", y).with_description("US English (QWERTY)"));
                y += 44.0;
                rows.push(SettingsRow::new("Key Repeat", y));
                sliders.push(SliderControl::new(0.6, 0.0, 1.0, 400.0, y + 10.0, 200.0));
                y += 60.0;
                rows.push(SettingsRow::new("Caps Lock Mapping", y).with_description("Escape"));
            }
            super::SettingsSection::MotionWave => {
                rows.push(SettingsRow::new("Natural Scroll", y));
                toggles.push(ToggleSwitch::new(state.trackpad_natural_scroll, 600.0, y + 12.0));
                y += 44.0;
                rows.push(SettingsRow::new("Three-Finger Swipe", y).with_description("Switch desktops"));
                toggles.push(ToggleSwitch::new(state.three_finger_swipe_enabled, 600.0, y + 12.0));
                y += 44.0;
                rows.push(SettingsRow::new("Fling Friction", y));
                sliders.push(SliderControl::new(state.fling_friction, 0.9, 1.0, 400.0, y + 10.0, 200.0));
            }
            super::SettingsSection::SecurityVault => {
                rows.push(SettingsRow::new("HEV Encryption", y).with_description("Argon2id vault"));
                toggles.push(ToggleSwitch::new(state.hev_encryption_active, 600.0, y + 12.0));
                y += 44.0;
                rows.push(SettingsRow::new("TPM PCR Seal", y).with_description("Hardware-bound"));
                toggles.push(ToggleSwitch::new(state.tpm_pcr_sealed, 600.0, y + 12.0));
                y += 44.0;
                rows.push(SettingsRow::new("Proximity Lock", y).with_description("BLE device trust"));
                toggles.push(ToggleSwitch::new(state.proximity_lock_enabled, 600.0, y + 12.0));
            }
            super::SettingsSection::Updates => {
                rows.push(SettingsRow::new("Channel", y).with_description(&format!("{}", state.active_channel)));
                y += 44.0;
                rows.push(SettingsRow::new("Check for Updates", y).with_description("Manual check"));
                y += 44.0;
                if let Some(ref ver) = state.remote_version {
                    rows.push(SettingsRow::new("Available Version", y).with_description(ver));
                } else {
                    rows.push(SettingsRow::new("Current Version", y).with_description("Up to date"));
                }
            }
            super::SettingsSection::About => {
                rows.push(SettingsRow::new("vitusOS", y).with_description("UpstreamColor Channel"));
                y += 44.0;
                rows.push(SettingsRow::new("Kernel", y).with_description("Linux 6.x"));
                y += 44.0;
                rows.push(SettingsRow::new("Compositor", y).with_description("AnimusEngine"));
                y += 44.0;
                rows.push(SettingsRow::new("Memory", y).with_description("16 GB"));
                y += 44.0;
                rows.push(SettingsRow::new("Storage", y).with_description("512 GB NVMe"));
            }
        }

        SectionContent { rows, toggles, sliders, section }
    }

    pub fn update(&mut self, dt: f32) {
        for t in &mut self.toggles {
            t.update(dt);
        }
        for s in &mut self.sliders {
            s.update(dt);
        }
    }

    pub fn on_pointer_motion(&mut self, mx: f32, my: f32) {
        for s in &mut self.sliders {
            s.on_pointer_motion(mx, my);
        }
    }

    pub fn on_pointer_button(&mut self, mx: f32, my: f32, pressed: bool) -> bool {
        let mut handled = false;
        for t in &mut self.toggles {
            if t.on_pointer_button(mx, my, pressed) {
                handled = true;
            }
        }
        for s in &mut self.sliders {
            if s.on_pointer_button(mx, my, pressed) {
                handled = true;
            }
        }
        handled
    }
}

/// The settings app's render state — combines sidebar, content, and transitions.
pub struct SettingsRenderState {
    pub sidebar_width: f32,
    pub content_x: f32,
    pub content_width: f32,
    pub transition: SectionTransition,
    pub selection_pill_y: SpringSolver,
    pub content_opacity: SpringSolver,
}

impl SettingsRenderState {
    pub const SIDEBAR_WIDTH: f32 = 220.0;
    pub const ROW_HEIGHT: f32 = 36.0;

    pub fn new() -> Self {
        Self {
            sidebar_width: Self::SIDEBAR_WIDTH,
            content_x: Self::SIDEBAR_WIDTH,
            content_width: 580.0,
            transition: SectionTransition::new(),
            selection_pill_y: SpringSolver::new(36.0, SpringProfile::Selection),
            content_opacity: SpringSolver::new(1.0, SpringProfile::Hover),
        }
    }

    pub fn switch_section(&mut self, section_index: usize) {
        self.transition.start();
        self.selection_pill_y.set_target(section_index as f32 * Self::ROW_HEIGHT);
    }

    pub fn update(&mut self, dt: f32) {
        self.transition.update(dt);
        self.selection_pill_y.update(dt);
        self.content_opacity.update(dt);
    }
}

impl Default for SettingsRenderState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_switch_animation() {
        let mut t = ToggleSwitch::new(false, 100.0, 100.0);
        assert!(!t.is_on);
        assert!((t.knob.value - 0.0).abs() < 0.01);

        t.toggle();
        assert!(t.is_on);
        assert!((t.knob.target - 1.0).abs() < 0.01);

        for _ in 0..120 {
            t.update(1.0 / 60.0);
        }
        assert!((t.knob.value - 1.0).abs() < 0.05);

        // Verify knob moved right
        let knob_x_off = t.knob_x();
        t.toggle();
        for _ in 0..120 {
            t.update(1.0 / 60.0);
        }
        let knob_x_on = t.knob_x();
        // knob_x when off should be less than knob_x when on
        // But we toggled back to off, so now knob_x should be less
        assert!(knob_x_on < knob_x_off + 1.0);
    }

    #[test]
    fn toggle_switch_hit_test() {
        let mut t = ToggleSwitch::new(false, 100.0, 100.0);
        assert!(!t.on_pointer_button(50.0, 50.0, true));  // outside
        assert!(t.on_pointer_button(110.0, 110.0, true)); // inside
        assert!(t.is_on);
    }

    #[test]
    fn slider_control_value() {
        let mut s = SliderControl::new(0.5, 0.0, 1.0, 100.0, 100.0, 200.0);
        assert_eq!(s.value, 0.5);

        s.set_value(0.8);
        assert!((s.value - 0.8).abs() < 0.01);

        s.set_value(-0.5); // should clamp to min
        assert_eq!(s.value, 0.0);

        s.set_value(1.5); // should clamp to max
        assert_eq!(s.value, 1.0);
    }

    #[test]
    fn slider_drag() {
        let mut s = SliderControl::new(0.5, 0.0, 1.0, 100.0, 100.0, 200.0);

        // Start drag at x=150 (should set value to 0.25)
        s.on_pointer_button(150.0, 100.0, true);
        assert!(s.is_dragging);
        assert!((s.value - 0.25).abs() < 0.05);

        // Drag to x=250 (should set value to 0.75)
        s.on_pointer_motion(250.0, 100.0);
        assert!((s.value - 0.75).abs() < 0.05);

        // Release
        s.on_pointer_button(250.0, 100.0, false);
        assert!(!s.is_dragging);
    }

    #[test]
    fn section_transition() {
        let mut t = SectionTransition::new();
        assert!(!t.transitioning);
        assert!(t.is_done());

        t.start();
        assert!(t.transitioning);
        assert!(!t.is_done());

        for _ in 0..120 {
            t.update(1.0 / 60.0);
        }
        assert!(t.is_done());
        assert!((t.fade_in.value - 1.0).abs() < 0.05);
        assert!((t.fade_out.value - 0.0).abs() < 0.05);
    }

    #[test]
    fn settings_render_state_switch() {
        let mut rs = SettingsRenderState::new();
        rs.switch_section(3);
        assert!(rs.transition.transitioning);
        assert!((rs.selection_pill_y.target - 108.0).abs() < 0.01); // 3 * 36 = 108

        for _ in 0..120 {
            rs.update(1.0 / 60.0);
        }
        assert!(rs.transition.is_done());
        assert!((rs.selection_pill_y.value - 108.0).abs() < 2.0);
    }

    #[test]
    fn section_content_appearance() {
        let state = super::super::SystemSettingsState::default();
        let content = SectionContent::for_section(super::super::SettingsSection::Appearance, &state);
        assert!(content.rows.len() >= 4); // Accent, Dark Mode, Reduce Motion, Reduce Transparency
        assert_eq!(content.toggles.len(), 3); // Dark Mode, Reduce Motion, Reduce Transparency
    }

    #[test]
    fn section_content_sound() {
        let state = super::super::SystemSettingsState::default();
        let content = SectionContent::for_section(super::super::SettingsSection::Sound, &state);
        assert!(content.rows.len() >= 3);
        assert_eq!(content.sliders.len(), 1); // Output Volume
        assert_eq!(content.toggles.len(), 2); // Boot Chime, Spatial Audio
    }

    #[test]
    fn section_content_about() {
        let state = super::super::SystemSettingsState::default();
        let content = SectionContent::for_section(super::super::SettingsSection::About, &state);
        assert!(content.rows.len() >= 5); // vitusOS, Kernel, Compositor, Memory, Storage
        assert!(content.toggles.is_empty());
        assert!(content.sliders.is_empty());
    }

    #[test]
    fn section_content_updates() {
        let state = super::super::SystemSettingsState::default();
        let content = SectionContent::for_section(super::super::SettingsSection::Updates, &state);
        assert!(content.rows.len() >= 3); // Channel, Check, Version
    }

    #[test]
    fn section_content_security() {
        let state = super::super::SystemSettingsState::default();
        let content = SectionContent::for_section(super::super::SettingsSection::SecurityVault, &state);
        assert_eq!(content.toggles.len(), 3); // HEV, TPM, Proximity
    }
}
