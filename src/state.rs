use crate::{
    canvas::Canvas, draw::Raycaster, map::TestMap, textures::TextureManager,
    voxel::VoxelModelManager,
};
use winit::{dpi::PhysicalSize, event::{KeyboardInput, DeviceEvent}};

pub struct State {
    canvas: Canvas,
    raycaster: Raycaster,
    models: VoxelModelManager,
    map: TestMap,
    textures: TextureManager,
}

impl State {
    pub fn new(canvas: Canvas) -> Self {
        let raycaster = Raycaster::new(
            2.0,
            0.5,
            2.0,
            90f32.to_radians(),
            canvas.width(),
            canvas.height(),
        );

        Self {
            canvas,
            raycaster,
            models: VoxelModelManager::init(),
            map: TestMap::new(),
            textures: TextureManager::init(),
        }
    }

    pub fn update(&mut self) {
        self.canvas.clear_data();
        self.raycaster.update();
        self.raycaster.cast_rays(
            &self.map,
            &self.models,
            &self.textures,
            self.canvas.data_mut(),
        );
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
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
