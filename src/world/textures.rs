const DEFAULT_TEXTURE_WIDTH: u32 = 2;
const DEFAULT_TEXTURE_HEIGHT: u32 = 2;
const DEFAULT_TEXTURE_RGBA: [u8; 16] = [
    200, 0, 200, 255, 0, 0, 0, 255, 0, 0, 0, 255, 200, 0, 200, 255,
];
const DEFAULT_TEXTURE_TRANSPARENCY: bool = false;

pub struct TextureManager {
    textures: Vec<TextureData>,
}

impl TextureManager {
    pub(super) fn new(mut textures: Vec<TextureData>) -> Self {
        let default_texture = TextureData {
            data: DEFAULT_TEXTURE_RGBA.to_vec(),
            width: DEFAULT_TEXTURE_WIDTH,
            height: DEFAULT_TEXTURE_HEIGHT,
            transparency: DEFAULT_TEXTURE_TRANSPARENCY,
        };
        textures.insert(0, default_texture);

        let empty_texture = TextureData {
            data: Vec::new(),
            width: 0,
            height: 0,
            transparency: true,
        };
        textures.insert(0, empty_texture);
        Self { textures }
    }

    pub(super) fn get_texture_data(&self, id: TextureID) -> TextureDataRef {
        self.textures[id.0].as_ref()
    }
}

// TODO maybe rename to TextureID
#[derive(Debug, Clone, Copy)]
pub struct TextureID(pub usize);

impl Default for TextureID {
    fn default() -> Self {
        Self(1)
    }
}

pub struct TextureData {
    data: Vec<u8>,
    width: u32,
    height: u32,
    transparency: bool,
}

impl TextureData {
    pub fn new(data: Vec<u8>, width: u32, height: u32, transparency: bool) -> Self {
        Self {
            data,
            width,
            height,
            transparency,
        }
    }
}

impl TextureData {
    fn as_ref(&self) -> TextureDataRef {
        TextureDataRef {
            data: &self.data,
            width: self.width,
            height: self.height,
            transparency: self.transparency,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureDataRef<'a> {
    pub data: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub transparency: bool,
}

impl<'a> TextureDataRef<'a> {
    // TODO probably unneeded
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
