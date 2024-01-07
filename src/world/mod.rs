mod parser;
pub mod portal;
pub mod textures;

use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use std::path::PathBuf;

use crate::player::Player;
use crate::render::PointXZ;
use crate::world::portal::{DummyPortal, Portal, PortalID};
use parser::{error::ParseError, WorldParser};
use textures::{Texture, TextureData, TextureManager};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoomID(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct SegmentID(pub usize);

pub struct World {
    segments: Vec<Segment>,
    segment_count: usize,
    texture_manager: TextureManager,

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
            portals: segment.portals.clone(),
            is_fully_generated: true,
        };
        room_counter += 1;

        let mut adjacent_rooms: Vec<Room> = starting_room
            .portals
            .iter_mut()
            .map(|portal| {
                // The first segment appears only once at the beginning, so skip it
                let rand_segment = segments[1..].choose(&mut rng).unwrap();
                let mut new_room = Room {
                    id: RoomID(room_counter),
                    segment_id: rand_segment.id,
                    portals: rand_segment.portals.clone(),
                    is_fully_generated: false,
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

        let segment_count = segments.len();
        Self {
            segments,
            segment_count,
            texture_manager: TextureManager::new(textures),
            rng,
            rooms,
        }
    }

    pub fn update(&mut self, player: &Player) {
        let mut next_id = self.rooms.len();
        let current_room = &mut self.rooms[player.get_current_room_id().0];
        if !current_room.is_fully_generated {
            let mut adjacent_rooms: Vec<Room> = current_room
                .portals
                .iter_mut()
                .filter(|portal| portal.link.is_none())
                .map(|portal| {
                    let rand_segment = self.segments[1..].choose(&mut self.rng).unwrap();
                    let mut new_room = Room {
                        id: RoomID(next_id),
                        segment_id: rand_segment.id,
                        portals: rand_segment.portals.clone(),
                        is_fully_generated: false,
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

    fn add_new_room(&mut self, segment_id: SegmentID) -> &mut Room {
        // Append the room at the end of the list.
        let room_id = RoomID(self.rooms.len());
        let segment = &self.segments[segment_id.0];
        let starting_room = Room {
            id: room_id,
            segment_id: segment.id,
            portals: segment.portals.clone(),
            is_fully_generated: false,
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

    pub fn get_room_data(&self, index: RoomID) -> RoomDataRef {
        let room = &self.rooms[index.0];
        RoomDataRef {
            segment: &self.segments[room.segment_id.0],
            portals: &room.portals,
        }
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
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

impl<'a> RoomDataRef<'a> {
    pub fn get_portal(&self, local_id: PortalID) -> Portal {
        self.portals[local_id.0]
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
}

// TODO Use a struct or type for dimensions instead
#[derive(Debug)]
pub struct Segment {
    id: SegmentID,
    name: String,
    dimensions: (u64, u64),
    tiles: Vec<Tile>,
    portals: Vec<Portal>,
    repeatable: bool,
}

impl Segment {
    pub fn new(
        id: SegmentID,
        name: String,
        dimensions: (u64, u64),
        tiles: Vec<Tile>,
        repeatable: bool,
    ) -> Self {
        // Create unlinked Portals from DummyPortals
        let portals = tiles
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

#[derive(Debug, Clone, Copy)]
pub struct TilePosition {
    pub x: u64,
    pub z: u64,
}

// TODO try removing Clone and Copy
#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub position: TilePosition,
    /// Texture of the bottom pillar walls.
    pub bottom_pillar_tex: Texture,
    /// Texture of the top pillar walls.
    pub top_pillar_tex: Texture,
    /// Texture of the bottom platform.
    pub ground_tex: Texture,
    /// Texture of the top platform.
    pub ceiling_tex: Texture,
    /// `Y-level` - starting lower bound of the bottom pillar;
    /// level from which the bottom pillar stretches.
    pub bottom_level: f32,
    /// `Y-level` - ending upper bound of the bottom pillar;
    /// area/platform on which the player is walking.
    pub ground_level: f32,
    /// `Y-level` - starting lower bound of the top pillar; the ceiling.
    pub ceiling_level: f32,
    /// `Y-level` - ending upper bound of the top pillar;
    /// level to which the top pillar stretches.
    pub top_level: f32,
    /// If the current tile should be a portal to different segment (map).
    pub portal: Option<DummyPortal>,
}
