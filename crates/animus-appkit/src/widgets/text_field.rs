use animus_physics::spring::{SpringSolver, SpringProfile};
use tracing::info;

pub struct AETextField {
    pub placeholder: String,
    pub value: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub is_focused: bool,
    
    // Focus ring animation
    pub focus_spring: SpringSolver,
}

impl AETextField {
    pub fn new(placeholder: impl Into<String>, width: f32) -> Self {
        Self {
            placeholder: placeholder.into(),
            value: String::new(),
            x: 0.0,
            y: 0.0,
            width,
            is_focused: false,
            focus_spring: SpringSolver::new(0.0, SpringProfile::Hover),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.focus_spring.update(dt);
    }

    pub fn hit_test(&self, mx: f32, my: f32) -> bool {
        let height = 28.0; // Fixed height per spec
        mx >= self.x && mx <= self.x + self.width && my >= self.y && my <= self.y + height
    }

    pub fn on_pointer_button(&mut self, mx: f32, my: f32, pressed: bool) {
        if pressed {
            self.is_focused = self.hit_test(mx, my);
            if self.is_focused {
                self.focus_spring.set_target(1.0);
                info!("AETextField focused");
            } else {
                self.focus_spring.set_target(0.0);
            }
        }
    }

    pub fn on_key(&mut self, _sym: u32, _mods: u32, _pressed: bool) {
        // Will be wired to wayland keyboard events
    }
}
