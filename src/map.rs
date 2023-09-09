use crate::object::ObjectType;

pub struct TestMap {
    map: Map<{Self::WIDTH as usize}, {Self::DEPTH as usize}>
}

impl TestMap {
    const WIDTH: u32 = TEST_MAP_WIDTH;
    const DEPTH: u32 = TEST_MAP_DEPTH;
    pub fn new() -> Self {
        Self {
            map: Map::new(TEST_MAP_DATA)
        }
    }

    #[inline]
    pub fn get_tile(&self, x: usize, z: usize) -> Tile {
        self.map.get_tile(x, z)
    }
}

/// A map where the player is positioned. Contains all map data.
/// The (0,0) coordinate is positioned at the bottom-left
/// and (`width`, `height`) at the top-right.
pub struct Map<const W: usize, const D: usize> {
    data: [[Tile; W]; D],
}
// TODO anti-aliasing
impl<const W: usize, const D: usize> Map<W, D> {
    pub fn new(raw_data: [[u32; W]; D]) -> Self {
        let mut data = [[Tile::Empty; W]; D];
        data.iter_mut().zip(raw_data).for_each(|(row, raw_row)| {
            row.iter_mut().zip(raw_row).for_each(|(tile, raw_tile)| *tile = Tile::from(raw_tile))
        });
        data.reverse();
        Self {
            data,
        }
    }

    ///Returns the value at the provided map coordinates.
    /// This game assumes that the y-axis points upwards, the z-axis forwards
    /// and the x-axis to the right so `x` represents moving left or right
    /// and `z` represents moving forward or backward on the map.
    /// Returns [`Tile::Void`] if coordinates are out of bounds.
    /*#[inline]
    pub fn get_value(&self, x: i32, z: i32) -> Tile {
        if z < 0 || z >= self.height as i32 || x < 0 || x >= self.width as i32 {
            return Tile::Void;
        }
        let index = (self.height as i32 - 1 - z) as usize * self.width as usize
            + x as usize;

        *self.data.get(index).unwrap_or(&Tile::Void)
    }*/
    #[inline]
    pub fn get_tile(&self, x: usize, z: usize) -> Tile {
        if let Some(row) = self.data.get(z) {
            return *row.get(x).unwrap_or(&Tile::Void)
        }
        return Tile::Void
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Tile {
    /// Empty walkable tile in map bounds.
    Empty,
    /// Non-walkable, non-transparent wall tile.
    Wall(WallTile),
    /// Represents all tiles which can have transparent parts.
    Transparent(TransparentTile),
    /// Represents tiles which have different roof and/or floor textures.
    TopBottom(TopBottom),
    /// Represents the space out of the map (non-tile).
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallTile {
    BlueBrick,
    LightPlank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransparentTile {
    /// Represents a wall tile with a partly transparent texture.
    TransparentWall(TransparentWall),
    /// Represents a tile which contains a voxel model.
    Object(ObjectType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransparentWall {
    Fence,
    BlueGlass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopBottom {
    Brick,
}

impl From<u32> for Tile {
    fn from(value: u32) -> Self {
        match value {
            0 => Tile::Empty,
            1 => Tile::Wall(WallTile::BlueBrick),
            2 => Tile::Wall(WallTile::LightPlank),
            3 => Tile::Transparent(TransparentTile::TransparentWall(
                TransparentWall::Fence,
            )),
            4 => Tile::Transparent(TransparentTile::TransparentWall(
                TransparentWall::BlueGlass,
            )),
            5 => Tile::Transparent(TransparentTile::Object(ObjectType::Cube)),
            6 => Tile::Transparent(TransparentTile::Object(ObjectType::Hole)),
            7 => Tile::Transparent(TransparentTile::Object(ObjectType::Voxel)),
            8 => {
                Tile::Transparent(TransparentTile::Object(ObjectType::Pillars))
            }
            9 => {
                Tile::Transparent(TransparentTile::Object(ObjectType::Damaged))
            }
            10 => Tile::TopBottom(TopBottom::Brick),
            _ => Tile::Void,
        }
    }
}

// TODO maybe add a number which also represents void
const TEST_MAP_WIDTH: u32 = 16;
const TEST_MAP_DEPTH: u32 = 16;
#[rustfmt::skip]
const TEST_MAP_DATA: [[u32; TEST_MAP_WIDTH as usize]; TEST_MAP_DEPTH as usize] = [
    [1, 1,  1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1],
    [1, 0,  1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1],
    [1, 0,  0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
    [1, 0,  0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0],
    [1, 0,  1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1],
    [1, 0,  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0,  0, 1, 3, 2, 3, 2, 2, 1, 1, 1, 1, 1, 0, 1],
    [1, 0,  0, 0, 3, 3, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1],
    [1, 0,  0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1],
    [1, 0, 10, 0, 0, 3, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1],
    [1, 0, 10, 0, 0, 6, 0, 0, 4, 0, 0, 0, 1, 0, 0, 1],
    [1, 0, 10, 0, 0, 7, 0, 0, 2, 0, 0, 0, 1, 0, 0, 1],
    [1, 0,  0, 0, 0, 8, 0, 0, 4, 2, 0, 0, 1, 0, 0, 1],
    [1, 0,  0, 0, 0, 9, 0, 0, 2, 0, 0, 0, 1, 0, 0, 1],
    [1, 1,  1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1],
    [1, 1,  1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
];
