use hashbrown::HashMap;
use image::{io::Reader as ImageReader, EncodableLayout};
use std::path::PathBuf;

use crate::{textures::{TextureData, Texture}, map_parser::error::PresetError};

use super::error::{VariableError, DimensionError, ParseError, TextureError, TileError};

struct MapParser {
    data: String,
    src_dir_path: PathBuf,

    textures: Vec<TextureData>,
    texture_map: HashMap<String, Texture>,
    next_texture_index: usize,

    preset_map: HashMap<String, TilePreset>,

    // 
    dimensions: (u32, u32),
    lvl1: f32,
    lvl2: f32,
    lvl3: f32,
    lvl4: f32,
}

impl MapParser {
    fn new<P: Into<PathBuf>>(src_path: P) -> Result<Self, ParseError> {
        let src_path: PathBuf = src_path.into().canonicalize()?;
        let data = std::fs::read_to_string(src_path.clone())?;
        Ok(Self {
            data,
            src_dir_path: src_path.parent().unwrap().to_path_buf(),

            textures: Vec::new(),
            texture_map: HashMap::new(),
            next_texture_index: 0,

            preset_map: HashMap::new(),

            dimensions: (0, 0),
            lvl1: -1.0,
            lvl2: -0.5,
            lvl3: 0.5,
            lvl4: 1.0,
        })
    }
    fn parse(mut self) -> Result<(), ParseError> {
        let data = self.data.clone();
        let mut lines = data
            .lines()
            .enumerate()
            .map(|(i, line)| {
                (i as u32, line.split("//").next().unwrap().trim())
            })
            .filter(|(_, line)| !line.is_empty());

        if let Some((i, dimensions_str)) = lines.next() {
            match self.parse_dimensions(dimensions_str) {
                Ok(_) => (),
                Err(e) => return Err(ParseError::Dimensions(e, i)),
            }
        } else {
            return Err(ParseError::Invalid);
        }

        for (i, line) in lines {
            let key = line.chars().next().unwrap();
            match key {
                '*' => match self.parse_variables(line) {
                    Err(e) => return Err(ParseError::Variable(e, i+1)),
                    _ => ()
                },
                '#' => match self.parse_texture(line) {
                    Err(e) => return Err(ParseError::Texture(e, i+1)),
                    _ => ()
                },
                '$' => match self.parse_preset(line) {
                    Err(e) => return Err(ParseError::Preset(e, i+1)),
                    _ => ()
                },
                '_' => (),
                k if k.is_ascii_digit() => (),
                _ => panic!("Unknown line key: '{}'", key),
            };
        }

        Ok(())
    }

    fn parse_dimensions(&mut self, src: &str) -> Result<(), DimensionError> {
        let split: Vec<&str> = src.split('x').collect();
        if split.len() != 2 {
            return Err(DimensionError::InvalidFormat(src.to_owned()));
        }
        let Ok(d1) = split[0].trim().parse::<u32>() else {
            return Err(DimensionError::ParseError(split[0].to_owned()));
        };
        let Ok(d2) = split[1].trim().parse::<u32>() else {
            return Err(DimensionError::ParseError(split[1].to_owned()));
        };
        if d1 == 0 || d2 == 0 {
            return Err(DimensionError::IllegalDimensions(d1, d2));
        }
        self.dimensions = (d1, d2);
        Ok(())
    }

