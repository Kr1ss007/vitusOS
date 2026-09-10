//! Notification Center -- Toast Banners with Actions, Timeout, Stacking (Part 42).
//!
//! Floating Altitude glass (48px Kawase Blur, 64% opacity) sliding in from
//! top-right with spring-driven stacking physics and auto-dismissal.
//!
//! Part 42 spec:
//! - Max 2 action buttons per notification (3+ truncated)
//! - Action button: 24px height, 6px corner, white 12% bg, 11px Inter Medium
//! - Default timeout: 5000ms fallback, max 30000ms, persistent = -1
//! - Click on body: dismiss + activate default action (if any)
//! - Default action key: "default"
//! - No notification history/center/persistence in unstable ISO
//! - Stacking: vertical, 8px gap, MARGIN_TOP=44px (below Panel)
//! - Stack offset applied by NotificationCenter, not individual toast

use animus_core::event_bus::EventBus;
use animus_core::events::{AEEvent, NotificationPayload};
use animus_physics::spring::{SpringProfile, SpringSolver};
use animus_render::altitude::SurfaceAltitude;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::info;

static NOTIFICATION_ID_SEQ: AtomicU64 = AtomicU64::new(1);

// Constants (Part 42 + Addendum M)
const NOTIFICATION_WIDTH: f32 = 320.0;
const NOTIFICATION_HEIGHT: f32 = 80.0;
const MARGIN_RIGHT: f32 = 16.0;
const MARGIN_TOP: f32 = 44.0;   // below Panel (28px) + 16px gap
const STACK_GAP: f32 = 8.0;
#[allow(dead_code)]
const CORNER_RADIUS: f32 = 12.0;
const MAX_TIMEOUT_MS: i32 = 30000;
const DEFAULT_TIMEOUT_MS: i32 = 5000;
const MAX_ACTION_BUTTONS: usize = 2;

/// A notification action button (Part 42.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub key: String,
    pub label: String,
    pub hover_alpha: SpringSolver,  // SPRING_HOVER (600,40)
}

/// A single toast notification banner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationToast {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub slide_x: SpringSolver,      // SPRING_NOTIFICATION (380,26): screenW -> slot
    pub stack_y: SpringSolver,      // SPRING_NOTIFICATION: vertical stack position
    pub opacity: SpringSolver,      // SPRING_NOTIFICATION: 0.0 -> 1.0
    pub time_remaining: f32,        // Seconds before auto-dismiss
    pub is_persistent: bool,
    pub is_dismissing: bool,
    pub is_dismissed: bool,
    pub actions: Vec<NotificationAction>,
    pub default_action_key: Option<String>,
    pub width: f32,
    pub height: f32,
}

impl NotificationToast {
    pub fn width() -> f32 { NOTIFICATION_WIDTH }
    pub fn height() -> f32 { NOTIFICATION_HEIGHT }
}

/// NotificationCenter manages active toast notifications.
pub struct NotificationCenter {
    pub toasts: RwLock<Vec<NotificationToast>>,
    pub altitude: SurfaceAltitude,
    screen_width: f32,
    bus: EventBus,
}

impl NotificationCenter {
    pub fn new(bus: EventBus) -> Self {
        Self {
            toasts: RwLock::new(Vec::new()),
            altitude: SurfaceAltitude::Floating,
            screen_width: 1920.0,
            bus,
        }
    }

    pub fn set_screen_width(&self, _w: f32) {
        // Used for slide-in start position calculation
    }

