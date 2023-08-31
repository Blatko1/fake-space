use crate::object::ObjectType;

/// A map where the player is positioned. Contains all map data.
/// The (0,0) coordinate is positioned at the bottom-left
/// and (`width`, `height`) at the top-right.
pub struct Map {
    width: u32,
    height: u32,
    data: Vec<Tile>,
}
// TODO anti-aliasing
impl Map {
    pub fn new(width: u32, height: u32, data: &[u32]) -> Self {
        assert_eq!(
            width * height,
            data.len() as u32,
            "Provided map dimensions {}x{} do not match with the map data size",
            width,
            height
        );

        let data = data.iter().map(|&v| Tile::from(v)).collect();

        Self {
            width,
            height,
            data,
        }
    }

    pub fn new_test() -> Self {
        Self::new(TEST_MAP_WIDTH, TEST_MAP_HEIGHT, TEST_MAP_DATA)
    }

    ///Returns the value at the provided map coordinates.
    /// This game assumes that the y-axis points upwards, the z-axis forwards
    /// and the x-axis to the right so `x` represents moving left or right
    /// and `z` represents moving forward or backward on the map.
    /// Returns [`Tile::Void`] if coordinates are out of bounds.
    #[inline]
    pub fn get_value(&self, x: i32, z: i32) -> Tile {
        if z < 0 || z >= self.height as i32 || x < 0 || x >= self.width as i32 {
            return Tile::Void;
        }
        let index = (self.height as i32 - 1 - z) as usize * self.width as usize
            + x as usize;

        self.data.get(index).unwrap_or(&Tile::Void).clone()
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Tile {
    /// Empty walkable tile.
    Empty,
    /// Non-walkable, non-transparent wall tile.
    Wall(WallTexture),
    /// Represents all tiles which have transparent parts.
    Transparent(TransparentTexture),
    /// Represents a tile which contains a voxel model and possible a Wall or a Transparent tile.
    Object(ObjectType),
    /// Represents space out of map (non-tile).
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallTexture {
    BlueBrick,
    LightPlank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransparentTexture {
    Fence,
    BlueGlass,
}

impl From<u32> for Tile {
    fn from(value: u32) -> Self {
        match value {
            0 => Tile::Empty,
            1 => Tile::Wall(WallTexture::BlueBrick),
            2 => Tile::Wall(WallTexture::LightPlank),
            3 => Tile::Transparent(TransparentTexture::Fence),
            4 => Tile::Transparent(TransparentTexture::BlueGlass),
            5 => Tile::Object(ObjectType::Cube),
            6 => Tile::Object(ObjectType::Hole),
            7 => Tile::Object(ObjectType::Voxel),
            _ => Tile::Void,
        }
    }
}

// TODO maybe add a number which also represents void
const TEST_MAP_WIDTH: u32 = 16;
const TEST_MAP_HEIGHT: u32 = 16;
#[rustfmt::skip]
const TEST_MAP_DATA: &[u32; (TEST_MAP_WIDTH * TEST_MAP_HEIGHT) as usize] = &[
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1,
    1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0,
    1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 1, 3, 2, 3, 2, 2, 1, 1, 1, 1, 1, 0, 1,
    1, 0, 0, 0, 3, 3, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 3, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 3, 0, 0, 4, 0, 0, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 4, 2, 0, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 6, 0, 0, 2, 0, 0, 0, 1, 0, 0, 1,
    1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];
