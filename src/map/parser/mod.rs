pub mod utils;
mod blueprint;

use std::path::PathBuf;

use blueprint::BlueprintParser;
use hashbrown::HashMap;
use image::{EncodableLayout, ImageReader};
use nom::{character::complete::{alphanumeric1, char}, combinator::{eof, fail, opt}, complete::bool, error::{context, convert_error, VerboseError}, multi::{separated_list1}, number::complete::float, sequence::Tuple, Finish, IResult};
use utils::{boolean, clean_input, comma_separator, expression_key, name_and_expression, path, separate_expressions, space, };

use crate::{models::{ModelData, ModelID}, textures::{TextureData, TextureID}};

use super::blueprint::{Blueprint, BlueprintID, SkyboxTextureIDs};

/// Takes as argument: (Option<&str>, &HashMap<String, TextureID>, target)
#[macro_export]
macro_rules! update_if_texture_exists {
    ($texture_name_opt:expr, $texture_map:expr, $target:expr) => {
        if let Some(tex_name) = $texture_name_opt {
            let Some(texture_id) = $texture_map.get(tex_name) else {
                return context("Texture doesn't exist", fail)(tex_name);
            };
            $target = *texture_id;
        }
    };
}

pub struct MapParser {
    parent_path: PathBuf,

    settings: MapSettings,
    textures: Vec<TextureData>,
    texture_map: HashMap<String, TextureID>,
    texture_count: usize,
    models: Vec<ModelData>,
    model_map: HashMap<String, ModelID>,
    model_count: usize,
    blueprints: Vec<Blueprint>,
}

impl<'a> MapParser {
    pub fn new<P: Into<PathBuf>>(
        parent_path: P,
    ) -> Self {
        Self {
            parent_path: parent_path.into(),

            settings: MapSettings::default(),
            textures: Vec::new(),
            texture_map: HashMap::new(),
            texture_count: 2,
            models: Vec::new(),
            model_map: HashMap::new(),
            model_count: 0,
            blueprints: Vec::new(),
        }
    }

    pub fn parse(
        mut self,
        input: &'a str,
    ) -> IResult<
        &'a str,
        (Vec<Blueprint>, Vec<TextureData>, Vec<ModelData>),
        VerboseError<&'a str>,
    > {
        let (_, expressions) = separate_expressions(input)?;
        for expr in expressions {
            let (expr, key) = expression_key(expr)?;
            match key {
                "#" => {
                    self.parse_texture(expr)?;
                }
                /*"~" => {
                    let (_, (model_name, model)) =
                        parse_vox_file(i, self.parent_path.clone())?;
                    if self.model_map.contains_key(&model_name) {
                        return context("Model with same name already exists", fail)(i);
                    }
                    self.models.push(model);
                    self.model_map.insert(model_name, ModelID(self.model_count));
                    self.model_count += 1;
                }*/
                "*" => {
                    self.parse_setting(expr)?;
                }
                "!" => {
                    
                        self.parse_blueprint(expr)?;
                }
                _ => return context("Unknown line key", fail)(key),
            };
        }
        Ok(("", (self.blueprints, self.textures, self.models)))
    }

    fn parse_texture(
        & mut self,
        input: &'a str,
    ) -> IResult<&'a str, (), VerboseError<&'a str>> {
    
        // TODO won't work if there are quotations in the file name
        let (i, (texture_name, texture_expression)) =
            name_and_expression(input.trim())?;

        let (_, (_, path, _, transparency, _)) =
            (space, path, comma_separator, boolean, eof).parse(texture_expression)?;
    
            if self.texture_map.contains_key(texture_name) {
                return context("Texture with same name already exists", fail)(texture_name);
            }

        let full_path = self.parent_path.join(path);
                    let data = match ImageReader::open(full_path) {
                        Ok(tex) => match tex.decode() {
                            Ok(d) => d,
                            // TODO test if these are correct! If the 'i' is correct arguemnt
                            Err(_) => return context("Texture decoding error", fail)(i),
                        },
                        Err(_) => return context("Texture not found", fail)(i),
                    };
                    let texture = TextureData::new(
                        data.to_rgba8().as_bytes().to_vec(),
                        data.width() as usize,
                        data.height() as usize,
                        transparency,
                    );

        self.textures.push(texture);
        self.texture_map
            .insert(texture_name.to_owned(), TextureID(self.texture_count));
        self.texture_count += 1;
        Ok((
            i,()
        ))
    }

    fn parse_setting(
        &mut self,
        input: &'a str,
    ) -> IResult<&'a str, (), VerboseError<&'a str>> {
        let (i, (identifier, values)) =
            name_and_expression(input)?;
    
        match identifier.trim() {
            "defaultHeights" => {
                let (_, (_, heights)) = 
                (space, separated_list1(comma_separator, opt(float))).parse(values)?;
                let [bottom_height, ground_height, ceiling_height, top_height] = heights.as_slice() else {
                    return context("Excatly 4 values needed (or three ',' separators)", fail)(values);
                };
                bottom_height.map(|height| self.settings.bottom_height = height);
                ground_height.map(|height| self.settings.ground_height = height);
                ceiling_height.map(|height| self.settings.ceiling_height = height);
                top_height.map(|height| self.settings.top_height = height);
            },
            "skybox" => {
                let (_, (skybox, eof)) = 
                (separated_list1(comma_separator, opt(alphanumeric1)), eof).parse(values)?;
                let [north, east, south, west, top, bottom] = skybox.as_slice() else {
                    return context("Excatly 6 values needed (or five ',' separators)", fail)(eof);
                };
                update_if_texture_exists!(*north, self.texture_map, self.settings.skybox_north);
                update_if_texture_exists!(*east, self.texture_map, self.settings.skybox_east);
                update_if_texture_exists!(*south, self.texture_map, self.settings.skybox_south);
                update_if_texture_exists!(*west, self.texture_map, self.settings.skybox_west);
                update_if_texture_exists!(*top, self.texture_map, self.settings.skybox_top);
                update_if_texture_exists!(*bottom, self.texture_map, self.settings.skybox_bottom);
            },
            _ => return context("Unknown setting identifier", fail)(identifier)
        };
        Ok((i, ()))
    }

    // TODO maybe add a deafult value for ambient light
