use std::sync::Arc;
use parking_lot::RwLock;
use tracing::info;

use animus_physics::AnimationClock;
use crate::event_bus::EventBus;
use crate::events::AEEvent;
use crate::hardware::{HardwareTopology, GpuDeviceInfo};
use crate::power::PowerManager;
use crate::sound::{SoundEngine, sounds};
use crate::state::StateManager;

pub struct AnimusEngine {
    pub hardware: Arc<HardwareTopology>,
    pub sound: Arc<SoundEngine>,
    pub event_bus: Arc<EventBus>,
    pub state: Arc<StateManager>,
    pub power: Arc<PowerManager>,
    pub clock: Arc<RwLock<AnimationClock>>,
}

impl AnimusEngine {
    /// Authoritative initialization of the entire Animus OS core.
    pub fn new() -> Self {
        let hardware = Arc::new(HardwareTopology::new());
        let sound = Arc::new(SoundEngine::new());
        let event_bus = Arc::new(EventBus::new());
        let state = Arc::new(StateManager::new((*event_bus).clone()));
        let power = Arc::new(PowerManager::new((*event_bus).clone()));
        let clock = Arc::new(RwLock::new(AnimationClock::new()));

        Self {
            hardware,
            sound,
            event_bus,
            state,
            power,
            clock,
        }
    }

    /// Executes the authoritative boot and hardware handoff sequence.
    pub fn boot_sequence(&self) {
        info!("=== AnimusEngine Boot Sequence Initialized ===");

        // 1. Authoritative GPU Topology Check
        self.hardware.detect_devices();
        if let Some(scanout) = self.hardware.primary_scanout_gpu() {
            info!("AnimusEngine: Bound Primary Scanout Controller -> {}", scanout.name);
            self.state.set("system.scanout_gpu", scanout.name);
        }
        if let Some(renderer) = self.hardware.primary_renderer_gpu() {
            info!("AnimusEngine: Bound Primary Compute Renderer -> {}", renderer.name);
            self.state.set("system.render_gpu", renderer.name);
        }

        // 2. Authoritative Audio Topology & Boot Chime
        self.sound.detect_audio_sinks();
        self.sound.play(sounds::BOOT_CHIME, 1.0);

        // 3. Register Core System State
        self.state.set("ui.scale", 1.0f64);
        self.state.set("ui.color_scheme", "dark".to_string());
        self.state.set("ui.reduced_motion", false);
        self.state.set("ui.accent_color", "#FF6B00".to_string()); // Space Orange

        // 4. Dispatch System Engine Ready Event
        self.event_bus.publish(AEEvent::EngineReady);
        info!("=== AnimusEngine Ready: All Subsystems Authoritative & Synchronized ===");
    }

    pub fn primary_scanout(&self) -> Option<GpuDeviceInfo> {
        self.hardware.primary_scanout_gpu()
    }

    pub fn primary_renderer(&self) -> Option<GpuDeviceInfo> {
        self.hardware.primary_renderer_gpu()
    }

    pub fn play_sound(&self, name: &str) {
        self.sound.play(name, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_boot_sequence_and_hardware_topology() {
        let engine = AnimusEngine::new();
        engine.boot_sequence();

        // Verify primary scanout and render GPU detection
        let scanout = engine.primary_scanout();
        assert!(scanout.is_some(), "Primary scanout GPU must be detected");
        assert!(scanout.unwrap().is_primary_scanout);

        let renderer = engine.primary_renderer();
        assert!(renderer.is_some(), "Primary compute renderer must be detected");
        assert!(renderer.unwrap().is_primary_renderer);

        // Verify state manager initialization
        assert_eq!(engine.state.get::<String>("ui.color_scheme"), Some("dark".to_string()));
        assert_eq!(engine.state.get::<String>("ui.accent_color"), Some("#FF6B00".to_string()));
        assert_eq!(engine.state.get::<bool>("ui.reduced_motion"), Some(false));
    }
}
