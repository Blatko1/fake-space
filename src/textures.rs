use crate::{map::{Tile, TopBottom, TransparentWall, WallTile}, world::EntityTexture};

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
            TextureData::from_data(blue_brick_data, 1.0, 1.0).unwrap();

        let fence_data = include_bytes!("../res/fence.png");
        let fence = TextureData::from_data(fence_data, -0.3125, 1.0).unwrap();

        let blue_glass_data = include_bytes!("../res/blue_glass.png");
        let blue_glass =
            TextureData::from_data(blue_glass_data, 1.0, 1.0).unwrap();

        let light_plank_data = include_bytes!("../res/light_plank.png");
        let light_plank =
            TextureData::from_data(light_plank_data, 1.0, 1.0).unwrap();

        let mossy_stone_data = include_bytes!("../res/mossy_stone.png");
        let mossy_stone =
            TextureData::from_data(mossy_stone_data, 1.0, 1.0).unwrap();

        Self {
            blue_brick,
            fence,
            blue_glass,
            light_plank,
            mossy_stone,
        }
    }

    #[inline]
    pub fn get_wall_tex(&self, tile: WallTile) -> TextureDataRef {
        match tile {
            WallTile::BlueBrick => self.blue_brick.as_ref(),
            WallTile::LightPlank => self.light_plank.as_ref(),
        }
    }

    #[inline]
    pub fn get_transparent_tex(&self, tile: TransparentWall) -> TextureDataRef {
        match tile {
            TransparentWall::Fence => self.fence.as_ref(),
            TransparentWall::BlueGlass => self.blue_glass.as_ref(),
        }
    }

    #[inline]
    pub fn get_floor_tex(&self, tile: Tile) -> TextureDataRef {
        match tile {
            Tile::TopBottom(top_bottom_tile) => match top_bottom_tile {
                TopBottom::TopAndBottomBrick => self.blue_brick.as_ref(),
            },
            _ => self.mossy_stone.as_ref(),
        }
    }

    #[inline]
    pub fn get_ceiling_tex(&self, tile: Tile) -> TextureDataRef {
        match tile {
            Tile::TopBottom(top_bottom_tile) => match top_bottom_tile {
                TopBottom::TopAndBottomBrick => self.blue_brick.as_ref(),
            },
            _ => self.light_plank.as_ref(),
        }
    }

    #[inline]
    pub fn get_entity_texture(&self, tex: EntityTexture) -> TextureDataRef {
        match tex {
           EntityTexture::Glass => self.blue_glass.as_ref(),
        }
    }
}

#[derive(Debug)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub top_height: f32,
    pub bottom_height: f32,
    pub texture: Vec<u8>,
    pub texture_darkened: Vec<u8>,
}

impl TextureData {
    fn from_data(
        data: &[u8],
        top_height: f32,
        bottom_height: f32,
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
    pub texture: &'a [u8],
    pub texture_darkened: &'a [u8],
}
