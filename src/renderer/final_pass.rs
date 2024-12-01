use crate::engine::{
    render_state::{GpuState, RenderState},
    render_state_ext::{
        binding::{Binding, BindingData, BindingEntry},
        pass::RenderPass,
        pipeline::{PipelineLayoutConfig, PushConstantConfig, RenderPipelineConfig},
        shader::Shader,
        texture::Texture,
        RenderStateExt,
    },
};

use super::{buffer::screen::ScreenBuffer, screen_quad::ScreenQuad};

pub struct FinalRenderContext {
    pub shader: Shader,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,

    pub screen_binding: Binding,
    pub texture_binding: Binding,

    pub surface_format: wgpu::TextureFormat,

    gpu_state: GpuState,
    screen_quad: ScreenQuad,
}

impl FinalRenderContext {
    pub fn new(
        render_state: &RenderState,
        input_texture: &Texture,
        screen_buffer: &ScreenBuffer,
        screen_quad: &ScreenQuad,
    ) -> Self {
        let texture_binding = Binding::new(
            &render_state,
            &[
                BindingEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding_data: BindingData::TextureView {
                        texture: input_texture,
                        texture_view: &input_texture.view(0..1, 0..1),
                    },
                    count: None,
                },
                BindingEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding_data: BindingData::TextureSampler {
                        sampler_type: wgpu::SamplerBindingType::Filtering,
                        texture: input_texture,
                    },
                    count: None,
                },
            ],
        );

        let screen_binding = Binding::new(
            &render_state,
            &[BindingEntry {
                visibility: wgpu::ShaderStages::FRAGMENT,
                binding_data: BindingData::Buffer {
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                    buffer: &screen_buffer.buffer,
                },
                count: None,
            }],
        );

        let shader = render_state.create_shader("assets/shaders/final.wgsl");

        let pipeline_layout = render_state.create_pipeline_layout(PipelineLayoutConfig {
            bind_group_layouts: &[
                screen_quad.vertex_index_binding.bind_group_layout(),
                screen_binding.bind_group_layout(),
                texture_binding.bind_group_layout(),
            ],
            push_constant_config: PushConstantConfig::default(),
        });

        let pipeline = render_state.create_render_pipeline(
            "Final Pass Render Pipeline",
            RenderPipelineConfig {
                layout: &pipeline_layout,
                vertex_buffer_layouts: &[],
                vertex: &screen_quad.vertex_shader,
                fragment: &shader,
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_state.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            },
        );

        Self {
            shader,
            pipeline_layout,
            pipeline,
            screen_binding,
            texture_binding,
            surface_format: render_state.config.format,
            gpu_state: render_state.get_gpu_state(),
            screen_quad: screen_quad.clone(),
        }
    }

    fn recreate_pipeline(&mut self) {
        self.pipeline = self.gpu_state.create_render_pipeline(
            "Final Pass Render Pipeline",
            RenderPipelineConfig {
                layout: &self.pipeline_layout,
                vertex_buffer_layouts: &[],
                vertex: &self.screen_quad.vertex_shader,
                fragment: &self.shader,
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            },
        );
    }

    fn update_input_texture(&mut self, input_texture: &Texture) {
        self.texture_binding = Binding::new(
            &self.gpu_state,
            &[
                BindingEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding_data: BindingData::TextureView {
                        texture: input_texture,
                        texture_view: &input_texture.view(0..1, 0..1),
                    },
                    count: None,
                },
                BindingEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding_data: BindingData::TextureSampler {
                        sampler_type: wgpu::SamplerBindingType::Filtering,
                        texture: input_texture,
                    },
                    count: None,
                },
            ],
        );
    }

    // since the input texture is recreated when the screen is resized, so does the binding for it in this pass.
    pub fn resize(&mut self, input_texture: &Texture) {
        self.update_input_texture(input_texture);
    }

    pub fn recompile_shaders(&mut self) {
        self.shader.recreate();
        self.recreate_pipeline();
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, surface_texture: &wgpu::SurfaceTexture) {
        let view = surface_texture.texture.create_view(&Default::default());

        let render_pass = RenderPass {
            name: "Final Render Pass",
            color_attachments: &[Some(&view)],
            pipeline: &self.pipeline,
            bindings: &[
                &self.screen_quad.vertex_index_binding,
                &self.screen_binding,
                &self.texture_binding,
            ],
            push_constants: None,
        };

        render_pass.draw(encoder);
    }
}
