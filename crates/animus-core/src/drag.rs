//! DragManager -- wl_data_device Drag and Drop (Part 35 of spec).
//!
//! Manages active drag operations, renders ghost image following cursor
//! with spring lag, and reports drag state to compositor hit-testing.
//!
//! Ghost image: 60% opacity, 48px icon, max 200px wide, 8px corner radius.
//! Ghost position lags cursor via SPRING_WINDOW_DRAG (800,35) -- weight feeling.
//! Drop target: 1px Space Orange border on accepted target (SPRING_HOVER).
//! Accepted drop: ghost springs into drop point (scale 1.0->0, SPRING_SELECTION).
//! Cancelled: ghost springs back to origin (SPRING_WINDOW_DRAG), fades out.

use animus_physics::spring::{SpringProfile, SpringSolver, SpringSolver2D};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::event_bus::EventBus;
use crate::events::AEEvent;

/// Payload type for drag operations (Part 35.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DragPayloadType {
    File,    // text/uri-list
    Text,    // text/plain;charset=utf-8
    Unknown, // other MIME type -- ghost shown, drop may fail
}

/// Drag payload carried during a drag operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragPayload {
    pub payload_type: DragPayloadType,
    pub mime_types: Vec<String>,
    /// First 40 chars for ghost label (filename for File, text for Text)
    pub preview: String,
    /// Icon path for File type; empty for Text
    pub icon_path: String,
    pub origin_x: f32,
    pub origin_y: f32,
}

/// Manages the active drag operation and ghost image rendering state.
pub struct DragManager {
    dragging: RwLock<bool>,
    payload: RwLock<Option<DragPayload>>,
    cursor_x: RwLock<f32>,
    cursor_y: RwLock<f32>,
    origin_x: RwLock<f32>,
    origin_y: RwLock<f32>,
    /// Ghost position lags cursor -- communicates weight
    ghost_pos: RwLock<SpringSolver2D>,
    /// Drop target highlight (1px Space Orange border on accepted target)
    drop_highlight: RwLock<SpringSolver>,
    over_valid_target: RwLock<bool>,
    bus: EventBus,
}

impl DragManager {
    pub const GHOST_OPACITY: f32 = 0.60;
    pub const GHOST_ICON_SIZE: f32 = 48.0;
    pub const GHOST_MAX_W: f32 = 200.0;
    pub const GHOST_CORNER: f32 = 8.0;

    pub fn new(bus: EventBus) -> Self {
        Self {
            dragging: RwLock::new(false),
            payload: RwLock::new(None),
            cursor_x: RwLock::new(0.0),
            cursor_y: RwLock::new(0.0),
            origin_x: RwLock::new(0.0),
            origin_y: RwLock::new(0.0),
            ghost_pos: RwLock::new(SpringSolver2D::new(0.0, 0.0, SpringProfile::WindowDrag)),
            drop_highlight: RwLock::new(SpringSolver::new(0.0, SpringProfile::Hover)),
            over_valid_target: RwLock::new(false),
            bus,
        }
    }

    pub fn is_dragging(&self) -> bool { *self.dragging.read() }

    /// Called when drag starts (Part 35.2).
    pub fn on_drag_start(&self, payload: DragPayload) {
        *self.dragging.write() = true;
        *self.origin_x.write() = payload.origin_x;
        *self.origin_y.write() = payload.origin_y;
        *self.cursor_x.write() = payload.origin_x;
        *self.cursor_y.write() = payload.origin_y;
        self.ghost_pos.write().snap(payload.origin_x, payload.origin_y);
        let event_payload = AEEvent_to_drag_payload(&payload);
        let origin_x = payload.origin_x;
        let origin_y = payload.origin_y;
        *self.payload.write() = Some(payload);
        *self.over_valid_target.write() = false;
        self.bus.publish(AEEvent::DragStart(event_payload));
        info!("DragManager: Drag started at ({:.0}, {:.0})", origin_x, origin_y);
    }

    /// Called every frame with current cursor position (Part 35.2).
    pub fn on_cursor_move(&self, x: f32, y: f32) {
        if !*self.dragging.read() { return; }
        *self.cursor_x.write() = x;
        *self.cursor_y.write() = y;
        self.ghost_pos.write().set_target(x, y);
        self.bus.publish(AEEvent::DragMotion { x, y });
    }

