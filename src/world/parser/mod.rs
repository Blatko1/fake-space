#[cfg(test)]
mod tests;

pub mod error;
mod segment_parser;

use std::path::PathBuf;

use hashbrown::HashMap;
use image::{io::Reader as ImageReader, EncodableLayout};

use super::{SkyboxTextures, Texture, TextureData};

use self::error::{ParseError, SegmentError, SettingError, TextureError};
use self::segment_parser::SegmentDataParser;

use super::{Segment, SegmentID, World};

pub struct WorldParser {
    data: String,
    dir_path: PathBuf,

    settings: Settings,
    textures: Vec<TextureData>,
    texture_map: HashMap<String, Texture>,
    texture_counter: usize,
    segments: Vec<Segment>,
}

impl WorldParser {
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self, ParseError> {
        let path: PathBuf = path.into().canonicalize()?;
        let data = std::fs::read_to_string(path.clone())?;
        Ok(Self {
            data,
            dir_path: path.parent().unwrap().to_path_buf(),

            settings: Settings::default(),
            textures: Vec::new(),
            texture_map: HashMap::new(),
            texture_counter: 0,
            segments: Vec::new(),
        })
    }

    pub fn parse(mut self) -> Result<World, ParseError> {
        let data = self.data.clone();

        // Remove comments, remove empty lines and trim data
        let lines = data
            .lines()
            .enumerate()
            .map(|(i, line)| (1 + i as u64, line.split("//").next().unwrap().trim()))
            .filter(|(_, line)| !line.is_empty());

        // Process each line
        for (i, line) in lines {
            // Identify each line by key
            let key = line.chars().next().unwrap();
            match key {
                // Mutates setting values through the function
                '*' => {
                    if let Err(e) = self.parse_setting(line) {
                        return Err(ParseError::SettingErr(e, i));
                    }
                }
                '#' => match self.parse_texture(line) {
                    Ok((name, tex)) => {
                        self.textures.push(tex);
                        self.texture_map
                            .insert(name, Texture::ID(self.texture_counter));
                        self.texture_counter += 1;
                    }
                    Err(e) => return Err(ParseError::TextureErr(e, i)),
                },
                '!' => match self.parse_segment(line, SegmentID(self.segments.len())) {
                    Ok(segment) => self.segments.push(segment),
                    Err(e) => return Err(ParseError::SegmentErr(e, i)),
                },
                _ => return Err(ParseError::UnknownKey(key.to_string(), i)),
            }
        }
        if self.segments.len() < 2 {
            return Err(ParseError::NotEnoughSegments(self.segments.len()));
        }
        Ok(World::new(self.segments, self.textures))
    }

    fn parse_segment(&self, line: &str, id: SegmentID) -> Result<Segment, SegmentError> {
        // Split the line and check for formatting errors
        let split: Vec<&str> = line.split('=').collect();
        if split.len() != 2 {
            return Err(SegmentError::InvalidFormat(line.to_owned()));
        }
        let name = split[0].trim();
        let expressions = split[1].trim();

        let mut segment_data = None;
        let mut repeatable = None;
        let mut ambient_light = None;
        for expr in expressions.split(',') {
            // Split the expression and check for formatting errors
            let split: Vec<&str> = expr.split(':').collect();
            if split.len() != 2 {
                return Err(SegmentError::InvalidFormat(expr.to_owned()));
            }
            let parameter = split[0].trim();
            let value = split[1].trim();

            // Identify the parameter and act accordingly
            match parameter {
                "src" => {
                    let full_path = self.dir_path.join(value);
                    let data = std::fs::read_to_string(full_path.clone())?;
                    let parsed = match SegmentDataParser::new(
                        &data,
                        &self.settings,
                        &self.texture_map,
                    )
                    .parse()
                    {
                        Ok(p) => p,
                        Err(e) => {
                            return Err(SegmentError::SegmentParseErr(
                                e,
                                full_path.to_string_lossy().to_string(),
                            ))
                        }
                    };
                    segment_data = Some(parsed);
                }
                "repeatable" => {
                    repeatable = match value.parse::<bool>() {
                        Ok(b) => Some(b),
                        Err(_) => {
                            return Err(SegmentError::BoolParseFail(value.to_owned()))
                        }
                    }
                }
                "ambient_light" => {
                    ambient_light = match value.parse::<f32>() {
                        Ok(b) => Some(b),
                        Err(_) => {
                            return Err(SegmentError::F32ParseFail(value.to_owned()))
                        }
                    }
                }
                _ => return Err(SegmentError::UnknownParameter(parameter.to_owned())),
            }
        }
        // Check if all needed information has been acquired
        let Some((dimensions, tiles)) = segment_data else {
            return Err(SegmentError::UnspecifiedSrc);
        };
        let Some(repeatable) = repeatable else {
            return Err(SegmentError::UnspecifiedRepetition);
        };
        let Some(ambient_light) = ambient_light else {
            return Err(SegmentError::UnspecifiedAmbientLight);
        };
        if ambient_light < 0.0 {
            return Err(SegmentError::InvalidAmbientLight(ambient_light.to_string()));
        }

        let skybox = SkyboxTextures {
            north: self.settings.skybox_north,
            east: self.settings.skybox_east,
            south: self.settings.skybox_south,
            west: self.settings.skybox_west,
            top: self.settings.skybox_top,
            bottom: self.settings.skybox_bottom,
        };

        Ok(Segment::new(
            id,
            name.to_owned(),
            dimensions,
            tiles,
            skybox,
            repeatable,
            ambient_light,
        ))
    }

