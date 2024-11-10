use std::{fmt::Debug, path::Path};

use binding::{WgpuBinding, WgpuBindingEntry};
use buffer::{BufferData, WgpuBuffer, WgpuBufferConfig, WgpuBufferType};
use pipeline::{WgpuComputePipelineConfig, WgpuPipelineLayoutConfig, WgpuRenderPipelineConfig};
use shader::{WgpuShader, WgslShaderSource};
use texture::{WgpuTexture, WgpuTextureConfig};
use wgpu::util::DeviceExt;

use super::render_state::{GpuState, RenderState};

pub mod binding;
pub mod buffer;
pub mod pass;
pub mod pipeline;
pub mod shader;
pub mod texture;
pub mod timestamp;

pub trait RenderStateExt {
    fn create_buffer<'a>(&self, name: &'a str, config: WgpuBufferConfig<'a>) -> WgpuBuffer;

    fn create_binding(&self, entries: &[WgpuBindingEntry]) -> WgpuBinding;

    fn create_texture<'a>(&self, name: &'a str, config: WgpuTextureConfig) -> WgpuTexture<'a>;

    fn create_shader<P: AsRef<Path> + Debug>(&self, relative_path: P) -> WgpuShader;

    fn create_pipeline_layout(&self, config: WgpuPipelineLayoutConfig) -> wgpu::PipelineLayout;

    fn create_compute_pipeline(
        &self,
        name: &str,
        config: WgpuComputePipelineConfig,
    ) -> wgpu::ComputePipeline;

    fn create_render_pipeline(
        &self,
        name: &str,
        config: WgpuRenderPipelineConfig,
    ) -> wgpu::RenderPipeline;
}

impl RenderStateExt for GpuState {
    fn create_buffer<'a>(&self, name: &'a str, config: WgpuBufferConfig<'a>) -> WgpuBuffer {
        let (buffer, len) = match config.data {
            BufferData::Init(data) => (
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(name),
                        contents: data,
                        usage: config.usage
                            | match config.ty {
                                WgpuBufferType::Storage => wgpu::BufferUsages::STORAGE,
                                WgpuBufferType::Uniform => wgpu::BufferUsages::UNIFORM,
                                WgpuBufferType::Vertex => wgpu::BufferUsages::VERTEX,
                                WgpuBufferType::Index => wgpu::BufferUsages::INDEX,
                            },
                    }),
                data.len(),
            ),
            BufferData::Uninit(len) => (
                self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(name),
                    size: len as u64,
                    usage: config.usage
                        | match config.ty {
                            WgpuBufferType::Storage => wgpu::BufferUsages::STORAGE,
                            WgpuBufferType::Uniform => wgpu::BufferUsages::UNIFORM,
                            WgpuBufferType::Vertex => wgpu::BufferUsages::VERTEX,
                            WgpuBufferType::Index => wgpu::BufferUsages::INDEX,
                        },
                    mapped_at_creation: false,
                }),
                len,
            ),
        };

        WgpuBuffer {
            buffer,
            ty: config.ty,
            len,
            gpu_state: self.clone(),
        }
    }

    fn create_binding(&self, entries: &[WgpuBindingEntry]) -> WgpuBinding {
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

        WgpuBinding {
            bind_group,
            bind_group_layout,
        }
    }

    fn create_texture<'a>(&self, name: &'a str, config: WgpuTextureConfig) -> WgpuTexture<'a> {
        let texture_descriptor = config.texture_descriptor(name);
        let sampler_descriptor = config.sampler_descriptor(name);

        let texture = self.device.create_texture(&texture_descriptor);
        let sampler = self.device.create_sampler(&sampler_descriptor);

        WgpuTexture {
            name,
            ty: config.ty,
            texture_descriptor,
            sampler_descriptor,
            texture,
            sampler,
            gpu_state: self.clone(),
        }
    }

    fn create_shader<P: AsRef<Path> + Debug>(&self, relative_path: P) -> WgpuShader {
        let mut source = WgslShaderSource::load(&relative_path);

        // so we can catch shader compilation errors instead of panicking
        self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let mut module = self.device.create_shader_module(source.desc());
        let err = pollster::block_on(self.device.pop_error_scope());

        if err.is_some() {
            source = WgslShaderSource::fallback(&relative_path);
            module = self.device.create_shader_module(source.desc());
        }

        WgpuShader {
            source,
            module,
            gpu_state: self.clone(),
        }
    }

    fn create_pipeline_layout(&self, config: WgpuPipelineLayoutConfig) -> wgpu::PipelineLayout {
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
        config: WgpuComputePipelineConfig,
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
        config: WgpuRenderPipelineConfig,
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

impl RenderStateExt for RenderState {
    fn create_buffer<'a>(&self, name: &'a str, config: WgpuBufferConfig<'a>) -> WgpuBuffer {
        self.get_gpu_state().create_buffer(name, config)
    }

    fn create_binding(&self, entries: &[WgpuBindingEntry]) -> WgpuBinding {
        self.get_gpu_state().create_binding(entries)
    }

    fn create_texture<'a>(&self, name: &'a str, config: WgpuTextureConfig) -> WgpuTexture<'a> {
        self.get_gpu_state().create_texture(name, config)
    }

    fn create_shader<P: AsRef<Path> + Debug>(&self, relative_path: P) -> WgpuShader {
        self.get_gpu_state().create_shader(relative_path)
    }

    fn create_pipeline_layout(&self, config: WgpuPipelineLayoutConfig) -> wgpu::PipelineLayout {
        self.get_gpu_state().create_pipeline_layout(config)
    }

    fn create_compute_pipeline(
        &self,
        name: &str,
        config: WgpuComputePipelineConfig,
    ) -> wgpu::ComputePipeline {
        self.get_gpu_state().create_compute_pipeline(name, config)
    }

    fn create_render_pipeline(
        &self,
        name: &str,
        config: WgpuRenderPipelineConfig,
    ) -> wgpu::RenderPipeline {
        self.get_gpu_state().create_render_pipeline(name, config)
    }
}
