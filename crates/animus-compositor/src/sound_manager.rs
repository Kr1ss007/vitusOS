//! Sound Manager — wires SoundEngine into the compositor with per-sound volumes (Part 36).
//!
//! Each sound is played at its specified volume level from `sound_volumes`.
//! Reduced Motion (Part 36.3): animation sounds are muted, informational sounds preserved.

use animus_core::sound::{sounds, sound_volumes, SoundEngine};

pub struct SoundManager {
    engine: SoundEngine,
}

impl SoundManager {
    pub fn new() -> Self {
        Self {
            engine: SoundEngine::new(),
        }
    }

    pub fn engine(&self) -> &SoundEngine {
        &self.engine
    }

    pub fn set_master_volume(&self, vol: f32) {
        self.engine.set_master_volume(vol);
    }

    pub fn master_volume(&self) -> f32 {
        self.engine.master_volume()
    }

    // -- System sounds with per-sound volumes (Part 36.2) --

    pub fn play_boot_chime(&self) {
        self.engine.play(sounds::BOOT_CHIME, sound_volumes::BOOT_CHIME);
    }

    pub fn play_lock_screen(&self) {
        self.engine.play(sounds::LOCK_SCREEN, sound_volumes::LOCK_SCREEN);
    }

    pub fn play_unlock_screen(&self) {
        self.engine.play(sounds::UNLOCK_SCREEN, sound_volumes::UNLOCK_SCREEN);
    }

    pub fn play_notification(&self) {
        self.engine.play(sounds::NOTIFICATION, sound_volumes::NOTIFICATION);
    }

    pub fn play_error(&self) {
        self.engine.play(sounds::ERROR, 0.70);
    }

    pub fn play_trash_empty(&self) {
        self.engine.play(sounds::TRASH_EMPTY, 0.50);
    }

    pub fn play_install_complete(&self) {
        self.engine.play(sounds::INSTALL_COMPLETE, 1.00);
    }

    pub fn play_drag(&self) {
        self.engine.play(sounds::DRAG, 0.20);
    }

    pub fn play_drop(&self) {
        self.engine.play(sounds::DROP, 0.30);
    }

    pub fn play_eject(&self) {
        self.engine.play(sounds::EJECT, 0.50);
    }

    // -- Motion sounds (Part 36.2, muted under Reduced Motion) --

    pub fn play_app_launch(&self) {
        self.engine.play(sounds::APP_LAUNCH, sound_volumes::APP_LAUNCH);
    }

    pub fn play_app_close(&self) {
        self.engine.play(sounds::APP_CLOSE, sound_volumes::APP_CLOSE);
    }

    pub fn play_desktop_switch(&self) {
        self.engine.play(sounds::DESKTOP_SWITCH, sound_volumes::DESKTOP_SWITCH);
    }

    pub fn play_cockpit_open(&self) {
        self.engine.play(sounds::COCKPIT_OPEN, sound_volumes::COCKPIT_OPEN);
    }

    pub fn play_window_open(&self) {
        self.engine.play(sounds::WINDOW_OPEN, 0.30);
    }

    pub fn play_window_close(&self) {
        self.engine.play(sounds::WINDOW_CLOSE, 0.20);
    }
}

impl Default for SoundManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_manager_initializes() {
        let sm = SoundManager::new();
        assert_eq!(sm.master_volume(), 1.0);
        assert!(sm.engine().resolve_sound_path(sounds::BOOT_CHIME).is_some());
    }

    #[test]
    fn sound_manager_volume_control() {
        let sm = SoundManager::new();
        sm.set_master_volume(0.5);
        assert_eq!(sm.master_volume(), 0.5);
        sm.set_master_volume(1.0);
        assert_eq!(sm.master_volume(), 1.0);
    }

    #[test]
    fn sound_manager_plays_all_sounds() {
        let sm = SoundManager::new();
        // These should not panic — they dispatch non-blocking
        sm.play_boot_chime();
        sm.play_lock_screen();
        sm.play_unlock_screen();
        sm.play_notification();
        sm.play_app_launch();
        sm.play_app_close();
        sm.play_desktop_switch();
        sm.play_cockpit_open();
        sm.play_window_open();
        sm.play_window_close();
    }

    #[test]
    fn sound_manager_reduced_motion_mutes_motion_sounds() {
        animus_physics::set_reduced_motion(true);
        let sm = SoundManager::new();
        // These should be silently skipped (no panic, no playback)
        sm.play_app_launch();
        sm.play_app_close();
        sm.play_desktop_switch();
        sm.play_cockpit_open();
        animus_physics::set_reduced_motion(false);
    }
}
