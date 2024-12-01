use std::collections::VecDeque;

use glam::UVec3;
use gpu_bytes::AsStd430;
use gpu_bytes_derive::{AsStd140, AsStd430};

use crate::engine::{
    render_state::{GpuState, RenderState},
    render_state_ext::{
        binding::{Binding, BindingData, BindingEntry},
        pass::ComputePass,
        pipeline::{ComputePipelineConfig, PipelineLayoutConfig, PushConstantConfig},
        texture::{Texture, TextureConfig, TextureType},
        RenderStateExt,
    },
};

use super::buffer::profiler::{ProfilerBuffer, PROFILER_STEP_COUNT, PROFILER_STEP_SIZE};

#[derive(Default, AsStd140, AsStd430)]
pub struct DebugRenderSettings {
    pub enabled: u32,
    pub texture_width: u32,
    pub texture_height: u32,
    pub step: u32,
}

pub struct DebugRenderContext<'a> {
    pub texture: Texture<'a>,
    pub binding: Binding,
    pub pipeline: wgpu::ComputePipeline,
    gpu_state: GpuState,
}

impl<'a> DebugRenderContext<'a> {
    pub fn new(
        render_state: &RenderState,
        input_texture: &Texture,
        profiler_buffer: &ProfilerBuffer,
    ) -> Self {
        let gpu_state = render_state.get_gpu_state();

        let texture = Self::create_texture(&gpu_state, input_texture);
        let binding = Self::create_binding(&gpu_state, &texture, profiler_buffer);

        let push_constant_size = DebugRenderSettings::default()
            .as_std430()
            .align()
            .as_slice()
            .len();

        let shader = gpu_state.create_shader("assets/shaders/debug.wgsl");

        let pipeline_layout = gpu_state.create_pipeline_layout(PipelineLayoutConfig {
            bind_group_layouts: &[binding.bind_group_layout()],
            push_constant_config: PushConstantConfig {
                compute: Some(0..(push_constant_size as u32)),
                ..Default::default()
            },
        });

        let pipeline = gpu_state.create_compute_pipeline(
            "Debug Compute Pipeline",
            ComputePipelineConfig {
                layout: &pipeline_layout,
                shader: &shader,
            },
        );

        Self {
            texture,
            binding,
            pipeline,
            gpu_state,
        }
    }

    fn create_texture<'b>(gpu_state: &GpuState, input_texture: &Texture) -> Texture<'b> {
        Texture::new(gpu_state,
            "Debug Texture",
            TextureConfig {
                ty: TextureType::Texture2d,
                format: input_texture.texture().format(),
                // 4 pixels per point
                width: 4 * PROFILER_STEP_COUNT as u32,
                height: 128,
                depth: 1,
                mips: 1,
                address_mode: wgpu::AddressMode::ClampToEdge,
                filter_mode: wgpu::FilterMode::Nearest,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            },
        )
    }

    fn create_binding(
        gpu_state: &GpuState,
        texture: &Texture,
        profiler_buffer: &ProfilerBuffer,
    ) -> Binding {
        Binding::new(
            gpu_state,
            &[
                BindingEntry {
                    visibility: wgpu::ShaderStages::COMPUTE,
                    binding_data: BindingData::TextureStorage {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        texture_view: &texture.view(0..1, 0..1),
                        texture,
                    },
                    count: None,
                },
                BindingEntry {
                    visibility: wgpu::ShaderStages::COMPUTE,
                    binding_data: BindingData::Buffer {
                        buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                        buffer: &profiler_buffer.buffer,
                    },
                    count: None,
                },
            ],
        )
    }

    pub fn on_profiler_update(
        &mut self,
        input_texture: &Texture,
        profiler_buffer: &ProfilerBuffer,
    ) {
        self.texture = Self::create_texture(&self.gpu_state, input_texture);
        self.binding = Self::create_binding(&self.gpu_state, &self.texture, profiler_buffer);
    }

    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        destination_texture: &Texture,
        enabled: bool,
    ) {
        if enabled {
            let workgroup_sizes = UVec3::new(8, 8, 1);

            let dimensions = UVec3::new(
                self.texture.texture().width(),
                self.texture.texture().height(),
                1,
            );

            let mut workgroups = dimensions / workgroup_sizes;

            // Add an extra workgroup in each dimension if the number we calculated doesn't cover the whole dimensions
            workgroups += (dimensions % workgroups).clamp(UVec3::ZERO, UVec3::ONE);

            let settings = DebugRenderSettings {
                enabled: enabled as u32,
                texture_width: self.texture.texture().width(),
                texture_height: self.texture.texture().height(),
                step: PROFILER_STEP_SIZE as u32,
            };

            let compute_pass = ComputePass {
                name: "Debug Render Pass",
                workgroups,
                pipeline: &self.pipeline,
                bindings: &[&self.binding],
                push_constants: Some(settings.as_std430()),
            };

            compute_pass.draw(encoder);

            // Copy the texture to the destination texture
            encoder.copy_texture_to_texture(
                self.texture.texture().as_image_copy(),
                destination_texture.texture().as_image_copy(),
                self.texture.texture().size(),
            );
        }
    }
}
