use std::ops::Deref;

use gpu_bytes::{AsStd140, AsStd430};
use wgpu::util::DeviceExt;

use crate::engine::render_state::{GpuContext, RenderState};

pub enum BufferData<'a> {
    Init(&'a [u8]),
    Uninit(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WgpuBufferType {
    Storage,
    Uniform,
    Vertex,
    Index,
}

pub struct WgpuBufferConfig<'a> {
    pub data: BufferData<'a>,
    pub ty: WgpuBufferType,
    pub usage: wgpu::BufferUsages,
}

pub struct WgpuBuffer {
    pub(in crate::engine::render_state_ext) buffer: wgpu::Buffer,
    pub(in crate::engine::render_state_ext) ty: WgpuBufferType,
    pub(in crate::engine::render_state_ext) len: usize,

    pub(in crate::engine::render_state_ext) ctx: GpuContext,
}

impl WgpuBuffer {
    pub(in crate::engine::render_state_ext) fn new<'a>(
        render_state: &RenderState,
        name: &'a str,
        config: WgpuBufferConfig<'a>,
    ) -> Self {
        let ctx = render_state.ctx();

        let (buffer, len) = match config.data {
            BufferData::Init(data) => (
                ctx.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(name),
                        contents: data,
                        usage: config.usage
                            | match config.ty {
                                WgpuBufferType::Storage => wgpu::BufferUsages::STORAGE,
                                WgpuBufferType::Uniform => wgpu::BufferUsages::UNIFORM,
                                WgpuBufferType::Vertex => wgpu::BufferUsages::VERTEX,
                                WgpuBufferType::Index => wgpu::BufferUsages::INDEX,
                            },
                    }),
                data.len(),
            ),
            BufferData::Uninit(len) => (
                ctx.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(name),
                    size: len as u64,
                    usage: config.usage
                        | match config.ty {
                            WgpuBufferType::Storage => wgpu::BufferUsages::STORAGE,
                            WgpuBufferType::Uniform => wgpu::BufferUsages::UNIFORM,
                            WgpuBufferType::Vertex => wgpu::BufferUsages::VERTEX,
                            WgpuBufferType::Index => wgpu::BufferUsages::INDEX,
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
            ctx,
        }
    }

    pub fn write<T: AsStd140 + AsStd430>(&self, data: &T) {
        match self.ty {
            WgpuBufferType::Storage => {
                let mut std430 = data.as_std430();
                std430.align();

                self.ctx.queue.write_buffer(self, 0, std430.as_slice());
            }
            WgpuBufferType::Uniform | WgpuBufferType::Vertex | WgpuBufferType::Index => {
                let mut std140 = data.as_std140();
                std140.align();

                self.ctx.queue.write_buffer(self, 0, std140.as_slice());
            }
        }
    }

    pub fn buffer_type(&self) -> WgpuBufferType {
        self.ty
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Deref for WgpuBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
