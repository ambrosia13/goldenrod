use std::marker::PhantomData;

use gpu_bytes::{AsStd140, AsStd430};

use crate::engine::{
    render_state::GpuState,
    render_state_ext::{
        buffer::{Buffer, BufferConfig, BufferData, BufferType},
        RenderStateExt,
    },
};

pub mod bvh;
pub mod object;
pub mod profiler;
pub mod screen;

/// Runtime-size arrays in storage buffers will allocate at least this many elements to avoid allocating
/// buffers on the gpu with zero size.
pub const MIN_DYNAMIC_BUFFER_CAPACITY: usize = 1;

pub trait UpdateFromSource<S> {
    fn update(&mut self, source: &S);
}

pub struct DynamicBuffer<T, S>
where
    T: Default + AsStd140 + AsStd430 + UpdateFromSource<S>,
{
    pub name: String,
    pub data: T,
    pub buffer: Buffer,
    gpu_state: GpuState,
    _marker: PhantomData<S>,
}

impl<T, S> DynamicBuffer<T, S>
where
    T: Default + AsStd140 + AsStd430 + UpdateFromSource<S>,
{
    pub fn new(name: &str, gpu_state: impl RenderStateExt) -> Self {
        let data = T::default();
        let buffer_size = data.as_std430().align().as_slice().len();

        Self {
            name: name.to_owned(),
            data,
            buffer: Buffer::new(
                &gpu_state,
                name,
                BufferConfig {
                    data: BufferData::Uninit(buffer_size),
                    ty: BufferType::Storage,
                    usage: wgpu::BufferUsages::COPY_DST,
                },
            ),
            gpu_state: gpu_state.as_gpu_state(),
            _marker: PhantomData,
        }
    }

    pub fn update(&mut self, source: &S) -> bool {
        self.data.update(source);

        let mut data = self.data.as_std430();
        data.align();

        let data_size = data.as_slice().len();

        // reallocate if the buffer can't hold the data
        if self.buffer.len() < data_size {
            log::info!("{} dynamic buffer reallocated", &self.name);

            self.buffer = Buffer::new(
                &self.gpu_state,
                &self.name,
                BufferConfig {
                    data: BufferData::Init(data.as_slice()),
                    ty: BufferType::Storage,
                    usage: wgpu::BufferUsages::COPY_DST,
                },
            );

            true
        } else {
            // write to existing buffer
            self.buffer.write(&self.data);

            false
        }
    }
}