    fn parse_variables(&mut self, src: &str) -> Result<(), VariableError> {
        let src = &src[1..];
        let split: Vec<&str> = src.split('=').collect();
        if split.len() != 2 {
            return Err(VariableError::InvalidFormat(src.to_owned()));
        }
        let variable = split[0].trim();
        let val = split[1].trim();

        match variable {
            "lvl1" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(VariableError::InvalidValue(val.to_owned()));
                };
                self.lvl1 = value;
            }
            "lvl2" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(VariableError::InvalidValue(val.to_owned()));
                };
                self.lvl2 = value;
            }
            "lvl3" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(VariableError::InvalidValue(val.to_owned()));
                };
                self.lvl3 = value;
            }
            "lvl4" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(VariableError::InvalidValue(val.to_owned()));
                };
                self.lvl4 = value;
            }
            _ => {
                return Err(VariableError::UnknownVariable(variable.to_owned()))
            }
        }

        Ok(())
    }

    fn parse_texture(&mut self, src: &str) -> Result<(), TextureError> {
        let src = &src[1..];
        let operands: Vec<&str> = src.split('=').collect();
        if operands.len() != 2 {
            return Err(TextureError::InvalidFormat(src.to_owned()));
        }
        let texture_name = operands[0].trim();
        let expressions = operands[1].trim();
        if self.texture_map.contains_key(texture_name) {
            return Err(TextureError::TextureAlreadyExists(texture_name.to_owned()))
        }

        let mut texture_data = None;
        let mut transparency = None;
        for expr in expressions.split(',') {
            let operands: Vec<&str> = expr.split(':').collect();
            if operands.len() != 2 {
                return Err(TextureError::InvalidExpressionFormat(
                    expr.to_owned(),
                ));
            }
            let parameter = operands[0].trim();
            let value = operands[1].trim();

            match parameter {
                "path" => {
                    let full_path = self.src_dir_path.join(value);
                    texture_data =
                        Some(ImageReader::open(full_path)?.decode()?);
                }
                "transparency" => {
                    transparency = match value.parse::<bool>() {
                        Ok(b) => Some(b),
                        Err(_) => {
                            return Err(TextureError::BoolParseFail(
                                value.to_string(),
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
        let Some(texture_data) = texture_data else {
            return Err(TextureError::UnspecifiedTexture);
        };
        let Some(transparency) = transparency else {
            return Err(TextureError::UnspecifiedTransparency);
        };
        let texture = TextureData::new(
            texture_data.to_rgba8().as_bytes().to_vec(),
            texture_data.width(),
            texture_data.height(),
            transparency,
        );
        self.textures.push(texture);
        self.texture_map.insert(texture_name.to_owned(), Texture::ID(self.next_texture_index));
        self.next_texture_index += 1;
        Ok(())
    }

    fn parse_preset(&mut self, src: &str) -> Result<(), PresetError> {
        let src = &src[1..];
        let split: Vec<&str> = src.split('=').collect();
        if split.len() != 2 {
            return Err(PresetError::InvalidFormat(src.to_owned()));
        }
        let variable = split[0].trim();
        let expressions = split[1].trim();

        let mut preset = TilePreset::default();
        for expr in expressions.split(',') {
            let operands: Vec<&str> = expr.split(':').collect();
            if operands.len() != 2 {
                return Err(TileError::InvalidExpressionFormat(
                    expr.to_owned(),
                ).into());
            }
            let parameter = operands[0].trim();
            let value = operands[1].trim();

            match parameter {
                "pillar1" | "pillar2" | "bottom" | "top" => {
                    let texture = match self.texture_map.get(value) {
                        Some(&t) => Some(t),
                        None => return Err(TileError::UnknownTexture(value.to_owned()).into()),
                    };
                    match parameter {
                        "pillar1" => preset.pillar1_tex = texture,
                        "pillar2" => preset.pillar2_tex = texture, 
                        "bottom" => preset.bottom_platform = texture,
                        "top" => preset.top_platform = texture,
                        _ => unreachable!()
                    }
                }
                "lvl1" | "lvl2" | "lvl3" | "lvl4" => {
                    let parsed = match value.parse::<f32>() {
                        Ok(n) => Some(n),
                        Err(_) => {
                            return Err(TileError::FloatParseFail(
                                value.to_string(),
                            ).into())
                        }
                    };

                    match parameter {
                        "lvl1" => preset.lvl1 = parsed,
                        "lvl2" => preset.lvl2 = parsed, 
                        "lvl3" => preset.lvl3 = parsed,
                        "lvl4" => preset.lvl4 = parsed,
                        _ => unreachable!()
                    }
                }
                _ => return Err(TileError::UnknownParameter(parameter.to_owned()).into())
            }
        }
        
        self.preset_map.insert(variable.to_owned(), preset);

        Ok(())
    }
}

#[derive(Debug, Default)]
struct TilePreset {
    pillar1_tex: Option<Texture>,
    pillar2_tex: Option<Texture>,
    bottom_platform: Option<Texture>,
    top_platform: Option<Texture>,
    lvl1: Option<f32>,
    lvl2: Option<f32>,
    lvl3: Option<f32>,
    lvl4: Option<f32>,
}

#[test]
fn parsing() {
    MapParser::new("../../new_syntax.txt")
        .unwrap()
        .parse()
        .unwrap();
}
