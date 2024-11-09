use gpu_bytes::{AsStd140, AsStd430};
use gpu_bytes_derive::{AsStd140, AsStd430};

use crate::{
    engine::{
        render_state::{GpuState, RenderState},
        render_state_ext::{
            buffer::{BufferData, WgpuBuffer, WgpuBufferConfig, WgpuBufferType},
            RenderStateExt,
        },
    },
    state::object::{Aabb, ObjectList, Plane, Sphere},
};

use super::MIN_STORAGE_ARRAY_CAPACITY;

pub trait UpdateFromObjectList {
    fn update(&mut self, object_list: &ObjectList);
}

pub struct ObjectListBuffer<T: AsStd140 + AsStd430 + UpdateFromObjectList + Default> {
    pub name: String,
    pub data: T,
    pub buffer: WgpuBuffer,
    gpu_state: GpuState,
}

impl<T: AsStd140 + AsStd430 + UpdateFromObjectList + Default> ObjectListBuffer<T> {
    pub fn new(name: &str, render_state: &RenderState) -> Self {
        let gpu_state = render_state.get_gpu_state();

        let data = T::default();
        let buffer_size = data.as_std430().align().as_slice().len();

        Self {
            data,
            buffer: gpu_state.create_buffer(
                name,
                WgpuBufferConfig {
                    data: BufferData::Uninit(buffer_size),
                    ty: WgpuBufferType::Storage,
                    usage: wgpu::BufferUsages::COPY_DST,
                },
            ),
            name: name.to_owned(),
            gpu_state,
        }
    }

    /// returns true if reallocated
    pub fn update(&mut self, object_list: &ObjectList) -> bool {
        self.data.update(object_list);

        let mut data = self.data.as_std430();
        data.align();

        let data_size = data.as_slice().len();

        // reallocate if the buffer can't hold the data
        if self.buffer.len() < data_size {
            log::info!("{} reallocated", &self.name);

            // reallocate the buffer to fit the data
            self.buffer = self.gpu_state.create_buffer(
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
        self.num_spheres = object_list.spheres().len() as u32;

        self.list = Vec::with_capacity(self.list.capacity());
        self.list.extend_from_slice(object_list.spheres());
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
        self.num_planes = object_list.planes().len() as u32;

        self.list = Vec::with_capacity(self.list.capacity());
        self.list.extend_from_slice(object_list.planes());
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
        self.num_aabbs = object_list.aabbs().len() as u32;

        self.list = Vec::with_capacity(self.list.capacity());
        self.list.extend_from_slice(object_list.aabbs());
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
