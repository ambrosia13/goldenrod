use input::Input;
use render_state::RenderState;
use time::Time;
use winit::event::{DeviceEvent, Event, WindowEvent};

pub mod input;
pub mod render_state;
pub mod time;
pub mod window;

pub fn run() {
    let (event_loop, window) = window::create_window();

    let mut render_state = pollster::block_on(RenderState::new(window.clone()));

    let mut input = Input::new();
    let mut time = Time::new();

    event_loop
        .run(move |event, control_flow| match event {
            Event::DeviceEvent {
                event:
                    DeviceEvent::MouseMotion {
                        delta: (delta_x, delta_y),
                    },
                ..
            } => input.set_mouse_delta(delta_x, delta_y),
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => control_flow.exit(),
                WindowEvent::Resized(size) => render_state.resize(size),
                WindowEvent::RedrawRequested => {
                    // We want another frame after this one
                    render_state.window.request_redraw();

                    match render_state.begin_frame() {
                        Ok((encoder, surface_texture)) => {
                            render_state.finish_frame(encoder, surface_texture);
                        }
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            let size = render_state.size;
                            render_state.resize(size);
                        }
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout");
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("Out of memory, exiting");
                            control_flow.exit();
                        }
                    }

                    time.update();
                }
                _ => {}
            },
            _ => {}
        })
        .unwrap();
}
