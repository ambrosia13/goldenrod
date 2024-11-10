use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;

use crate::{
    state::{
        bvh::BoundingVolumeHierarchy,
        camera::Camera,
        material::Material,
        object::{ObjectList, Sphere},
    },
    util,
};

use super::{input::Input, render_state::RenderState, time::Time};

pub const RANDOM_SCENE_KEY: KeyCode = KeyCode::KeyK;

pub struct EngineState {
    pub input: Input,
    pub time: Time,

    pub camera: Camera,
    pub object_list: ObjectList,
    pub bounding_volume_hierarchy: BoundingVolumeHierarchy,
}

impl EngineState {
    pub fn new(render_state: &RenderState) -> Self {
        let input = Input::new();
        let time = Time::new();

        let mut camera = Camera::new(Vec3::ZERO, Vec3::NEG_Z, 45.0, render_state.size, 1.0, 100.0);

        camera.position = Vec3::new(0.0, 0.0, 10.0);
        camera.look_at(Vec3::ZERO);

        let mut object_list = ObjectList::new();
        object_list.mesh_test_scene();

        let bounding_volume_hierarchy = BoundingVolumeHierarchy::from_objects(&mut object_list);

        Self {
            input,
            time,
            camera,
            object_list,
            bounding_volume_hierarchy,
        }
    }

    pub fn update(&mut self) {
        if self.input.keys.just_pressed(RANDOM_SCENE_KEY) {
            self.object_list.mesh_test_scene();
        }

        if self.bounding_volume_hierarchy.version != self.object_list.version() {
            log::info!("Rebuilding BVH");

            self.bounding_volume_hierarchy =
                BoundingVolumeHierarchy::from_objects(&mut self.object_list);
        }

        self.camera.update_position(&self.input, &self.time);
    }

    pub fn post_frame_update(&mut self) {
        self.input.update();
        self.time.update();
    }
}
