use std::path::PathBuf;
use crate::
    textures::{Texture, TextureData, TextureManager}
;

use super::parser::{WorldParser, error::ParseError};

pub struct World {
    // TODO remove this 'pub'
    pub segments: Vec<Segment>,
    texture_manager: TextureManager,
}

impl World {
    pub fn new(segments: Vec<Segment>, textures: Vec<TextureData>) -> Self {
        Self {
            segments,
            texture_manager: TextureManager::new(textures),
        }
    }

    pub fn from_path<P: Into<PathBuf>>(path: P) -> Result<Self, ParseError> {
        WorldParser::new(path)?.parse()
    }

    pub fn texture_manager(&self) -> &TextureManager {
        &self.texture_manager
    }
}

#[derive(Debug)]
pub struct Segment {
    id: String,
    dimensions: (u32, u32),
    tiles: Vec<Tile>,
    repeatable: bool,
}

impl Segment {
    pub fn new(
        id: String,
        dimensions: (u32, u32),
        tiles: Vec<Tile>,
        repeatable: bool,
    ) -> Self {
        Self {
            id,
            dimensions,
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
        // TODO do something about i32 arguments
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
}
