use crate::{
    canvas::Canvas, draw::Raycaster, map::TestMap, textures::TextureManager,
    voxel::VoxelModelManager,
};
use winit::{dpi::PhysicalSize, event::KeyboardInput};

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

    pub fn process_input(&mut self, keyboard: KeyboardInput) {
        self.raycaster.process_input(keyboard);
    }

    pub fn on_surface_lost(&self) {
        self.canvas.on_surface_lost()
    }
}
