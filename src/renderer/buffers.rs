use glam::{Mat3, Mat4, Vec3};
use gpu_bytes::{AsStd140, AsStd430, Std140Bytes, Std430Bytes};
use gpu_bytes_derive::{AsStd140, AsStd430};

use crate::{
    engine::{
        render_state::RenderState,
        render_state_ext::{
            buffer::{BufferData, WgpuBuffer, WgpuBufferConfig, WgpuBufferType},
            RenderStateExt,
        },
    },
    state::{
        camera::Camera,
        material::Material,
        object::{ObjectList, Sphere},
    },
};

/// Runtime-size arrays in storage buffers will allocate at least this many elements to avoid allocating
/// buffers on the gpu with zero size.
pub const MIN_STORAGE_ARRAY_CAPACITY: usize = 1;

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
    pub fn update(&mut self, render_state: &RenderState) {
        self.width = render_state.size.width;
        self.height = render_state.size.height;
        self.frame_count = self.frame_count.wrapping_add(1);
    }
}

#[derive(AsStd140, AsStd430, Default)]
pub struct ScreenUniform {
    pub camera: CameraUniform,
    pub view: ViewUniform,
}

impl ScreenUniform {
    pub fn update(&mut self, camera: &Camera, render_state: &RenderState) {
        self.camera.update(camera);
        self.view.update(render_state);
    }
}

pub struct ScreenBuffer {
    pub data: ScreenUniform,
    pub buffer: WgpuBuffer,
}

impl ScreenBuffer {
    pub fn new(render_state: &RenderState) -> Self {
        let data = ScreenUniform::default();
        let buffer_size = data.as_std430().align().as_slice().len();

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

#[derive(AsStd140, AsStd430)]
pub struct SphereListUniform {
    pub num_spheres: u32,
    pub list: Vec<Sphere>,
}

impl SphereListUniform {
    pub fn update(&mut self, object_list: &ObjectList) {
        self.num_spheres = object_list.spheres.len() as u32;

        self.list = Vec::with_capacity(self.list.capacity());
        self.list.extend_from_slice(&object_list.spheres);
        // self.list = object_list.spheres.clone();
        // self.list
        //     .reserve(object_list.spheres.capacity() - self.list.capacity());
    }
}

impl Default for SphereListUniform {
    fn default() -> Self {
        Self {
            num_spheres: 0,
            list: Vec::with_capacity(MIN_STORAGE_ARRAY_CAPACITY),
        }
    }
}

pub struct SphereListBuffer {
    pub data: SphereListUniform,
    pub buffer: WgpuBuffer,
}

impl SphereListBuffer {
    pub fn new(render_state: &RenderState) -> Self {
        let data = SphereListUniform::default();
        let buffer_size = data.as_std430().align().as_slice().len();

        Self {
            data,
            buffer: render_state.create_buffer(
                "Sphere List Buffer",
                WgpuBufferConfig {
                    data: BufferData::Uninit(buffer_size),
                    ty: WgpuBufferType::Storage,
                    usage: wgpu::BufferUsages::COPY_DST,
                },
            ),
        }
    }

    /// returns true if reallocated
    pub fn update(&mut self, render_state: &RenderState, object_list: &ObjectList) -> bool {
        self.data.update(object_list);

        let mut data = self.data.as_std430();
        data.align();

        let data_size = data.as_slice().len();

        // reallocate if the buffer can't hold the data
        if self.buffer.len() < data_size {
            log::info!("Sphere list buffer reallocated");

            // reallocate the buffer to fit the data
            self.buffer = render_state.create_buffer(
                "Sphere List Buffer",
                WgpuBufferConfig {
                    data: BufferData::Init(data.as_slice()),
                    ty: WgpuBufferType::Storage,
                    usage: wgpu::BufferUsages::COPY_DST,
                },
            );

            true
        } else {
            // write to the existing buffer
            self.buffer.write(&render_state.queue, &self.data);

            false
        }
    }
}
