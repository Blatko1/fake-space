use std::ops::RangeInclusive;

use hashbrown::HashMap;

use crate::{textures::Texture, world::map::Tile};

use super::{
    error::{DimensionError, PresetError, SegmentParseError, TileError},
    Settings,
};

#[derive(Debug)]
pub(super) struct SegmentDataParser<'a> {
    data: &'a str,
    settings: &'a Settings,

    preset_map: HashMap<String, TilePreset>,
    texture_map: &'a HashMap<String, Texture>,
    tiles: Vec<TilePreset>,
}

impl<'a> SegmentDataParser<'a> {
    pub(super) fn new(
        data: &'a str,
        settings: &'a Settings,
        texture_map: &'a HashMap<String, Texture>,
    ) -> Self {
        Self {
            data,
            settings,

            preset_map: HashMap::new(),
            texture_map,
            tiles: Vec::new(),
        }
    }
    pub(super) fn parse(
        mut self,
    ) -> Result<((u32, u32), Vec<Tile>), SegmentParseError> {
        // Remove comments, remove empty lines and trim data
        let mut lines = self
            .data
            .lines()
            .enumerate()
            .map(|(i, line)| {
                (1 + i as u32, line.split("//").next().unwrap().trim())
            })
            .filter(|(_, line)| !line.is_empty());

        let dimensions = match lines.next() {
            Some((i, dimensions_str)) => match self
                .parse_dimensions(dimensions_str)
            {
                Ok(d) => d,
                Err(e) => return Err(SegmentParseError::DimensionsErr(e, i)),
            },
            None => return Err(SegmentParseError::Invalid),
        };
        self.tiles =
            vec![TilePreset::default(); (dimensions.0 * dimensions.1) as usize];

        for (i, line) in lines {
            let key = line.chars().next().unwrap();
            match key {
                '$' => match self.parse_preset(line) {
                    Ok((id, preset)) => {
                        self.preset_map.insert(id, preset);
                    }
                    Err(e) => return Err(SegmentParseError::PresetErr(e, i)),
                },
                k if k.is_ascii_digit() => {
                    if let Err(e) = self.parse_tile_def(line) {
                        return Err(SegmentParseError::TileErr(e, i));
                    }
                }
                _ => {
                    return Err(SegmentParseError::UnknownKey(
                        key.to_string(),
                        i,
                    ))
                }
            };
        }

        let mut tiles = Vec::with_capacity(self.tiles.len());
        for (i, tile) in self.tiles.into_iter().enumerate() {
            let t = match tile.to_tile(self.settings) {
                Ok(t) => t,
                Err(e) => return Err(SegmentParseError::InvalidLevels(i+1, e.0, e.1, e.2, e.3)),
            };
            tiles.push(t);
        }

        Ok((dimensions, tiles))
    }

    fn parse_dimensions(
        &mut self,
        line: &str,
    ) -> Result<(u32, u32), DimensionError> {
        let split: Vec<&str> = line.split('x').collect();
        if split.len() != 2 {
            return Err(DimensionError::InvalidFormat(line.to_owned()));
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

    fn parse_preset(
        &mut self,
        line: &str,
    ) -> Result<(String, TilePreset), PresetError> {
        // Split the line and check for formatting errors
        let line = &line[1..];
        let split: Vec<&str> = line.split('=').collect();
        if split.len() != 2 {
            return Err(PresetError::InvalidFormat(line.to_owned()));
        }
        let identifier = split[0].trim();
        let expressions = split[1].trim();

        let preset = self.parse_tile_expressions(expressions)?;

        Ok((identifier.to_owned(), preset))
    }

    fn parse_tile_def(&mut self, line: &str) -> Result<(), TileError> {
        // Split the line and check for formatting errors
        let split: Vec<&str> = line.split('=').collect();
        if split.len() != 2 {
            return Err(TileError::InvalidFormat(line.to_owned()));
        }
        let index_str = split[0].trim();
        let expressions_str = split[1].trim();

        let index = Self::parse_index(index_str)?;
        let new_tile = self.parse_tile_expressions(expressions_str)?;

        for i in index {
            if let Some(tile) = self.tiles.get_mut(i) {
                tile.overwrite_with(&new_tile)
            } else {
                return Err(TileError::IndexOutOfRange(
                    index_str.to_owned(),
                    self.tiles.len(),
                ));
            }
        }

        Ok(())
    }

    fn parse_index(index: &str) -> Result<RangeInclusive<usize>, TileError> {
        let split: Vec<&str> = index.split('-').collect();
        match split[..] {
            // If the index is only one number
            [i_str] => match i_str.trim().parse::<usize>() {
                Ok(i) => {
                    if i == 0 {
                        return Err(TileError::IndexIsZero(index.to_owned()));
                    }
                    Ok((i - 1)..=(i - 1))
                }
                Err(_) => {
                    Err(TileError::IndexUsizeParseFail(i_str.to_string()))
                }
            },
            // If index is an inclusive range
            [from_str, to_str] => {
                let from = match from_str.trim().parse::<usize>() {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(TileError::IndexUsizeParseFail(
                            from_str.to_string(),
                        ))
                    }
                };
                let to = match to_str.trim().parse::<usize>() {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(TileError::IndexUsizeParseFail(
                            to_str.to_string(),
                        ))
                    }
                };
                if from > to || from == 0 {
                    return Err(TileError::InvalidIndexRange(
                        index.to_owned(),
                        from,
                        to,
                    ));
                }
                Ok((from - 1)..=(to - 1))
            }
            _ => Err(TileError::InvalidIndexFormat(index.to_owned())),
        }
    }

