use crate::{canvas::Canvas, player::Player, window::Window};
use rand::Rng;
use winit::{
    dpi::PhysicalSize,
    event::{KeyboardInput, VirtualKeyCode},
};

pub struct State {
    window: Window,
    canvas: Canvas,

    player: Player,

    should_exit: bool,
}

impl State {
    pub fn new(window: Window, width: u32, height: u32) -> Self {
        let mut canvas =
            Canvas::new(window.device(), window.config().format, width, height);

        let mut rng = rand::thread_rng();
        for pixel in canvas.data_mut().chunks_exact_mut(4) {
            pixel[0] = 100;
            pixel[1] = 200;
            pixel[2] = 250;
            pixel[3] = 255;
        }

        Self {
            window,
            canvas,

            player: Player::default(),

            should_exit: false,
        }
    }

    pub fn update(&mut self) {
        let mut data = self.canvas.data_mut();
        let pos_x = ((self.player.x / (MAP_WIDTH * TILE_SIZE) as f32) * self.canvas.width() as f32) as u32;
        let pos_y = ((self.player.y / (MAP_HEIGHT * TILE_SIZE) as f32) * self.canvas.height() as f32) as u32;
        
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

    pub fn process_input(&mut self, input: KeyboardInput) {
        self.player.process_input(input);

        if let Some(code) = input.virtual_keycode {
            match code {
                VirtualKeyCode::Escape => self.should_exit = true,
                _ => (),
            }
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn window(&mut self) -> &mut Window {
        &mut self.window
    }
}

const TILE_SIZE: usize = 10;
const MAP_WIDTH: usize = 16;
const MAP_HEIGHT: usize = 16;
const MAP: [usize; MAP_WIDTH * MAP_HEIGHT] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0,
    0, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1,
];
