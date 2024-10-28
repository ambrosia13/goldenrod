use std::ops::{Deref, Range};

use gpu_bytes::{AsStd140, AsStd430};

use super::{binding::WgpuBinding, shader::WgpuShader};

#[derive(Debug, Default)]
pub struct WgpuPushConstantConfig {
    pub vertex: Option<Range<u32>>,
    pub fragment: Option<Range<u32>>,
    pub compute: Option<Range<u32>>,
}

impl WgpuPushConstantConfig {
    pub fn as_ranges(&self) -> Vec<wgpu::PushConstantRange> {
        let mut ranges = Vec::new();

        if let Some(vertex) = &self.vertex {
            ranges.push(wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: vertex.clone(),
            });
        }

        if let Some(fragment) = &self.fragment {
            ranges.push(wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::FRAGMENT,
                range: fragment.clone(),
            });
        }

        if let Some(compute) = &self.compute {
            ranges.push(wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: compute.clone(),
            });
        }

        ranges
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
