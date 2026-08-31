//! Spatial Animation Context (AnimusContext)
//!
//! Stores origin geometry and semantic source when spawning new surfaces
//! (e.g. from a Pathfinder search result, Dock icon, or notification).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextOriginType {
    ScreenCenter,
    DockIcon,
    PathfinderResult,
    NotificationCard,
    ParentWindow,
    CursorPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimusContext {
    pub origin_type: ContextOriginType,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for AnimusContext {
    fn default() -> Self {
        Self::center()
    }
}

impl AnimusContext {
    pub fn center() -> Self {
        Self {
            origin_type: ContextOriginType::ScreenCenter,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn from_dock(x: f32, y: f32, icon_size: f32) -> Self {
        Self {
            origin_type: ContextOriginType::DockIcon,
            x,
            y,
            width: icon_size,
            height: icon_size,
        }
    }

    pub fn from_pathfinder_result(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            origin_type: ContextOriginType::PathfinderResult,
            x,
            y,
            width: w,
            height: h,
        }
    }

    pub fn from_cursor(x: f32, y: f32) -> Self {
        Self {
            origin_type: ContextOriginType::CursorPosition,
            x,
            y,
            width: 1.0,
            height: 1.0,
        }
    }
}
