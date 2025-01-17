use std::{path::{Path, PathBuf}, process::id};

use hashbrown::HashMap;
use image::{EncodableLayout, ImageReader};
use tiled::{Loader, PropertyValue, TileLayer};

use crate::{raycaster::PointXZ, textures::{TextureData, TextureID}};

use super::{blueprint::{Blueprint, BlueprintID, SkyboxTextureIDs, Tile}, portal::{DummyPortal, Orientation, PortalID}};

pub struct MapParser {
    parent_path: PathBuf,
    textures: HashMap<String, TextureData>,
    texture_count: usize,
}

impl MapParser {

    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            parent_path: path.as_ref().canonicalize().unwrap(),
            textures: HashMap::new(),
            texture_count: 0
        }
    }

pub fn parse(mut self) -> (Blueprint, Vec<TextureData>) {
    let map = Loader::new().load_tmx_map(self.parent_path.clone()).unwrap();

    let TileLayer::Finite(tile_layer) = map.get_layer(0).unwrap().as_tile_layer().unwrap() else {
        panic!()
    };

    let mut portal_count = 0;
    let mut tiles = Vec::new();
    for y in 0..tile_layer.height() as i32 {
        for x in 0..tile_layer.width() as i32 {
            let tile = tile_layer.get_tile(x, y).unwrap();
            let properties = &tile.get_tile().unwrap().properties;
            let PropertyValue::FloatValue(bottom_height) = *properties.get("bottom_height").unwrap() else {
                panic!()
            };
            let PropertyValue::FloatValue(ground_height) = *properties.get("ground_height").unwrap() else {
                panic!()
            };
            let PropertyValue::FloatValue(ceiling_height) = *properties.get("ceiling_height").unwrap() else {
                panic!()
            };
            let PropertyValue::FloatValue(top_height) = *properties.get("top_height").unwrap() else {
                panic!()
            };
            
            let PropertyValue::FileValue(bottom_texture_path) = properties.get("bottom_texture").unwrap() else {
                panic!()
            };
            let bottom_wall_tex = self.parse_texture(bottom_texture_path.to_owned());
            let PropertyValue::FileValue(ground_texture_path) = properties.get("ground_texture").unwrap() else {
                panic!()
            };
            let ground_tex = self.parse_texture(ground_texture_path.to_owned());
            let PropertyValue::FileValue(ceiling_texture_path) = properties.get("ceiling_texture").unwrap() else {
                panic!()
            };
            let ceiling_tex = self.parse_texture(ceiling_texture_path.to_owned());
            let PropertyValue::FileValue(top_texture_path) = properties.get("top_texture").unwrap() else {
                panic!()
            };
            let top_wall_tex = self.parse_texture(top_texture_path.to_owned());

            let portal = if let Some(portal_direction) = properties.get("portal_direction") {
                let PropertyValue::StringValue(portal_direction) = portal_direction else {
                    panic!()
                };
                if !portal_direction.is_empty() {
                    let orientation = match portal_direction.as_str() {
                        "N" => Orientation::North,
                        "E" => Orientation::East,
                        "S" => Orientation::South,
                        "W" => Orientation::West,
                        _ => panic!()
                    };
                    let portal = DummyPortal {
                        id: PortalID(portal_count),
                        orientation,
                    };
                    portal_count += 1;
                    Some(portal)
                } else {
                    None
                }
            } else {
                None
            };


            let tile = Tile {
                position: PointXZ::new(x as u64, y as u64),
                bottom_wall_tex,
                top_wall_tex,
                ground_tex,
                ceiling_tex,
                bottom_height,
                ground_height,
                ceiling_height,
                top_height,
                portal,
                object: None,
            };
            tiles.push(tile);
        }
        
    }
    let skybox = SkyboxTextureIDs {
        north: TextureID::default(),
        east: TextureID::default(),
        south: TextureID::default(),
        west: TextureID::default(),
        top: TextureID::default(),
        bottom: TextureID::default(),
    };(Blueprint::new(BlueprintID(0), (map.width as u64, map.height as u64), tiles, skybox, false, 1.0), self.textures.drain().map(|(_, v)| v).collect())
    
}

fn parse_texture(&mut self, texture_path: String) -> TextureID {
    if !texture_path.is_empty() {
        if let Some(tex) = self.textures.get(&texture_path) {
            tex.id()
        } else {
            println!("path: {:?}, tex_path: {}", self.parent_path, texture_path);
            let data = ImageReader::open(self.parent_path.join("..").join(&texture_path)).unwrap().decode().unwrap();
            let id = TextureID(self.texture_count);
            self.texture_count += 1;

            let texture = TextureData::new(
                id,
                data.to_rgba8().as_bytes().to_vec(),
                data.width() as usize,
                data.height() as usize,
                false,
            );
            self.textures.insert(texture_path, texture);

            id
        }
    } else {
        TextureID::default()
    }
}

}