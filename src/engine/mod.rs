use std::sync::Arc;

use glam::Vec3;
use input::Input;
use render_state::RenderState;
use time::Time;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

use crate::{
    renderer::{
        buffers::{AabbListBuffer, PlaneListBuffer, ScreenBuffer, SphereListBuffer},
        final_pass::FinalRenderContext,
        raytrace::RaytraceRenderContext,
        screen_quad::ScreenQuad,
    },
    state::{
        camera::Camera,
        object::ObjectList,
    },
};

pub mod input;
pub mod render_state;
pub mod render_state_ext;
pub mod time;

pub struct EngineState<'a> {
    pub input: Input,
    pub time: Time,

    pub camera: Camera,
    pub object_list: ObjectList,

    pub screen_buffer: ScreenBuffer,

    pub object_buffer_version: u32,
    pub sphere_list_buffer: SphereListBuffer,
    pub plane_list_buffer: PlaneListBuffer,
    pub aabb_list_buffer: AabbListBuffer,

    pub screen_quad: ScreenQuad,
    pub raytrace_render_context: RaytraceRenderContext<'a>,
    pub final_render_context: FinalRenderContext,
}

impl<'a> EngineState<'a> {
    pub fn new(render_state: &RenderState) -> Self {
        let input = Input::new();
        let time = Time::new();

        let camera = Camera::new(
            Vec3::ZERO,
            Vec3::NEG_Z,
            45.0,
            render_state.size,
            0.005,
            500.0,
        );

        let mut object_list = ObjectList::new();
        object_list.random_scene();

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
            input,
            time,
            camera,
            object_list,
            screen_buffer,
            object_buffer_version,
            sphere_list_buffer,
            plane_list_buffer,
            aabb_list_buffer,
            screen_quad,
            raytrace_render_context,
            final_render_context,
        }
    }

    pub fn update_object_buffers(&mut self, render_state: &RenderState) {
        // If the object buffers don't reflect the current object list, update those
        if self.object_buffer_version != self.object_list.version {
            log::info!("Updating object buffers");

            #[rustfmt::skip]
            let update_object_bindings = 
                self.sphere_list_buffer.update(&self.object_list) | 
                self.plane_list_buffer.update(&self.object_list) | 
                self.aabb_list_buffer.update(&self.object_list);

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
            self.object_buffer_version = self.object_list.version;
        }
    }
}

pub enum AppState<'a> {
    Uninit,
    Init {
        window: Arc<Window>,
        render_state: RenderState,
        engine_state: EngineState<'a>,
    },
}

pub struct App<'a> {
    state: AppState<'a>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            state: AppState::Uninit,
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if matches!(&self.state, AppState::Uninit) {
            let window_attributes = WindowAttributes::default()
                .with_title("goldenrod rendering engine")
                .with_maximized(true);

            let window = event_loop
                .create_window(window_attributes)
                .expect("Couldn't create window");

            let window = Arc::new(window);

            let render_state = pollster::block_on(RenderState::new(window.clone()));
            let engine_state = EngineState::new(&render_state);

            self.state = AppState::Init {
                window,
                render_state,
                engine_state,
            };

            log::info!("App state initialized");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let AppState::Init {
            window,
            render_state,
            engine_state,
        } = &mut self.state
        else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                input::handle_keyboard_input_event(&mut engine_state.input, event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                input::handle_mouse_input_event(&mut engine_state.input, state, button);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, lines_y) => lines_y / 20.0,
                    winit::event::MouseScrollDelta::PixelDelta(physical_position) => {
                        physical_position.y as f32
                    },
                };

                engine_state.camera.fov += delta * 25.0;
                engine_state.camera.fov = f32::clamp(engine_state.camera.fov, 30.0, 150.0);
            }

            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                engine_state.camera.reconfigure_aspect(size);
                render_state.resize(size);

                engine_state.raytrace_render_context.resize(size);
                engine_state.final_render_context.resize(&engine_state.raytrace_render_context.color_texture);
            }
            WindowEvent::RedrawRequested => {
                // We want another frame after this one
                render_state.window.request_redraw();

                let (mut encoder, surface_texture) = match render_state.begin_frame() {
                    Ok(r) => r,
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        render_state.reconfigure();
                        return;
                    }
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout");
                        return;
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("Out of memory, exiting");
                        event_loop.exit();
                        return;
                    }
                };

                engine_state.update_object_buffers(render_state);

                engine_state.camera.update_position(&engine_state.input, &engine_state.time);

                engine_state.screen_buffer.update(render_state, &engine_state.camera);

                engine_state.raytrace_render_context.draw(&mut encoder);
                engine_state.final_render_context.draw(&mut encoder, &surface_texture, &engine_state.screen_quad);

                render_state.finish_frame(encoder, surface_texture);

                engine_state.input.update();
                engine_state.time.update();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let AppState::Init { engine_state, .. } = &mut self.state else {
            return;
        };

        let EngineState { input, camera, .. } = engine_state;

        if let DeviceEvent::MouseMotion {
            delta: (delta_x, delta_y),
        } = event
        {
            input.set_mouse_delta(delta_x, delta_y);
            camera.update_rotation(input, 0.1);
        }
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
    }
}

pub fn run() {
    let event_loop = EventLoop::new().expect("Couldn't create window event loop");
    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}
