use std::fmt::Debug;

use crate::map::blueprint::SkyboxTextureIDs;

pub struct TextureArray {
    textures: Vec<TextureData>,
}

impl TextureArray {
    pub(super) fn new(mut textures: Vec<TextureData>) -> Self {
        let default_texture = TextureData {
            id: DEFAULT_TEXTURE_ID,
            data: DEFAULT_TEXTURE_RGBA.to_vec(),
            width: DEFAULT_TEXTURE_WIDTH,
            height: DEFAULT_TEXTURE_HEIGHT,
            transparency: DEFAULT_TEXTURE_TRANSPARENCY,
        };
        textures.insert(0, default_texture);

        Self { textures }
    }

    pub fn get_skybox_textures(&self, skybox: &SkyboxTextureIDs) -> SkyboxTexturesRef {
        SkyboxTexturesRef {
            north: self.get_texture_data(skybox.north),
            east: self.get_texture_data(skybox.east),
            south: self.get_texture_data(skybox.south),
            west: self.get_texture_data(skybox.west),
            top: self.get_texture_data(skybox.top),
            bottom: self.get_texture_data(skybox.bottom),
        }
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
        Self(0)
    }
}

pub struct TextureData {
    id: TextureID,
    data: Vec<u8>,
    width: usize,
    height: usize,
    transparency: bool,
}

impl TextureData {
    pub fn new(
        id: TextureID,
        data: Vec<u8>,
        width: usize,
        height: usize,
        transparency: bool,
    ) -> Self {
        Self {
            id,
            data,
            width,
            height,
            transparency,
        }
    }

    pub fn id(&self) -> TextureID {
        self.id
    }

    fn as_ref(&self) -> TextureDataRef {
        TextureDataRef {
            data: &self.data,
            width: self.width,
            height: self.height,
            transparency: self.transparency,
        }
    }
}

// TODO probably not needed
impl Debug for TextureData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextureData")
            .field("data size: {}", &self.data.len())
            .field(
                "color channels: {}",
                &(self.data.len() / (self.width * self.height)),
            )
            .field("width", &self.width)
            .field("height", &self.height)
            .field("transparency", &self.transparency)
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureDataRef<'a> {
    pub data: &'a [u8],
    pub width: usize,
    pub height: usize,
    pub transparency: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SkyboxTexturesRef<'a> {
    pub north: TextureDataRef<'a>,
    pub east: TextureDataRef<'a>,
    pub south: TextureDataRef<'a>,
    pub west: TextureDataRef<'a>,
    pub top: TextureDataRef<'a>,
    pub bottom: TextureDataRef<'a>,
}

const DEFAULT_TEXTURE_ID: TextureID = TextureID(0);
const DEFAULT_TEXTURE_WIDTH: usize = 2;
const DEFAULT_TEXTURE_HEIGHT: usize = 2;
const DEFAULT_TEXTURE_RGBA: [u8; 16] = [
    200, 0, 200, 255, 0, 0, 0, 255, 0, 0, 0, 255, 200, 0, 200, 255,
];
const DEFAULT_TEXTURE_TRANSPARENCY: bool = false;