fn parse_blueprint(
    &mut self,
    input: &'a str,
) -> IResult<&'a str,(),VerboseError<&'a str>,
> {
    let (i, (_, name, _, _, _, path, _, ambient_light, _, repeatable, _)) =
        (space, alphanumeric1, space, char('='), space, path, comma_separator, float, comma_separator, boolean, eof).parse(input)?;

        let full_path = self.parent_path.join(path);
        let data = match std::fs::read_to_string(full_path.clone()) {
            Ok(d) => d,
            Err(_) => return context("Blueprint file not found", fail)(i),
        };
        let clean_data = clean_input(data);
        let (_, (dimensions, tiles)) = match BlueprintParser::new(self.settings)
            .parse(&clean_data, &self.texture_map)
            .finish()
        {
            Ok(p) => p,
            Err(e) => panic!(
                "blueprint parse error for blueprint {}: {}",
                full_path.display(),
                convert_error(clean_data.as_str(), e)
            ),
        };
        if ambient_light < 0.0 {
            return context("Invalid ambient light", fail)(i);
        }

    let skybox = SkyboxTextureIDs {
        north: self.settings.skybox_north,
        east: self.settings.skybox_east,
        south: self.settings.skybox_south,
        west: self.settings.skybox_west,
        top: self.settings.skybox_top,
        bottom: self.settings.skybox_bottom,
    };

    let blueprint = Blueprint::new(
        BlueprintID(self.blueprints.len()),
        dimensions,
        tiles,
        skybox,
        repeatable,
        ambient_light,
    );
    self.blueprints.push(blueprint);

    Ok((
        i,
        (),
    ))
}
}

// TODO add a setting for a default texture
#[derive(Debug, Clone, Copy)]
pub(super) struct MapSettings {
    pub bottom_height: f32,
    pub ground_height: f32,
    pub ceiling_height: f32,
    pub top_height: f32,
    pub skybox_north: TextureID,
    pub skybox_east: TextureID,
    pub skybox_south: TextureID,
    pub skybox_west: TextureID,
    pub skybox_top: TextureID,
    pub skybox_bottom: TextureID,
}

// TODO maybe require for all map settings to be specified ???????
impl Default for MapSettings {
    fn default() -> Self {
        Self {             
            bottom_height: -1.0,
            ground_height: -0.5,
            ceiling_height: 0.5,
            top_height: 1.0,
            skybox_north: Default::default(),
            skybox_east: Default::default(),
            skybox_south: Default::default(),
            skybox_west: Default::default(),
            skybox_top: Default::default(),
            skybox_bottom: Default::default(),
             }
    }
}
