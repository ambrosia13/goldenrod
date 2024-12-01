use gpu_bytes::AsStd430;
use gpu_bytes_derive::AsStd430;
use winit::dpi::PhysicalSize;

use crate::engine::{
    render_state::{GpuState, RenderState},
    render_state_ext::{
        binding::{Binding, BindingData, BindingEntry},
        pass::RenderPass,
        pipeline::{PipelineLayoutConfig, PushConstantConfig, RenderPipelineConfig},
        shader::Shader,
        texture::{Texture, TextureConfig, TextureType},
        RenderStateExt,
    },
};

use super::{buffer::screen::ScreenBuffer, screen_quad::ScreenQuad};

#[derive(AsStd430)]
struct LodInfo {
    pub current_lod: u32,
    pub max_lod: u32,
}

pub struct BloomRenderContext<'a> {
    pub downsample_pipeline: wgpu::RenderPipeline,
    pub downsample_pipeline_layout: wgpu::PipelineLayout,
    pub downsample_shader: Shader,
    pub downsample_bindings: Vec<Binding>,
    pub downsample_texture: Texture<'a>,

    pub first_upsample_pipeline: wgpu::RenderPipeline,
    pub first_upsample_pipeline_layout: wgpu::PipelineLayout,
    pub first_upsample_shader: Shader,
    pub first_upsample_binding: Binding,
    pub upsample_pipeline: wgpu::RenderPipeline,
    pub upsample_pipeline_layout: wgpu::PipelineLayout,
    pub upsample_shader: Shader,
    pub upsample_bindings: Vec<Binding>,
    pub upsample_texture: Texture<'a>,

    pub bloom_texture: Texture<'a>,
    pub merge_pipeline: wgpu::RenderPipeline,
    pub merge_pipeline_layout: wgpu::PipelineLayout,
    pub merge_shader: Shader,
    pub merge_binding: Binding,

    pub push_constant_config: PushConstantConfig,

    pub mip_levels: u32,

    gpu_state: GpuState,
    screen_quad: ScreenQuad,
}

impl<'a> BloomRenderContext<'a> {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
    pub const ADDRESS_MODE: wgpu::AddressMode = wgpu::AddressMode::ClampToBorder;

    pub fn new(
        render_state: &RenderState,
        screen_quad: &ScreenQuad,
        input_texture: &Texture,
        screen_buffer: &ScreenBuffer,
    ) -> Self {
        let mip_levels = Self::calculate_mip_levels(
            input_texture.texture().width(),
            input_texture.texture().height(),
        );

        let push_constant_config = PushConstantConfig {
            fragment: Some(0..8),
            ..Default::default()
        };

        let gpu_state = render_state.get_gpu_state();

        let (downsample_texture, upsample_texture, bloom_texture) =
            Self::create_bloom_textures(&gpu_state, render_state.size, mip_levels);

        let (downsample_bindings, first_upsample_binding, upsample_bindings, merge_binding) =
            Self::create_bindings(
                &gpu_state,
                &downsample_texture,
                &upsample_texture,
                input_texture,
                screen_buffer,
                mip_levels,
            );

        let downsample_shader =
            gpu_state.create_shader("assets/shaders/bloom/bloom_downsample.wgsl");
        let (downsample_pipeline_layout, downsample_pipeline) = Self::create_pipelines(
            &gpu_state,
            "Bloom Downsample Render Pipeline",
            &downsample_bindings[0],
            &push_constant_config,
            screen_quad,
            &downsample_shader,
        );

        let first_upsample_shader =
            gpu_state.create_shader("assets/shaders/bloom/bloom_upsample_first.wgsl");
        let (first_upsample_pipeline_layout, first_upsample_pipeline) = Self::create_pipelines(
            &gpu_state,
            "First Bloom Upsample Render Pipeline",
            &first_upsample_binding,
            &push_constant_config,
            screen_quad,
            &first_upsample_shader,
        );

        let upsample_shader = gpu_state.create_shader("assets/shaders/bloom/bloom_upsample.wgsl");
        let (upsample_pipeline_layout, upsample_pipeline) = Self::create_pipelines(
            &gpu_state,
            "Bloom Upsample Render Pipeline",
            &upsample_bindings[0],
            &push_constant_config,
            screen_quad,
            &upsample_shader,
        );

        let merge_shader = gpu_state.create_shader("assets/shaders/bloom/bloom_merge.wgsl");
        let (merge_pipeline_layout, merge_pipeline) = Self::create_pipelines(
            &gpu_state,
            "Bloom Merge Render Pipeline",
            &merge_binding,
            &push_constant_config,
            screen_quad,
            &merge_shader,
        );

        Self {
            downsample_pipeline,
            downsample_pipeline_layout,
            downsample_shader,
            downsample_bindings,
            downsample_texture,
            first_upsample_pipeline,
            first_upsample_pipeline_layout,
            first_upsample_shader,
            first_upsample_binding,
            upsample_pipeline,
            upsample_pipeline_layout,
            upsample_shader,
            upsample_bindings,
            upsample_texture,
            bloom_texture,
            merge_pipeline,
            merge_pipeline_layout,
            merge_shader,
            merge_binding,
            push_constant_config,
            mip_levels,
            gpu_state,
            screen_quad: screen_quad.clone(),
        }
    }

