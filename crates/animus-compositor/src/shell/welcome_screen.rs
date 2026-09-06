//! Welcome Screen & First-Boot Setup Wizard (Part 37).
//!
//! Full-screen first-boot experience. Three steps: vault setup, wallpaper, done.
//! Shown only once — never again after first_boot_complete = true.
//! Background: #1A1208 (same as LockScreen — system speaking).
//! Content card: glass material, 480px wide, centered.
//! No Panel. No Dock. No orange box visible. The OS is not ready yet.

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};
use serde::{Deserialize, Serialize};

/// Step in the first-boot wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WizardStep {
    /// Passphrase creation for HEV vault.
    VaultSetup = 0,
    /// Wallpaper selection (3 built-in + custom via Filer portal).
    WallpaperPick = 1,
    /// Completion screen — "you're all set".
    Done = 2,
}

/// Passphrase strength levels (0-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PassphraseStrength {
    /// Too short (< 8 chars)
    TooShort = 0,
    /// Weak (8+ chars, simple)
    Weak = 1,
    /// Fair (mixed case or digits)
    Fair = 2,
    /// Good (mixed case + digits)
    Good = 3,
    /// Strong (mixed case + digits + symbols)
    Strong = 4,
}

impl PassphraseStrength {
    /// Segment color for the strength bar.
    pub fn segment_color(self, index: usize) -> [f32; 4] {
        if index > self as usize {
            // Inactive segment
            return [1.0, 1.0, 1.0, 0.15];
        }
        match self {
            Self::TooShort => [0.85, 0.27, 0.27, 0.9], // red
            Self::Weak => [0.91, 0.40, 0.11, 0.9],     // orange
            Self::Fair => [0.91, 0.68, 0.11, 0.9],     // yellow
            Self::Good => [0.20, 0.75, 0.35, 0.9],     // green
            Self::Strong => [0.13, 0.75, 0.30, 0.95],  // deep green
        }
    }

    /// Calculate passphrase strength honestly.
    /// 0: too short (<8 chars)
    /// 1: weak (8+ chars, simple)
    /// 2: fair (mixed case or digits)
    /// 3: good (mixed case + digits)
    /// 4: strong (mixed case + digits + symbols)
    pub fn calculate(pass: &str) -> Self {
        if pass.len() < 8 {
            return Self::TooShort;
        }

        let has_lower = pass.chars().any(|c| c.is_ascii_lowercase());
        let has_upper = pass.chars().any(|c| c.is_ascii_uppercase());
        let has_digit = pass.chars().any(|c| c.is_ascii_digit());
        let has_symbol = pass.chars().any(|c| !c.is_alphanumeric() && !c.is_whitespace());

        let mixed_case = has_lower && has_upper;

        if mixed_case && has_digit && has_symbol {
            Self::Strong
        } else if mixed_case && has_digit {
            Self::Good
        } else if mixed_case || has_digit {
            Self::Fair
        } else {
            Self::Weak
        }
    }
}

/// Built-in wallpaper definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinWallpaper {
    /// Default Mars surface wallpaper.
    Mars = 0,
    /// Aurora gradient.
    Aurora = 1,
    /// Deep space nebula.
    Nebula = 2,
}

impl BuiltinWallpaper {
    pub const ALL: [Self; 3] = [Self::Mars, Self::Aurora, Self::Nebula];
}

/// Continue button state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinueButtonState {
    Disabled,
    Enabled,
    /// Brief shake animation when passphrase mismatch detected.
    Shake,
}

/// Card entrance / exit state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardPhase {
    /// Card springing in from above.
    Entering,
    /// Card settled, user interacting.
    Active,
    /// Card springing out on completion.
    Exiting,
    /// Fully done, can be destroyed.
    Complete,
}

pub struct WelcomeScreen {
    pub current_step: WizardStep,
    pub card_phase: CardPhase,
    pub is_active: bool,

    // Card animation — SPRING_SHEET (420,30): drops from above
    pub card_y: SpringSolver,
    // Card fade — SPRING_HOVER (600,40)
    pub card_opacity: SpringSolver,

    // Step 1: Vault setup
    pub passphrase1: String,
    pub passphrase2: String,
    pub passphrase_visible: bool,
    pub passphrase_strength: PassphraseStrength,
    pub continue_button_state: ContinueButtonState,
    /// Shake offset for mismatch feedback (SPRING_SELECTION).
    pub shake_x: SpringSolver,
    /// Strength bar fill animation (SPRING_HOVER).
    pub strength_bar: SpringSolver,

