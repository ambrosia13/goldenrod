use std::sync::Arc;

use glam::Vec3;
use input::Input;
use render_state::RenderState;
use time::Time;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::KeyCode,
    window::{Window, WindowAttributes},
};

use crate::{
    renderer::{
        buffers::{ScreenBuffer, SphereListBuffer},
        final_pass::FinalRenderContext,
        raytrace::RaytraceRenderContext,
        screen_quad::ScreenQuad,
    },
    state::{
        camera::Camera,
        material::Material,
        object::{ObjectList, Sphere},
    },
};

pub mod input;
pub mod render_state;
pub mod render_state_ext;
pub mod time;
pub mod window;

pub struct EngineState<'a> {
    pub input: Input,
    pub time: Time,

    pub camera: Camera,
    pub object_list: ObjectList,

    pub screen_buffer: ScreenBuffer,
    pub sphere_list_buffer: SphereListBuffer,

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
            0.0001,
            1000.0,
        );

        let object_list = ObjectList::new();

        let screen_buffer = ScreenBuffer::new(render_state);
        let sphere_list_buffer = SphereListBuffer::new(render_state);

        let screen_quad = ScreenQuad::new(render_state);
        let raytrace_render_context =
            RaytraceRenderContext::new(render_state, &screen_buffer, &sphere_list_buffer);
        let final_render_context = FinalRenderContext::new(
            render_state,
            &render_state.surface.get_current_texture().unwrap(),
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
            sphere_list_buffer,
            screen_quad,
            raytrace_render_context,
            final_render_context,
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

        let EngineState {
            input,
            time,
            camera,
            object_list,
            screen_buffer,
            sphere_list_buffer,
            screen_quad,
            raytrace_render_context,
            final_render_context,
        } = engine_state;

        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                input::handle_keyboard_input_event(input, event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                input::handle_mouse_input_event(input, state, button);
            }

            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                camera.reconfigure_aspect(size);
                render_state.resize(size);

                raytrace_render_context.resize(render_state);
                final_render_context.resize(render_state, &raytrace_render_context.color_texture);
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

                if input.keys.just_pressed(KeyCode::KeyT) {
                    let offset = object_list.spheres.len() as f32;
                    object_list.push_sphere(Sphere::new(
                        Vec3::ZERO + Vec3::X * offset,
                        0.5,
                        Material::random(),
                    ));

                    log::info!("Updating sphere list buffer");

                    // When the buffer is reallocated, we need to update its binding in every pass that uses it
                    if sphere_list_buffer.update(render_state, object_list) {
                        raytrace_render_context.on_object_update(render_state, sphere_list_buffer);
                    }
                }

                camera.update(input, time);

                screen_buffer.update(render_state, camera);

                raytrace_render_context.draw(&mut encoder);
                final_render_context.draw(&mut encoder, &surface_texture, screen_quad);

                render_state.finish_frame(encoder, surface_texture);

                input.update();
                time.update();
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

        let EngineState { input, .. } = engine_state;

        if let DeviceEvent::MouseMotion {
            delta: (delta_x, delta_y),
        } = event
        {
            input.set_mouse_delta(delta_x, delta_y);
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
