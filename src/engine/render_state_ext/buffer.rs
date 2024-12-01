use std::ops::Deref;

use gpu_bytes::{AsStd140, AsStd430};

use crate::engine::render_state::GpuState;

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