    /// Called when drop occurs over a target (Part 35.3).
    pub fn on_drop(&self, x: f32, y: f32) {
        if !*self.dragging.read() { return; }
        let payload = self.payload.read().clone();
        *self.dragging.write() = false;
        *self.payload.write() = None;

        // Ghost springs into drop point: scale 1.0 -> 0
        self.ghost_pos.write().set_target(x, y);

        self.bus.publish(AEEvent::DragDrop { x, y });
        if let Some(p) = payload {
            info!("DragManager: Dropped at ({:.0}, {:.0}) type={:?}", x, y, p.payload_type);
        }
    }

    /// Called when drag is cancelled (Part 35.3).
    pub fn on_drag_cancel(&self) {
        if !*self.dragging.read() { return; }
        *self.dragging.write() = false;

        // Ghost springs back to origin
        let ox = *self.origin_x.read();
        let oy = *self.origin_y.read();
        self.ghost_pos.write().set_target(ox, oy);

        self.bus.publish(AEEvent::DragCancel);
        *self.payload.write() = None;
        info!("DragManager: Drag cancelled -- ghost returning to origin");
    }

    /// Sets whether the cursor is over a valid drop target.
    pub fn set_over_valid_target(&self, valid: bool) {
        if *self.over_valid_target.read() != valid {
            *self.over_valid_target.write() = valid;
            self.drop_highlight.write().set_target(if valid { 1.0 } else { 0.0 });
        }
    }

    /// Returns ghost image position for rendering.
    pub fn ghost_position(&self) -> (f32, f32) {
        self.ghost_pos.read().values()
    }

    pub fn cursor_position(&self) -> (f32, f32) {
        (*self.cursor_x.read(), *self.cursor_y.read())
    }

    pub fn drop_highlight_alpha(&self) -> f32 {
        self.drop_highlight.read().value
    }

    /// Ticks the ghost position spring.
    pub fn update(&self, dt: f32) {
        self.ghost_pos.write().update(dt);
        self.drop_highlight.write().update(dt);
    }
}

/// Helper: convert DragPayload to AEEvent::DragStart payload
fn AEEvent_to_drag_payload(payload: &DragPayload) -> crate::events::DragPayload {
    use crate::events::{DragPayload as EventPayload, DragPayloadType as EventType};
    let pt = match payload.payload_type {
        DragPayloadType::File => EventType::File,
        DragPayloadType::Text => EventType::Text,
        DragPayloadType::Unknown => EventType::Unknown,
    };
    EventPayload {
        payload_type: pt,
        data: payload.preview.as_bytes().to_vec(),
        mime_types: payload.mime_types.clone(),
        origin_x: payload.origin_x,
        origin_y: payload.origin_y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drag_lifecycle() {
        let bus = EventBus::new();
        let dm = DragManager::new(bus);

        assert!(!dm.is_dragging());

        dm.on_drag_start(DragPayload {
            payload_type: DragPayloadType::File,
            mime_types: vec!["text/uri-list".to_string()],
            preview: "document.pdf".to_string(),
            icon_path: "/usr/share/icons/vitusos/pdf.svg".to_string(),
            origin_x: 100.0,
            origin_y: 200.0,
        });
        assert!(dm.is_dragging());

        dm.on_cursor_move(150.0, 250.0);
        let (gx, gy) = dm.ghost_position();
        assert!((gx - 100.0).abs() < 1.0); // ghost starts at origin

        // Tick ghost spring
        dm.update(0.016);
        let (gx2, _) = dm.ghost_position();
        assert!(gx2 > gx); // ghost moves toward cursor

        dm.on_drop(300.0, 400.0);
        assert!(!dm.is_dragging());
    }

    #[test]
    fn test_drag_cancel() {
        let bus = EventBus::new();
        let dm = DragManager::new(bus);

        dm.on_drag_start(DragPayload {
            payload_type: DragPayloadType::Text,
            mime_types: vec!["text/plain".to_string()],
            preview: "Hello world".to_string(),
            icon_path: String::new(),
            origin_x: 50.0,
            origin_y: 50.0,
        });
        assert!(dm.is_dragging());

        dm.on_drag_cancel();
        assert!(!dm.is_dragging());

        // Ghost target should be back at origin
        let (gx, gy) = dm.ghost_position();
        dm.update(0.016);
        // Ghost should be moving toward origin (50, 50)
    }

    #[test]
    fn test_drop_target_highlight() {
        let bus = EventBus::new();
        let dm = DragManager::new(bus);

        assert_eq!(dm.drop_highlight_alpha(), 0.0);

        dm.set_over_valid_target(true);
        dm.update(0.016);
        assert!(dm.drop_highlight_alpha() > 0.0);

        dm.set_over_valid_target(false);
        dm.update(0.016);
        // Alpha should be decreasing back to 0
    }
}
