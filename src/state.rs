use winit::{
    dpi::PhysicalSize,
    event::{KeyboardInput, VirtualKeyCode},
};

use crate::utils::Window;

pub struct State {
    window: Window,

    should_exit: bool
}

impl State {
    pub fn new(window: Window) -> Self {
        Self { window, should_exit: false }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window.resize(new_size)
    }

    pub fn process_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
            self.should_exit = true;
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }
}
