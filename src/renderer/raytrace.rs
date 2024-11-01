use glam::UVec3;
use winit::dpi::PhysicalSize;

use crate::engine::{
    render_state::{GpuState, RenderState},
    render_state_ext::{
        binding::{WgpuBinding, WgpuBindingData, WgpuBindingEntry},
        pass::WgpuComputePass,
        pipeline::{WgpuComputePipelineConfig, WgpuPipelineLayoutConfig, WgpuPushConstantConfig},
        shader::WgpuShader,
        texture::{WgpuTexture, WgpuTextureConfig, WgpuTextureType},
        RenderStateExt,
    },
};

use super::buffers::{AabbListBuffer, PlaneListBuffer, ScreenBuffer, SphereListBuffer};

pub struct RaytraceRenderContext<'a> {
    pub color_texture: WgpuTexture<'a>,
    pub color_texture_copy: WgpuTexture<'a>,

    pub shader: WgpuShader,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::ComputePipeline,

    pub screen_binding: WgpuBinding,
    pub object_binding: WgpuBinding,
    pub texture_binding: WgpuBinding,

    gpu_state: GpuState,
}

impl<'a> RaytraceRenderContext<'a> {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

    pub fn new(
        render_state: &RenderState,
        screen_buffer: &ScreenBuffer,
        sphere_list_buffer: &SphereListBuffer,
        plane_list_buffer: &PlaneListBuffer,
        aabb_list_buffer: &AabbListBuffer,
    ) -> Self {
        let color_texture_config = WgpuTextureConfig {
            ty: WgpuTextureType::Texture2d,
            format: Self::TEXTURE_FORMAT,
            width: render_state.size.width,
            height: render_state.size.height,
            depth: 1,
            mips: 1,
            address_mode: wgpu::AddressMode::ClampToEdge,
            filter_mode: wgpu::FilterMode::Linear,
            usage: wgpu::TextureUsages::empty(),
        };

        let color_texture = render_state.create_texture(
            "Raytrace Color Texture",
            WgpuTextureConfig {
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                ..color_texture_config.clone()
            },
        );

        let color_texture_copy = render_state.create_texture(
            "Raytrace Color Texture Copy",
            WgpuTextureConfig {
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
                ..color_texture_config
            },
        );

        let screen_binding = render_state.create_binding(&[WgpuBindingEntry {
            visibility: wgpu::ShaderStages::COMPUTE,
            binding_data: WgpuBindingData::Buffer {
                buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                buffer: &screen_buffer.buffer,
            },
            count: None,
        }]);

        let object_binding = render_state.create_binding(&[
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::Buffer {
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                    buffer: &sphere_list_buffer.buffer,
                },
                count: None,
            },
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::Buffer {
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                    buffer: &plane_list_buffer.buffer,
                },
                count: None,
            },
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::Buffer {
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                    buffer: &aabb_list_buffer.buffer,
                },
                count: None,
            },
        ]);

        let texture_binding = render_state.create_binding(&[
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::TextureStorage {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    texture_view: &color_texture.view(0..1, 0..1),
                    texture: &color_texture,
                },
                count: None,
            },
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::TextureStorage {
                    access: wgpu::StorageTextureAccess::ReadOnly,
                    texture_view: &color_texture_copy.view(0..1, 0..1),
                    texture: &color_texture_copy,
                },
                count: None,
            },
        ]);

        let shader = render_state.create_shader("assets/shaders/raytrace.wgsl");

        let pipeline_layout = render_state.create_pipeline_layout(WgpuPipelineLayoutConfig {
            bind_group_layouts: &[
                screen_binding.bind_group_layout(),
                object_binding.bind_group_layout(),
                texture_binding.bind_group_layout(),
            ],
            push_constant_config: WgpuPushConstantConfig::default(),
        });

        let pipeline = render_state.create_compute_pipeline(
            "Raytrace Compute Pipeline",
            WgpuComputePipelineConfig {
                layout: &pipeline_layout,
                shader: &shader,
            },
        );

        Self {
            color_texture,
            color_texture_copy,
            shader,
            pipeline_layout,
            pipeline,
            screen_binding,
            object_binding,
            texture_binding,
            gpu_state: render_state.get_gpu_state(),
        }
    }

    fn recreate_pipeline(&mut self) {
        self.pipeline = self.gpu_state.create_compute_pipeline(
            "Raytrace Compute Pipeline",
            WgpuComputePipelineConfig {
                layout: &self.pipeline_layout,
                shader: &self.shader,
            },
        );
    }

    fn recreate_textures(&mut self, new_size: PhysicalSize<u32>) {
        self.color_texture.resize(new_size.width, new_size.height);
        self.color_texture_copy
            .resize(new_size.width, new_size.height);

        // texture binding needs to be recreated because we just recreated the textures
        // but the pipeline layout doesn't need to be recreated, since the layout remains the same, just the data is different
        self.texture_binding = self.gpu_state.create_binding(&[
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::TextureStorage {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    texture_view: &self.color_texture.view(0..1, 0..1),
                    texture: &self.color_texture,
                },
                count: None,
            },
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::TextureStorage {
                    access: wgpu::StorageTextureAccess::ReadOnly,
                    texture_view: &self.color_texture_copy.view(0..1, 0..1),
                    texture: &self.color_texture_copy,
                },
                count: None,
            },
        ]);
    }

    fn recreate_object_binding(
        &mut self,
        sphere_list_buffer: &SphereListBuffer,
        plane_list_buffer: &PlaneListBuffer,
        aabb_list_buffer: &AabbListBuffer,
    ) {
        self.object_binding = self.gpu_state.create_binding(&[
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::Buffer {
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                    buffer: &sphere_list_buffer.buffer,
                },
                count: None,
            },
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::Buffer {
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                    buffer: &plane_list_buffer.buffer,
                },
                count: None,
            },
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::Buffer {
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                    buffer: &aabb_list_buffer.buffer,
                },
                count: None,
            },
        ]);
    }

    pub fn recompile_shaders(&mut self) {
        self.shader.recreate();
        self.recreate_pipeline();
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.recreate_textures(new_size);
    }

    pub fn on_object_update(
        &mut self,
        sphere_list_buffer: &SphereListBuffer,
        plane_list_buffer: &PlaneListBuffer,
        aabb_list_buffer: &AabbListBuffer,
    ) {
        self.recreate_object_binding(sphere_list_buffer, plane_list_buffer, aabb_list_buffer);
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_texture_to_texture(
            self.color_texture.texture().as_image_copy(),
            self.color_texture_copy.texture().as_image_copy(),
            self.color_texture.texture().size(),
        );

        let workgroup_sizes = UVec3::new(8, 8, 1);
        let dimensions = UVec3::new(
            self.color_texture.texture().width(),
            self.color_texture.texture().height(),
            1,
        );

        let mut workgroups = dimensions / workgroup_sizes;

        // Add an extra workgroup in each dimension if the number we calculated doesn't cover the whole dimensions
        workgroups += (dimensions % workgroups).clamp(UVec3::ZERO, UVec3::ONE);

        let compute_pass = WgpuComputePass {
            name: "Raytrace Pass",
            workgroups,
            pipeline: &self.pipeline,
            bindings: &[
                &self.screen_binding,
                &self.object_binding,
                &self.texture_binding,
            ],
            push_constants: None,
        };

        compute_pass.draw(encoder);
    }
}
