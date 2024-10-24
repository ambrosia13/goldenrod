use glam::{UVec2, UVec3};

use crate::engine::render_state_ext::{
    binding::WgpuBinding, pass::WgpuComputePass, shader::WgpuShader, texture::WgpuTexture,
};

pub mod buffers;

pub struct RaytraceRenderContext<'a> {
    pub color_texture: WgpuTexture<'a>,
    pub color_texture_copy: WgpuTexture<'a>,

    pub shader: WgpuShader,
    pub pipeline: wgpu::ComputePipeline,

    pub screen_binding: WgpuBinding,
    pub object_binding: WgpuBinding,
    pub texture_binding: WgpuBinding,
}

impl<'a> RaytraceRenderContext<'a> {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

    pub fn new() -> Self {
        todo!()
    }

    pub fn update(&mut self) {
        todo!()
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder) {
        let workgroup_sizes = UVec3::new(8, 8, 1);
        let workgroups = UVec2::new(
            self.color_texture.texture().width() + 1,
            self.color_texture.texture().height() + 1,
        )
        .extend(1)
            / workgroup_sizes;

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
            shader: &self.shader,
        };

        compute_pass.draw(encoder);
    }
}
