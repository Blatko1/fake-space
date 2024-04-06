mod parser;
pub mod portal;
pub mod textures;

use rand::{rngs::ThreadRng, seq::SliceRandom};
use std::path::PathBuf;

use crate::player::render::PointXZ;
use crate::voxel::{VoxelModelDataRef, VoxelModelID, VoxelModelManager};
use crate::world::portal::{DummyPortal, Portal, PortalID};
use parser::{error::ParseError, WorldParser};
use textures::{TextureData, TextureID, TextureManager};

use self::textures::TextureDataRef;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoomID(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct SegmentID(pub usize);

pub struct World {
    segments: Vec<Segment>,
    texture_manager: TextureManager,
    voxel_model_manager: VoxelModelManager,

    rng: ThreadRng,
    // Each room has index which is the position in this Vec
    rooms: Vec<Room>,
}

impl World {
    pub fn from_path<P: Into<PathBuf>>(path: P) -> Result<Self, ParseError> {
        WorldParser::new(path)?.parse()
    }

    // TODO starting segment is always '0' and main room is '1'
    pub fn new(segments: Vec<Segment>, textures: Vec<TextureData>) -> Self {
        let mut rooms = Vec::new();
        let mut room_counter = 0;
        let mut rng = rand::thread_rng();

        // Select the first segment which repeats only once
        let segment = &segments[0];
        let mut starting_room = Room {
            id: RoomID(room_counter),
            segment_id: segment.id,
            portals: segment.unlinked_portals.clone(),
            is_fully_generated: true,
            skybox: segment.skybox,
            ambient_light_intensity: segment.ambient_light_intensity,
        };
        room_counter += 1;

        let root_segment = &segments[1];
        let mut adjacent_rooms: Vec<Room> = starting_room
            .portals
            .iter_mut()
            .map(|portal| {
                let mut new_room = Room {
                    id: RoomID(room_counter),
                    segment_id: root_segment.id,
                    portals: root_segment.unlinked_portals.clone(),
                    is_fully_generated: false,
                    skybox: root_segment.skybox,
                    ambient_light_intensity: root_segment.ambient_light_intensity,
                };
                let room_rand_portal = new_room.portals.choose_mut(&mut rng).unwrap();
                // Connect the two portals:
                // Connect the starting room with new random room
                portal.link = Some((new_room.id, room_rand_portal.id));
                // Connect the new random room with the starting room
                room_rand_portal.link = Some((starting_room.id, portal.id));
                room_counter += 1;

                new_room
            })
            .collect();
        rooms.push(starting_room);
        rooms.append(&mut adjacent_rooms);

        Self {
            segments,
            texture_manager: TextureManager::new(textures),
            voxel_model_manager: VoxelModelManager::init(),
            rng,
            rooms,
        }
    }

    pub fn update(&mut self, current_room_id: RoomID) {
        self.fully_generate_room(current_room_id);
        // The current room is now fully generated
        let current_room = &mut self.rooms[current_room_id.0];
        // Clone needed for the borrow checker. Is there a better solution???
        current_room
            .portals
            .clone()
            .iter()
            .for_each(|portal| self.fully_generate_room(portal.link.unwrap().0));
    }

    fn fully_generate_room(&mut self, room_id: RoomID) {
        let mut next_id = self.rooms.len();
        let current_room = &mut self.rooms[room_id.0];
        // Check if current room has all adjacent rooms generated
        if !current_room.is_fully_generated {
            // Generated adjacent rooms where needed
            let mut adjacent_rooms: Vec<Room> = current_room
                .portals
                .iter_mut()
                .filter(|portal| portal.link.is_none())
                .map(|portal| {
                    // Skip the first (two) rooms since they appear only once
                    let rand_segment = self.segments[1..].choose(&mut self.rng).unwrap();
                    let mut new_room = Room {
                        id: RoomID(next_id),
                        segment_id: rand_segment.id,
                        portals: rand_segment.unlinked_portals.clone(),
                        is_fully_generated: false,
                        skybox: rand_segment.skybox,
                        ambient_light_intensity: rand_segment.ambient_light_intensity,
                    };
                    let room_rand_portal =
                        new_room.portals.choose_mut(&mut self.rng).unwrap();

                    // Connect the two portals:
                    // Connect the starting room with new random room
                    portal.link = Some((new_room.id, room_rand_portal.id));
                    // Connect the new random room with the starting room
                    room_rand_portal.link = Some((current_room.id, portal.id));
                    next_id += 1;
                    new_room
                })
                .collect();
            current_room.is_fully_generated = true;

            self.rooms.append(&mut adjacent_rooms);
        }
    }

    pub fn get_room_data(&self, index: RoomID) -> RoomRef {
        let room = &self.rooms[index.0];
        RoomRef {
            segment: &self.segments[room.segment_id.0],
            data: room,
        }
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    pub fn get_texture(&self, id: TextureID) -> TextureDataRef {
        self.texture_manager.get_texture_data(id)
    }

    pub fn get_skybox_textures(&self, skybox: &SkyboxTextureIDs) -> SkyboxTexturesRef {
        SkyboxTexturesRef {
            north: self.texture_manager.get_texture_data(skybox.north),
            east: self.texture_manager.get_texture_data(skybox.east),
            south: self.texture_manager.get_texture_data(skybox.south),
            west: self.texture_manager.get_texture_data(skybox.west),
            top: self.texture_manager.get_texture_data(skybox.top),
            bottom: self.texture_manager.get_texture_data(skybox.bottom),
        }
    }

    pub fn get_voxel_model(&self, id: VoxelModelID) -> VoxelModelDataRef {
        self.voxel_model_manager.get_model(id)
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
}

// TODO remove 'pub'
#[derive(Debug)]
pub struct Room {
    id: RoomID,
    segment_id: SegmentID,
    // Each portal has its own index which is the position in this Vec
    portals: Vec<Portal>,
    is_fully_generated: bool,
    skybox: SkyboxTextureIDs,
    ambient_light_intensity: f32,
}

impl Room {
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

// TODO Use a struct or type for dimensions instead
/// A map segment (room) with immutable data.
/// You can mutate room data in a [`Room`] struct.
#[derive(Debug)]
pub struct Segment {
    id: SegmentID,
    name: String,
    dimensions: (u64, u64),
    tiles: Vec<Tile>,
    unlinked_portals: Vec<Portal>,
    skybox: SkyboxTextureIDs,
    repeatable: bool,
    ambient_light_intensity: f32,
}

impl Segment {
    pub fn new(
        id: SegmentID,
        name: String,
        dimensions: (u64, u64),
        tiles: Vec<Tile>,
        skybox: SkyboxTextureIDs,
        repeatable: bool,
        ambient_light_intensity: f32,
    ) -> Self {
        // Create unlinked Portals from DummyPortals
        let unlinked_portals = tiles
            .iter()
            .filter(|tile| tile.portal.is_some())
            .map(|tile| {
                let dummy = tile.portal.unwrap();
                Portal {
                    id: dummy.id,
                    direction: dummy.direction,
                    position: tile.position,
                    center: PointXZ {
                        x: tile.position.x as f32 + 0.5,
                        z: tile.position.z as f32 + 0.5,
                    },
                    ground_level: tile.ground_level,
                    link: None,
                }
            })
            .collect();
        Self {
            id,
            name,
            dimensions,
            unlinked_portals,
            tiles,
            skybox,
            repeatable,
            ambient_light_intensity,
        }
    }

    /// Returns the value at the provided map coordinates.
    /// Parsed arguments are assumed to be in map bound and correct.
    /// This game assumes that the y-axis points upwards, the z-axis forwards
    /// and the x-axis to the right so `x` represents moving left or right
    /// and `z` represents moving forward or backward on the map.
    /// Returns [`Tile::Void`] if coordinates are out of bounds.
    #[inline]
    pub fn get_tile(&self, x: i64, z: i64) -> Option<&Tile> {
        // TODO do something about i64 arguments and 'if' conditions
        if x >= self.dimensions.0 as i64
            || x < 0
            || z >= self.dimensions.1 as i64
            || z < 0
        {
            return None;
        }
        self.tiles
            .get(z as usize * self.dimensions.0 as usize + x as usize)
    }

    pub fn get_skybox(&self) -> SkyboxTextureIDs {
        self.skybox
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TilePosition {
    pub x: u64,
    pub z: u64,
}

// TODO try removing Clone and Copy
#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub position: TilePosition,
    /// Texture of the bottom wall walls.
    pub bottom_wall_tex: TextureID,
    /// Texture of the top wall walls.
    pub top_wall_tex: TextureID,
    /// Texture of the bottom platform.
    pub ground_tex: TextureID,
    /// Texture of the top platform.
    pub ceiling_tex: TextureID,
    /// `Y-level` - starting lower bound of the bottom wall;
    /// level from which the bottom wall stretches.
    pub bottom_level: f32,
    /// `Y-level` - ending upper bound of the bottom wall;
    /// area/platform on which the player is walking.
    pub ground_level: f32,
    /// `Y-level` - starting lower bound of the top wall; the ceiling.
    pub ceiling_level: f32,
    /// `Y-level` - ending upper bound of the top wall;
    /// level to which the top wall stretches.
    pub top_level: f32,
    /// If the current tile should be a portal to different segment (map).
    pub portal: Option<DummyPortal>,
    pub voxel_model: Option<VoxelModelID>,
}

#[derive(Copy, Clone, Debug)]
pub struct SkyboxTextureIDs {
    pub north: TextureID,
    pub east: TextureID,
    pub south: TextureID,
    pub west: TextureID,
    pub top: TextureID,
    pub bottom: TextureID,
}

#[derive(Debug)]
pub struct SkyboxTexturesRef<'a> {
    pub north: TextureDataRef<'a>,
    pub east: TextureDataRef<'a>,
    pub south: TextureDataRef<'a>,
    pub west: TextureDataRef<'a>,
    pub top: TextureDataRef<'a>,
    pub bottom: TextureDataRef<'a>,
}