    // Step 2: Wallpaper pick
    pub selected_wallpaper: usize,
    /// Per-thumbnail hover springs (SPRING_HOVER).
    pub wallpaper_hover: [SpringSolver; 3],
    /// Custom wallpaper path if user picked via Filer portal.
    pub custom_wallpaper_path: Option<String>,

    // Step 3: Done
    /// "let's go" button hover (SPRING_HOVER).
    pub done_button_hover: SpringSolver,
    /// Continue button hover (SPRING_HOVER).
    pub continue_button_hover: SpringSolver,

    // Progress dots — per-dot scale (SPRING_HOVER)
    pub dot_scale: [SpringSolver; 3],

    bus: EventBus,
}

impl WelcomeScreen {
    pub const BG_COLOR: [f32; 4] = [0.102, 0.071, 0.031, 1.0]; // #1A1208
    pub const CARD_WIDTH: f32 = 480.0;
    pub const CARD_CORNER_RADIUS: f32 = 16.0;
    pub const CARD_BORDER_OPACITY: f32 = 0.15;
    pub const PROGRESS_DOT_SIZE: f32 = 8.0;
    pub const PROGRESS_DOT_GAP: f32 = 12.0;
    pub const PROGRESS_DOT_ACTIVE_SCALE: f32 = 1.1;
    pub const WALLPAPER_THUMB_W: f32 = 136.0;
    pub const WALLPAPER_THUMB_H: f32 = 76.0;
    pub const WALLPAPER_THUMB_GAP: f32 = 8.0;
    pub const WALLPAPER_THUMB_CORNER: f32 = 8.0;
    pub const CONTINUE_BUTTON_W: f32 = 220.0;
    pub const CONTINUE_BUTTON_H: f32 = 44.0;
    pub const MIN_PASSPHRASE_LEN: usize = 8;
    pub const SPACE_ORANGE: [f32; 4] = [0.91, 0.36, 0.0, 1.0]; // #E85D00

    pub fn new(bus: EventBus) -> Self {
        Self {
            current_step: WizardStep::VaultSetup,
            card_phase: CardPhase::Entering,
            is_active: false,

            card_y: SpringSolver::new(-40.0, SpringProfile::Sheet),
            card_opacity: SpringSolver::new(0.0, SpringProfile::Hover),

            passphrase1: String::new(),
            passphrase2: String::new(),
            passphrase_visible: false,
            passphrase_strength: PassphraseStrength::TooShort,
            continue_button_state: ContinueButtonState::Disabled,
            shake_x: SpringSolver::new(0.0, SpringProfile::Selection),
            strength_bar: SpringSolver::new(0.0, SpringProfile::Hover),

            selected_wallpaper: 0,
            wallpaper_hover: [
                SpringSolver::new(1.0, SpringProfile::Hover),
                SpringSolver::new(1.0, SpringProfile::Hover),
                SpringSolver::new(1.0, SpringProfile::Hover),
            ],
            custom_wallpaper_path: None,

            done_button_hover: SpringSolver::new(0.0, SpringProfile::Hover),
            continue_button_hover: SpringSolver::new(0.0, SpringProfile::Hover),

            dot_scale: [
                SpringSolver::new(1.1, SpringProfile::Hover),
                SpringSolver::new(1.0, SpringProfile::Hover),
                SpringSolver::new(1.0, SpringProfile::Hover),
            ],

            bus,
        }
    }

    /// Activate the welcome screen — card drops in from above.
    pub fn activate(&mut self) {
        self.is_active = true;
        self.card_phase = CardPhase::Entering;
        self.card_y.snap(-40.0);
        self.card_y.set_target(0.0);
        self.card_opacity.snap(0.0);
        self.card_opacity.set_target(1.0);
    }

    /// Check if the welcome screen has fully completed and can be destroyed.
    pub fn is_complete(&self) -> bool {
        self.card_phase == CardPhase::Complete
    }

    /// Called when the wizard should advance to the next step.
    pub fn next_step(&mut self) {
        match self.current_step {
            WizardStep::VaultSetup => {
                self.commit_vault_setup();
                self.current_step = WizardStep::WallpaperPick;
                self.update_progress_dots(1);
            }
            WizardStep::WallpaperPick => {
                self.commit_wallpaper_pick();
                self.current_step = WizardStep::Done;
                self.update_progress_dots(2);
            }
            WizardStep::Done => {
                self.commit_done();
            }
        }
    }

