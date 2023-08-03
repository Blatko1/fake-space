use crate::{canvas::Canvas, window::Window};
use rand::Rng;
use winit::{
    dpi::PhysicalSize,
    event::{KeyboardInput, VirtualKeyCode},
};

pub struct State {
    window: Window,
    canvas: Canvas,

    should_exit: bool,
}

impl State {
    pub fn new(window: Window, width: u32, height: u32) -> Self {
        let mut canvas =
            Canvas::new(window.device(), window.config().format, width, height);

        let mut rng = rand::thread_rng();
        for pixel in canvas.data_mut().chunks_exact_mut(4) {
            pixel[0] = (255.0 * rng.gen::<f32>()) as u8;
            pixel[1] = (255.0 * rng.gen::<f32>()) as u8;
            pixel[2] = (255.0 * rng.gen::<f32>()) as u8;
            pixel[3] = 255;
        }

        Self {
            window,
            canvas,

            should_exit: false,
        }
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        self.canvas.render(&self.window)
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window.resize(new_size);
        self.canvas.resize(
            self.window.queue(),
            new_size.width as f32,
            new_size.height as f32,
        );
    }

    pub fn process_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
            self.should_exit = true;
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn window(&mut self) -> &mut Window {
        &mut self.window
    }
}
