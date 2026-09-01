//! Notification Center & Floating Toast Banners.
//!
//! Floating Altitude Glass (48px Kawase Blur) sliding in from the top-right
//! with spring-driven stacking physics and auto-dismissal.

use animus_core::event_bus::EventBus;
use animus_core::events::{AEEvent, NotificationPayload};
use animus_physics::spring::{SpringProfile, SpringSolver};
use animus_render::altitude::SurfaceAltitude;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::info;

static NOTIFICATION_ID_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationToast {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub slide_x: SpringSolver,  // SPRING_SELECTION (400, 28): +360.0 (offscreen) -> 0.0 (onscreen)
    pub opacity: SpringSolver,  // SPRING_SELECTION (400, 28): 0.0 -> 1.0
    pub time_remaining: f32,    // Seconds remaining before auto-dismiss
    pub is_persistent: bool,
    pub is_dismissing: bool,
    pub action_labels: Vec<String>,
}

pub struct NotificationCenter {
    pub toasts: RwLock<Vec<NotificationToast>>,
    pub altitude: SurfaceAltitude, // Floating (48px Kawase Blur, 64% Opacity)
    bus: EventBus,
}

impl NotificationCenter {
    pub fn new(bus: EventBus) -> Self {
        Self {
            toasts: RwLock::new(Vec::new()),
            altitude: SurfaceAltitude::Floating,
            bus,
        }
    }

    /// Posts a new notification banner.
    pub fn post(&self, payload: NotificationPayload) -> u64 {
        let id = NOTIFICATION_ID_SEQ.fetch_add(1, Ordering::SeqCst);
        let timeout = if payload.timeout_ms < 0 {
            f32::INFINITY
        } else {
            payload.timeout_ms as f32 / 1000.0
        };

        let mut toast = NotificationToast {
            id,
            title: payload.title.clone(),
            body: payload.body.clone(),
            slide_x: SpringSolver::new(360.0, SpringProfile::Selection),
            opacity: SpringSolver::new(0.0, SpringProfile::Selection),
            time_remaining: timeout,
            is_persistent: payload.is_persistent,
            is_dismissing: false,
            action_labels: payload.action_labels.clone(),
        };

        // Trigger entrance spring
        toast.slide_x.set_target(0.0);
        toast.opacity.set_target(1.0);

        self.toasts.write().push(toast);
        let title_for_log = payload.title.clone();
        self.bus.publish(AEEvent::NotificationPosted(payload));
        info!("NotificationCenter: Posted notification #{} -> '{}'", id, title_for_log);
        id
    }

    /// Dismisses a notification with a slide-out spring animation.
    pub fn dismiss(&self, id: u64) {
        let mut toasts = self.toasts.write();
        if let Some(toast) = toasts.iter_mut().find(|t| t.id == id) {
            toast.is_dismissing = true;
            toast.slide_x.set_target(380.0);
            toast.opacity.set_target(0.0);
            self.bus.publish(AEEvent::NotificationDismissed { id });
            info!("NotificationCenter: Dismissed notification #{}", id);
        }
    }

    /// Updates spring physics and auto-dismissal timers.
    pub fn update(&self, dt: f32) {
        let mut toasts = self.toasts.write();
        let mut to_dismiss = Vec::new();

        for toast in toasts.iter_mut() {
            toast.slide_x.update(dt);
            toast.opacity.update(dt);

            if !toast.is_persistent && !toast.is_dismissing {
                toast.time_remaining -= dt;
                if toast.time_remaining <= 0.0 {
                    toast.is_dismissing = true;
                    toast.slide_x.set_target(380.0);
                    toast.opacity.set_target(0.0);
                    to_dismiss.push(toast.id);
                }
            }
        }

        // Clean up completely faded toasts
        toasts.retain(|t| !t.is_dismissing || t.opacity.value > 0.01);

        for id in to_dismiss {
            self.bus.publish(AEEvent::NotificationDismissed { id });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_center_lifecycle() {
        let bus = EventBus::new();
        let nc = NotificationCenter::new(bus);

        let id = nc.post(NotificationPayload {
            title: "Kernel Security".to_string(),
            body: "TPM 2.0 PCR Sealing Verified".to_string(),
            timeout_ms: 3000,
            is_persistent: false,
            action_keys: vec!["ok".to_string()],
            action_labels: vec!["Acknowledge".to_string()],
        });

        assert_eq!(nc.toasts.read().len(), 1);
        assert_eq!(nc.toasts.read()[0].id, id);

        // Update physics
        nc.update(0.1);
        assert!(nc.toasts.read()[0].slide_x.value < 360.0);

        nc.dismiss(id);
        assert!(nc.toasts.read()[0].is_dismissing);
    }
}
