pub struct World {
    entities: Vec<Entity>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    pub fn new_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }
}

pub struct Entity {
    pos_x: f32,
    pos_y: f32,
    texture: &'static [u8],
}

impl Entity {
    pub fn new(x: f32, y: f32, texture: &'static [u8]) -> Self {
        Self {
            pos_x: x,
            pos_y: y,
            texture,
        }
    }
}
