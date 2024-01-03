use crate::{
    backend::Canvas, render::RayCaster, voxel::VoxelModelManager, world::world::World,
};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, KeyboardInput},
};

pub struct State {
    canvas: Canvas,
    raycaster: RayCaster,
    models: VoxelModelManager,
    world: World,
}

impl State {
    pub fn new(canvas: Canvas, world: World) -> Self {
        let raycaster = RayCaster::new(
            2.0,
            0.0,
            2.0,
            90f32.to_radians(),
            canvas.width(),
            canvas.height(),
        );

        Self {
            canvas,
            raycaster,
            models: VoxelModelManager::init(),
            world,
        }
    }

    pub fn update(&mut self) {
        self.canvas.clear_buffer();
        self.raycaster.update(&mut self.world);
        self.raycaster
            .cast_and_draw(&mut self.world, self.canvas.buffer_mut());
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.canvas.render()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.canvas.resize(new_size);
    }

    pub fn process_keyboard_input(&mut self, event: KeyboardInput) {
        self.raycaster.process_keyboard_input(event);
    }

    pub fn process_mouse_input(&mut self, event: DeviceEvent) {
        self.raycaster.process_mouse_input(event);
    }

    pub fn on_surface_lost(&self) {
        self.canvas.on_surface_lost()
    }
}
