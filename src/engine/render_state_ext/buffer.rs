use std::ops::Deref;

use gpu_bytes::{AsStd140, AsStd430};
use wgpu::util::DeviceExt;

use crate::engine::render_state::GpuState;

use super::RenderStateExt;

pub enum BufferData<'a> {
    Init(&'a [u8]),
    Uninit(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    Storage,
    Uniform,
    Vertex,
    Index,
}

pub struct BufferConfig<'a> {
    pub data: BufferData<'a>,
    pub ty: BufferType,
    pub usage: wgpu::BufferUsages,
}

pub struct Buffer {
    pub(in crate::engine::render_state_ext) buffer: wgpu::Buffer,
    pub(in crate::engine::render_state_ext) ty: BufferType,
    pub(in crate::engine::render_state_ext) len: usize,

    pub(in crate::engine::render_state_ext) gpu_state: GpuState,
}

impl Buffer {
    pub fn new<'a>(
        gpu_state: &impl RenderStateExt,
        name: &'a str,
        config: BufferConfig<'a>,
    ) -> Self {
        let (buffer, len) = match config.data {
            BufferData::Init(data) => (
                gpu_state
                    .device()
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(name),
                        contents: data,
                        usage: config.usage
                            | match config.ty {
                                BufferType::Storage => wgpu::BufferUsages::STORAGE,
                                BufferType::Uniform => wgpu::BufferUsages::UNIFORM,
                                BufferType::Vertex => wgpu::BufferUsages::VERTEX,
                                BufferType::Index => wgpu::BufferUsages::INDEX,
                            },
                    }),
                data.len(),
            ),
            BufferData::Uninit(len) => (
                gpu_state.device().create_buffer(&wgpu::BufferDescriptor {
                    label: Some(name),
                    size: len as u64,
                    usage: config.usage
                        | match config.ty {
                            BufferType::Storage => wgpu::BufferUsages::STORAGE,
                            BufferType::Uniform => wgpu::BufferUsages::UNIFORM,
                            BufferType::Vertex => wgpu::BufferUsages::VERTEX,
                            BufferType::Index => wgpu::BufferUsages::INDEX,
                        },
                    mapped_at_creation: false,
                }),
                len,
            ),
        };

        Self {
            buffer,
            ty: config.ty,
            len,
            gpu_state: gpu_state.as_gpu_state(),
        }
    }

    pub fn write<T: AsStd140 + AsStd430>(&self, data: &T) {
        match self.ty {
            BufferType::Storage => {
                let mut std430 = data.as_std430();
                std430.align();

                self.gpu_state
                    .queue
                    .write_buffer(self, 0, std430.as_slice());
            }
            BufferType::Uniform | BufferType::Vertex | BufferType::Index => {
                let mut std140 = data.as_std140();
                std140.align();

                self.gpu_state
                    .queue
                    .write_buffer(self, 0, std140.as_slice());
            }
        }
    }

    pub fn buffer_type(&self) -> BufferType {
        self.ty
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Deref for Buffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