    fn parse_setting(&mut self, line: &str) -> Result<(), SettingError> {
        // Split the line and check for formatting errors
        let line = &line[1..];
        let split: Vec<&str> = line.split('=').collect();
        if split.len() != 2 {
            return Err(SettingError::InvalidFormat(line.to_owned()));
        }
        let setting = split[0].trim();
        let val = split[1].trim();

        // Identify the setting parameter and act accordingly
        match setting {
            "bottomL" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(SettingError::InvalidF32Value(val.to_owned()));
                };
                self.settings.bottom_level = value;
            }
            "groundL" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(SettingError::InvalidF32Value(val.to_owned()));
                };
                self.settings.ground_level = value;
            }
            "ceilingL" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(SettingError::InvalidF32Value(val.to_owned()));
                };
                self.settings.ceiling_level = value;
            }
            "topL" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(SettingError::InvalidF32Value(val.to_owned()));
                };
                self.settings.top_level = value;
            }
            "skyboxNorth" => {
                let Some(&texture_id) = self.texture_map.get(val) else {
                    return Err(SettingError::UnknownTexture(val.to_owned()));
                };
                self.settings.skybox_north = texture_id;
            }
            "skyboxEast" => {
                let Some(&texture_id) = self.texture_map.get(val) else {
                    return Err(SettingError::UnknownTexture(val.to_owned()));
                };
                self.settings.skybox_east = texture_id;
            }
            "skyboxSouth" => {
                let Some(&texture_id) = self.texture_map.get(val) else {
                    return Err(SettingError::UnknownTexture(val.to_owned()));
                };
                self.settings.skybox_south = texture_id;
            }
            "skyboxWest" => {
                let Some(&texture_id) = self.texture_map.get(val) else {
                    return Err(SettingError::UnknownTexture(val.to_owned()));
                };
                self.settings.skybox_west = texture_id;
            }
            "skyboxTop" => {
                let Some(&texture_id) = self.texture_map.get(val) else {
                    return Err(SettingError::UnknownTexture(val.to_owned()));
                };
                self.settings.skybox_top = texture_id;
            }
            "skyboxBottom" => {
                let Some(&texture_id) = self.texture_map.get(val) else {
                    return Err(SettingError::UnknownTexture(val.to_owned()));
                };
                self.settings.skybox_bottom = texture_id;
            }
            _ => return Err(SettingError::UnknownSetting(setting.to_owned())),
        }

        Ok(())
    }

    fn parse_texture(&self, line: &str) -> Result<(String, TextureData), TextureError> {
        // Split the line and check for formatting errors
        let line = &line[1..];
        let operands: Vec<&str> = line.split('=').collect();
        if operands.len() != 2 {
            return Err(TextureError::InvalidFormat(line.to_owned()));
        }
        let texture_name = operands[0].trim();
        let expressions = operands[1].trim();

        // There can't be multiple texture with the same name
        if self.texture_map.contains_key(texture_name) {
            return Err(TextureError::TextureAlreadyExists(texture_name.to_owned()));
        }

        let mut texture_data = None;
        let mut transparency = None;
        for expr in expressions.split(',') {
            // Split the expression and check for formatting errors
            let operands: Vec<&str> = expr.split(':').collect();
            if operands.len() != 2 {
                return Err(TextureError::InvalidExpressionFormat(expr.to_owned()));
            }
            let parameter = operands[0].trim();
            let value = operands[1].trim();

            // Identify the parameter and act accordingly
            match parameter {
                "path" => {
                    let full_path = self.dir_path.join(value);
                    texture_data = Some(ImageReader::open(full_path)?.decode()?);
                }
                "transparency" => {
                    transparency = match value.parse::<bool>() {
                        Ok(b) => Some(b),
                        Err(_) => {
                            return Err(TextureError::BoolParseFail(value.to_owned()))
                        }
                    }
                }
                _ => {
                    return Err(TextureError::UnknownExpressionParameter(
                        parameter.to_owned(),
                    ))
                }
            }
        }
        // Check if all needed information is acquired
        let Some(texture_data) = texture_data else {
            return Err(TextureError::UnspecifiedSrc);
        };
        let Some(transparency) = transparency else {
            return Err(TextureError::UnspecifiedTransparency);
        };

        // Store the texture with an unique ID
        let texture = TextureData::new(
            texture_data.to_rgba8().as_bytes().to_vec(),
            texture_data.width(),
            texture_data.height(),
            transparency,
        );

        Ok((texture_name.to_owned(), texture))
    }
}

#[derive(Debug)]
pub(super) struct Settings {
    pub bottom_level: f32,
    pub ground_level: f32,
    pub ceiling_level: f32,
    pub top_level: f32,
    pub skybox_north: Texture,
    pub skybox_east: Texture,
    pub skybox_south: Texture,
    pub skybox_west: Texture,
    pub skybox_top: Texture,
    pub skybox_bottom: Texture,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bottom_level: -1.0,
            ground_level: -0.5,
            ceiling_level: 0.5,
            top_level: 1.0,
            skybox_north: Texture::Empty,
            skybox_east: Texture::Empty,
            skybox_south: Texture::Empty,
            skybox_west: Texture::Empty,
            skybox_top: Texture::Empty,
            skybox_bottom: Texture::Empty,
        }
    }
}

#[test]
fn parsing() {
    WorldParser::new("maps/world.txt").unwrap().parse().unwrap();
}
