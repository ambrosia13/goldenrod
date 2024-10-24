use glam::{Mat3, Mat4, Vec3};
use gpu_bytes::{AsStd430, Std430Bytes};
use gpu_bytes_derive::{AsStd140, AsStd430};

use crate::{
    engine::{
        render_state::RenderState,
        render_state_ext::{
            buffer::{BufferData, WgpuBuffer, WgpuBufferConfig, WgpuBufferType},
            RenderStateExt,
        },
    },
    state::{camera::Camera, material::Material},
};

#[derive(AsStd140, AsStd430, Default)]
pub struct CameraUniform {
    view_projection_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,

    inverse_view_projection_matrix: Mat4,
    inverse_view_matrix: Mat4,
    inverse_projection_matrix: Mat4,

    previous_view_projection_matrix: Mat4,
    previous_view_matrix: Mat4,
    previous_projection_matrix: Mat4,

    position: Vec3,
    previous_position: Vec3,

    view: Vec3,
    previous_view: Vec3,

    right: Vec3,
    up: Vec3,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, camera: &Camera) {
        self.previous_projection_matrix = self.view_projection_matrix;
        self.previous_view_matrix = self.view_matrix;
        self.previous_projection_matrix = self.projection_matrix;

        self.view_matrix = camera.view_matrix();
        self.projection_matrix = camera.projection_matrix();
        self.view_projection_matrix = self.projection_matrix * self.view_matrix;

        self.inverse_view_matrix = self.view_matrix.inverse();
        self.inverse_projection_matrix = self.projection_matrix.inverse();
        self.inverse_view_projection_matrix = self.view_projection_matrix.inverse();

        self.previous_position = self.position;
        self.position = camera.position;

        self.previous_view = self.view;
        self.view = camera.forward();

        self.right = camera.right();
        self.up = camera.up();
    }
}

#[derive(AsStd140, AsStd430, Default)]
pub struct ViewUniform {
    width: u32,
    height: u32,
    frame_count: u32,
}

impl ViewUniform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, render_state: &RenderState) {
        self.width = render_state.size.width;
        self.height = render_state.size.height;
        self.frame_count = self.frame_count.wrapping_add(1);
    }
}

#[derive(AsStd140, AsStd430, Default)]
pub struct ScreenUniform {
    camera: CameraUniform,
    view: ViewUniform,
}

impl ScreenUniform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, camera: &Camera, render_state: &RenderState) {
        self.camera.update(camera);
        self.view.update(render_state);
    }
}

pub struct ScreenBuffer {
    data: ScreenUniform,
    buffer: WgpuBuffer,
}

impl ScreenBuffer {
    pub fn new(render_state: &RenderState) -> Self {
        let data = ScreenUniform::new();
        let buffer_size = data.as_std430().as_slice().len();

        Self {
            data,
            buffer: render_state.create_buffer(
                "Screen Uniforms Buffer",
                WgpuBufferConfig {
                    data: BufferData::Uninit(buffer_size),
                    ty: WgpuBufferType::Storage,
                    usage: wgpu::BufferUsages::COPY_DST,
                },
            ),
        }
    }

    pub fn update(&mut self, render_state: &RenderState, camera: &Camera) {
        self.data.update(camera, render_state);
        self.buffer.write(&render_state.queue, &self.data);
    }
}
