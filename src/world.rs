use std::slice::Iter;

use glam::Vec3;

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

    #[inline]
    pub fn entity_iter(&self) -> Iter<Entity> {
        self.entities.iter()
    }

    #[inline]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }
}

pub struct Entity {
    pos: Vec3,
    texture: EntityTexture,
}

impl Entity {
    pub fn new(x: f32, y: f32, z: f32, texture: EntityTexture) -> Self {
        Self {
            pos: Vec3::new(x, y, z),
            texture,
        }
    }

    #[inline]
    pub fn pos(&self) -> Vec3 {
        self.pos
    }

    #[inline]
    pub fn texture(&self) -> EntityTexture {
        self.texture
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EntityTexture {
    Glass
}
