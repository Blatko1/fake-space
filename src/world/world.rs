use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

use crate::textures::{Texture, TextureData, TextureManager};
use std::path::PathBuf;

use super::parser::{error::ParseError, WorldParser};

pub type RoomIndex = usize;
pub type SegmentIndex = usize;
pub type TileIndex = usize;
pub type PortalIndex = usize;

pub struct World {
    // TODO maybe store these two into Assets struct
    // TODO remove this 'pub'
    pub segments: Vec<Segment>,
    texture_manager: TextureManager,

    rng: ThreadRng,
    //player: Player,
    pub current_room_index: RoomIndex,
    // Each room has index which is the position in this Vec
    pub rooms: Vec<Room>,
}

impl World {
    fn _new(segments: Vec<Segment>, textures: Vec<TextureData>) -> Self {
        Self {
            segments,
            texture_manager: TextureManager::new(textures),
            rng: rand::thread_rng(),
            current_room_index: 0,
            rooms: Vec::new(),
        }
    }

    pub fn new(segments: Vec<Segment>, textures: Vec<TextureData>) -> Self {
        let mut world = Self::_new(segments, textures);
        world.add_start_room();
        world
    }

    fn add_start_room(&mut self) {
        let start_seg = self.segments.first().unwrap();
        let start_room = Room {
            segment_index: 0,
            portals: start_seg.portals.clone(),
        }
    }

    pub fn from_path<P: Into<PathBuf>>(path: P) -> Result<Self, ParseError> {
        WorldParser::new(path)?.parse()
    }

    pub fn texture_manager(&self) -> &TextureManager {
        &self.texture_manager
    }

    pub fn get_current_room(&self) -> RoomRef {
        self.get_room(self.current_room_index)
    }

    pub fn get_room(&self, index: RoomIndex) -> RoomRef {
        let room = &self.rooms[index];
        let segment = &self.segments[index];
        RoomRef {
            segment_index: room.segment_index,
            segment: segment,
            portals: &room.portals,
        }
    }
}

#[derive(Debug)]
pub struct RoomRef<'a> {
    pub segment_index: SegmentIndex,
    pub segment: &'a Segment,
    pub portals: &'a [Portal],
}

// TODO remove 'pub'
#[derive(Debug)]
pub struct Room {
    //id: RoomID,
    pub segment_index: SegmentIndex,
    // Each portal has its own index which is the position in this Vec
    pub portals: Vec<Portal>,
}

#[derive(Debug)]
pub struct Segment {
    //id: SegmentIndex,
    name: String,
    dimensions: (u32, u32),
    tiles: Vec<Tile>,
    portals: Vec<Portal>,
    repeatable: bool,
}

impl Segment {
    pub fn new(
        name: String,
        dimensions: (u32, u32),
        tiles: Vec<Tile>,
        portals: Vec<Portal>,
        repeatable: bool,
    ) -> Self {
        Self {
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

#[derive(Debug, Clone, Copy)]
pub struct Portal {
    pub index: PortalIndex,
    pub direction: PortalDirection,
    pub tile_index: TileIndex,
    pub connection: Option<(RoomIndex, PortalIndex)>,
}

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
pub enum PortalDirection {
    North,
    South,
    East,
    West,
}