    fn parse_tile_expressions(
        &self,
        expressions: &str,
    ) -> Result<TilePreset, TileError> {
        let mut preset = TilePreset::default();
        for expr in expressions.split(',') {
            // Split the expression and check for formatting errors
            let operands: Vec<&str> = expr.trim().split(':').collect();
            match operands[..] {
                [s] if s.is_empty() => continue,
                // If the expression is only one word with a preceding '$' sign,
                // then load that preset in this preset
                [e] => {
                    if let Some(preset_str) = e.strip_prefix('$') {
                        match self.preset_map.get(preset_str) {
                            Some(preset_expr) => {
                                preset.overwrite_with(preset_expr);
                                continue;
                            }
                            None => {
                                return Err(TileError::UnknownPreset(
                                    preset_str.to_owned(),
                                ))
                            }
                        }
                    } else {
                        return Err(TileError::InvalidExpressionFormat(
                            expr.to_owned(),
                        ));
                    }
                }
                [_, _] => (),
                _ => {
                    return Err(TileError::InvalidExpressionFormat(
                        expr.to_owned(),
                    ))
                }
            }

            let parameter = operands[0].trim();
            let value = operands[1].trim();

            // Identify the parameter and act accordingly
            match parameter {
                // If the parameter is one of these, the value should be a *texture name*
                "pillar1" | "pillar2" | "bottom" | "top" => {
                    let texture = match self.texture_map.get(value) {
                        Some(&t) => Some(t),
                        None => {
                            return Err(TileError::UnknownTexture(
                                value.to_owned(),
                            ))
                        }
                    };
                    match parameter {
                        "pillar1" => preset.pillar1_tex = texture,
                        "pillar2" => preset.pillar2_tex = texture,
                        "bottom" => preset.bottom_platform = texture,
                        "top" => preset.top_platform = texture,
                        _ => unreachable!(),
                    }
                }
                // If the parameter is one of these, the value should be a *number*
                "lvl1" | "lvl2" | "lvl3" | "lvl4" => {
                    let parsed = match value.parse::<f32>() {
                        Ok(n) => Some(n),
                        Err(_) => {
                            return Err(TileError::FloatParseFail(
                                value.to_string(),
                            ))
                        }
                    };

                    match parameter {
                        "lvl1" => preset.lvl1 = parsed,
                        "lvl2" => preset.lvl2 = parsed,
                        "lvl3" => preset.lvl3 = parsed,
                        "lvl4" => preset.lvl4 = parsed,
                        _ => unreachable!(),
                    }
                }
                _ => {
                    return Err(TileError::UnknownParameter(
                        parameter.to_owned(),
                    ))
                }
            }
        }
        Ok(preset)
    }
}

#[derive(Debug, Default, Clone)]
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

impl TilePreset {
    /// Overwrites all old values with new ones.
    /// Doesn't replace if the new value is `None`.
    fn overwrite_with(&mut self, other: &Self) {
        if let Some(pillar1) = other.pillar1_tex {
            self.pillar1_tex.replace(pillar1);
        }
        if let Some(pillar2) = other.pillar2_tex {
            self.pillar2_tex.replace(pillar2);
        }
        if let Some(bottom_platform) = other.bottom_platform {
            self.bottom_platform.replace(bottom_platform);
        }
        if let Some(top_platform) = other.top_platform {
            self.top_platform.replace(top_platform);
        }
        if let Some(lvl1) = other.lvl1 {
            self.lvl1.replace(lvl1);
        }
        if let Some(lvl2) = other.lvl2 {
            self.lvl2.replace(lvl2);
        }
        if let Some(lvl3) = other.lvl3 {
            self.lvl3.replace(lvl3);
        }
        if let Some(lvl4) = other.lvl4 {
            self.lvl4.replace(lvl4);
        }
    }

    /// Fills the `None` values with default ones and convert to [`Tile`].
    /// Compares levels to each other to find errors (lvl1 <= lvl2 < lvl3 <= lvl4).
    fn to_tile(&self, settings: &Settings) -> Result<Tile, (f32, f32, f32, f32)> {
        let level1 = self.lvl1.unwrap_or(settings.lvl1);
        let level2 = self.lvl2.unwrap_or(settings.lvl2);
        let level3 = self.lvl3.unwrap_or(settings.lvl3);
        let level4 = self.lvl4.unwrap_or(settings.lvl4);
        if level1 <= level2 && level2 < level3 && level3 <= level4 {
            Ok(Tile {
                pillar1_tex: self.pillar1_tex.unwrap_or_default(),
                pillar2_tex: self.pillar2_tex.unwrap_or_default(),
                bottom_platform_tex: self.bottom_platform.unwrap_or_default(),
                top_platform_tex: self.top_platform.unwrap_or_default(),
                level1,
                level2,
                level3,
                level4,
            })
        } else {
            Err((level1, level2, level3, level4))
        }
    }
}
