use glam::UVec3;
use winit::dpi::PhysicalSize;

use crate::engine::{
    render_state::{GpuState, RenderState},
    render_state_ext::{
        binding::{Binding, BindingData, BindingEntry},
        pass::ComputePass,
        pipeline::{ComputePipelineConfig, PipelineLayoutConfig, PushConstantConfig},
        shader::{Shader, ShaderSource},
        texture::{self, Texture, TextureConfig, TextureType},
        RenderStateExt,
    },
};

use super::buffer::{
    bvh::BvhBuffer,
    object::{AabbListBuffer, PlaneListBuffer, SphereListBuffer, TriangleListBuffer},
    screen::ScreenBuffer,
};

pub struct RaytraceRenderContext<'a> {
    pub color_texture: Texture<'a>,
    pub color_texture_copy: Texture<'a>,

    pub shader: Shader,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::ComputePipeline,

    pub screen_binding: Binding,
    pub object_binding: Binding,
    pub lut_binding: Binding,
    pub texture_binding: Binding,

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
        triangle_list_buffer: &TriangleListBuffer,
        bvh_buffer: &BvhBuffer,
    ) -> Self {
        let gpu_state = render_state.get_gpu_state();

        let color_texture_config = TextureConfig {
            ty: TextureType::Texture2d,
            format: Self::TEXTURE_FORMAT,
            width: render_state.size.width,
            height: render_state.size.height,
            depth: 1,
            mips: 1,
            address_mode: wgpu::AddressMode::ClampToEdge,
            filter_mode: wgpu::FilterMode::Linear,
            usage: wgpu::TextureUsages::empty(),
        };

        let color_texture = Texture::new(
            &gpu_state,
            "Raytrace Color Texture",
            TextureConfig {
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                ..color_texture_config.clone()
            },
        );

        let color_texture_copy = Texture::new(
            &gpu_state,
            "Raytrace Color Texture Copy",
            TextureConfig {
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
                ..color_texture_config
            },
        );

        let (wavelength_to_xyz_lut, rgb_to_spectral_intensity_lut, cubemap) =
            Self::load_luts(&gpu_state);

        let screen_binding = Binding::new(
            &gpu_state,
            &[BindingEntry {
                visibility: wgpu::ShaderStages::COMPUTE,
                binding_data: BindingData::Buffer {
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                    buffer: &screen_buffer.buffer,
                },
                count: None,
            }],
        );

        let object_binding = Self::create_object_binding(
            &gpu_state,
            sphere_list_buffer,
            plane_list_buffer,
            aabb_list_buffer,
            triangle_list_buffer,
            bvh_buffer,
        );

        let lut_binding = Binding::new(
            &gpu_state,
            &[
                BindingEntry {
                    visibility: wgpu::ShaderStages::COMPUTE,
                    binding_data: BindingData::TextureStorage {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        texture_view: &wavelength_to_xyz_lut.view(0..1, 0..1),
                        texture: &wavelength_to_xyz_lut,
                    },
                    count: None,
                },
                BindingEntry {
                    visibility: wgpu::ShaderStages::COMPUTE,
                    binding_data: BindingData::TextureStorage {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        texture_view: &rgb_to_spectral_intensity_lut.view(0..1, 0..1),
                        texture: &rgb_to_spectral_intensity_lut,
                    },
                    count: None,
                },
                BindingEntry {
                    visibility: wgpu::ShaderStages::COMPUTE,
                    binding_data: BindingData::TextureView {
                        texture: &cubemap,
                        texture_view: &cubemap.view(0..1, 0..6),
                    },
                    count: None,
                },
                BindingEntry {
                    visibility: wgpu::ShaderStages::COMPUTE,
                    binding_data: BindingData::TextureSampler {
                        sampler_type: wgpu::SamplerBindingType::Filtering,
                        texture: &cubemap,
                    },
                    count: None,
                },
            ],
        );

        let texture_binding =
            Self::create_texture_binding(&gpu_state, &color_texture, &color_texture_copy);

        let shader = Shader::new(
            &render_state,
            ShaderSource::load_wgsl("assets/shaders/raytrace.wgsl"),
        );

        let pipeline_layout = render_state.create_pipeline_layout(PipelineLayoutConfig {
            bind_group_layouts: &[
                screen_binding.bind_group_layout(),
                object_binding.bind_group_layout(),
                lut_binding.bind_group_layout(),
                texture_binding.bind_group_layout(),
            ],
            push_constant_config: PushConstantConfig::default(),
        });

        let pipeline = render_state.create_compute_pipeline(
            "Raytrace Compute Pipeline",
            ComputePipelineConfig {
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

    pub fn load_luts(gpu_state: &GpuState) -> (Texture, Texture, Texture) {
        let wavelength_to_xyz_path = std::env::current_dir()
            .unwrap()
            .join("assets/textures/lut/wavelength_to_xyz");

        let wavelength_to_xyz_bytes = std::fs::read(&wavelength_to_xyz_path).unwrap_or_else(|_| {
            panic!(
                "Couldn't read texture file; expected at {:?}",
                wavelength_to_xyz_path
            );
        });

        let rgb_to_spectral_intensity_path = std::env::current_dir()
            .unwrap()
            .join("assets/textures/lut/rgb_to_spectral_intensity");

        let rgb_to_spectral_intensity_bytes = std::fs::read(&rgb_to_spectral_intensity_path)
            .unwrap_or_else(|_| {
                panic!(
                    "Couldn't read texture file; expected at {:?}",
                    rgb_to_spectral_intensity_path
                );
            });

        // divide the number of bytes by the bytes per pixel to get number of pixels
        let lut_size =
            wavelength_to_xyz_bytes.len() as u32 / (std::mem::size_of::<f32>() as u32 * 4);

        let wavelength_to_xyz_lut = Texture::new(
            gpu_state,
            "Wavelength to XYZ LUT",
            TextureConfig {
                ty: TextureType::Texture1d,
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
            wavelength_to_xyz_lut.as_image_copy(),
            &wavelength_to_xyz_bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(wavelength_to_xyz_bytes.len() as u32),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: lut_size,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        // divide the number of bytes by the bytes per pixel to get number of pixels
        let lut_size =
            rgb_to_spectral_intensity_bytes.len() as u32 / (std::mem::size_of::<f32>() as u32 * 4);

        let rgb_to_spectral_intensity_lut = Texture::new(
            gpu_state,
            "RGB to Spectral Intensity",
            TextureConfig {
                ty: TextureType::Texture1d,
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
            rgb_to_spectral_intensity_lut.as_image_copy(),
            &rgb_to_spectral_intensity_bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(rgb_to_spectral_intensity_bytes.len() as u32),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: lut_size,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let cubemap = texture::create_cubemap_texture(
            gpu_state,
            "Sky Cubemap",
            "assets/textures/cubemap/meadow",
            4096,
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureUsages::TEXTURE_BINDING,
        )
        .unwrap();

        (
            wavelength_to_xyz_lut,
            rgb_to_spectral_intensity_lut,
            cubemap,
        )
    }

    fn create_texture_binding(
        gpu_state: &GpuState,
        texture: &Texture,
        texture_copy: &Texture,
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
                    binding_data: BindingData::TextureStorage {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        texture_view: &texture_copy.view(0..1, 0..1),
                        texture: texture_copy,
                    },
                    count: None,
                },
            ],
        )
    }

    fn create_object_binding(
        gpu_state: &GpuState,
        sphere_list_buffer: &SphereListBuffer,
        plane_list_buffer: &PlaneListBuffer,
        aabb_list_buffer: &AabbListBuffer,
        triangle_list_buffer: &TriangleListBuffer,
        bvh_buffer: &BvhBuffer,
    ) -> Binding {
        Binding::new(
            gpu_state,
            &[
                BindingEntry {
                    visibility: wgpu::ShaderStages::COMPUTE,
                    binding_data: BindingData::Buffer {
                        buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                        buffer: &sphere_list_buffer.buffer,
                    },
                    count: None,
                },
                BindingEntry {
                    visibility: wgpu::ShaderStages::COMPUTE,
                    binding_data: BindingData::Buffer {
                        buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                        buffer: &plane_list_buffer.buffer,
                    },
                    count: None,
                },
                BindingEntry {
                    visibility: wgpu::ShaderStages::COMPUTE,
                    binding_data: BindingData::Buffer {
                        buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                        buffer: &aabb_list_buffer.buffer,
                    },
                    count: None,
                },
                BindingEntry {
                    visibility: wgpu::ShaderStages::COMPUTE,
                    binding_data: BindingData::Buffer {
                        buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                        buffer: &triangle_list_buffer.buffer,
                    },
                    count: None,
                },
                BindingEntry {
                    visibility: wgpu::ShaderStages::COMPUTE,
                    binding_data: BindingData::Buffer {
                        buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
                        buffer: &bvh_buffer.buffer,
                    },
                    count: None,
                },
            ],
        )
    }

    fn recreate_pipeline(&mut self) {
        self.pipeline = self.gpu_state.create_compute_pipeline(
            "Raytrace Compute Pipeline",
            ComputePipelineConfig {
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
        triangle_list_buffer: &TriangleListBuffer,
        bvh_buffer: &BvhBuffer,
    ) {
        self.object_binding = Self::create_object_binding(
            &self.gpu_state,
            sphere_list_buffer,
            plane_list_buffer,
            aabb_list_buffer,
            triangle_list_buffer,
            bvh_buffer,
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
        triangle_list_buffer: &TriangleListBuffer,
        bvh_buffer: &BvhBuffer,
    ) {
        self.recreate_object_binding(
            sphere_list_buffer,
            plane_list_buffer,
            aabb_list_buffer,
            triangle_list_buffer,
            bvh_buffer,
        );
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_texture_to_texture(
            self.color_texture.as_image_copy(),
            self.color_texture_copy.as_image_copy(),
            self.color_texture.size(),
        );

        let workgroup_sizes = UVec3::new(8, 8, 1);
        let dimensions = UVec3::new(self.color_texture.width(), self.color_texture.height(), 1);

        let mut workgroups = dimensions / workgroup_sizes;

        // Add an extra workgroup in each dimension if the number we calculated doesn't cover the whole dimensions
        workgroups += (dimensions % workgroups).clamp(UVec3::ZERO, UVec3::ONE);

        let compute_pass = ComputePass {
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
