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

#[derive(Debug)]
pub struct TextureData {
    data: Vec<u8>,
    width: u32,
    height: u32,
    transparency: bool,
    repeating: bool,

    light_shade: Vec<u8>,
    medium_shade: Vec<u8>,
}

impl TextureData {
    pub fn new(
        data: Vec<u8>,
        width: u32,
        height: u32,
        transparency: bool,
        repeating: bool,
    ) -> Self {
        let mut light_shade = data.clone();
        let mut medium_shade = data.clone();

        light_shade.chunks_exact_mut(4).for_each(|rgba| {
            rgba[0] = (rgba[0] as f32 * 0.85) as u8;
            rgba[1] = (rgba[1] as f32 * 0.85) as u8;
            rgba[2] = (rgba[2] as f32 * 0.85) as u8;
        });
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
            repeating,
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
            repeating: self.repeating,
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
    pub repeating: bool,

    pub light_shade: &'a [u8],
    pub medium_shade: &'a [u8],
}

impl<'a> TextureDataRef<'a> {
    const EMPTY: Self = Self {
        data: &[],
        width: 0,
        height: 0,
        transparency: true,
        repeating: false,
        light_shade: &[],
        medium_shade: &[],
    };

    const DEFAULT_TEXTURE_WIDTH: u32 = 2;
    const DEFAULT_TEXTURE_HEIGHT: u32 = 2;
    const DEFAULT_TEXTURE_RGBA: &[u8] = &[
        200, 0, 200, 255, 0, 0, 0, 255, 0, 0, 0, 255, 200, 0, 200, 255,
    ];
    const DEFAULT_TEXTURE_RGBA_LIGHT_SHADE: &[u8] = &[
        170, 0, 170, 255, 0, 0, 0, 255, 0, 0, 0, 255, 170, 0, 170, 255,
    ];
    const DEFAULT_TEXTURE_RGBA_MEDIUM_SHADE: &[u8] = &[
        130, 0, 130, 255, 0, 0, 0, 255, 0, 0, 0, 255, 130, 0, 130, 255,
    ];

    const DEFAULT: Self = Self {
        data: Self::DEFAULT_TEXTURE_RGBA,
        width: Self::DEFAULT_TEXTURE_WIDTH,
        height: Self::DEFAULT_TEXTURE_HEIGHT,
        transparency: false,
        repeating: false,
        light_shade: Self::DEFAULT_TEXTURE_RGBA_LIGHT_SHADE,
        medium_shade: Self::DEFAULT_TEXTURE_RGBA_MEDIUM_SHADE,
    };

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/*use crate::map::{BoundType, FullWallType, TransparentWallType};

pub struct TextureManager {
    blue_brick: TextureData,
    fence: TextureData,
    blue_glass: TextureData,
    light_plank: TextureData,
    mossy_stone: TextureData,
}

impl TextureManager {
    pub fn init() -> Self {
        let blue_brick_data = include_bytes!("../res/blue_brick.png");
        let blue_brick =
            TextureData::from_data(blue_brick_data, 1.0, 1.0, false).unwrap();

        let fence_data = include_bytes!("../res/fence.png");
        let fence =
            TextureData::from_data(fence_data, -0.3125, 1.0, true).unwrap();

        let blue_glass_data = include_bytes!("../res/blue_glass.png");
        let blue_glass =
            TextureData::from_data(blue_glass_data, 1.0, 1.0, true).unwrap();

        let light_plank_data = include_bytes!("../res/light_plank.png");
        let light_plank =
            TextureData::from_data(light_plank_data, 1.0, 1.0, false).unwrap();

        let mossy_stone_data = include_bytes!("../res/mossy_stone.png");
        let mossy_stone =
            TextureData::from_data(mossy_stone_data, 1.0, 1.0, false).unwrap();

        Self {
            blue_brick,
            fence,
            blue_glass,
            light_plank,
            mossy_stone,
        }
    }

    #[inline]
    pub fn get_full_wall_tex(&self, tile: FullWallType) -> TextureDataRef {
        match tile {
            FullWallType::BlueBrick => self.blue_brick.as_ref(),
            FullWallType::LightPlank => self.light_plank.as_ref(),
        }
    }

    #[inline]
    pub fn get_transparent_wall_tex(
        &self,
        tile: TransparentWallType,
    ) -> TextureDataRef {
        match tile {
            TransparentWallType::Fence => self.fence.as_ref(),
            TransparentWallType::BlueGlass => self.blue_glass.as_ref(),
        }
    }

    #[inline]
    pub fn get_bound_tex(&self, tile: BoundType) -> TextureDataRef {
        match tile {
            BoundType::MossyStone => self.mossy_stone.as_ref(),
            BoundType::Brick => self.blue_brick.as_ref(),
            BoundType::Empty => todo!(),
            BoundType::LightPlank => self.light_plank.as_ref(),
        }
    }
}

#[derive(Debug)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub top_height: f32,
    pub bottom_height: f32,
    has_transparency: bool,
    pub texture: Vec<u8>,
    pub texture_darkened: Vec<u8>,
}

impl TextureData {
    fn from_data(
        data: &[u8],
        top_height: f32,
        bottom_height: f32,
        has_transparency: bool,
    ) -> Option<Self> {
        if top_height < 0.0 {
            assert!(
                -top_height < bottom_height,
                "The `top height` ({}) goes further down \
                than the `bottom height` ({}).",
                top_height,
                bottom_height
            );
        }
        if bottom_height < 0.0 {
            assert!(
                -bottom_height < top_height,
                "The `bottom height` ({}) goes further down \
                than the `top height` ({}).",
                bottom_height,
                top_height
            );
        }
        let blue_brick_img = image::load_from_memory(data).unwrap();
        let texture = blue_brick_img.to_rgba8().to_vec();
        let mut texture_darkened = texture.clone();
        texture_darkened.chunks_mut(4).for_each(|rgba| {
            rgba[0] = rgba[0].saturating_sub(15);
            rgba[1] = rgba[1].saturating_sub(15);
            rgba[2] = rgba[2].saturating_sub(15);
        });
        Some(Self {
            width: blue_brick_img.width(),
            height: blue_brick_img.height(),
            top_height,
            bottom_height,
            has_transparency,
            texture,
            texture_darkened,
        })
    }

    #[inline]
    fn as_ref(&self) -> TextureDataRef {
        TextureDataRef {
            width: self.width,
            height: self.height,
            top_height: self.top_height,
            bottom_height: self.bottom_height,
            has_transparency: self.has_transparency,
            texture: self.texture.as_slice(),
            texture_darkened: self.texture_darkened.as_slice(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureDataRef<'a> {
    pub width: u32,
    pub height: u32,
    pub top_height: f32,
    pub bottom_height: f32,
    pub has_transparency: bool,
    pub texture: &'a [u8],
    pub texture_darkened: &'a [u8],
}

impl<'a> Default for TextureDataRef<'a> {
    fn default() -> Self {
        Self {
            width: 4,
            height: 4,
            top_height: 1.0,
            bottom_height: 1.0,
            has_transparency: false,
            texture: &[0, 0, 0, 255],
            texture_darkened: &[0, 0, 0, 255],
        }
    }
}
*/
