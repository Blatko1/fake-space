use std::{
    fs,
    path::{Path},
};

use glam::Vec2;
use image::{EncodableLayout, ImageReader};
use tiled::{Loader, PropertyValue, TileLayer};

use crate::{
    raycaster::PointXZ,
    textures::{ TextureData, TextureID},
};

use super::{
    portal::{ Orientation, Portal, PortalID}, tilemap::{Skybox, Tile, Tilemap, TilemapID}
};

pub fn parse<P: AsRef<Path>>(path: P) -> (Vec<Tilemap>, Vec<TextureData>) {
    let texture_dir_path = path.as_ref().join("textures");
    let texture_dir =
        fs::read_dir(texture_dir_path).expect("Couldn't find 'texture' dir");
    let texture_array: Vec<(String, TextureData)> = texture_dir
        .flatten()
        .enumerate()
        .filter(|(_, texture)| texture.metadata().unwrap().is_file())
        .map(|(i, texture)| {
            let texture_name = texture.file_name().to_str().unwrap().to_owned();
            println!("name: {}", texture_name);
            let data = ImageReader::open(texture.path()).unwrap().decode().unwrap();
            (texture_name, TextureData::new(
                TextureID(i),
                data.to_rgba8().as_bytes().to_vec(),
                data.width() as usize,
                data.height() as usize,
                false,
            ))
        }).collect();

    let blueprint_dir_path = path.as_ref().join("blueprints");
    let blueprint_count = fs::read_dir(&blueprint_dir_path).expect("Couldn't find 'blueprints' dir").count();
    let blueprint_dir = fs::read_dir(blueprint_dir_path).unwrap();
    let mut blueprints = Vec::with_capacity(blueprint_count);
    for blueprint_path in blueprint_dir.flatten() {
        let blueprint_name = blueprint_path.file_name().to_str().unwrap().to_owned();
        let tmx_path = blueprint_path.path().join(format!("{}.tmx", blueprint_name));
        let tiled_data = Loader::new()
            .load_tmx_map(tmx_path)
            .unwrap();

        let map_properties = &tiled_data.properties;
        let PropertyValue::FloatValue(ambient_light) = map_properties.get("ambient_light").unwrap().to_owned() else { panic!()};
        let PropertyValue::StringValue(skybox_north_name) = map_properties.get("skybox_north").unwrap() else { panic!()};
        let PropertyValue::StringValue(skybox_east_name) = map_properties.get("skybox_east").unwrap() else { panic!()};
        let PropertyValue::StringValue(skybox_south_name) = map_properties.get("skybox_south").unwrap() else { panic!()};
        let PropertyValue::StringValue(skybox_west_name) = map_properties.get("skybox_west").unwrap() else { panic!()};
        let PropertyValue::StringValue(skybox_top_name) = map_properties.get("skybox_top").unwrap() else { panic!()};
        let PropertyValue::StringValue(skybox_bottom_name) = map_properties.get("skybox_bottom").unwrap() else { panic!()};

        let skybox_north = texture_array.iter().position(|(name, _)| name == skybox_north_name);
        let skybox_east = texture_array.iter().position(|(name, _)| name == skybox_east_name);
        let skybox_south = texture_array.iter().position(|(name, _)| name == skybox_south_name);
        let skybox_west = texture_array.iter().position(|(name, _)| name == skybox_west_name);
        let skybox_top = texture_array.iter().position(|(name, _)| name == skybox_top_name);
        let skybox_bottom = texture_array.iter().position(|(name, _)| name == skybox_bottom_name);

        // TODO find a better solution instead of idx+1 everywhere
        let default_skybox = Skybox {
            north: skybox_north.map(|idx| TextureID(idx+1)).unwrap_or_default(),
            east: skybox_east.map(|idx| TextureID(idx+1)).unwrap_or_default(),
            south: skybox_south.map(|idx| TextureID(idx+1)).unwrap_or_default(),
            west: skybox_west.map(|idx| TextureID(idx+1)).unwrap_or_default(),
            top: skybox_top.map(|idx| TextureID(idx+1)).unwrap_or_default(),
            bottom: skybox_bottom.map(|idx| TextureID(idx+1)).unwrap_or_default(),
        };

        let TileLayer::Finite(tile_layer) =
            tiled_data.get_layer(0).unwrap().as_tile_layer().unwrap()
        else {
            panic!()
        };

        let width = tile_layer.width() as i32;
        let height = tile_layer.height() as i32; 
        let mut tiles = Vec::with_capacity((width * height) as usize);
        let mut portals = Vec::new();
        for y in 0..height {
            for x in 0..width {
                // Reverse the y direction
                let tile_properties = &tile_layer.get_tile(x, height - y - 1).unwrap().get_tile().unwrap().properties;
                let PropertyValue::FloatValue(bottom_height) = tile_properties.get("bottom_height").unwrap().to_owned() else { panic!()};
                let PropertyValue::FloatValue(ground_height) = tile_properties.get("ground_height").unwrap().to_owned() else { panic!()};
                let PropertyValue::FloatValue(ceiling_height) = tile_properties.get("ceiling_height").unwrap().to_owned() else { panic!()};
                let PropertyValue::FloatValue(top_height) = tile_properties.get("top_height").unwrap().to_owned() else { panic!()};
                let PropertyValue::StringValue(bottom_texture_name) = tile_properties.get("bottom_texture").unwrap() else { panic!()};
                let PropertyValue::StringValue(ground_texture_name) = tile_properties.get("ground_texture").unwrap() else { panic!()};
                let PropertyValue::StringValue(ceiling_texture_name) = tile_properties.get("ceiling_texture").unwrap() else { panic!()};
                let PropertyValue::StringValue(top_texture_name) = tile_properties.get("top_texture").unwrap() else { panic!()};
                let PropertyValue::StringValue(portal_direction) = tile_properties.get("portal_direction").unwrap() else {panic!()};
                
                let bottom_texture = texture_array.iter().position(|(name, _)| name == bottom_texture_name);
                let ground_texture = texture_array.iter().position(|(name, _)| name == ground_texture_name);
                let ceiling_texture = texture_array.iter().position(|(name, _)| name == ceiling_texture_name);
                let top_texture = texture_array.iter().position(|(name, _)| name == top_texture_name);
                
                let position = PointXZ { x: x as u64, z: y as u64 };
                let portal_id = if !portal_direction.is_empty() {
                    let direction = match portal_direction.as_str() {
                        "N" => Vec2::Y,
                        "E" => Vec2::X,
                        "S" => Vec2::NEG_Y,
                        "W" => Vec2::NEG_X,
                        _ => panic!()
                    };
                    let id = PortalID(portals.len());
                    let portal = Portal {
                        id,
                        direction,
                        position,
                        center: Vec2::new(
                            position.x as f32 + 0.5,
                            position.z as f32 + 0.5,
                        ),
                        ground_height,
                        destination: None,
                    };
                    portals.push(portal);
                    Some(id)
                } else { None };

                let tile = Tile {
                    position,
                    bottom_wall_tex: bottom_texture.map(|idx| TextureID(idx+1)).unwrap_or_default(),
                    top_wall_tex: top_texture.map(|idx| TextureID(idx+1)).unwrap_or_default(),
                    ground_tex: ground_texture.map(|idx| TextureID(idx+1)).unwrap_or_default(),
                    ceiling_tex: ceiling_texture.map(|idx| TextureID(idx+1)).unwrap_or_default(),
                    bottom_height,
                    ground_height,
                    ceiling_height,
                    top_height,
                    portal_id,
                    object: None,
                };
                tiles.push(tile);
            }
        }

        let blueprint = Tilemap {
            id: TilemapID(blueprints.len()),
            dimensions: (width as u64, height as u64),
            tiles,
            unlinked_portals: portals,
            default_skybox,
            repeatable: false,
            default_ambient_light: ambient_light,
        };
        blueprints.push(blueprint);
    }

    let textures = texture_array.into_iter().map(|(_, texture_data)| texture_data).collect();

    (blueprints, textures)
}
