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

use crate::{renderer::buffers::ScreenBuffer, state::camera::Camera};

pub mod input;
pub mod render_state;
pub mod render_state_ext;
pub mod time;
pub mod window;

pub struct EngineState {
    pub input: Input,
    pub time: Time,

    pub camera: Camera,
    pub screen_buffer: ScreenBuffer,
}

impl EngineState {
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

        let screen_buffer = ScreenBuffer::new(render_state);

        Self {
            input,
            time,
            camera,
            screen_buffer,
        }
    }
}

pub enum AppState {
    Uninit,
    Init {
        window: Arc<Window>,
        render_state: RenderState,
        engine_state: EngineState,
    },
}

pub struct App {
    state: AppState,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Uninit,
        }
    }
}

#[allow(unused)]
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let AppState::Uninit = &self.state {
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
            screen_buffer,
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
            }
            WindowEvent::RedrawRequested => {
                // We want another frame after this one
                render_state.window.request_redraw();

                let (encoder, surface_texture) = match render_state.begin_frame() {
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

                input.update();
                time.update();

                camera.update(input, time);

                screen_buffer.update(render_state, camera);

                render_state.finish_frame(encoder, surface_texture);
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let AppState::Init {
            window,
            render_state,
            engine_state,
        } = &mut self.state
        else {
            return;
        };

        let EngineState {
            input,
            time,
            camera,
            screen_buffer,
        } = engine_state;

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
