use crate::{
    canvas::{Canvas, Pixel},
    player::Player,
    window::Window,
};
use rand::Rng;
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, KeyboardInput},
};

pub struct State {
    window: Window,
    canvas: Canvas,

    player: Player,

    should_exit: bool,
}

impl State {
    pub fn new(window: Window, width: u32, height: u32) -> Self {
        let canvas =
            Canvas::new(window.device(), window.config().format, width, height);

        Self {
            window,
            canvas,

            player: Player::new(2.0, 2.0, 90.0),

            should_exit: false,
        }
    }

    pub fn update(&mut self) {
        self.player.update();

        self.canvas.clear_data();
        let width = self.canvas.width();
        let height = self.canvas.height();
        let data = self.canvas.data_mut();
        self.player.cast_rays(width, height, data);
        //let hits = self.player.cast_rays(width, height);
        //for hit in hits {
        //    let pos_x = ((hit.pos.x / MAP_WIDTH as f32) * width as f32) as u32;
        //    let pos_y = (((MAP_HEIGHT as f32 - hit.pos.y) / MAP_HEIGHT as f32)
        //        * height as f32) as u32;
        //    if ((pos_x + pos_y * width) as usize) < (data.len()) {
        //        data[(pos_x + pos_y * width) as usize] = Pixel {
        //            r: 150,
        //            g: 150,
        //            b: 150,
        //            a: 150,
        //        }
        //    }
        //}

        //let lines = self.player.cast_rays(width, height);
        //for (e, line) in lines.iter().enumerate() {
        //    for i in line.0..line.1 {
        //        data[(i as usize)*width as usize + e] = Pixel {
        //            r: 100,
        //            g: 150,
        //            b: 0,
        //            a: 150,
        //        }
        //    }
        //}

        //let pos_x =
        //    ((self.player.pos.x / MAP_WIDTH as f32) * width as f32) as u32;
        //let pos_y = (((MAP_HEIGHT as f32 - self.player.pos.y)
        //    / MAP_HEIGHT as f32)
        //    * height as f32) as u32;
        //data[(pos_x + pos_y * width) as usize] = Pixel {
        //    r: 250,
        //    g: 250,
        //    b: 250,
        //    a: 250,
        //};
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

    pub fn process_input(&mut self, keyboard: KeyboardInput) {
        self.player.process_input(keyboard);
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn window(&mut self) -> &mut Window {
        &mut self.window
    }
}

pub const MAP_WIDTH: usize = 10;
pub const MAP_HEIGHT: usize = 10;
pub const MAP: [[u32; MAP_WIDTH]; MAP_HEIGHT] = [
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 1, 1, 1, 1, 1, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 1, 0, 1, 0, 0, 1, 0, 1],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
];