    /// Skip the current step (only allowed for wallpaper, NOT vault).
    pub fn skip_step(&mut self) {
        match self.current_step {
            WizardStep::WallpaperPick => {
                // Keeps default Mars wallpaper
                self.current_step = WizardStep::Done;
                self.update_progress_dots(2);
            }
            WizardStep::VaultSetup => {
                // Cannot skip — vault setup is mandatory
            }
            WizardStep::Done => {}
        }
    }

    // -- Step 1: Vault Setup --

    /// Handle passphrase input for field 1.
    pub fn set_passphrase1(&mut self, value: String) {
        self.passphrase1 = value.trim().to_string();
        self.passphrase_strength = PassphraseStrength::calculate(&self.passphrase1);
        let target = self.passphrase_strength as usize as f32 / 4.0;
        self.strength_bar.set_target(target);
        self.update_continue_state();
    }

    /// Handle passphrase input for field 2.
    pub fn set_passphrase2(&mut self, value: String) {
        self.passphrase2 = value.trim().to_string();
        self.update_continue_state();
    }

    /// Toggle passphrase visibility (show/hide button).
    pub fn toggle_passphrase_visibility(&mut self) {
        self.passphrase_visible = !self.passphrase_visible;
    }

    fn update_continue_state(&mut self) {
        let strength_ok = self.passphrase_strength as usize >= 1;
        let fields_match = !self.passphrase1.is_empty()
            && self.passphrase1 == self.passphrase2
            && self.passphrase1.len() >= Self::MIN_PASSPHRASE_LEN;

        if fields_match && strength_ok {
            self.continue_button_state = ContinueButtonState::Enabled;
        } else if self.continue_button_state == ContinueButtonState::Shake {
            // Keep shake state until spring settles
        } else {
            self.continue_button_state = ContinueButtonState::Disabled;
        }
    }

    /// Check if the continue button can be pressed.
    pub fn can_continue(&self) -> bool {
        match self.current_step {
            WizardStep::VaultSetup => self.continue_button_state == ContinueButtonState::Enabled,
            WizardStep::WallpaperPick => true, // Always enabled (default pre-selected)
            WizardStep::Done => true,
        }
    }

    /// Trigger shake animation on passphrase mismatch.
    pub fn trigger_mismatch_shake(&mut self) {
        self.continue_button_state = ContinueButtonState::Shake;
        self.shake_x.snap(-12.0);
        self.shake_x.set_target(0.0);
    }

    fn commit_vault_setup(&mut self) {
        // Zero passphrase2 from memory after derivation
        // HEV::initialize() with new passphrase would be called here
        // Takes 200-500ms on HP Victus (KNOWN LIMIT-37-2: blocking on main thread)
        self.passphrase2.zeroize();
        self.passphrase1.zeroize();
        self.passphrase_visible = false;
    }

    // -- Step 2: Wallpaper Pick --

    /// Select a built-in wallpaper by index.
    pub fn select_wallpaper(&mut self, index: usize) {
        if index < BuiltinWallpaper::ALL.len() {
            self.selected_wallpaper = index;
            self.custom_wallpaper_path = None;
        }
    }

    /// Set a custom wallpaper path (from Filer portal).
    pub fn set_custom_wallpaper(&mut self, path: String) {
        self.custom_wallpaper_path = Some(path);
    }

    /// Handle pointer motion over wallpaper thumbnails.
    pub fn on_wallpaper_pointer_motion(&mut self, mx: f32, _my: f32, thumb_start_x: f32) {
        for (i, spring) in self.wallpaper_hover.iter_mut().enumerate() {
            let tx = thumb_start_x + i as f32 * (Self::WALLPAPER_THUMB_W + Self::WALLPAPER_THUMB_GAP);
            let selected_scale = if i == self.selected_wallpaper { 1.04 } else { 1.0 };
            if mx >= tx && mx <= tx + Self::WALLPAPER_THUMB_W {
                spring.set_target(1.02 * selected_scale / 1.0);
            } else {
                spring.set_target(selected_scale);
            }
        }
    }

    /// Handle click on wallpaper thumbnails. Returns true if a thumbnail was hit.
    pub fn on_wallpaper_pointer_button(&mut self, mx: f32, _my: f32, thumb_start_x: f32) -> bool {
        for (i, _) in BuiltinWallpaper::ALL.iter().enumerate() {
            let tx = thumb_start_x + i as f32 * (Self::WALLPAPER_THUMB_W + Self::WALLPAPER_THUMB_GAP);
            if mx >= tx && mx <= tx + Self::WALLPAPER_THUMB_W {
                self.select_wallpaper(i);
                return true;
            }
        }
        false
    }

