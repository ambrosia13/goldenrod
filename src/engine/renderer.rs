use crate::renderer::{buffers::{AabbListBuffer, PlaneListBuffer, ScreenBuffer, SphereListBuffer}, final_pass::FinalRenderContext, raytrace::RaytraceRenderContext, screen_quad::ScreenQuad};

use super::{engine_state::EngineState, render_state::RenderState};


pub struct Renderer<'a> {
    pub raytrace_render_context: RaytraceRenderContext<'a>,
    pub final_render_context: FinalRenderContext,

    pub screen_quad: ScreenQuad,

    pub screen_buffer: ScreenBuffer,

    pub object_buffer_version: u32,
    pub sphere_list_buffer: SphereListBuffer,
    pub plane_list_buffer: PlaneListBuffer,
    pub aabb_list_buffer: AabbListBuffer,
}

impl<'a> Renderer<'a> {
    pub fn init(render_state: &RenderState) -> Self {
        let screen_buffer = ScreenBuffer::new(render_state);

        let object_buffer_version = 0;
        let sphere_list_buffer = SphereListBuffer::new("Sphere List Buffer", render_state);
        let plane_list_buffer = PlaneListBuffer::new("Plane List Buffer", render_state);
        let aabb_list_buffer = AabbListBuffer::new("AABB List Buffer", render_state);

        let screen_quad = ScreenQuad::new(render_state);

        let raytrace_render_context = RaytraceRenderContext::new(
            render_state,
            &screen_buffer,
            &sphere_list_buffer,
            &plane_list_buffer,
            &aabb_list_buffer,
        );

        let final_render_context = FinalRenderContext::new(
            render_state,
            &raytrace_render_context.color_texture,
            &screen_buffer,
            &screen_quad,
        );

        Self {
            raytrace_render_context,
            final_render_context,
            screen_quad,
            screen_buffer,
            object_buffer_version,
            sphere_list_buffer,
            plane_list_buffer,
            aabb_list_buffer,
        }
    }

    pub fn update_object_buffers(&mut self, engine_state: &EngineState) {
        // If the object buffers don't reflect the current object list, update those
        if self.object_buffer_version != engine_state.object_list.version() {
            log::info!("Updating object buffers");

            #[rustfmt::skip]
            let update_object_bindings = 
                self.sphere_list_buffer.update(&engine_state.object_list) | 
                self.plane_list_buffer.update(&engine_state.object_list) | 
                self.aabb_list_buffer.update(&engine_state.object_list);

            // if updating the object buffers caused a reallocation, update the bindings so the raytracer
            // has access to the new buffers
            if update_object_bindings {
                self.raytrace_render_context.on_object_update(
                    &self.sphere_list_buffer,
                    &self.plane_list_buffer,
                    &self.aabb_list_buffer,
                );
            }

            // update the version to match
            self.object_buffer_version = engine_state.object_list.version();
        }
    }

    pub fn update(&mut self, render_state: &RenderState, engine_state: &EngineState, encoder: &mut wgpu::CommandEncoder, surface_texture: &wgpu::SurfaceTexture) {
        self.update_object_buffers(engine_state);

        self
            .screen_buffer
            .update(render_state, &engine_state.camera);

            self.raytrace_render_context.draw(encoder);
            self.final_render_context.draw(
            encoder,
            surface_texture,
            &self.screen_quad,
        );

    }
}