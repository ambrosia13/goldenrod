use texture::{WgpuTexture, WgpuTextureConfig};

use super::render_state::RenderState;

pub mod texture;

pub trait RenderStateExt {
    fn create_texture<'a>(&self, name: &'a str, config: WgpuTextureConfig) -> WgpuTexture<'a>;
}

impl RenderStateExt for RenderState {
    fn create_texture<'a>(&self, name: &'a str, config: WgpuTextureConfig) -> WgpuTexture<'a> {
        WgpuTexture::new(self, name, config)
    }
}
