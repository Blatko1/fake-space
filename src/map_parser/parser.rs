use image::{io::Reader as ImageReader, EncodableLayout};
use std::path::PathBuf;

use crate::textures::TextureData;

use super::error::{ConstantError, DimensionError, ParseError, TextureError};

struct MapParser {
    data: String,
    src_dir_path: PathBuf,

    //
    dimensions: (u32, u32),
    lvl1: Option<f32>,
    lvl2: Option<f32>,
    lvl3: Option<f32>,
    lvl4: Option<f32>,
}

impl MapParser {
    fn new<P: Into<PathBuf>>(src_path: P) -> Result<Self, ParseError> {
        let src_path: PathBuf = src_path.into().canonicalize()?;
        let data = std::fs::read_to_string(src_path.clone())?;
        Ok(Self {
            data,
            src_dir_path: src_path.parent().unwrap().to_path_buf(),
            dimensions: (0, 0),
            lvl1: None,
            lvl2: None,
            lvl3: None,
            lvl4: None,
        })
    }
    fn parse(mut self) -> Result<(), ParseError> {
        let mut lines = self
            .data
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

        //TODO add a macro for simpler error parsing
        for (i, line) in lines {
            let key = line.chars().next().unwrap();
            match key {
                '*' => self.parse_constant(line),
                '#' => self.parse_texture(line),
                '$' => (),
                '_' => (),
                k if k.is_ascii_digit() => (),
                _ => panic!("Unknown line key {}", key),
            };
        }

        Ok(())
    }

    fn parse_dimensions(&mut self, src: &str) -> Result<(), DimensionError> {
        let mut split: Vec<&str> = src.split('x').collect();
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

    fn parse_constant(&mut self, src: &str) -> Result<(), ConstantError> {
        let src = &src[1..];
        let split: Vec<&str> = src.split('=').collect();
        if split.len() != 2 {
            return Err(ConstantError::InvalidFormat(src.to_owned()));
        }
        let variable = split[0].trim();
        let val = split[1].trim();

        match variable {
            "lvl1" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(ConstantError::InvalidValue(val.to_owned()));
                };
                self.lvl1 = Some(value);
            }
            "lvl2" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(ConstantError::InvalidValue(val.to_owned()));
                };
                self.lvl2 = Some(value);
            }
            "lvl3" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(ConstantError::InvalidValue(val.to_owned()));
                };
                self.lvl3 = Some(value);
            }
            "lvl4" => {
                let Ok(value) = val.parse::<f32>() else {
                    return Err(ConstantError::InvalidValue(val.to_owned()));
                };
                self.lvl4 = Some(value);
            }
            _ => {
                return Err(ConstantError::UnknownVariable(variable.to_owned()))
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
                            return Err(TextureError::FailedBoolParse(
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
        textures.push(texture);
        texture_indices
            .insert(texture_name.to_string(), Texture::ID(texture_index));
        assert_eq!(textures.len(), texture_indices.len());
        Ok((textures, texture_indices))
    }
}

#[test]
fn parsing() {
    MapParser::new("../../new_syntax.txt")
        .unwrap()
        .parse()
        .unwrap();
}