    /// Posts a new notification banner (Part 42).
    /// Max 2 action buttons (3+ truncated). Default timeout 5000ms.
    /// Persistent (timeout_ms = -1) stays until user dismisses.
    pub fn post(&self, payload: NotificationPayload) -> u64 {
        let id = NOTIFICATION_ID_SEQ.fetch_add(1, Ordering::SeqCst);

        // Clamp timeout (Part 42.3)
        let timeout_secs = if payload.timeout_ms < 0 {
            f32::INFINITY // persistent
        } else {
            let clamped = payload.timeout_ms.min(MAX_TIMEOUT_MS).max(0);
            if clamped == 0 {
                DEFAULT_TIMEOUT_MS as f32 / 1000.0
            } else {
                clamped as f32 / 1000.0
            }
        };

        // Build action buttons (max 2, Part 42.2)
        let actions: Vec<NotificationAction> = payload.action_keys.iter()
            .zip(payload.action_labels.iter())
            .take(MAX_ACTION_BUTTONS)
            .map(|(key, label)| NotificationAction {
                key: key.clone(),
                label: label.clone(),
                hover_alpha: SpringSolver::new(0.0, SpringProfile::Hover),
            })
            .collect();

        let default_action_key = payload.action_keys.iter()
            .find(|k| k.as_str() == "default")
            .cloned()
            .or_else(|| if payload.action_keys.is_empty() { None } else { Some(payload.action_keys[0].clone()) });

        let mut toast = NotificationToast {
            id,
            title: payload.title.clone(),
            body: payload.body.clone(),
            slide_x: SpringSolver::new(self.screen_width, SpringProfile::Notification)
                .eliminate_on_reduced_motion(true),
            stack_y: SpringSolver::new(0.0, SpringProfile::Notification),
            opacity: SpringSolver::new(0.0, SpringProfile::Notification)
                .eliminate_on_reduced_motion(true),
            time_remaining: timeout_secs,
            is_persistent: payload.is_persistent,
            is_dismissing: false,
            is_dismissed: false,
            actions,
            default_action_key,
            width: NOTIFICATION_WIDTH,
            height: NOTIFICATION_HEIGHT,
        };

        // Trigger entrance spring
        let target_x = self.screen_width - NOTIFICATION_WIDTH - MARGIN_RIGHT;
        toast.slide_x.set_target(target_x);
        toast.opacity.set_target(1.0);

        self.toasts.write().push(toast);
        self.recalculate_stack_positions();

        self.bus.publish(AEEvent::NotificationPosted(payload));
        info!("NotificationCenter: Posted notification #{} -> '{}'", id, id);
        id
    }

    /// Dismisses a notification with slide-out animation.
    pub fn dismiss(&self, id: u64) {
        let mut toasts = self.toasts.write();
        if let Some(toast) = toasts.iter_mut().find(|t| t.id == id) {
            toast.is_dismissing = true;
            toast.slide_x.set_target(self.screen_width + 50.0);
            toast.opacity.set_target(0.0);
            self.bus.publish(AEEvent::NotificationDismissed { id });
            info!("NotificationCenter: Dismissed notification #{}", id);
        }
    }

    /// Click on notification body: dismiss + activate default action (Part 42.3).
    pub fn on_click(&self, id: u64) {
        let toasts = self.toasts.read();
        if let Some(toast) = toasts.iter().find(|t| t.id == id) {
            if let Some(ref action_key) = toast.default_action_key {
                info!("NotificationCenter: Default action '{}' triggered for #{}", action_key, id);
                // In production: DBusBridge sends ActionInvoked signal
            }
        }
        drop(toasts);
        self.dismiss(id);
    }

    /// Click on an action button: invoke action + dismiss (Part 42.2).
    pub fn on_action_click(&self, notification_id: u64, action_key: &str) {
        info!("NotificationCenter: Action '{}' triggered for #{}", action_key, notification_id);
        // In production: DBusBridge sends ActionInvoked signal with action key
        self.dismiss(notification_id);
    }

    /// Recalculates vertical stack positions for all toasts (Part 42, Addendum M).
    fn recalculate_stack_positions(&self) {
        let toasts = self.toasts.read();
        let mut _y_offset = MARGIN_TOP;
        for toast in toasts.iter() {
            if !toast.is_dismissing {
                // stack_y target = _y_offset (applied via set_target on next update)
                _y_offset += NOTIFICATION_HEIGHT + STACK_GAP;
            }
        }
    }

