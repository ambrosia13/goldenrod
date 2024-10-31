use std::ops::Range;

use crate::engine::render_state::{GpuContext, RenderState};

#[derive(Debug, Clone, Copy)]
pub enum WgpuTextureType {
    Texture1d,

    Texture2d,
    Texture2dArray,
    TextureCube,
    TextureCubeArray,

    Texture3d,
}

impl WgpuTextureType {
    pub fn dimension(self) -> wgpu::TextureDimension {
        match self {
            WgpuTextureType::Texture1d => wgpu::TextureDimension::D1,
            WgpuTextureType::Texture2d => wgpu::TextureDimension::D2,
            WgpuTextureType::Texture2dArray => wgpu::TextureDimension::D2,
            WgpuTextureType::TextureCube => wgpu::TextureDimension::D2,
            WgpuTextureType::TextureCubeArray => wgpu::TextureDimension::D2,
            WgpuTextureType::Texture3d => wgpu::TextureDimension::D3,
        }
    }

    pub fn view_dimension(self) -> wgpu::TextureViewDimension {
        match self {
            WgpuTextureType::Texture1d => wgpu::TextureViewDimension::D1,
            WgpuTextureType::Texture2d => wgpu::TextureViewDimension::D2,
            WgpuTextureType::Texture2dArray => wgpu::TextureViewDimension::D2Array,
            WgpuTextureType::TextureCube => wgpu::TextureViewDimension::Cube,
            WgpuTextureType::TextureCubeArray => wgpu::TextureViewDimension::CubeArray,
            WgpuTextureType::Texture3d => wgpu::TextureViewDimension::D3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WgpuTextureConfig {
    pub ty: WgpuTextureType,
    pub format: wgpu::TextureFormat,

    pub width: u32,
    pub height: u32,
    pub depth: u32,

    pub mips: u32,

    pub address_mode: wgpu::AddressMode,
    pub filter_mode: wgpu::FilterMode,

    pub usage: wgpu::TextureUsages,
}

impl WgpuTextureConfig {
    pub fn texture_descriptor<'a>(&self, name: &'a str) -> wgpu::TextureDescriptor<'a> {
        wgpu::TextureDescriptor {
            label: Some(name),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: self.depth,
            },
            mip_level_count: self.mips,
            sample_count: 1,
            dimension: self.ty.dimension(),
            format: self.format,
            usage: self.usage,
            view_formats: &[],
        }
    }

    pub fn sampler_descriptor<'a>(&self, name: &'a str) -> wgpu::SamplerDescriptor<'a> {
        wgpu::SamplerDescriptor {
            label: Some(name),
            address_mode_v: self.address_mode,
            address_mode_u: self.address_mode,
            address_mode_w: self.address_mode,
            mag_filter: self.filter_mode,
            min_filter: self.filter_mode,
            mipmap_filter: self.filter_mode,
            ..Default::default()
        }
    }
}

pub struct WgpuTexture<'a> {
    pub(in crate::engine::render_state_ext) name: &'a str,

    pub(in crate::engine::render_state_ext) ty: WgpuTextureType,

    pub(in crate::engine::render_state_ext) texture_descriptor: wgpu::TextureDescriptor<'a>,
    pub(in crate::engine::render_state_ext) sampler_descriptor: wgpu::SamplerDescriptor<'a>,

    pub(in crate::engine::render_state_ext) texture: wgpu::Texture,
    pub(in crate::engine::render_state_ext) sampler: wgpu::Sampler,

    pub(in crate::engine::render_state_ext) ctx: GpuContext,
}

impl<'a> WgpuTexture<'a> {
    pub(in crate::engine::render_state_ext) fn new(
        render_state: &RenderState,
        name: &'a str,
        config: WgpuTextureConfig,
    ) -> Self {
        let ctx = render_state.ctx();

        let texture_descriptor = config.texture_descriptor(name);
        let sampler_descriptor = config.sampler_descriptor(name);

        let texture = ctx.device.create_texture(&texture_descriptor);
        let sampler = ctx.device.create_sampler(&sampler_descriptor);

        Self {
            name,
            ty: config.ty,
            texture_descriptor,
            sampler_descriptor,
            texture,
            sampler,
            ctx,
        }
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.texture_descriptor.size.width = new_width;
        self.texture_descriptor.size.height = new_height;

        self.recreate();
    }

    pub fn set_descriptor(
        &'a mut self,
        texture_descriptor: wgpu::TextureDescriptor<'a>,
        sampler_descriptor: wgpu::SamplerDescriptor<'a>,
    ) {
        self.texture_descriptor = wgpu::TextureDescriptor {
            label: Some(self.name),
            ..texture_descriptor
        };

        self.sampler_descriptor = wgpu::SamplerDescriptor {
            label: Some(self.name),
            ..sampler_descriptor
        };

        self.recreate();
    }

    fn recreate(&mut self) {
        self.texture = self.ctx.device.create_texture(&self.texture_descriptor);
        self.sampler = self.ctx.device.create_sampler(&self.sampler_descriptor);
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn view(&self, mip_range: Range<u32>, layer_range: Range<u32>) -> wgpu::TextureView {
        self.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("{} View", self.name)),
            format: Some(self.texture_descriptor.format),
            dimension: Some(self.view_dimension()),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: mip_range.start,
            mip_level_count: Some(mip_range.end - mip_range.start),
            base_array_layer: layer_range.start,
            array_layer_count: Some(layer_range.end - layer_range.start),
        })
    }

    pub fn dimension(&self) -> wgpu::TextureDimension {
        self.ty.dimension()
    }

    pub fn view_dimension(&self) -> wgpu::TextureViewDimension {
        self.ty.view_dimension()
    }
}
