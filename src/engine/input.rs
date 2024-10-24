use std::{collections::HashSet, fmt::Debug};

use glam::DVec2;
use winit::{
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::KeyCode,
};

pub struct Input {
    pub keys: ButtonInputs<KeyCode>,
    pub mouse_buttons: ButtonInputs<MouseButton>,
    mouse_delta: DVec2,
}

impl Input {
    pub fn new() -> Self {
        Self {
            keys: ButtonInputs::new(),
            mouse_buttons: ButtonInputs::new(),
            mouse_delta: DVec2::ZERO,
        }
    }

    pub fn set_mouse_delta(&mut self, delta_x: f64, delta_y: f64) {
        self.mouse_delta = DVec2::new(delta_x, delta_y);
    }

    pub fn mouse_delta(&self) -> DVec2 {
        self.mouse_delta
    }

    pub fn update(&mut self) {
        self.keys.update();
        self.mouse_buttons.update();
    }
}

pub struct ButtonInputs<T>
where
    T: Copy + Eq + std::hash::Hash,
{
    pressed: HashSet<T>,
    just_pressed: HashSet<T>,
    just_released: HashSet<T>,
}

impl<T> ButtonInputs<T>
where
    T: Copy + Eq + std::hash::Hash,
{
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
        }
    }

    pub fn press(&mut self, input: T) {
        if self.pressed.insert(input) {
            self.just_pressed.insert(input);
        }
    }

    pub fn release(&mut self, input: T) {
        if self.pressed.remove(&input) {
            self.just_released.insert(input);
        }
    }

    pub fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn pressed(&self, input: T) -> bool {
        self.pressed.contains(&input)
    }

    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed.contains(&input)
    }

    pub fn just_released(&self, input: T) -> bool {
        self.just_released.contains(&input)
    }
}

pub fn handle_keyboard_input_event(input: &mut Input, event: KeyEvent) {
    let key = match event.physical_key {
        winit::keyboard::PhysicalKey::Code(key_code) => key_code,
        winit::keyboard::PhysicalKey::Unidentified(native_key_code) => {
            log::warn!("Unidentified physical key press: {:?}", native_key_code);
            return;
        }
    };

    match event.state {
        winit::event::ElementState::Pressed => input.keys.press(key),
        winit::event::ElementState::Released => input.keys.release(key),
    }
}

pub fn handle_mouse_input_event(input: &mut Input, state: ElementState, button: MouseButton) {
    match state {
        winit::event::ElementState::Pressed => input.mouse_buttons.press(button),
        winit::event::ElementState::Released => input.mouse_buttons.release(button),
    }
}