    /// Updates spring physics, auto-dismissal timers, and stack positions.
    pub fn update(&self, dt: f32) {
        let mut toasts = self.toasts.write();

        // Calculate stack positions for non-dismissing toasts
        let mut stack_y = MARGIN_TOP;
        for toast in toasts.iter_mut() {
            if !toast.is_dismissing {
                toast.stack_y.set_target(stack_y);
                stack_y += NOTIFICATION_HEIGHT + STACK_GAP;
            }
        }

        let mut to_dismiss = Vec::new();

        for toast in toasts.iter_mut() {
            toast.slide_x.update(dt);
            toast.stack_y.update(dt);
            toast.opacity.update(dt);

            // Update action button hover springs
            for action in &mut toast.actions {
                action.hover_alpha.update(dt);
            }

            // Auto-dismiss timer (Part 42.3)
            if !toast.is_persistent && !toast.is_dismissing {
                toast.time_remaining -= dt;
                if toast.time_remaining <= 0.0 {
                    toast.is_dismissing = true;
                    toast.slide_x.set_target(self.screen_width + 50.0);
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

    pub fn live_count(&self) -> usize {
        self.toasts.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_lifecycle() {
        let bus = EventBus::new();
        let nc = NotificationCenter::new(bus);

        let id = nc.post(NotificationPayload {
            title: "Test".to_string(),
            body: "Body text".to_string(),
            timeout_ms: 3000,
            is_persistent: false,
            action_keys: vec!["ok".to_string()],
            action_labels: vec!["OK".to_string()],
        });

        assert_eq!(nc.live_count(), 1);

        nc.update(0.1);
        assert!(nc.toasts.read()[0].slide_x.value < nc.screen_width);

        nc.dismiss(id);
        assert!(nc.toasts.read()[0].is_dismissing);
    }

    #[test]
    fn test_action_buttons_max_2() {
        let bus = EventBus::new();
        let nc = NotificationCenter::new(bus);

        nc.post(NotificationPayload {
            title: "Test".to_string(),
            body: "Body".to_string(),
            timeout_ms: 5000,
            is_persistent: false,
            action_keys: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            action_labels: vec!["A".to_string(), "B".to_string(), "C".to_string()],
        });

        let toasts = nc.toasts.read();
        assert_eq!(toasts[0].actions.len(), MAX_ACTION_BUTTONS); // Truncated to 2
    }

    #[test]
    fn test_persistent_notification() {
        let bus = EventBus::new();
        let nc = NotificationCenter::new(bus);

        let id = nc.post(NotificationPayload {
            title: "Critical".to_string(),
            body: "Save your work".to_string(),
            timeout_ms: -1, // persistent
            is_persistent: true,
            ..Default::default()
        });

        // Tick many frames -- should NOT auto-dismiss
        for _ in 0..100 {
            nc.update(0.1);
        }
        assert_eq!(nc.live_count(), 1);

        // Manual dismiss
        nc.dismiss(id);
        for _ in 0..100 {
            nc.update(0.1);
        }
        assert_eq!(nc.live_count(), 0);
    }

    #[test]
    fn test_auto_dismiss_timeout() {
        let bus = EventBus::new();
        let nc = NotificationCenter::new(bus);

        nc.post(NotificationPayload {
            title: "Test".to_string(),
            body: "Auto-dismiss".to_string(),
            timeout_ms: 100, // 100ms = 0.1s
            is_persistent: false,
            ..Default::default()
        });

        // Tick past timeout
        for _ in 0..20 {
            nc.update(0.1);
        }
        // Should be dismissing or dismissed
        let toasts = nc.toasts.read();
        assert!(toasts.is_empty() || toasts[0].is_dismissing);
    }

    #[test]
    fn test_click_dismisses_and_activates() {
        let bus = EventBus::new();
        let nc = NotificationCenter::new(bus);

        let id = nc.post(NotificationPayload {
            title: "Test".to_string(),
            body: "Click me".to_string(),
            timeout_ms: 5000,
            is_persistent: false,
            action_keys: vec!["default".to_string()],
            action_labels: vec!["Open".to_string()],
        });

        nc.on_click(id);
        assert!(nc.toasts.read()[0].is_dismissing);
    }

    #[test]
    fn test_stacking() {
        let bus = EventBus::new();
        let nc = NotificationCenter::new(bus);

        nc.post(NotificationPayload {
            title: "First".to_string(),
            body: "".to_string(),
            timeout_ms: 5000,
            ..Default::default()
        });

        nc.post(NotificationPayload {
            title: "Second".to_string(),
            body: "".to_string(),
            timeout_ms: 5000,
            ..Default::default()
        });

        nc.update(0.1); // Let stack positions update

        let toasts = nc.toasts.read();
        assert_eq!(toasts.len(), 2);
        // Second toast should have higher stack_y target (below first)
        assert!(toasts[1].stack_y.target > toasts[0].stack_y.target);
    }
}
