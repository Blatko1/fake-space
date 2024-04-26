mod parser;
pub mod portal;
pub mod textures;
pub mod model;

use nom::error::convert_error;
use nom::Finish;
use rand::Rng;
use rand::{rngs::ThreadRng, seq::SliceRandom};
use std::path::PathBuf;

use crate::player::render::PointXZ;
use crate::world::portal::{DummyPortal, Portal, PortalID};
use parser::WorldParser;
use textures::{TextureData, TextureID, TextureManager};

use self::model::{ModelData, ModelDataRef, ModelID, ModelManager};
use self::parser::cleanup_input;
use self::textures::TextureDataRef;

const VOXEL_CHANCE: f64 = 0.1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoomID(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct SegmentID(pub usize);

pub struct World {
    segments: Vec<Segment>,
    texture_manager: TextureManager,
    model_manager: ModelManager,

    rng: ThreadRng,
    // Each room has index which is the position in this Vec
    rooms: Vec<Room>,
}

impl World {
    pub fn from_path<P: Into<PathBuf>>(path: P) -> std::io::Result<Self> {
        // TODO this 'unwrap()'
        let path: PathBuf = path.into().canonicalize()?;
        let parent_path = path.parent().unwrap().to_path_buf();
        let input = cleanup_input(std::fs::read_to_string(path)?);
        match WorldParser::new(&input, parent_path)?.parse().finish() {
            Ok((_, world)) => Ok(world),
            Err(e) => {
                println!("verbose errors: \n{}", convert_error(input.as_str(), e));
                panic!()
            }
        }
    }

    // TODO starting segment is always '0' and main room is '1'
    pub fn new(
        segments: Vec<Segment>,
        textures: Vec<TextureData>,
        models: Vec<ModelData>,
    ) -> Self {
        let model_manager = ModelManager::new(models);
        let mut rooms = Vec::new();
        let mut room_counter = 0;
        let mut rng = rand::thread_rng();

        // Select the first segment which repeats only once
        let segment = &segments[0];
        let mut starting_room = Room::new_with_rng_objects(
            RoomID(room_counter),
            segment,
            &mut rng,
            model_manager.model_list(),
        );
        room_counter += 1;

        let root_segment = &segments[1];
        let mut adjacent_rooms: Vec<Room> = starting_room
            .portals
            .iter_mut()
            .map(|portal| {
                let mut new_room = Room::new_with_rng_objects(
                    RoomID(room_counter),
                    root_segment,
                    &mut rng,
                    model_manager.model_list(),
                );
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
            model_manager,
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
                    let mut new_room = Room::new_with_rng_objects(
                        RoomID(next_id),
                        rand_segment,
                        &mut self.rng,
                        self.model_manager.model_list(),
                    );
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

    pub fn get_texture(&self, id: TextureID) -> TextureDataRef {
        self.texture_manager.get_texture_data(id)
    }

    pub fn get_model(&self, id: ModelID) -> ModelDataRef {
        self.model_manager.get_model_data(id)
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

    pub fn collect_dbg_data(&self) -> WorldDebugData {
        WorldDebugData {
            room_count: self.rooms.len() as u64,
        }
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

// TODO remove 'pub'
#[derive(Debug)]
pub struct Room {
    id: RoomID,
    segment_id: SegmentID,
    // Each portal has its own index which is the position in this Vec
    portals: Vec<Portal>,
    objects: Vec<Option<ModelID>>,
    is_fully_generated: bool,
    skybox: SkyboxTextureIDs,
    ambient_light_intensity: f32,
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

    pub fn new_with_rng_objects(
        id: RoomID,
        segment: &Segment,
        rng: &mut ThreadRng,
        model_list: &[ModelID],
    ) -> Self {
        let mut room = Self::new(id, segment);
        if !model_list.is_empty() {
            room.objects.iter_mut().for_each(|p| {
                if rng.gen_bool(VOXEL_CHANCE) {
                    let rand_voxel_model = model_list.choose(rng).unwrap();
                    p.replace(*rand_voxel_model);
                }
            });
        }
        room
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

    // TODO show in dbg
    pub fn id(&self) -> RoomID {
        self.id
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
    object_placeholders: Vec<Option<ModelID>>,
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
                    center: PointXZ::new(
                        tile.position.x as f32 + 0.5,
                        tile.position.z as f32 + 0.5,
                    ),
                    ground_level: tile.ground_level,
                    link: None,
                }
            })
            .collect();

        let object_placeholders =
            tiles.iter().filter(|tile| tile.object.is_some()).count();
        Self {
            id,
            name,
            dimensions,
            unlinked_portals,
            tiles,
            object_placeholders: vec![None; object_placeholders],
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
}

// TODO try removing Clone and Copy
#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub position: PointXZ<u64>,
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

    pub object: Option<ObjectID>,
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectID(pub usize);

#[derive(Copy, Clone, Debug)]
pub struct SkyboxTextureIDs {
    pub north: TextureID,
    pub east: TextureID,
    pub south: TextureID,
    pub west: TextureID,
    pub top: TextureID,
    pub bottom: TextureID,
}

#[derive(Debug, Clone, Copy)]
pub struct SkyboxTexturesRef<'a> {
    pub north: TextureDataRef<'a>,
    pub east: TextureDataRef<'a>,
    pub south: TextureDataRef<'a>,
    pub west: TextureDataRef<'a>,
    pub top: TextureDataRef<'a>,
    pub bottom: TextureDataRef<'a>,
}

#[derive(Debug)]
pub struct WorldDebugData {
    pub room_count: u64,
}
