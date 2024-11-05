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
    pub lut_binding: WgpuBinding,
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
        let gpu_state = render_state.get_gpu_state();

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

        let color_texture = gpu_state.create_texture(
            "Raytrace Color Texture",
            WgpuTextureConfig {
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                ..color_texture_config.clone()
            },
        );

        let color_texture_copy = gpu_state.create_texture(
            "Raytrace Color Texture Copy",
            WgpuTextureConfig {
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
                ..color_texture_config
            },
        );

        let bytes = std::fs::read(
            std::env::current_dir()
                .unwrap()
                .join("assets/textures/lut/wavelength_to_xyz"),
        )
        .unwrap();

        // divide the number of bytes by the bytes per pixel to get number of pixels
        let lut_size = bytes.len() as u32 / (std::mem::size_of::<f32>() as u32 * 4);

        let wavelength_to_xyz_lut = gpu_state.create_texture(
            "Wavelength to XYZ LUT",
            WgpuTextureConfig {
                ty: WgpuTextureType::Texture1d,
                format: wgpu::TextureFormat::Rgba32Float,
                width: lut_size,
                height: 1,
                depth: 1,
                mips: 1,
                address_mode: wgpu::AddressMode::ClampToEdge,
                filter_mode: wgpu::FilterMode::Linear,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            },
        );

        gpu_state.queue.write_texture(
            wavelength_to_xyz_lut.texture().as_image_copy(),
            &bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes.len() as u32),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: lut_size,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let bytes = std::fs::read(
            std::env::current_dir()
                .unwrap()
                .join("assets/textures/lut/rgb_to_spectral_intensity"),
        )
        .unwrap();

        // divide the number of bytes by the bytes per pixel to get number of pixels
        let lut_size = bytes.len() as u32 / (std::mem::size_of::<f32>() as u32 * 4);

        let rgb_to_spectral_intensity_lut = gpu_state.create_texture(
            "RGB to Spectral Intensity",
            WgpuTextureConfig {
                ty: WgpuTextureType::Texture1d,
                format: wgpu::TextureFormat::Rgba32Float,
                width: lut_size,
                height: 1,
                depth: 1,
                mips: 1,
                address_mode: wgpu::AddressMode::ClampToEdge,
                filter_mode: wgpu::FilterMode::Linear,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            },
        );

        gpu_state.queue.write_texture(
            rgb_to_spectral_intensity_lut.texture().as_image_copy(),
            &bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes.len() as u32),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: lut_size,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let screen_binding = gpu_state.create_binding(&[WgpuBindingEntry {
            visibility: wgpu::ShaderStages::COMPUTE,
            binding_data: WgpuBindingData::Buffer {
                buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                buffer: &screen_buffer.buffer,
            },
            count: None,
        }]);

        let object_binding = Self::create_object_binding(
            &gpu_state,
            sphere_list_buffer,
            plane_list_buffer,
            aabb_list_buffer,
        );

        let lut_binding = gpu_state.create_binding(&[
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::TextureStorage {
                    access: wgpu::StorageTextureAccess::ReadOnly,
                    texture_view: &wavelength_to_xyz_lut.view(0..1, 0..1),
                    texture: &wavelength_to_xyz_lut,
                },
                count: None,
            },
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::TextureStorage {
                    access: wgpu::StorageTextureAccess::ReadOnly,
                    texture_view: &rgb_to_spectral_intensity_lut.view(0..1, 0..1),
                    texture: &rgb_to_spectral_intensity_lut,
                },
                count: None,
            },
        ]);

        let texture_binding =
            Self::create_texture_binding(&gpu_state, &color_texture, &color_texture_copy);

        let shader = render_state.create_shader("assets/shaders/raytrace.wgsl");

        let pipeline_layout = render_state.create_pipeline_layout(WgpuPipelineLayoutConfig {
            bind_group_layouts: &[
                screen_binding.bind_group_layout(),
                object_binding.bind_group_layout(),
                lut_binding.bind_group_layout(),
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
            lut_binding,
            texture_binding,
            gpu_state: render_state.get_gpu_state(),
        }
    }

    fn create_texture_binding(
        gpu_state: &GpuState,
        texture: &WgpuTexture,
        texture_copy: &WgpuTexture,
    ) -> WgpuBinding {
        gpu_state.create_binding(&[
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::TextureStorage {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    texture_view: &texture.view(0..1, 0..1),
                    texture,
                },
                count: None,
            },
            WgpuBindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: WgpuBindingData::TextureStorage {
                    access: wgpu::StorageTextureAccess::ReadOnly,
                    texture_view: &texture_copy.view(0..1, 0..1),
                    texture: texture_copy,
                },
                count: None,
            },
        ])
    }

    fn create_object_binding(
        gpu_state: &GpuState,
        sphere_list_buffer: &SphereListBuffer,
        plane_list_buffer: &PlaneListBuffer,
        aabb_list_buffer: &AabbListBuffer,
    ) -> WgpuBinding {
        gpu_state.create_binding(&[
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
        ])
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
        self.texture_binding = Self::create_texture_binding(
            &self.gpu_state,
            &self.color_texture,
            &self.color_texture_copy,
        );
    }

    fn recreate_object_binding(
        &mut self,
        sphere_list_buffer: &SphereListBuffer,
        plane_list_buffer: &PlaneListBuffer,
        aabb_list_buffer: &AabbListBuffer,
    ) {
        self.object_binding = Self::create_object_binding(
            &self.gpu_state,
            sphere_list_buffer,
            plane_list_buffer,
            aabb_list_buffer,
        );
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
                &self.lut_binding,
                &self.texture_binding,
            ],
            push_constants: None,
        };

        compute_pass.draw(encoder);
    }
}
