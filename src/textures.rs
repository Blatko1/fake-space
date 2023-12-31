pub struct TextureManager {
    textures: Vec<TextureData>,
}

impl TextureManager {
    pub fn new(textures: Vec<TextureData>) -> Self {
        Self { textures }
    }

    pub fn get(&self, id: Texture) -> TextureDataRef {
        match id {
            Texture::ID(id) => self.textures.get(id).unwrap().as_ref(),
            Texture::Default => TextureDataRef::DEFAULT,
            Texture::Empty => TextureDataRef::EMPTY,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Texture {
    ID(usize),
    #[default]
    Default,
    Empty,
}

pub struct TextureData {
    data: Vec<u8>,
    width: u32,
    height: u32,
    transparency: bool,

    light_shade: Vec<u8>,
    medium_shade: Vec<u8>,
}

impl TextureData {
    pub fn new(data: Vec<u8>, width: u32, height: u32, transparency: bool) -> Self {
        let mut light_shade = data.clone();
        let mut medium_shade = data.clone();

        // Generate a light shade:
        light_shade.chunks_exact_mut(4).for_each(|rgba| {
            rgba[0] = (rgba[0] as f32 * 0.85) as u8;
            rgba[1] = (rgba[1] as f32 * 0.85) as u8;
            rgba[2] = (rgba[2] as f32 * 0.85) as u8;
        });

        // Generate a heavier shade:
        medium_shade.chunks_exact_mut(4).for_each(|rgba| {
            rgba[0] = (rgba[0] as f32 * 0.65) as u8;
            rgba[1] = (rgba[1] as f32 * 0.65) as u8;
            rgba[2] = (rgba[2] as f32 * 0.65) as u8;
        });
        Self {
            data,
            width,
            height,
            transparency,
            light_shade,
            medium_shade,
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
            light_shade: &self.light_shade,
            medium_shade: &self.medium_shade,
        }
    }
}

#[derive(Debug)]
pub struct TextureDataRef<'a> {
    pub data: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub transparency: bool,

    pub light_shade: &'a [u8],
    pub medium_shade: &'a [u8],
}

impl<'a> TextureDataRef<'a> {
    const EMPTY: Self = Self {
        data: &[],
        width: 0,
        height: 0,
        transparency: true,
        light_shade: &[],
        medium_shade: &[],
    };

    const DEFAULT_TEXTURE_WIDTH: u32 = 2;
    const DEFAULT_TEXTURE_HEIGHT: u32 = 2;
    const DEFAULT_TEXTURE_RGBA: &'static [u8] = &[
        200, 0, 200, 255, 0, 0, 0, 255, 0, 0, 0, 255, 200, 0, 200, 255,
    ];
    const DEFAULT_TEXTURE_RGBA_LIGHT_SHADE: &'static [u8] = &[
        170, 0, 170, 255, 0, 0, 0, 255, 0, 0, 0, 255, 170, 0, 170, 255,
    ];
    const DEFAULT_TEXTURE_RGBA_MEDIUM_SHADE: &'static [u8] = &[
        130, 0, 130, 255, 0, 0, 0, 255, 0, 0, 0, 255, 130, 0, 130, 255,
    ];

    const DEFAULT: Self = Self {
        data: Self::DEFAULT_TEXTURE_RGBA,
        width: Self::DEFAULT_TEXTURE_WIDTH,
        height: Self::DEFAULT_TEXTURE_HEIGHT,
        transparency: false,
        light_shade: Self::DEFAULT_TEXTURE_RGBA_LIGHT_SHADE,
        medium_shade: Self::DEFAULT_TEXTURE_RGBA_MEDIUM_SHADE,
    };

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
