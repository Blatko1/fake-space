#[cfg(test)]
mod tests;

pub mod map_parser;
mod parse_error;

use std::str::FromStr;

use crate::{
    textures::{Texture, TextureData},
    voxel::VoxelModelType,
};

use self::{map_parser::MapParser, parse_error::MapParseError};

pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<MapTile>,
}

impl Map {
    pub fn from_file_str(
        data: &str,
    ) -> Result<(Self, Vec<TextureData>), MapParseError> {
        let ((w, h), tiles, textures) =
            MapParser::from_path("./maps/map1.txt")?.parse()?;
        Ok((
            Self {
                width: w,
                height: h,
                tiles,
            },
            textures,
        ))
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

/*/// Represents all tiles not including ceiling or floor tiles.
/// Additionally, contains a non-tile `Void` type.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ObjectType {
    MossyStone,
    BlueBrick,
    LightPlank,
    Fence,
    BlueGlass,
    ///// Non-transparent wall tile, possibly with modified height.
    //FullWall(FullWallType),
    ///// A wall tile which contains transparent parts or has modified height.
    //TransparentWall(TransparentWallType),
    /// Empty tile inside the map bounds.
    Empty,
}
*/
/*pub struct TestMap {
    map: Map<{ Self::WIDTH as usize }, { Self::DEPTH as usize }>,
}

impl TestMap {
    const WIDTH: u32 = TEST_MAP_WIDTH;
    const DEPTH: u32 = TEST_MAP_DEPTH;
    pub fn new() -> Self {
        Self {
            map: Map::new(
                TEST_MAP_OBJECT_DATA,
                TEST_MAP_FLOOR_DATA,
                TEST_MAP_CEILING_DATA,
            ),
        }
    }

    #[inline]
    pub fn get_tile(&self, x: usize, z: usize) -> MapTile {
        self.map.get_tile(x, z)
    }
}*/

//// A map where the player is positioned. Contains all map data.
//// The (0,0) coordinate is positioned at the bottom-left
//// and (`width`, `height`) at the top-right.
/*pub struct Map<const W: usize, const D: usize> {
    data: [[MapTile; W]; D],
}

impl<const W: usize, const D: usize> Map<W, D> {
    pub fn new(
        raw_object_data: [[u32; W]; D],
        raw_floor_data: [[u32; W]; D],
        raw_ceiling_data: [[u32; W]; D],
    ) -> Self {
        let mut data = [[MapTile::VOID; W]; D];
        // Merge three provided maps into one:
        data.iter_mut()
            .zip(raw_object_data)
            .zip(raw_floor_data)
            .zip(raw_ceiling_data)
            .for_each(
                |(
                    ((row, object_data_row), floor_data_row),
                    ceiling_data_row,
                )| {
                    row.iter_mut()
                        .zip(object_data_row)
                        .zip(floor_data_row)
                        .zip(ceiling_data_row)
                        .for_each(
                            |(
                                ((tile, object_data_tile), floor_data_tile),
                                ceiling_data_tile,
                            )| {
                                let object = ObjectType::from(object_data_tile);
                                let floor = BoundType::from(floor_data_tile);
                                let ceiling =
                                    BoundType::from(ceiling_data_tile);
                                *tile = MapTile {
                                    object,
                                    object_top: BoundType::Brick,
                                    object_bottom: BoundType::Brick,
                                    floor,
                                    ceiling,
                                    obj_top_height: f32::INFINITY,
                                    obj_bottom_height: f32::INFINITY,
                                };
                            },
                        );
                },
            );
        // Reverse map rows so the bottom row would be the starting tile at 0.
        data.reverse();
        Self { data }
    }

    /// Returns the value at the provided map coordinates.
    /// This game assumes that the y-axis points upwards, the z-axis forwards
    /// and the x-axis to the right so `x` represents moving left or right
    /// and `z` represents moving forward or backward on the map.
    /// Returns [`Tile::Void`] if coordinates are out of bounds.
    #[inline]
    pub fn get_tile(&self, x: usize, z: usize) -> MapTile {
        if let Some(row) = self.data.get(z) {
            if let Some(&tile) = row.get(x) {
                return tile;
            }
        }
        // If out of map bounds:
        MapTile::VOID
    }
}*/

// A voxel model object tile which possibly contains
// transparent/hollow parts.
//VoxelModel(VoxelModelType),

/*#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullWallType {
    BlueBrick,
    LightPlank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransparentWallType {
    Fence,
    BlueGlass,
}*/

//// Represents a floor or a ceiling or a top or a bottom part of an object.
/*#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundType {
    Empty,
    MossyStone,
    LightPlank,
    Brick,
}

impl From<u32> for ObjectType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Empty,
            1 => Self::FullWall(FullWallType::BlueBrick),
            2 => Self::FullWall(FullWallType::LightPlank),
            3 => Self::TransparentWall(TransparentWallType::Fence),
            4 => Self::TransparentWall(TransparentWallType::BlueGlass),
            5 => Self::VoxelModel(VoxelModelType::Cube),
            6 => Self::VoxelModel(VoxelModelType::CubeHole),
            7 => Self::VoxelModel(VoxelModelType::Voxel),
            8 => Self::VoxelModel(VoxelModelType::Pillars),
            9 => Self::VoxelModel(VoxelModelType::Damaged),
            _ => Self::Void,
        }
    }
}

impl From<u32> for BoundType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::MossyStone,
            1 => Self::Brick,
            _ => Self::Empty,
        }
    }
}

// TODO maybe add a number which also represents void
const TEST_MAP_WIDTH: u32 = 16;
const TEST_MAP_DEPTH: u32 = 16;
#[rustfmt::skip]
const TEST_MAP_OBJECT_DATA: [[u32; TEST_MAP_WIDTH as usize]; TEST_MAP_DEPTH as usize] = [
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1],
    [1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0],
    [1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 1, 3, 2, 3, 2, 2, 1, 1, 1, 1, 1, 0, 1],
    [1, 0, 0, 0, 3, 3, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1],
    [1, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1],
    [1, 0, 0, 0, 0, 3, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1],
    [1, 0, 0, 0, 0, 6, 0, 0, 4, 0, 0, 0, 1, 0, 0, 1],
    [1, 0, 0, 0, 0, 7, 0, 0, 2, 0, 0, 0, 1, 0, 0, 1],
    [1, 0, 0, 0, 0, 8, 0, 0, 4, 2, 0, 0, 1, 0, 0, 1],
    [1, 0, 0, 0, 0, 9, 0, 0, 2, 0, 0, 0, 1, 0, 0, 1],
    [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
];
#[rustfmt::skip]
const TEST_MAP_OBJECT_TOP_HEIGHT: [[f32; TEST_MAP_WIDTH as usize]; TEST_MAP_DEPTH as usize] = [
    [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0],
    [1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0],
    [1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0],
    [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 1.0, 3.0, 2.0, 3.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 0.0, 3.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 0.0, 0.0, 6.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 0.0, 0.0, 7.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 0.0, 0.0, 8.0, 0.0, 0.0, 4.0, 2.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 0.0, 0.0, 9.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
    [1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
];
#[rustfmt::skip]
const TEST_MAP_FLOOR_DATA: [[u32; TEST_MAP_WIDTH as usize]; TEST_MAP_DEPTH as usize] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];
#[rustfmt::skip]
const TEST_MAP_CEILING_DATA: [[u32; TEST_MAP_WIDTH as usize]; TEST_MAP_DEPTH as usize] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];
*/
