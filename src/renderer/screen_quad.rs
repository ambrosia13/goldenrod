use std::sync::Arc;

use glam::Vec2;
use gpu_bytes::Std430Bytes;
use gpu_bytes_derive::{AsStd140, AsStd430};

use crate::engine::{
    render_state::RenderState,
    render_state_ext::{
        binding::{Binding, BindingData, BindingEntry},
        buffer::{Buffer, BufferConfig, BufferData, BufferType},
        shader::Shader,
        RenderStateExt,
    },
};

#[derive(Clone, Copy, AsStd140, AsStd430)]
pub struct ScreenVertex {
    position: Vec2,
    uv: Vec2,
    texcoord: Vec2,
}

impl ScreenVertex {
    const VERTICES: &'static [Self] = &[
        Self {
            position: Vec2::new(-1.0, -1.0),
            uv: Vec2::new(0.0, 1.0),
            texcoord: Vec2::new(0.0, 0.0),
        },
        Self {
            position: Vec2::new(1.0, -1.0),
            uv: Vec2::new(1.0, 1.0),
            texcoord: Vec2::new(1.0, 0.0),
        },
        Self {
            position: Vec2::new(1.0, 1.0),
            uv: Vec2::new(1.0, 0.0),
            texcoord: Vec2::new(1.0, 1.0),
        },
        Self {
            position: Vec2::new(-1.0, 1.0),
            uv: Vec2::new(0.0, 0.0),
            texcoord: Vec2::new(0.0, 1.0),
        },
    ];

    const INDICES: &'static [u32] = &[0, 1, 2, 0, 2, 3];

    pub fn vertices_std430() -> Std430Bytes {
        let mut buf = Std430Bytes::new();
        buf.write_array(Self::VERTICES);
        buf
    }

    pub fn indices_std430() -> Std430Bytes {
        let mut buf = Std430Bytes::new();
        buf.write_array(Self::INDICES);
        buf
    }
}

#[derive(Clone)]
pub struct ScreenQuad {
    pub vertex_storage_buffer: Arc<Buffer>,
    pub index_storage_buffer: Arc<Buffer>,

    pub vertex_index_binding: Arc<Binding>,
    pub vertex_shader: Arc<Shader>,
}

impl ScreenQuad {
    pub fn new(render_state: &RenderState) -> Self {
        let vertex_storage_buffer = render_state.create_buffer(
            "Screen Vertex Storage Buffer",
            BufferConfig {
                data: BufferData::Init(ScreenVertex::vertices_std430().align().as_slice()),
                ty: BufferType::Storage,
                usage: wgpu::BufferUsages::empty(),
            },
        );

        let index_storage_buffer = render_state.create_buffer(
            "Screen Index Storage Buffer",
            BufferConfig {
                data: BufferData::Init(ScreenVertex::indices_std430().align().as_slice()),
                ty: BufferType::Storage,
                usage: wgpu::BufferUsages::empty(),
            },
        );

        let vertex_index_binding = render_state.create_binding(&[
            BindingEntry {
                visibility: wgpu::ShaderStages::VERTEX,
                binding_data: BindingData::Buffer {
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                    buffer: &vertex_storage_buffer,
                },
                count: None,
            },
            BindingEntry {
                visibility: wgpu::ShaderStages::VERTEX,
                binding_data: BindingData::Buffer {
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                    buffer: &index_storage_buffer,
                },
                count: None,
            },
        ]);

        let vertex_shader = render_state.create_shader("assets/shaders/frame_vertex.wgsl");

        let vertex_storage_buffer = Arc::new(vertex_storage_buffer);
        let index_storage_buffer = Arc::new(index_storage_buffer);
        let vertex_index_binding = Arc::new(vertex_index_binding);
        let vertex_shader = Arc::new(vertex_shader);

        Self {
            vertex_storage_buffer,
            index_storage_buffer,
            vertex_index_binding,
            vertex_shader,
        }
    }
}
