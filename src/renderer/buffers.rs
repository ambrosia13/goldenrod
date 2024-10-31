use glam::{Mat4, Vec3};
use gpu_bytes::{AsStd140, AsStd430};
use gpu_bytes_derive::{AsStd140, AsStd430};

use crate::{
    engine::{
        render_state::{GpuContext, RenderState},
        render_state_ext::{
            buffer::{BufferData, WgpuBuffer, WgpuBufferConfig, WgpuBufferType},
            RenderStateExt,
        },
    },
    state::{
        camera::Camera,
        object::{Aabb, ObjectList, Plane, Sphere},
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
        self.buffer.write(&self.data);
    }
}

pub trait UpdateFromObjectList {
    fn update(&mut self, object_list: &ObjectList);
}

pub struct ObjectListBuffer<T: AsStd140 + AsStd430 + UpdateFromObjectList + Default> {
    pub name: String,
    pub data: T,
    pub buffer: WgpuBuffer,
    ctx: GpuContext,
}

impl<T: AsStd140 + AsStd430 + UpdateFromObjectList + Default> ObjectListBuffer<T> {
    pub fn new(name: &str, render_state: &RenderState) -> Self {
        let ctx = render_state.ctx();

        let data = T::default();
        let buffer_size = data.as_std430().align().as_slice().len();

        Self {
            data,
            buffer: render_state.create_buffer(
                name,
                WgpuBufferConfig {
                    data: BufferData::Uninit(buffer_size),
                    ty: WgpuBufferType::Storage,
                    usage: wgpu::BufferUsages::COPY_DST,
                },
            ),
            name: name.to_owned(),
            ctx,
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
            log::info!("{} reallocated", &self.name);

            // reallocate the buffer to fit the data
            self.buffer = render_state.create_buffer(
                &self.name,
                WgpuBufferConfig {
                    data: BufferData::Init(data.as_slice()),
                    ty: WgpuBufferType::Storage,
                    usage: wgpu::BufferUsages::COPY_DST,
                },
            );

            true
        } else {
            // write to the existing buffer
            self.buffer.write(&self.data);

            false
        }
    }
}

#[derive(AsStd140, AsStd430)]
pub struct SphereListUniform {
    pub num_spheres: u32,
    pub list: Vec<Sphere>,
}

impl UpdateFromObjectList for SphereListUniform {
    fn update(&mut self, object_list: &ObjectList) {
        self.num_spheres = object_list.spheres.len() as u32;

        self.list = Vec::with_capacity(self.list.capacity());
        self.list.extend_from_slice(&object_list.spheres);
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

#[derive(AsStd140, AsStd430)]
pub struct PlaneListUniform {
    pub num_planes: u32,
    pub list: Vec<Plane>,
}

impl UpdateFromObjectList for PlaneListUniform {
    fn update(&mut self, object_list: &ObjectList) {
        self.num_planes = object_list.planes.len() as u32;

        self.list = Vec::with_capacity(self.list.capacity());
        self.list.extend_from_slice(&object_list.planes);
    }
}

impl Default for PlaneListUniform {
    fn default() -> Self {
        Self {
            num_planes: 0,
            list: Vec::with_capacity(MIN_STORAGE_ARRAY_CAPACITY),
        }
    }
}

#[derive(AsStd140, AsStd430)]
pub struct AabbListUniform {
    pub num_aabbs: u32,
    pub list: Vec<Aabb>,
}

impl UpdateFromObjectList for AabbListUniform {
    fn update(&mut self, object_list: &ObjectList) {
        self.num_aabbs = object_list.aabbs.len() as u32;

        self.list = Vec::with_capacity(self.list.capacity());
        self.list.extend_from_slice(&object_list.aabbs);
    }
}

impl Default for AabbListUniform {
    fn default() -> Self {
        Self {
            num_aabbs: 0,
            list: Vec::with_capacity(MIN_STORAGE_ARRAY_CAPACITY),
        }
    }
}

pub type SphereListBuffer = ObjectListBuffer<SphereListUniform>;
pub type PlaneListBuffer = ObjectListBuffer<PlaneListUniform>;
pub type AabbListBuffer = ObjectListBuffer<AabbListUniform>;

// pub struct SphereListBuffer {
//     pub data: SphereListUniform,
//     pub buffer: WgpuBuffer,
// }

// impl SphereListBuffer {
//     pub fn new(render_state: &RenderState) -> Self {
//         let data = SphereListUniform::default();
//         let buffer_size = data.as_std430().align().as_slice().len();

//         Self {
//             data,
//             buffer: render_state.create_buffer(
//                 "Sphere List Buffer",
//                 WgpuBufferConfig {
//                     data: BufferData::Uninit(buffer_size),
//                     ty: WgpuBufferType::Storage,
//                     usage: wgpu::BufferUsages::COPY_DST,
//                 },
//             ),
//         }
//     }

//     /// returns true if reallocated
//     pub fn update(&mut self, render_state: &RenderState, object_list: &ObjectList) -> bool {
//         self.data.update(object_list);

//         let mut data = self.data.as_std430();
//         data.align();

//         let data_size = data.as_slice().len();

//         // reallocate if the buffer can't hold the data
//         if self.buffer.len() < data_size {
//             log::info!("Sphere list buffer reallocated");

//             // reallocate the buffer to fit the data
//             self.buffer = render_state.create_buffer(
//                 "Sphere List Buffer",
//                 WgpuBufferConfig {
//                     data: BufferData::Init(data.as_slice()),
//                     ty: WgpuBufferType::Storage,
//                     usage: wgpu::BufferUsages::COPY_DST,
//                 },
//             );

//             true
//         } else {
//             // write to the existing buffer
//             self.buffer.write(&render_state.queue, &self.data);

//             false
//         }
//     }
// }

// pub struct PlaneListBuffer {
//     pub data: PlaneListUniform,
//     pub buffer: WgpuBuffer,
// }

// impl PlaneListBuffer {
//     pub fn new(render_state: &RenderState) -> Self {
//         let data = PlaneListUniform::default();
//         let buffer_size = data.as_std430().align().as_slice().len();

//         Self {
//             data,
//             buffer: render_state.create_buffer(
//                 "Sphere List Buffer",
//                 WgpuBufferConfig {
//                     data: BufferData::Uninit(buffer_size),
//                     ty: WgpuBufferType::Storage,
//                     usage: wgpu::BufferUsages::COPY_DST,
//                 },
//             ),
//         }
//     }

//     /// returns true if reallocated
//     pub fn update(&mut self, render_state: &RenderState, object_list: &ObjectList) -> bool {
//         self.data.update(object_list);

//         let mut data = self.data.as_std430();
//         data.align();

//         let data_size = data.as_slice().len();

//         // reallocate if the buffer can't hold the data
//         if self.buffer.len() < data_size {
//             log::info!("Sphere list buffer reallocated");

//             // reallocate the buffer to fit the data
//             self.buffer = render_state.create_buffer(
//                 "Sphere List Buffer",
//                 WgpuBufferConfig {
//                     data: BufferData::Init(data.as_slice()),
//                     ty: WgpuBufferType::Storage,
//                     usage: wgpu::BufferUsages::COPY_DST,
//                 },
//             );

//             true
//         } else {
//             // write to the existing buffer
//             self.buffer.write(&render_state.queue, &self.data);

//             false
//         }
//     }
// }
