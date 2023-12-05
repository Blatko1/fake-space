use std::path::PathBuf;

use crate::{
    map_parser::{parse_error::MapParseError, MapParser},
    textures::{Texture, TextureManager},
};

// TODO maybe the Map struct should store the TextureManager
// because all textures are bound to a their own map.
pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<MapTile>,
    texture_manager: TextureManager,
}

impl Map {
    pub fn from_path<P: Into<PathBuf>>(path: P) -> Result<Self, MapParseError> {
        let ((w, h), tiles, textures) = MapParser::from_path(path)?.parse()?;
        let texture_manager = TextureManager::new(textures);
        Ok(Self {
            width: w,
            height: h,
            tiles,
            texture_manager,
        })
    }

    /// Returns the value at the provided map coordinates.
    /// Parsed arguments are assumed to be in map bound and correct.
    /// This game assumes that the y-axis points upwards, the z-axis forwards
    /// and the x-axis to the right so `x` represents moving left or right
    /// and `z` represents moving forward or backward on the map.
    /// Returns [`Tile::Void`] if coordinates are out of bounds.
    #[inline]
    pub fn get_tile(&self, x: i32, z: i32) -> Option<&MapTile> {
        // TODO do something about i32 arguments
        if x >= self.width as i32 || x < 0 || z >= self.height as i32 || z < 0 {
            return None;
        }
        self.tiles.get(z as usize * self.width + x as usize)
    }

    pub fn texture_manager(&self) -> &TextureManager {
        &self.texture_manager
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapTile {
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
