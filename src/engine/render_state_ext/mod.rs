use std::{fmt::Debug, path::Path};

use binding::{Binding, BindingEntry};
use buffer::{Buffer, BufferConfig, BufferData, BufferType};
use pipeline::{ComputePipelineConfig, PipelineLayoutConfig, RenderPipelineConfig};
use shader::{Shader, ShaderSource};
use texture::{Texture, TextureConfig};
use wgpu::util::DeviceExt;

use super::render_state::{GpuState, RenderState};

pub mod binding;
pub mod buffer;
pub mod pass;
pub mod pipeline;
pub mod shader;
pub mod texture;

pub trait RenderStateExt {
    fn as_gpu_state(&self) -> GpuState;

    fn device(&self) -> &wgpu::Device;

    fn queue(&self) -> &wgpu::Queue;

    fn create_binding(&self, entries: &[BindingEntry]) -> Binding;

    fn create_texture<'a>(&self, name: &'a str, config: TextureConfig) -> Texture<'a>;

    fn create_shader<P: AsRef<Path> + Debug>(&self, relative_path: P) -> Shader;

    fn create_pipeline_layout(&self, config: PipelineLayoutConfig) -> wgpu::PipelineLayout;

    fn create_compute_pipeline(
        &self,
        name: &str,
        config: ComputePipelineConfig,
    ) -> wgpu::ComputePipeline;

    fn create_render_pipeline(
        &self,
        name: &str,
        config: RenderPipelineConfig,
    ) -> wgpu::RenderPipeline;
}

impl RenderStateExt for GpuState {
    fn as_gpu_state(&self) -> GpuState {
        self.clone()
    }

    fn device(&self) -> &wgpu::Device {
        &self.device
    }

    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    fn create_binding(&self, entries: &[BindingEntry]) -> Binding {
        let entries: Vec<_> = entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                (
                    wgpu::BindGroupEntry {
                        binding: index as u32,
                        resource: entry.binding_data.binding_resource(),
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: index as u32,
                        visibility: entry.visibility,
                        ty: entry.binding_data.binding_type(),
                        count: entry.count,
                    },
                )
            })
            .collect();

        let bind_group_entries: Vec<_> = entries.iter().map(|(bge, _)| bge.clone()).collect();
        let bind_group_layout_entries: Vec<_> = entries.iter().map(|&(_, bgle)| bgle).collect();

        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &bind_group_layout_entries,
                });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &bind_group_entries,
        });

        Binding {
            bind_group,
            bind_group_layout,
        }
    }

    fn create_texture<'a>(&self, name: &'a str, config: TextureConfig) -> Texture<'a> {
        let texture_descriptor = config.texture_descriptor(name);
        let sampler_descriptor = config.sampler_descriptor(name);

        let texture = self.device.create_texture(&texture_descriptor);
        let sampler = self.device.create_sampler(&sampler_descriptor);

        Texture {
            name,
            ty: config.ty,
            texture_descriptor,
            sampler_descriptor,
            texture,
            sampler,
            gpu_state: self.clone(),
        }
    }

    fn create_shader<P: AsRef<Path> + Debug>(&self, relative_path: P) -> Shader {
        let mut source = ShaderSource::load(&relative_path);

        // so we can catch shader compilation errors instead of panicking
        self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let mut module = self.device.create_shader_module(source.desc());
        let err = pollster::block_on(self.device.pop_error_scope());

        if err.is_some() {
            source = ShaderSource::fallback(&relative_path);
            module = self.device.create_shader_module(source.desc());
        }

        Shader {
            source,
            module,
            gpu_state: self.clone(),
        }
    }

    fn create_pipeline_layout(&self, config: PipelineLayoutConfig) -> wgpu::PipelineLayout {
        self.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: config.bind_group_layouts,
                push_constant_ranges: &config.push_constant_config.as_ranges(),
            })
    }

    fn create_compute_pipeline(
        &self,
        name: &str,
        config: ComputePipelineConfig,
    ) -> wgpu::ComputePipeline {
        self.device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: Some(config.layout),
                module: config.shader.module(),
                entry_point: "compute",
                compilation_options: Default::default(),
                cache: None,
            })
    }

    fn create_render_pipeline(
        &self,
        name: &str,
        config: RenderPipelineConfig,
    ) -> wgpu::RenderPipeline {
        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(name),
                layout: Some(config.layout),
                vertex: wgpu::VertexState {
                    module: config.vertex.module(),
                    entry_point: "vertex",
                    compilation_options: Default::default(),
                    buffers: config.vertex_buffer_layouts,
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: config.fragment.module(),
                    entry_point: "fragment",
                    compilation_options: Default::default(),
                    targets: config.targets,
                }),
                multiview: None,
                cache: None,
            })
    }
}

impl RenderStateExt for &RenderState {
    fn as_gpu_state(&self) -> GpuState {
        self.get_gpu_state()
    }

    fn device(&self) -> &wgpu::Device {
        &self.device
    }

    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    fn create_binding(&self, entries: &[BindingEntry]) -> Binding {
        self.get_gpu_state().create_binding(entries)
    }

    fn create_texture<'a>(&self, name: &'a str, config: TextureConfig) -> Texture<'a> {
        self.get_gpu_state().create_texture(name, config)
    }

    fn create_shader<P: AsRef<Path> + Debug>(&self, relative_path: P) -> Shader {
        self.get_gpu_state().create_shader(relative_path)
    }

    fn create_pipeline_layout(&self, config: PipelineLayoutConfig) -> wgpu::PipelineLayout {
        self.get_gpu_state().create_pipeline_layout(config)
    }

    fn create_compute_pipeline(
        &self,
        name: &str,
        config: ComputePipelineConfig,
    ) -> wgpu::ComputePipeline {
        self.get_gpu_state().create_compute_pipeline(name, config)
    }

    fn create_render_pipeline(
        &self,
        name: &str,
        config: RenderPipelineConfig,
    ) -> wgpu::RenderPipeline {
        self.get_gpu_state().create_render_pipeline(name, config)
    }
}
