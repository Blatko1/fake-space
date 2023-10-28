use crate::{
    canvas::Canvas,
    draw::Raycaster, //textures::TextureManager,
    map::{Map},
    voxel::VoxelModelManager,
    textures::{Texture, TextureData, TextureManager}
};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, KeyboardInput},
};

pub struct State {
    canvas: Canvas,
    raycaster: Raycaster,
    models: VoxelModelManager,
    map: Map,
    textures: TextureManager,
}

impl State {
    pub fn new(canvas: Canvas, map: Map, textures: Vec<TextureData>) -> Self {
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
            map,
            textures: TextureManager::new(textures),
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
