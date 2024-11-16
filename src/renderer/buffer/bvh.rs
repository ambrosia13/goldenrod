use gpu_bytes::AsStd430;
use gpu_bytes_derive::{AsStd140, AsStd430};

use crate::{
    engine::{
        render_state::{GpuState, RenderState},
        render_state_ext::{
            buffer::{BufferData, WgpuBuffer, WgpuBufferConfig, WgpuBufferType},
            RenderStateExt,
        },
    },
    state::bvh::{BoundingVolumeHierarchy, BvhNode},
};

use super::MIN_DYNAMIC_BUFFER_CAPACITY;

#[derive(AsStd140, AsStd430)]
pub struct BvhUniform {
    num_nodes: u32,
    nodes: Vec<BvhNode>,
}

impl BvhUniform {
    pub fn update(&mut self, bvh: &BoundingVolumeHierarchy) {
        self.num_nodes = bvh.nodes().len() as u32;
        self.nodes = bvh.nodes().to_vec();
    }
}

impl Default for BvhUniform {
    fn default() -> Self {
        Self {
            num_nodes: 0,
            nodes: Vec::with_capacity(MIN_DYNAMIC_BUFFER_CAPACITY),
        }
    }
}

pub struct BvhBuffer {
    pub data: BvhUniform,
    pub buffer: WgpuBuffer,
    gpu_state: GpuState,
}

impl BvhBuffer {
    pub fn new(render_state: &RenderState) -> Self {
        let gpu_state = render_state.get_gpu_state();

        let data = BvhUniform::default();
        let buffer_size = data.as_std430().align().as_slice().len();

        Self {
            data,
            buffer: gpu_state.create_buffer(
                "Bounding Volume Hierarchy Buffer",
                WgpuBufferConfig {
                    data: BufferData::Uninit(buffer_size),
                    ty: WgpuBufferType::Storage,
                    usage: wgpu::BufferUsages::COPY_DST,
                },
            ),
            gpu_state,
        }
    }

    pub fn update(&mut self, bvh: &BoundingVolumeHierarchy) -> bool {
        self.data.update(bvh);

        let mut data = self.data.as_std430();
        data.align();

        let data_size = data.as_slice().len();

        if self.buffer.len() < data_size {
            log::info!("BVH Buffer reallocated");

            self.buffer = self.gpu_state.create_buffer(
                "Bounding Volume Hierarchy Buffer",
                WgpuBufferConfig {
                    data: BufferData::Init(data.as_slice()),
                    ty: WgpuBufferType::Storage,
                    usage: wgpu::BufferUsages::COPY_DST,
                },
            );

            true
        } else {
            self.buffer.write(&self.data);

            false
        }
    }
}
