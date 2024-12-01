use std::num::NonZero;

use crate::engine::render_state;

use super::{buffer::Buffer, texture::Texture};

pub enum BindingData<'resource, 'r>
where
    'resource: 'r,
{
    TextureView {
        texture: &'r Texture<'resource>,
        texture_view: &'r wgpu::TextureView,
    },
    TextureSampler {
        sampler_type: wgpu::SamplerBindingType,
        texture: &'r Texture<'resource>,
    },
    TextureStorage {
        access: wgpu::StorageTextureAccess,
        texture_view: &'r wgpu::TextureView,
        texture: &'r Texture<'resource>,
    },
    Buffer {
        buffer_type: wgpu::BufferBindingType,
        buffer: &'r Buffer,
    },
}

impl<'resource, 'r> BindingData<'resource, 'r>
where
    'resource: 'r,
{
    pub fn binding_type(&self) -> wgpu::BindingType {
        match *self {
            BindingData::TextureView {
                texture,
                texture_view: _,
            } => wgpu::BindingType::Texture {
                sample_type: texture
                    .texture_descriptor
                    .format
                    .sample_type(None, Some(render_state::WGPU_FEATURES))
                    .unwrap(),
                view_dimension: texture.view_dimension(),
                multisampled: false,
            },
            BindingData::TextureSampler {
                sampler_type,
                texture: _,
            } => wgpu::BindingType::Sampler(sampler_type),
            BindingData::TextureStorage {
                access,
                texture_view: _,
                texture,
            } => wgpu::BindingType::StorageTexture {
                access,
                format: texture.texture_descriptor.format,
                view_dimension: texture.view_dimension(),
            },
            BindingData::Buffer {
                buffer_type,
                buffer: _,
            } => wgpu::BindingType::Buffer {
                ty: buffer_type,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource<'r> {
        match self {
            BindingData::TextureView {
                texture: _,
                texture_view,
            } => wgpu::BindingResource::TextureView(texture_view),
            BindingData::TextureSampler {
                sampler_type: _,
                texture,
            } => wgpu::BindingResource::Sampler(&texture.sampler),
            BindingData::TextureStorage {
                access: _,
                texture_view,
                texture: _,
            } => wgpu::BindingResource::TextureView(texture_view),
            BindingData::Buffer {
                buffer_type: _,
                buffer,
            } => buffer.as_entire_binding(),
        }
    }
}

pub struct BindingEntry<'resource, 'r> {
    pub visibility: wgpu::ShaderStages,
    pub binding_data: BindingData<'resource, 'r>,
    pub count: Option<NonZero<u32>>,
}

pub struct Binding {
    pub(in crate::engine) bind_group: wgpu::BindGroup,
    pub(in crate::engine) bind_group_layout: wgpu::BindGroupLayout,
}

impl Binding {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}
