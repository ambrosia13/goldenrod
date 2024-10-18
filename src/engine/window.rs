use std::sync::Arc;

use winit::{
    event_loop::EventLoop,
    window::{CursorGrabMode, Window, WindowBuilder},
};

pub fn create_window() -> (EventLoop<()>, Arc<Window>) {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    window.set_title("goldenrod rendering engine");

    // Set the cursor grab mode to one that is supported by the system.
    window
        .set_cursor_grab(CursorGrabMode::Confined)
        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
        .unwrap();
    window.set_cursor_visible(false);

    window.set_maximized(true);

    let window = Arc::new(window);

    (event_loop, window)
}
