use rand::{rngs::ThreadRng, Rng, seq::SliceRandom};

use crate::textures::{Texture, TextureData, TextureManager};
use std::path::PathBuf;

use super::parser::{error::ParseError, WorldParser};

pub type TileIndex = usize;

#[derive(Debug, Clone, Copy)]
pub struct RoomID(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct SegmentID(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct PortalLocalID(pub usize);

pub struct World {
    // TODO maybe store these two into Assets struct
    // TODO remove this 'pub'
    pub segments: Vec<Segment>,
    segment_count: usize,
    texture_manager: TextureManager,

    rng: ThreadRng,
    //player: Player,
    pub current_room_id: RoomID,
    // Each room has index which is the position in this Vec
    pub rooms: Vec<Room>,
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
            portals: segment.portals.clone(),
        };
        room_counter += 1;

        let mut adjacent_rooms: Vec<Room> = starting_room.portals.iter_mut().map(|portal| {
            // The first segment appears only once at the beginning, so skip it
            let rand_segment = segments[1..].choose(&mut rng).unwrap();
            let mut room = Room {
                id: RoomID(room_counter),
                segment_id: rand_segment.id,
                portals: rand_segment.portals.clone(),
            };
            let room_rand_portal = room.portals.choose_mut(&mut rng).unwrap();
            
            // Connect the two portals:
            // Connect the starting room with new random room
            portal.connection = Some((room.id, room_rand_portal.id));
            // Connect the new random room with the starting room
            room_rand_portal.connection = Some((starting_room.id, portal.id));
            room_counter += 1;

            room
        }).collect();
        rooms.push(starting_room);
        rooms.append(&mut adjacent_rooms);
        for r in rooms.iter() {
            println!("r: {:?}, portals: {:?}, segment: {:?}", r.id, r.portals, r.segment_id);
        }

        let segment_count = segments.len();
        Self {
            segments,
            segment_count,
            texture_manager: TextureManager::new(textures),
            rng,
            current_room_id: RoomID(0),
            rooms,
        }
    }

    fn add_new_room(&mut self, segment_id: SegmentID) -> &mut Room {
        // Append the room at the end of the list.
        let room_id = RoomID(self.rooms.len());
        let segment = &self.segments[segment_id.0];
        let mut starting_room = Room {
            id: room_id,
            segment_id: segment.id,
            portals: segment.portals.clone(),
        };
        self.rooms.push(starting_room);
        self.rooms.last_mut().unwrap()
    }

    fn add_new_random_room(&mut self) -> &mut Room {
        let rand_segment_id = self.get_random_segment_id();
        self.add_new_room(rand_segment_id)
    }

    fn get_random_segment_id(&mut self) -> SegmentID {
        // The first segment repeats only once at the beginning
        SegmentID(self.rng.gen_range(1..self.segment_count))
    }

    pub fn get_current_room_data(&self) -> RoomDataRef {
        self.get_room_data(self.current_room_id)
    }

    pub fn get_room_data(&self, index: RoomID) -> RoomDataRef {
        let room = &self.rooms[index.0];
        RoomDataRef {
            segment: &self.segments[room.segment_id.0],
            portals: &room.portals,
        }
    }

    pub fn texture_manager(&self) -> &TextureManager {
        &self.texture_manager
    }
}

#[derive(Debug)]
pub struct RoomDataRef<'a> {
    pub segment: &'a Segment,
    pub portals: &'a [Portal],
}

// TODO remove 'pub'
#[derive(Debug)]
pub struct Room {
    id: RoomID,
    pub segment_id: SegmentID,
    // Each portal has its own index which is the position in this Vec
    pub portals: Vec<Portal>,
}

#[derive(Debug)]
pub struct Segment {
    id: SegmentID,
    name: String,
    dimensions: (u32, u32),
    tiles: Vec<Tile>,
    portals: Vec<Portal>,
    repeatable: bool,
}

impl Segment {
    pub fn new(
        id: SegmentID,
        name: String,
        dimensions: (u32, u32),
        tiles: Vec<Tile>,
        portals: Vec<Portal>,
        repeatable: bool,
    ) -> Self {
        Self {
            id,
            name,
            dimensions,
            portals,
            tiles,
            repeatable,
        }
    }

    /// Returns the value at the provided map coordinates.
    /// Parsed arguments are assumed to be in map bound and correct.
    /// This game assumes that the y-axis points upwards, the z-axis forwards
    /// and the x-axis to the right so `x` represents moving left or right
    /// and `z` represents moving forward or backward on the map.
    /// Returns [`Tile::Void`] if coordinates are out of bounds.
    #[inline]
    pub fn get_tile(&self, x: i32, z: i32) -> Option<&Tile> {
        // TODO do something about i32 arguments and 'if' conditions
        if x >= self.dimensions.0 as i32
            || x < 0
            || z >= self.dimensions.0 as i32
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
    /// Texture of the lower pillar walls.
    pub pillar1_tex: Texture,
    /// Texture of the upper pillar walls.
    pub pillar2_tex: Texture,
    /// Texture of the bottom platform.
    pub bottom_platform_tex: Texture,
    /// Texture of the top platform.
    pub top_platform_tex: Texture,
    /// `Y-level` - starting lower bound of the bottom pillar;
    /// level from which the bottom pillar stretches.
    pub level1: f32,
    /// `Y-level` - ending upper bound of the bottom pillar;
    /// area/platform on which the player is walking.
    pub level2: f32,
    /// `Y-level` - starting lower bound of the top pillar; the ceiling.
    pub level3: f32,
    /// `Y-level` - ending upper bound of the top pillar;
    /// level to which the top pillar stretches.
    pub level4: f32,
    /// If the current tile should be a portal to different segment (map).
    pub portal: Option<Portal>,
}

#[derive(Debug, Clone, Copy)]
pub struct Portal {
    pub id: PortalLocalID,
    pub direction: PortalDirection,
    pub tile_index: TileIndex,
    pub connection: Option<(RoomID, PortalLocalID)>,
}

#[derive(Debug, Clone, Copy)]
pub enum PortalDirection {
    North,
    South,
    East,
    West,
}
