use crate::{
    canvas::Canvas,
    draw::Raycaster,
    map::Map,
    object::ModelManager,
    textures::BLUE_GLASS,
    world::{Entity, World},
};
use winit::{dpi::PhysicalSize, event::KeyboardInput};

pub struct State {
    canvas: Canvas,
    raycaster: Raycaster,
    models: ModelManager,
    map: Map,
    world: World,
}

impl State {
    pub fn new(canvas: Canvas) -> Self {
        let raycaster = Raycaster::new(
            2.0,
            0.5,
            2.0,
            90f32.to_radians(),
            80f32.to_radians(),
            canvas.width(),
            canvas.height(),
        );
        let mut world = World::new();
        world.new_entity(Entity::new(5.0, 5.0, BLUE_GLASS));

        Self {
            canvas,
            raycaster,
            models: ModelManager::init(),
            map: Map::new_test(),
            world,
        }
    }

    pub fn update(&mut self) {
        self.canvas.clear_data();
        self.raycaster.update();
        self.raycaster.cast_rays(&self.map);
        self.raycaster.render(&self.models, self.canvas.data_mut());
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
