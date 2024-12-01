use std::ops::Range;

use super::shader::Shader;

#[derive(Debug, Default, Clone)]
pub struct PushConstantConfig {
    pub vertex: Option<Range<u32>>,
    pub fragment: Option<Range<u32>>,
    pub compute: Option<Range<u32>>,
}

impl PushConstantConfig {
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

pub struct PipelineLayoutConfig<'a> {
    pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    pub push_constant_config: PushConstantConfig,
}

pub struct ComputePipelineConfig<'a> {
    pub layout: &'a wgpu::PipelineLayout,
    pub shader: &'a Shader,
}

pub struct RenderPipelineConfig<'a> {
    pub layout: &'a wgpu::PipelineLayout,
    pub vertex_buffer_layouts: &'a [wgpu::VertexBufferLayout<'a>],
    pub vertex: &'a Shader,
    pub fragment: &'a Shader,
    pub targets: &'a [Option<wgpu::ColorTargetState>],
}
