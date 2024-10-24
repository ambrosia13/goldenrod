use std::ops::Deref;

use gpu_bytes::{AsStd140, AsStd430};
use wgpu::util::DeviceExt;

pub enum BufferData<'a> {
    Init(&'a [u8]),
    Uninit(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WgpuBufferType {
    Storage,
    Uniform,
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
}

impl WgpuBuffer {
    pub(in crate::engine::render_state_ext) fn new<'a>(
        device: &wgpu::Device,
        name: &'a str,
        config: WgpuBufferConfig<'a>,
    ) -> Self {
        let (buffer, len) = match config.data {
            BufferData::Init(data) => (
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(name),
                    contents: data,
                    usage: config.usage
                        | match config.ty {
                            WgpuBufferType::Storage => wgpu::BufferUsages::STORAGE,
                            WgpuBufferType::Uniform => wgpu::BufferUsages::UNIFORM,
                        },
                }),
                data.len(),
            ),
            BufferData::Uninit(len) => (
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(name),
                    size: len as u64,
                    usage: config.usage
                        | match config.ty {
                            WgpuBufferType::Storage => wgpu::BufferUsages::STORAGE,
                            WgpuBufferType::Uniform => wgpu::BufferUsages::UNIFORM,
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
        }
    }

    pub fn write<T: AsStd140 + AsStd430>(&self, queue: &wgpu::Queue, data: &T) {
        match self.ty {
            WgpuBufferType::Storage => {
                let std430 = data.as_std430();
                queue.write_buffer(self, 0, std430.as_slice());
            }
            WgpuBufferType::Uniform => {
                let std140 = data.as_std140();
                queue.write_buffer(self, 0, std140.as_slice());
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
