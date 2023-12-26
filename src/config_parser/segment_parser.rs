use hashbrown::HashMap;

use crate::{textures::Texture, world::map::Tile};

use super::{parser::Settings, error::{DimensionError, PresetError, TileError, SegmentParseError}};

#[derive(Debug)]
pub(super) struct SegmentDataParser<'a> {
    data: &'a str,
    settings: &'a Settings,

    preset_map: HashMap<String, TilePreset>,
    texture_map: &'a HashMap<String, Texture>,
    tiles: Vec<Tile>
}

impl<'a> SegmentDataParser<'a> {
    pub(super) fn new(data: &'a str, settings: &'a Settings, texture_map: &'a HashMap<String, Texture>) -> Self {
        Self {
            data,
            settings,

            preset_map: HashMap::new(),
            texture_map,
            tiles: Vec::new()
        }
    }
    pub(super) fn parse(mut self) -> Result<((u32, u32), Vec<Tile>), SegmentParseError> {
        // Remove comments, remove empty lines and trim data
        let mut lines = self.data
            .lines()
            .enumerate()
            .map(|(i, line)| {
                (1+i as u32, line.split("//").next().unwrap().trim())
            })
            .filter(|(_, line)| !line.is_empty());

        let dimensions = match lines.next() {
            Some((i, dimensions_str)) => match self.parse_dimensions(dimensions_str) {
                Ok(d) => d,
                Err(e) => return Err(SegmentParseError::DimensionsErr(e, i)),
            },
            None => return Err(SegmentParseError::Invalid)
        };

        for (i, line) in lines {
            let key = line.chars().next().unwrap();
            match key {
                '$' => match self.parse_preset(line) {
                    Err(e) => return Err(SegmentParseError::PresetErr(e, i)),
                    _ => ()
                },
                k if k.is_ascii_digit() => (),
                _ => return Err(SegmentParseError::UnknownKey(key.to_string(), i)),
            };
        }

        Ok((dimensions, self.tiles))
    }

    fn parse_dimensions(&mut self, src: &str) -> Result<(u32, u32), DimensionError> {
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
        Ok((d1, d2))
    }

    fn parse_preset(&mut self, src: &str) -> Result<(), PresetError> {
        // Split the line and check for formatting errors
        let src = &src[1..];
        let split: Vec<&str> = src.split('=').collect();
        if split.len() != 2 {
            return Err(PresetError::InvalidFormat(src.to_owned()));
        }
        let identifier = split[0].trim();
        let expressions = split[1].trim();

        let mut preset = TilePreset::default();
        for expr in expressions.split(',') {
            // Split the expression and check for formatting errors
            let operands: Vec<&str> = expr.split(':').collect();
            if operands.len() != 2 {
                return Err(TileError::InvalidExpressionFormat(
                    expr.to_owned(),
                ).into());
            }
            let parameter = operands[0].trim();
            let value = operands[1].trim();

            // Identify the parameter and act accordingly
            match parameter {
                // If the parameter is one of these, the value should be a *texture name*
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
                // If the parameter is one of these, the value should be a *number*
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
        
        self.preset_map.insert(identifier.to_owned(), preset);

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