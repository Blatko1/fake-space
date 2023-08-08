use crate::{canvas::Canvas, map::Map, raycaster::Raycaster, window::Window};
use winit::{
    dpi::PhysicalSize,
    event::KeyboardInput,
};

pub struct State {
    window: Window,
    canvas: Canvas,
    raycaster: Raycaster,
    map: Map,
}

impl State {
    pub fn new(window: Window, width: u32, height: u32) -> Self {
        let canvas = Canvas::new(
            window.device(),
            window.config(),
            window.config().format,
            width,
            height,
        );
        let raycaster = Raycaster::new(
            2.0,
            2.0,
            90f32.to_radians(),
            80f32.to_radians(),
            width,
            height,
        );

        Self {
            window,
            canvas,
            raycaster,
            map: Map::new_test(),
        }
    }

    pub fn update(&mut self) {
        self.canvas.clear_data();
        self.raycaster.update();
        self.raycaster.cast_rays(&self.map);
        self.raycaster.render(self.canvas.data_mut());
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        self.canvas.render(&self.window)
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window.resize(new_size);
        self.canvas
            .resize(self.window.queue(), self.window.config());
    }

    pub fn process_input(&mut self, keyboard: KeyboardInput) {
        self.raycaster.process_input(keyboard);
    }

    pub fn window(&mut self) -> &mut Window {
        &mut self.window
    }
}
