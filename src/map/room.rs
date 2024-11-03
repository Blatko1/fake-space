use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

use crate::{models::ModelID, textures::{TextureArray, TextureDataRef}};

use super::{portal::{Portal, PortalID}, segment::{ObjectID, Segment, SegmentID, SkyboxTextureIDs}};

const VOXEL_CHANCE: f64 = 0.3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoomID(pub usize);

// TODO remove 'pub'
#[derive(Debug)]
pub struct Room {
    pub(super) id: RoomID,
    pub(super) segment_id: SegmentID,
    // Each portal has its own index which is the position in this Vec
    pub(super) portals: Vec<Portal>,
    pub(super) objects: Vec<Option<ModelID>>,
    pub(super) is_fully_generated: bool,
    pub(super) skybox: SkyboxTextureIDs,
    pub(super) ambient_light_intensity: f32,
}

impl Room {
    pub fn new(id: RoomID, segment: &Segment) -> Self {
        Self {
            id,
            segment_id: segment.id,
            portals: segment.unlinked_portals.clone(),
            objects: segment.object_placeholders.clone(),
            is_fully_generated: false,
            skybox: segment.skybox,
            ambient_light_intensity: segment.ambient_light_intensity,
        }
    }

    // TODO show in dbg
    pub fn get_portals(&self) -> &[Portal] {
        &self.portals
    }

    pub fn ambient_light_intensity(&self) -> f32 {
        self.ambient_light_intensity
    }

    pub fn skybox(&self) -> &SkyboxTextureIDs {
        &self.skybox
    }
}

#[derive(Debug)]
pub struct RoomRef<'a> {
    pub segment: &'a Segment,
    pub data: &'a Room,
}

impl<'a> RoomRef<'a> {
    pub fn get_portal(&self, local_id: PortalID) -> Portal {
        self.data.portals[local_id.0]
    }

    pub fn get_object(&self, local_id: ObjectID) -> Option<ModelID> {
        self.data.objects[local_id.0]
    }
}
