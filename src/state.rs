use crate::{
    backend::Canvas, voxel::VoxelModelManager, world::{world::{World, RoomID}}, player::{Player}, render::camera::Camera,
};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, KeyboardInput},
};

pub struct State {
    canvas: Canvas,
    models: VoxelModelManager,

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

        Self {
            canvas,
            models: VoxelModelManager::init(),

            world,
            player: Player::new(camera, RoomID(0))
        }
    }

    pub fn update(&mut self) {
        self.player.update(&self.world);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.canvas.clear_buffer();
        self.player
            .cast_and_draw(&self.world, self.canvas.mut_column_iterator());
        self.canvas.render()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.canvas.resize(new_size);
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
