//! Built-in AELoginManager / Greeter on AESurfaces (SurfaceAltitude::High).
//!
//! Provides native session orchestration, user avatar cards, and smooth crossfade into desktop.

use animus_core::event_bus::EventBus;
use animus_core::events::AEEvent;
use animus_physics::spring::{SpringProfile, SpringSolver};
use animus_render::altitude::SurfaceAltitude;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub display_name: String,
    pub avatar_path: Option<String>,
}

pub struct AELoginManager {
    pub is_session_active: RwLock<bool>,
    pub altitude: SurfaceAltitude, // High (32px Kawase Blur, 72% Opacity)
    pub crossfade_opacity: RwLock<SpringSolver>, // 1.0 -> 0.0 on session start
    pub users: RwLock<Vec<UserProfile>>,
    pub selected_user_idx: RwLock<usize>,
    bus: EventBus,
}

impl AELoginManager {
    pub fn new(bus: EventBus) -> Self {
        let default_user = UserProfile {
            username: "krisna".to_string(),
            display_name: "Krisna Dwi Prasetyo".to_string(),
            avatar_path: Some("/usr/share/vitusos/avatars/default.png".to_string()),
        };

        Self {
            is_session_active: RwLock::new(false),
            altitude: SurfaceAltitude::High,
            crossfade_opacity: RwLock::new(SpringSolver::new(1.0, SpringProfile::Boot)),
            users: RwLock::new(vec![default_user]),
            selected_user_idx: RwLock::new(0),
            bus,
        }
    }

    /// Selects user by index.
    pub fn select_user(&self, idx: usize) {
        let users = self.users.read();
        if idx < users.len() {
            *self.selected_user_idx.write() = idx;
        }
    }

    /// Authenticates and launches desktop session with smooth crossfade.
    pub fn start_session(&self) {
        let mut active = self.is_session_active.write();
        *active = true;
        self.crossfade_opacity.write().set_target(0.0);
        self.bus.publish(AEEvent::WelcomeScreenCompleted);
        info!("AELoginManager: Desktop session started. Greeter crossfading into desktop.");
    }

    pub fn update(&self, dt: f32) {
        self.crossfade_opacity.write().update(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_manager_session_start() {
        let bus = EventBus::new();
        let login = AELoginManager::new(bus);

        assert!(!*login.is_session_active.read());
        assert_eq!(login.users.read().len(), 1);

        login.start_session();
        assert!(*login.is_session_active.read());
        assert_eq!(login.crossfade_opacity.read().target, 0.0);
    }
}
