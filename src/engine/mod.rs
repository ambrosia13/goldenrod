use std::sync::Arc;

use engine_state::EngineState;
use render_state::RenderState;
use renderer::Renderer;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

pub mod engine_state;
pub mod input;
pub mod render_state;
pub mod render_state_ext;
pub mod renderer;
pub mod time;

#[allow(clippy::large_enum_variant)]
pub enum AppState<'a> {
    Uninit,
    Init {
        window: Arc<Window>,
        render_state: RenderState,
        engine_state: EngineState,
        renderer: Renderer<'a>,
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
            let renderer = Renderer::init(&render_state);

            self.state = AppState::Init {
                window,
                render_state,
                engine_state,
                renderer,
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
            renderer,
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
                    }
                };

                engine_state.camera.fov += delta * 25.0;
                engine_state.camera.fov = f32::clamp(engine_state.camera.fov, 30.0, 150.0);
            }

            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                engine_state.camera.reconfigure_aspect(size);
                render_state.resize(size);

                renderer.resize(size);
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

                engine_state.update();
                renderer.update(render_state, engine_state, &mut encoder, &surface_texture);

                render_state.finish_frame(encoder, surface_texture);

                engine_state.post_frame_update();
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
