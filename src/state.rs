use crate::hud::Hud;
use crate::{
    backend::Canvas,
    player::Player,
    render::camera::Camera,
    voxel::VoxelModelManager,
    world::{RoomID, World},
};
use wgpu_text::glyph_brush::ab_glyph::FontVec;
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, KeyboardInput},
};

pub struct State {
    canvas: Canvas,
    models: VoxelModelManager,
    hud: Hud,

    world: World,
    player: Player,
}

impl State {
    pub fn new(canvas: Canvas, world: World) -> Self {
        let camera = Camera::new(
            10.5,
            1.0,
            14.5,
            90f32.to_radians(),
            90f32.to_radians(),
            canvas.width(),
            canvas.height(),
        );
        // TODO change/fix this later
        let font_data = include_bytes!("../res/DejaVuSans.ttf").to_owned().to_vec();
        let font = FontVec::try_from_vec(font_data).unwrap();
        let hud = Hud::new(canvas.gfx(), font);

        Self {
            canvas,
            models: VoxelModelManager::init(),
            hud,

            world,
            player: Player::new(camera, RoomID(0)),
        }
    }

    pub fn update(&mut self) {
        self.player.update(&self.world);
        self.world.update(&self.player);
        self.hud.update(&self.world, &self.player);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.canvas.clear_buffer();
        self.player
            .cast_and_draw(&self.world, self.canvas.mut_column_iterator());
        // TODO return the result instead of unwrap
        self.hud.queue_data(self.canvas.gfx()).unwrap();
        self.canvas.render(&self.hud)
    }

    pub fn resize(&mut self,  new_size: PhysicalSize<u32>) {
        self.canvas.resize(new_size);
        self.hud.on_resize(&self.canvas);
    }

    pub fn process_keyboard_input(&mut self, event: KeyboardInput) {
        self.player.process_keyboard_input(event);
    }

    pub fn process_mouse_input(&mut self, event: DeviceEvent) {
        self.player.process_mouse_input(event);
    }

    pub fn on_surface_lost(&self) {
        self.canvas.on_surface_lost()
    }
}
