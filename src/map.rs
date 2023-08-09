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
    /// Returns [`Tile::Void`] if coordinates are out of bounds.
    #[inline]
    pub fn get_value(&self, x: i32, y: i32) -> Tile {
        if y < 0 || y >= self.height as i32 || x < 0 || x >= self.width as i32 {
            return Tile::Void;
        }
        let index = (self.height as i32 - 1 - y) as usize * self.width as usize
            + x as usize;

        *self.data.get(index).unwrap_or(&Tile::Void)
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width as u32
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    /// Empty walkable tile.
    Empty,
    /// Non-walkable tile.
    Wall,
    /// Represents all tiles which have transparent parts.
    Transparent,
    /// Represents space out of map (non-tile).
    Void,
}

impl From<u32> for Tile {
    fn from(value: u32) -> Self {
        match value {
            0 => Tile::Empty,
            1 => Tile::Wall,
            2 => Tile::Transparent,
            _ => Tile::Void,
        }
    }
}

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
    1, 0, 0, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1,
    1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];
