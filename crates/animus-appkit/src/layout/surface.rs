use animus_physics::spring::{SpringSolver, SpringProfile};

pub struct AESurface {
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
}

pub struct AEWindow {
    pub surface: AESurface,
    pub has_traffic_lights: bool,
    pub traffic_light_hover: SpringSolver, // 0 -> 1 on hover over traffic lights
}

impl AEWindow {
    pub fn new(width: f32, height: f32, has_traffic_lights: bool) -> Self {
        Self {
            surface: AESurface { width, height, corner_radius: 12.0 },
            has_traffic_lights,
            traffic_light_hover: SpringSolver::new(0.0, SpringProfile::Hover),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.traffic_light_hover.update(dt);
    }
}

pub struct AESidebar {
    pub width: f32,
    pub is_collapsed: bool,
    pub slide_spring: SpringSolver,
}

impl AESidebar {
    pub fn new(width: f32) -> Self {
        Self {
            width,
            is_collapsed: false,
            slide_spring: SpringSolver::new(1.0, SpringProfile::Scroll),
        }
    }

    pub fn toggle(&mut self) {
        self.is_collapsed = !self.is_collapsed;
        self.slide_spring.set_target(if self.is_collapsed { 0.0 } else { 1.0 });
    }

    pub fn update(&mut self, dt: f32) {
        self.slide_spring.update(dt);
    }
}

pub struct AEToolbar {
    pub height: f32,
}

impl AEToolbar {
    pub fn new() -> Self {
        Self { height: 48.0 }
    }
}

pub struct AEContent {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
