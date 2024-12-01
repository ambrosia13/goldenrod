use std::{fmt::Debug, ops::Range, path::Path};

use crate::{
    engine::render_state::{GpuState, RenderState},
    util,
};

use super::RenderStateExt;

#[derive(Debug, Clone, Copy)]
pub enum TextureType {
    Texture1d,

    Texture2d,
    Texture2dArray,
    TextureCube,
    TextureCubeArray,

    Texture3d,
}

impl TextureType {
    pub fn dimension(self) -> wgpu::TextureDimension {
        match self {
            TextureType::Texture1d => wgpu::TextureDimension::D1,
            TextureType::Texture2d => wgpu::TextureDimension::D2,
            TextureType::Texture2dArray => wgpu::TextureDimension::D2,
            TextureType::TextureCube => wgpu::TextureDimension::D2,
            TextureType::TextureCubeArray => wgpu::TextureDimension::D2,
            TextureType::Texture3d => wgpu::TextureDimension::D3,
        }
    }

    pub fn view_dimension(self) -> wgpu::TextureViewDimension {
        match self {
            TextureType::Texture1d => wgpu::TextureViewDimension::D1,
            TextureType::Texture2d => wgpu::TextureViewDimension::D2,
            TextureType::Texture2dArray => wgpu::TextureViewDimension::D2Array,
            TextureType::TextureCube => wgpu::TextureViewDimension::Cube,
            TextureType::TextureCubeArray => wgpu::TextureViewDimension::CubeArray,
            TextureType::Texture3d => wgpu::TextureViewDimension::D3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextureConfig {
    pub ty: TextureType,
    pub format: wgpu::TextureFormat,

    pub width: u32,
    pub height: u32,
    pub depth: u32,

    pub mips: u32,

    pub address_mode: wgpu::AddressMode,
    pub filter_mode: wgpu::FilterMode,

    pub usage: wgpu::TextureUsages,
}

impl TextureConfig {
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

pub struct Texture<'a> {
    pub(in crate::engine::render_state_ext) name: &'a str,

    pub(in crate::engine::render_state_ext) ty: TextureType,

    pub texture_descriptor: wgpu::TextureDescriptor<'a>,
    pub sampler_descriptor: wgpu::SamplerDescriptor<'a>,

    pub(in crate::engine::render_state_ext) texture: wgpu::Texture,
    pub(in crate::engine::render_state_ext) sampler: wgpu::Sampler,

    pub(in crate::engine::render_state_ext) gpu_state: GpuState,
}

impl<'a> Texture<'a> {
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

    pub fn recreate(&mut self) {
        self.texture = self
            .gpu_state
            .device
            .create_texture(&self.texture_descriptor);
        self.sampler = self
            .gpu_state
            .device
            .create_sampler(&self.sampler_descriptor);
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

pub fn create_cubemap_texture<'a, P: AsRef<Path> + Debug>(
    gpu_state: &GpuState,
    name: &'a str,
    path: P,
    size: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> Result<Texture<'a>, std::io::Error> {
    let parent_path = std::env::current_dir().unwrap();
    let path = parent_path.join(&path);

    let faces = ["px", "nx", "py", "ny", "pz", "nz"];
    let paths = faces.map(|f| path.join(f));

    let images: Result<Vec<Vec<u8>>, _> = paths.into_iter().map(std::fs::read).collect();
    let images = images?;

    let bytes_per_pixel = format.target_pixel_byte_cost().unwrap();

    let texture = gpu_state.create_texture(
        name,
        TextureConfig {
            ty: TextureType::TextureCube,
            format,
            width: size,
            height: size,
            depth: 6,
            mips: 1,
            address_mode: wgpu::AddressMode::ClampToEdge,
            filter_mode: wgpu::FilterMode::Linear,
            usage: usage | wgpu::TextureUsages::COPY_DST,
        },
    );

    for (index, image) in images.iter().enumerate() {
        gpu_state.queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: texture.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: index as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size * bytes_per_pixel),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
        );
    }

    Ok(texture)
}