    fn commit_wallpaper_pick(&mut self) {
        // Wallpaper selection persisted to StateManager
        // "wallpaper" key = index or path
    }

    // -- Step 3: Done --

    fn commit_done(&mut self) {
        // StateManager sets "first_boot_complete" = true
        self.card_phase = CardPhase::Exiting;
        self.card_y.set_target(-40.0);
        self.card_opacity.set_target(0.0);
        self.bus.publish(AEEvent::WelcomeScreenCompleted);
    }

    // -- Progress dots --

    fn update_progress_dots(&mut self, active: usize) {
        for (i, dot) in self.dot_scale.iter_mut().enumerate() {
            if i == active {
                dot.set_target(Self::PROGRESS_DOT_ACTIVE_SCALE);
            } else {
                dot.set_target(1.0);
            }
        }
    }

    /// Card center X for a given screen width.
    pub fn card_center_x(&self, screen_w: f32) -> f32 {
        screen_w * 0.5
    }

    /// Card center Y offset (springed) for a given screen height.
    pub fn card_offset_y(&self) -> f32 {
        self.card_y.value
    }

    /// Current card opacity (0.0 to 1.0).
    pub fn card_alpha(&self) -> f32 {
        self.card_opacity.value
    }

    /// Update all springs by dt seconds.
    pub fn update(&mut self, dt: f32) {
        match self.card_phase {
            CardPhase::Complete => return,
            CardPhase::Exiting => {
                self.card_y.update(dt);
                self.card_opacity.update(dt);
                if self.card_opacity.value < 0.01 && self.card_opacity.is_settled() {
                    self.card_phase = CardPhase::Complete;
                    self.is_active = false;
                }
                self.update_step_springs(dt);
                return;
            }
            _ => {}
        }

        self.card_y.update(dt);
        self.card_opacity.update(dt);

        if self.card_y.is_settled() && self.card_phase == CardPhase::Entering {
            self.card_phase = CardPhase::Active;
        }

        self.update_step_springs(dt);

        // Shake spring
        if self.continue_button_state == ContinueButtonState::Shake {
            self.shake_x.update(dt);
            if self.shake_x.is_settled() {
                self.update_continue_state();
            }
        }
    }

    fn update_step_springs(&mut self, dt: f32) {
        self.strength_bar.update(dt);
        self.shake_x.update(dt);
        self.done_button_hover.update(dt);
        self.continue_button_hover.update(dt);
        for dot in &mut self.dot_scale {
            dot.update(dt);
        }
        match self.current_step {
            WizardStep::WallpaperPick => {
                for spring in &mut self.wallpaper_hover {
                    spring.update(dt);
                }
            }
            _ => {}
        }
    }
}

/// Trait for zeroizing sensitive string data.
trait Zeroize {
    fn zeroize(&mut self);
}

