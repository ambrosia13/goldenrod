use std::ops::{Deref, Range};

use gpu_bytes::{AsStd140, AsStd430};

use super::{binding::WgpuBinding, shader::WgpuShader};

#[derive(Debug, Default)]
pub struct WgpuPushConstantConfig {
    vertex: Option<Range<u32>>,
    fragment: Option<Range<u32>>,
    compute: Option<Range<u32>>,
}

impl WgpuPushConstantConfig {
    pub fn as_ranges(&self) -> [wgpu::PushConstantRange; 3] {
        [
            wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: self.vertex.clone().unwrap_or(0..0),
            },
            wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::FRAGMENT,
                range: self.fragment.clone().unwrap_or(0..0),
            },
            wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: self.compute.clone().unwrap_or(0..0),
            },
        ]
    }
}

pub struct WgpuPipelineLayoutConfig<'a> {
    pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    pub push_constant_config: WgpuPushConstantConfig,
}

pub struct WgpuComputePipelineConfig<'a> {
    pub layout: &'a wgpu::PipelineLayout,
    pub shader: &'a WgpuShader,
}

pub struct WgpuRenderPipelineConfig<'a> {
    pub layout: &'a wgpu::PipelineLayout,
    pub vertex_buffer_layouts: &'a [wgpu::VertexBufferLayout<'a>],
    pub vertex: &'a WgpuShader,
    pub fragment: &'a WgpuShader,
    pub targets: &'a [Option<wgpu::ColorTargetState>],
}