    fn calculate_mip_levels(width: u32, height: u32) -> u32 {
        let min_dim = width.min(height);
        f32::log2(min_dim as f32) as u32
    }

    // returns (downsample_texture, upsample_texture, bloom_texture)
    fn create_bloom_textures<'b>(
        gpu_state: &GpuState,
        size: PhysicalSize<u32>,
        mip_levels: u32,
    ) -> (Texture<'b>, Texture<'b>, Texture<'b>) {
        let config = TextureConfig {
            ty: TextureType::Texture2d,
            format: BloomRenderContext::TEXTURE_FORMAT,
            width: size.width,
            height: size.height,
            depth: 1,
            mips: mip_levels,
            address_mode: BloomRenderContext::ADDRESS_MODE,
            filter_mode: wgpu::FilterMode::Linear,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
        };

        (
            Texture::new(gpu_state, "Bloom Downsample Texture", config.clone()),
            Texture::new(gpu_state, "Bloom Upsample Texture", config.clone()),
            Texture::new(
                gpu_state,
                "Bloom Texture",
                TextureConfig { mips: 1, ..config },
            ),
        )
    }

    fn create_bindings(
        gpu_state: &GpuState,
        downsample_texture: &Texture,
        upsample_texture: &Texture,
        input_texture: &Texture,
        screen_buffer: &ScreenBuffer,
        mip_levels: u32,
    ) -> (Vec<Binding>, Binding, Vec<Binding>, Binding) {
        let mut downsample_bindings = Vec::with_capacity(mip_levels as usize);
        let mut upsample_bindings = Vec::with_capacity(mip_levels as usize);

        downsample_bindings.push(Binding::new(
            gpu_state,
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
                BindingEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding_data: BindingData::Buffer {
                        buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                        buffer: &screen_buffer.buffer,
                    },
                    count: None,
                },
            ],
        ));

        for target_mip in 1..mip_levels {
            downsample_bindings.push(Binding::new(
                gpu_state,
                &[
                    BindingEntry {
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        binding_data: BindingData::TextureView {
                            texture: downsample_texture,
                            texture_view: &downsample_texture
                                .view((target_mip - 1)..target_mip, 0..1),
                        },
                        count: None,
                    },
                    BindingEntry {
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        binding_data: BindingData::TextureSampler {
                            sampler_type: wgpu::SamplerBindingType::Filtering,
                            texture: downsample_texture,
                        },
                        count: None,
                    },
                    BindingEntry {
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        binding_data: BindingData::Buffer {
                            buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                            buffer: &screen_buffer.buffer,
                        },
                        count: None,
                    },
                ],
            ));
        }

        let first_upsample_binding = Binding::new(
            gpu_state,
            &[
                BindingEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding_data: BindingData::TextureView {
                        texture: downsample_texture,
                        texture_view: &downsample_texture.view((mip_levels - 1)..mip_levels, 0..1),
                    },
                    count: None,
                },
                BindingEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding_data: BindingData::TextureSampler {
                        sampler_type: wgpu::SamplerBindingType::Filtering,
                        texture: downsample_texture,
                    },
                    count: None,
                },
            ],
        );

        for target_mip in 0..(mip_levels - 1) {
            upsample_bindings.push(Binding::new(
                gpu_state,
                &[
                    BindingEntry {
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        binding_data: BindingData::TextureView {
                            texture: upsample_texture,
                            texture_view: &upsample_texture
                                .view((target_mip + 1)..(target_mip + 2), 0..1),
                        },
                        count: None,
                    },
                    BindingEntry {
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        binding_data: BindingData::TextureSampler {
                            sampler_type: wgpu::SamplerBindingType::Filtering,
                            texture: upsample_texture,
                        },
                        count: None,
                    },
                    BindingEntry {
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        binding_data: BindingData::TextureView {
                            texture: downsample_texture,
                            texture_view: &downsample_texture
                                .view(target_mip..(target_mip + 1), 0..1),
                        },
                        count: None,
                    },
                    BindingEntry {
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        binding_data: BindingData::TextureSampler {
                            sampler_type: wgpu::SamplerBindingType::Filtering,
                            texture: downsample_texture,
                        },
                        count: None,
                    },
                    BindingEntry {
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        binding_data: BindingData::Buffer {
                            buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                            buffer: &screen_buffer.buffer,
                        },
                        count: None,
                    },
                ],
            ))
        }

        let merge_binding = Binding::new(
            gpu_state,
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
                BindingEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding_data: BindingData::TextureView {
                        texture: upsample_texture,
                        texture_view: &upsample_texture.view(0..1, 0..1),
                    },
                    count: None,
                },
                BindingEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    binding_data: BindingData::TextureSampler {
                        sampler_type: wgpu::SamplerBindingType::Filtering,
                        texture: upsample_texture,
                    },
                    count: None,
                },
            ],
        );

        (
            downsample_bindings,
            first_upsample_binding,
            upsample_bindings,
            merge_binding,
        )
    }

    fn create_pipelines(
        gpu_state: &GpuState,
        name: &str,
        binding: &Binding,
        push_constant_config: &PushConstantConfig,
        screen_quad: &ScreenQuad,
        shader: &Shader,
    ) -> (wgpu::PipelineLayout, wgpu::RenderPipeline) {
        let layout = gpu_state.create_pipeline_layout(PipelineLayoutConfig {
            bind_group_layouts: &[
                screen_quad.vertex_index_binding.bind_group_layout(),
                binding.bind_group_layout(),
            ],
            push_constant_config: push_constant_config.clone(),
        });

        let pipeline = gpu_state.create_render_pipeline(
            name,
            RenderPipelineConfig {
                layout: &layout,
                vertex_buffer_layouts: &[],
                vertex: &screen_quad.vertex_shader,
                fragment: shader,
                targets: &[Some(wgpu::ColorTargetState {
                    format: BloomRenderContext::TEXTURE_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::all(),
                })],
            },
        );

        (layout, pipeline)
    }

    fn recreate_textures(&mut self, new_size: PhysicalSize<u32>) {
        self.mip_levels = Self::calculate_mip_levels(new_size.width, new_size.height);

        self.bloom_texture.resize(new_size.width, new_size.height);

        self.downsample_texture.texture_descriptor.size.width = new_size.width;
        self.downsample_texture.texture_descriptor.size.height = new_size.height;
        self.downsample_texture.texture_descriptor.mip_level_count = self.mip_levels;
        self.downsample_texture.recreate();

        self.upsample_texture.texture_descriptor.size.width = new_size.width;
        self.upsample_texture.texture_descriptor.size.height = new_size.height;
        self.upsample_texture.texture_descriptor.mip_level_count = self.mip_levels;
        self.upsample_texture.recreate();
    }

    fn recreate_bindings(&mut self, input_texture: &Texture, screen_buffer: &ScreenBuffer) {
        (
            self.downsample_bindings,
            self.first_upsample_binding,
            self.upsample_bindings,
            self.merge_binding,
        ) = Self::create_bindings(
            &self.gpu_state,
            &self.downsample_texture,
            &self.upsample_texture,
            input_texture,
            screen_buffer,
            self.mip_levels,
        );
    }

    fn recreate_pipelines(&mut self) {
        (self.downsample_pipeline_layout, self.downsample_pipeline) = Self::create_pipelines(
            &self.gpu_state,
            "Bloom Downsample Render Pipeline",
            &self.downsample_bindings[0],
            &self.push_constant_config,
            &self.screen_quad,
            &self.downsample_shader,
        );

        (
            self.first_upsample_pipeline_layout,
            self.first_upsample_pipeline,
        ) = Self::create_pipelines(
            &self.gpu_state,
            "First Bloom Upsample Render Pipeline",
            &self.first_upsample_binding,
            &self.push_constant_config,
            &self.screen_quad,
            &self.first_upsample_shader,
        );

        (self.upsample_pipeline_layout, self.upsample_pipeline) = Self::create_pipelines(
            &self.gpu_state,
            "Bloom Upsample Render Pipeline",
            &self.upsample_bindings[0],
            &self.push_constant_config,
            &self.screen_quad,
            &self.upsample_shader,
        );

        (self.merge_pipeline_layout, self.merge_pipeline) = Self::create_pipelines(
            &self.gpu_state,
            "Bloom Merge Render Pipeline",
            &self.merge_binding,
            &self.push_constant_config,
            &self.screen_quad,
            &self.merge_shader,
        );
    }

    pub fn recompile_shaders(&mut self) {
        self.downsample_shader.recreate();
        self.first_upsample_shader.recreate();
        self.upsample_shader.recreate();
        self.merge_shader.recreate();

        self.recreate_pipelines();
    }

    pub fn resize(
        &mut self,
        new_size: PhysicalSize<u32>,
        input_texture: &Texture,
        screen_buffer: &ScreenBuffer,
    ) {
        self.recreate_textures(new_size);
        self.recreate_bindings(input_texture, screen_buffer);
    }

    fn draw_downsample(&self, encoder: &mut wgpu::CommandEncoder) {
        for target_mip in 0..self.mip_levels {
            let view = self
                .downsample_texture
                .view(target_mip..(target_mip + 1), 0..1);

            let render_pass = RenderPass {
                name: &format!("Bloom Downsample Pass (lod = {})", target_mip),
                color_attachments: &[Some(&view)],
                pipeline: &self.downsample_pipeline,
                bindings: &[
                    &self.screen_quad.vertex_index_binding,
                    &self.downsample_bindings[target_mip as usize],
                ],
                push_constants: Some(vec![(
                    wgpu::ShaderStages::FRAGMENT,
                    LodInfo {
                        current_lod: target_mip,
                        max_lod: self.mip_levels,
                    }
                    .as_std430(),
                )]),
            };

            render_pass.draw(encoder);
        }
    }

    fn draw_upsample(&self, encoder: &mut wgpu::CommandEncoder) {
        let first_view = self
            .upsample_texture
            .view((self.mip_levels - 1)..self.mip_levels, 0..1);

        let first_render_pass = RenderPass {
            name: "First Bloom Upsample Render Pass",
            color_attachments: &[Some(&first_view)],
            pipeline: &self.first_upsample_pipeline,
            bindings: &[
                &self.screen_quad.vertex_index_binding,
                &self.first_upsample_binding,
            ],
            push_constants: Some(vec![(
                wgpu::ShaderStages::FRAGMENT,
                LodInfo {
                    current_lod: self.mip_levels - 1,
                    max_lod: self.mip_levels,
                }
                .as_std430(),
            )]),
        };

        first_render_pass.draw(encoder);

        for target_mip in (0..(self.mip_levels - 1)).rev() {
            let view = self
                .upsample_texture
                .view(target_mip..(target_mip + 1), 0..1);

            let render_pass = RenderPass {
                name: &format!("Bloom Upsample Render Pass (lod = {})", target_mip),
                color_attachments: &[Some(&view)],
                pipeline: &self.upsample_pipeline,
                bindings: &[
                    &self.screen_quad.vertex_index_binding,
                    &self.upsample_bindings[target_mip as usize],
                ],
                push_constants: Some(vec![(
                    wgpu::ShaderStages::FRAGMENT,
                    LodInfo {
                        current_lod: target_mip,
                        max_lod: self.mip_levels,
                    }
                    .as_std430(),
                )]),
            };

            render_pass.draw(encoder);
        }
    }

    fn draw_merge(&self, encoder: &mut wgpu::CommandEncoder) {
        let view = self.bloom_texture.view(0..1, 0..1);

        let render_pass = RenderPass {
            name: "Bloom Merge Render Pass",
            color_attachments: &[Some(&view)],
            pipeline: &self.merge_pipeline,
            bindings: &[&self.screen_quad.vertex_index_binding, &self.merge_binding],
            push_constants: Some(vec![(
                wgpu::ShaderStages::FRAGMENT,
                LodInfo {
                    current_lod: 0,
                    max_lod: self.mip_levels,
                }
                .as_std430(),
            )]),
        };

        render_pass.draw(encoder);
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder) {
        self.draw_downsample(encoder);
        self.draw_upsample(encoder);
        self.draw_merge(encoder);
    }
}
