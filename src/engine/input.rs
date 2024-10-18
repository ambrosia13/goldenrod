use glam::DVec2;

#[derive(Debug, Default)]
pub struct Input {
    mouse_delta: DVec2,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_mouse_delta(&mut self, delta_x: f64, delta_y: f64) {
        self.mouse_delta = DVec2::new(delta_x, delta_y);
    }

    pub fn mouse_delta(&self) -> DVec2 {
        self.mouse_delta
    }
}
