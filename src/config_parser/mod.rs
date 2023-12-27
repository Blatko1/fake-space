#[cfg(test)]
mod tests;

pub mod error;
mod segment_parser;

use std::path::PathBuf;

use hashbrown::HashMap;
use image::{io::Reader as ImageReader, EncodableLayout};

use crate::textures::{Texture, TextureData};
use crate::world::map::{Segment, World};

use self::error::{ParseError, SegmentError, SettingError, TextureError};
use self::segment_parser::SegmentDataParser;

pub struct ConfigParser {
    data: String,
    dir_path: PathBuf,

    settings: Settings,
    textures: Vec<TextureData>,
    texture_map: HashMap<String, Texture>,
    texture_counter: usize,
    segments: Vec<Segment>,
}

impl ConfigParser {
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
            .map(|(i, line)| {
                (1 + i as u32, line.split("//").next().unwrap().trim())
            })
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
                '!' => match self.parse_segment(line) {
                    Ok(segment) => self.segments.push(segment),
                    Err(e) => return Err(ParseError::SegmentErr(e, i)),
                },
                _ => return Err(ParseError::UnknownKey(key.to_string(), i)),
            }
        }
        Ok(World::new(self.segments, self.textures))
    }

    fn parse_segment(&self, line: &str) -> Result<Segment, SegmentError> {
        // Split the line and check for formatting errors
        let split: Vec<&str> = line.split('=').collect();
        if split.len() != 2 {
            return Err(SegmentError::InvalidFormat(line.to_owned()));
        }
        let identifier = split[0].trim();
        let expressions = split[1].trim();

        let mut segment_tiles = None;
        let mut repeatable = None;
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
                    segment_tiles = Some(parsed);
                }
                "repeatable" => {
                    repeatable = match value.parse::<bool>() {
                        Ok(b) => Some(b),
                        Err(_) => {
                            return Err(SegmentError::BoolParseFail(
                                value.to_owned(),
                            ))
                        }
                    }
                }
                _ => {
                    return Err(SegmentError::UnknownParameter(
                        parameter.to_owned(),
                    ))
                }
            }
        }
        // Check if all needed information is acquired
        let Some((dimensions, tiles)) = segment_tiles else {
            return Err(SegmentError::UnspecifiedSrc);
        };
        let Some(repeatable) = repeatable else {
            return Err(SegmentError::UnspecifiedRepetition);
        };

        Ok(Segment::new(
            identifier.to_owned(),
            dimensions,
            tiles,
            repeatable,
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
            "lvl1" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(SettingError::InvalidValue(val.to_owned()));
                };
                self.settings.lvl1 = value;
            }
            "lvl2" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(SettingError::InvalidValue(val.to_owned()));
                };
                self.settings.lvl2 = value;
            }
            "lvl3" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(SettingError::InvalidValue(val.to_owned()));
                };
                self.settings.lvl3 = value;
            }
            "lvl4" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(SettingError::InvalidValue(val.to_owned()));
                };
                self.settings.lvl4 = value;
            }
            _ => return Err(SettingError::UnknownSetting(setting.to_owned())),
        }

        Ok(())
    }

    fn parse_texture(
        &self,
        line: &str,
    ) -> Result<(String, TextureData), TextureError> {
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
            return Err(TextureError::TextureAlreadyExists(
                texture_name.to_owned(),
            ));
        }

        let mut texture_data = None;
        let mut transparency = None;
        for expr in expressions.split(',') {
            // Split the expression and check for formatting errors
            let operands: Vec<&str> = expr.split(':').collect();
            if operands.len() != 2 {
                return Err(TextureError::InvalidExpressionFormat(
                    expr.to_owned(),
                ));
            }
            let parameter = operands[0].trim();
            let value = operands[1].trim();

            // Identify the parameter and act accordingly
            match parameter {
                "path" => {
                    let full_path = self.dir_path.join(value);
                    texture_data =
                        Some(ImageReader::open(full_path)?.decode()?);
                }
                "transparency" => {
                    transparency = match value.parse::<bool>() {
                        Ok(b) => Some(b),
                        Err(_) => {
                            return Err(TextureError::BoolParseFail(
                                value.to_owned(),
                            ))
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
    pub lvl1: f32,
    pub lvl2: f32,
    pub lvl3: f32,
    pub lvl4: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lvl1: -1.0,
            lvl2: -0.5,
            lvl3: 0.5,
            lvl4: 1.0,
        }
    }
}

#[test]
fn parsing() {
    ConfigParser::new("maps/config.txt")
        .unwrap()
        .parse()
        .unwrap();
}
