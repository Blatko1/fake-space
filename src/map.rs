use crate::voxel::VoxelModelType;

pub struct TestMap {
    map: Map<{ Self::WIDTH as usize }, { Self::DEPTH as usize }>,
}

impl TestMap {
    const WIDTH: u32 = TEST_MAP_WIDTH;
    const DEPTH: u32 = TEST_MAP_DEPTH;
    pub fn new() -> Self {
        Self {
            map: Map::new(TEST_MAP_OBJECT_DATA, TEST_MAP_FLOOR_DATA, TEST_MAP_CEILING_DATA),
        }
    }

    #[inline]
    pub fn get_tile(&self, x: usize, z: usize) -> MapTile {
        self.map.get_tile(x, z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapTile {
    pub object_tile: ObjectType,
    pub floor_tile: FloorType,
    pub ceiling_tile: CeilingType,
}

impl MapTile {
    pub const VOID: Self = MapTile {
        object_tile: ObjectType::Void,
        floor_tile: FloorType::DEFAULT,
        ceiling_tile: CeilingType::DEFAULT,
    };
}

/// A map where the player is positioned. Contains all map data.
/// The (0,0) coordinate is positioned at the bottom-left
/// and (`width`, `height`) at the top-right.
pub struct Map<const W: usize, const D: usize> {
    data: [[MapTile; W]; D],
}
// TODO anti-aliasing
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
                                let object_tile = ObjectType::from(object_data_tile);
                                let floor_tile = FloorType::from(floor_data_tile);
                                let ceiling_tile = CeilingType::from(ceiling_data_tile);
                                *tile = MapTile {
                                    object_tile,
                                    floor_tile,
                                    ceiling_tile,
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
}
// TODO divide object tiles and floor_ceiling tiles into separate enums
// TODO divide the map into three different maps with same sizes, one with
// normal object tiles data, one with floor data and one with ceiling data.
// Then put all three tile data in a struct for each tile, put it in an array
// and use it as a map.

/// Represents all tiles not including ceiling or floor tiles.
/// Additionally, contains a non-tile `Void` type.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ObjectType {
    /// Non-transparent wall tile.
    FullWall(FullWallType),
    /// A wall tile which contains transparent parts.
    TransparentWall(TransparentWallType),
    /// A voxel model object tile which possibly contains
    /// transparent/hollow parts.
    VoxelModel(VoxelModelType),
    /// Empty tile inside the map bounds.
    Empty,
    /// Represents the space out of the map bounds (non-tile).
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullWallType {
    BlueBrick,
    LightPlank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransparentWallType {
    Fence,
    BlueGlass,
}

/// Represents a type of a floor tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorType {
    MossyStone,
    Brick,
}

impl FloorType {
    const DEFAULT: FloorType = FloorType::MossyStone;
}

/// Represents a type of a ceiling tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeilingType {
    LightPlank,
    Brick,
}

impl CeilingType {
    const DEFAULT: CeilingType = CeilingType::LightPlank;
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

impl From<u32> for FloorType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::MossyStone,
            1 => Self::Brick,
            _ => Self::DEFAULT,
        }
    }
}

impl From<u32> for CeilingType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::LightPlank,
            1 => Self::Brick,
            _ => Self::DEFAULT,
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
    [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
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