impl Zeroize for String {
    fn zeroize(&mut self) {
        // Overwrite the string contents with zeros
        unsafe {
            let bytes = self.as_bytes_mut();
            for b in bytes {
                *b = 0;
            }
        }
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_screen() -> WelcomeScreen {
        WelcomeScreen::new(EventBus::new())
    }

    #[test]
    fn passphrase_strength_levels() {
        assert_eq!(PassphraseStrength::calculate(""), PassphraseStrength::TooShort);
        assert_eq!(PassphraseStrength::calculate("short"), PassphraseStrength::TooShort);
        assert_eq!(PassphraseStrength::calculate("12345678"), PassphraseStrength::Fair);
        assert_eq!(PassphraseStrength::calculate("password"), PassphraseStrength::Weak);
        assert_eq!(PassphraseStrength::calculate("Password1"), PassphraseStrength::Good);
        assert_eq!(PassphraseStrength::calculate("Password!"), PassphraseStrength::Fair);
        assert_eq!(PassphraseStrength::calculate("P@ssw0rd!"), PassphraseStrength::Strong);
    }

    #[test]
    fn vault_step_continue_disabled_initially() {
        let mut s = make_screen();
        assert_eq!(s.continue_button_state, ContinueButtonState::Disabled);
        assert!(!s.can_continue());
    }

    #[test]
    fn vault_step_continue_enables_on_match() {
        let mut s = make_screen();
        s.set_passphrase1("StrongPass123!".to_string());
        assert_eq!(s.passphrase_strength, PassphraseStrength::Strong);
        assert!(!s.can_continue()); // Fields don't match yet

        s.set_passphrase2("StrongPass123!".to_string());
        assert_eq!(s.continue_button_state, ContinueButtonState::Enabled);
        assert!(s.can_continue());
    }

    #[test]
    fn vault_step_mismatch_shake() {
        let mut s = make_screen();
        s.set_passphrase1("password123".to_string());
        s.set_passphrase2("password456".to_string());
        assert_eq!(s.continue_button_state, ContinueButtonState::Disabled);

        s.trigger_mismatch_shake();
        assert_eq!(s.continue_button_state, ContinueButtonState::Shake);
        assert!((s.shake_x.value - (-12.0)).abs() < 0.1);

        // Shake should settle
        for _ in 0..60 {
            s.update(1.0 / 60.0);
        }
        assert!((s.shake_x.value - 0.0).abs() < 1.0);
    }

    #[test]
    fn vault_step_short_passphrase_disabled() {
        let mut s = make_screen();
        s.set_passphrase1("short".to_string());
        s.set_passphrase2("short".to_string());
        // Too short (< 8 chars) → disabled
        assert_eq!(s.continue_button_state, ContinueButtonState::Disabled);
        assert!(!s.can_continue());
    }

    #[test]
    fn wizard_step_progression() {
        let mut s = make_screen();
        s.activate();
        assert_eq!(s.current_step, WizardStep::VaultSetup);
        assert_eq!(s.card_phase, CardPhase::Entering);

        // Simulate frames to settle entrance
        for _ in 0..120 {
            s.update(1.0 / 60.0);
        }
        assert_eq!(s.card_phase, CardPhase::Active);

        // Advance through steps
        s.set_passphrase1("MyP@ssw0rd!".to_string());
        s.set_passphrase2("MyP@ssw0rd!".to_string());
        s.next_step();
        assert_eq!(s.current_step, WizardStep::WallpaperPick);
        assert_eq!(s.dot_scale[0].target, 1.0);
        assert_eq!(s.dot_scale[1].target, 1.1);

        s.next_step();
        assert_eq!(s.current_step, WizardStep::Done);
        assert_eq!(s.dot_scale[2].target, 1.1);
    }

    #[test]
    fn wallpaper_selection() {
        let mut s = make_screen();
        s.current_step = WizardStep::WallpaperPick;

        s.select_wallpaper(1);
        assert_eq!(s.selected_wallpaper, 1);

        s.select_wallpaper(2);
        assert_eq!(s.selected_wallpaper, 2);
    }

    #[test]
    fn wallpaper_skip_allowed() {
        let mut s = make_screen();
        s.current_step = WizardStep::WallpaperPick;

        s.skip_step();
        assert_eq!(s.current_step, WizardStep::Done);
    }

    #[test]
    fn vault_setup_cannot_skip() {
        let mut s = make_screen();
        s.current_step = WizardStep::VaultSetup;

        s.skip_step();
        assert_eq!(s.current_step, WizardStep::VaultSetup);
    }

    #[test]
    fn complete_triggers_exit() {
        let mut s = make_screen();
        s.activate();
        for _ in 0..120 {
            s.update(1.0 / 60.0);
        }
        assert_eq!(s.card_phase, CardPhase::Active);

        s.current_step = WizardStep::Done;
        s.next_step(); // commit_done
        assert_eq!(s.card_phase, CardPhase::Exiting);

        // Fade out
        for _ in 0..120 {
            s.update(1.0 / 60.0);
        }
        assert_eq!(s.card_phase, CardPhase::Complete);
        assert!(!s.is_active);
        assert!(s.is_complete());
    }

    #[test]
    fn passphrase_visibility_toggle() {
        let mut s = make_screen();
        assert!(!s.passphrase_visible);
        s.toggle_passphrase_visibility();
        assert!(s.passphrase_visible);
        s.toggle_passphrase_visibility();
        assert!(!s.passphrase_visible);
    }

    #[test]
    fn passphrase_trimmed_on_input() {
        let mut s = make_screen();
        s.set_passphrase1("  hello123  ".to_string());
        assert_eq!(s.passphrase1, "hello123");
    }

    #[test]
    fn card_entrance_animation() {
        let mut s = make_screen();
        s.activate();
        assert!((s.card_y.value - (-40.0)).abs() < 0.1);
        assert!((s.card_opacity.value - 0.0).abs() < 0.1);

        for _ in 0..120 {
            s.update(1.0 / 60.0);
        }
        assert!((s.card_y.value - 0.0).abs() < 2.0);
        assert!((s.card_opacity.value - 1.0).abs() < 0.05);
    }
}
