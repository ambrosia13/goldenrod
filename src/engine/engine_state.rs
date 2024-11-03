use glam::Vec3;
use winit::keyboard::KeyCode;

use crate::state::{camera::Camera, object::ObjectList};

use super::{input::Input, render_state::RenderState, time::Time};

pub const RANDOM_SCENE_KEY: KeyCode = KeyCode::KeyK;

pub struct EngineState {
    pub input: Input,
    pub time: Time,

    pub camera: Camera,
    pub object_list: ObjectList,
}

impl EngineState {
    pub fn new(render_state: &RenderState) -> Self {
        let input = Input::new();
        let time = Time::new();

        let camera = Camera::new(Vec3::ZERO, Vec3::NEG_Z, 45.0, render_state.size, 1.0, 100.0);

        let mut object_list = ObjectList::new();
        object_list.random_scene();

        Self {
            input,
            time,
            camera,
            object_list,
        }
    }

    pub fn update(&mut self) {
        if self.input.keys.just_pressed(RANDOM_SCENE_KEY) {
            self.object_list.random_scene();
        }

        self.camera.update_position(&self.input, &self.time);
    }

    pub fn post_frame_update(&mut self) {
        self.input.update();
        self.time.update();
    }
}
