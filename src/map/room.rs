use glam::Vec2;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

use crate::{
    models::ModelID,
    textures::{TextureArray, TextureDataRef},
};

use super::{
    tilemap::{Tilemap, TilemapID, ObjectID, Skybox},
    portal::{Orientation, Portal, PortalID, Rotation},
};

const VOXEL_CHANCE: f64 = 0.3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoomID(pub usize);

// TODO remove 'pub'
#[derive(Debug)]
pub struct Room {
    pub(super) id: RoomID,
    pub(super) tilemap_id: TilemapID,
    // Each portal has its own index which is the position in this Vec
    pub(super) portals: Vec<Portal>,
    //pub(super) objects: Vec<Option<ModelID>>,
    pub(super) is_fully_generated: bool,
    pub(super) skybox: Skybox,
    pub(super) ambient_light_intensity: f32,

    // TODO finish orientation for skyboxes to remain in place
    /// To which side is the room oriented to or to where points the room north
    pub direction: Vec2,
}

impl Room {
    pub fn new(id: RoomID, tilemap: &Tilemap, direction: Vec2) -> Self {
        Self {
            id,
            tilemap_id: tilemap.id,
            portals: tilemap.unlinked_portals.clone(),
            //objects: blueprint.object_placeholders.clone(),
            is_fully_generated: false,
            skybox: tilemap.default_skybox,
            ambient_light_intensity: tilemap.default_ambient_light,

            direction,
        }
    }

    // TODO show in dbg
    pub fn get_portals(&self) -> &[Portal] {
        &self.portals
    }

    pub fn ambient_light_intensity(&self) -> f32 {
        self.ambient_light_intensity
    }

    pub fn skybox(&self) -> &Skybox {
        &self.skybox
    }
}

#[derive(Debug)]
pub struct RoomRef<'a> {
    pub tilemap: &'a Tilemap,
    pub data: &'a Room,
}

impl<'a> RoomRef<'a> {
    pub fn get_portal(&self, local_id: PortalID) -> Portal {
        self.data.portals[local_id.0]
    }

    //pub fn get_object(&self, local_id: ObjectID) -> Option<ModelID> {
    //    self.data.objects[local_id.0]
    //}
}
