use winit::{dpi::PhysicalSize, keyboard::KeyCode};

use crate::renderer::{
    bloom::BloomRenderContext,
    buffer::{
        bvh::BvhBuffer,
        object::{AabbListBuffer, PlaneListBuffer, SphereListBuffer},
        screen::ScreenBuffer,
    },
    final_pass::FinalRenderContext,
    raytrace::RaytraceRenderContext,
    screen_quad::ScreenQuad,
};

use super::{engine_state::EngineState, render_state::RenderState};

pub const RECOMPILE_SHADERS_KEY: KeyCode = KeyCode::KeyR;

pub struct Renderer<'a> {
    pub raytrace_render_context: RaytraceRenderContext<'a>,
    pub bloom_render_context: BloomRenderContext<'a>,
    pub final_render_context: FinalRenderContext,

    pub screen_quad: ScreenQuad,

    pub screen_buffer: ScreenBuffer,

    pub object_buffer_version: u32,
    pub sphere_list_buffer: SphereListBuffer,
    pub plane_list_buffer: PlaneListBuffer,
    pub aabb_list_buffer: AabbListBuffer,
    pub bvh_buffer: BvhBuffer,
}

impl<'a> Renderer<'a> {
    pub fn init(render_state: &RenderState) -> Self {
        let screen_buffer = ScreenBuffer::new(render_state);

        let object_buffer_version = 0;
        let sphere_list_buffer = SphereListBuffer::new("Sphere List Buffer", render_state);
        let plane_list_buffer = PlaneListBuffer::new("Plane List Buffer", render_state);
        let aabb_list_buffer = AabbListBuffer::new("AABB List Buffer", render_state);

        let bvh_buffer = BvhBuffer::new(render_state);

        let screen_quad = ScreenQuad::new(render_state);

        let raytrace_render_context = RaytraceRenderContext::new(
            render_state,
            &screen_buffer,
            &sphere_list_buffer,
            &plane_list_buffer,
            &aabb_list_buffer,
            &bvh_buffer,
        );

        let bloom_render_context = BloomRenderContext::new(
            render_state,
            &screen_quad,
            &raytrace_render_context.color_texture,
            &screen_buffer,
        );

        let final_render_context = FinalRenderContext::new(
            render_state,
            &bloom_render_context.bloom_texture,
            &screen_buffer,
            &screen_quad,
        );

        Self {
            raytrace_render_context,
            bloom_render_context,
            final_render_context,
            screen_quad,
            screen_buffer,
            object_buffer_version,
            sphere_list_buffer,
            plane_list_buffer,
            aabb_list_buffer,
            bvh_buffer,
        }
    }

    pub fn update_object_buffers(&mut self, engine_state: &EngineState) {
        // If the object buffers don't reflect the current object list, update those
        if self.object_buffer_version != engine_state.object_list.version() {
            log::info!("Updating object buffers");

            let update_object_bindings = self.sphere_list_buffer.update(&engine_state.object_list)
                | self.plane_list_buffer.update(&engine_state.object_list)
                | self.aabb_list_buffer.update(&engine_state.object_list)
                | self
                    .bvh_buffer
                    .update(&engine_state.bounding_volume_hierarchy);

            // if updating the object buffers caused a reallocation, update the bindings so the raytracer
            // has access to the new buffers
            if update_object_bindings {
                self.raytrace_render_context.on_object_update(
                    &self.sphere_list_buffer,
                    &self.plane_list_buffer,
                    &self.aabb_list_buffer,
                    &self.bvh_buffer,
                );
            }

            // update the version to match
            self.object_buffer_version = engine_state.object_list.version();
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.raytrace_render_context.resize(new_size);
        self.bloom_render_context.resize(
            new_size,
            &self.raytrace_render_context.color_texture,
            &self.screen_buffer,
        );
        self.final_render_context
            .resize(&self.bloom_render_context.bloom_texture);
    }

    pub fn update(
        &mut self,
        render_state: &RenderState,
        engine_state: &EngineState,
        encoder: &mut wgpu::CommandEncoder,
        surface_texture: &wgpu::SurfaceTexture,
    ) {
        if engine_state.input.keys.just_pressed(RECOMPILE_SHADERS_KEY) {
            self.raytrace_render_context.recompile_shaders();
            self.bloom_render_context.recompile_shaders();
            self.final_render_context.recompile_shaders();
        }

        self.update_object_buffers(engine_state);

        self.screen_buffer
            .update(render_state, &engine_state.camera);

        self.raytrace_render_context.draw(encoder);
        self.bloom_render_context.draw(encoder);
        self.final_render_context.draw(encoder, surface_texture);
    }
}
