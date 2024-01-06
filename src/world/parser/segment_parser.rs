use std::{ops::RangeInclusive, str::FromStr};

use hashbrown::HashMap;

use crate::
    textures::Texture
;
use crate::world::portal::{DummyPortal, PortalDirection, PortalID};
use crate::world::{Tile, TilePosition};

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
    ) -> Result<((u64, u64), Vec<Tile>), SegmentParseError> {
        // Remove comments, remove empty lines and trim data
        let mut lines = self
            .data
            .lines()
            .enumerate()
            .map(|(i, line)| (1 + i as u64, line.split("//").next().unwrap().trim()))
            .filter(|(_, line)| !line.is_empty());

        let dimensions = match lines.next() {
            Some((i, dimensions_str)) => match self.parse_dimensions(dimensions_str) {
                Ok(d) => d,
                Err(e) => return Err(SegmentParseError::DimensionsErr(e, i)),
            },
            None => return Err(SegmentParseError::Invalid),
        };
        self.tiles = vec![TilePreset::default(); (dimensions.0 * dimensions.1) as usize];

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
                _ => return Err(SegmentParseError::UnknownKey(key.to_string(), i)),
            };
        }

        let mut tiles = Vec::with_capacity(self.tiles.len());
        let mut portal_id = 0;
        for (i, tile) in self.tiles.into_iter().enumerate() {
            // Fill the `None` values with default ones and convert to [`Tile`], then
            // compare levels to each other to find error (lvl1 <= lvl2 < lvl3 <= lvl4)
            let bottom_level = tile.bottom_level.unwrap_or(self.settings.bottom_level);
            let ground_level = tile.ground_level.unwrap_or(self.settings.ground_level);
            let ceiling_level = tile.ceiling_level.unwrap_or(self.settings.ceiling_level);
            let top_level = tile.top_level.unwrap_or(self.settings.top_level);
            if !(bottom_level <= ground_level
                && ground_level < ceiling_level
                && ceiling_level <= top_level)
            {
                return Err(SegmentParseError::InvalidLevels(
                    i + 1,
                    bottom_level,
                    ground_level,
                    ceiling_level,
                    top_level,
                ));
            }
            let portal = match tile.portal_dir {
                Some(direction) => {
                    let dummy = DummyPortal {
                        id: PortalID(portal_id),
                        direction,
                    };
                    portal_id += 1;
                    Some(dummy)
                }
                None => None,
            };
            let position = TilePosition {
                x: i as u64 % dimensions.0,
                z: i as u64 / dimensions.0,
            };
            let t = Tile {
                position,
                bottom_pillar_tex: tile.bottom_pillar_tex.unwrap_or_default(),
                top_pillar_tex: tile.top_pillar_tex.unwrap_or_default(),
                ground_tex: tile.ground_tex.unwrap_or_default(),
                ceiling_tex: tile.ceiling_tex.unwrap_or_default(),
                bottom_level,
                ground_level,
                ceiling_level,
                top_level,
                portal,
            };

            tiles.push(t);
        }
        if portal_id == 0 {
            return Err(SegmentParseError::NoPortalsSpecified);
        }

        Ok((dimensions, tiles))
    }

    fn parse_dimensions(&mut self, line: &str) -> Result<(u64, u64), DimensionError> {
        let split: Vec<&str> = line.split('x').collect();
        if split.len() != 2 {
            return Err(DimensionError::InvalidFormat(line.to_owned()));
        }
        let Ok(d1) = split[0].trim().parse::<u64>() else {
            return Err(DimensionError::ParseError(split[0].to_owned()));
        };
        let Ok(d2) = split[1].trim().parse::<u64>() else {
            return Err(DimensionError::ParseError(split[1].to_owned()));
        };
        if d1 == 0 || d2 == 0 {
            return Err(DimensionError::IllegalDimensions(d1, d2));
        }
        Ok((d1, d2))
    }

    fn parse_preset(&mut self, line: &str) -> Result<(String, TilePreset), PresetError> {
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
                Err(_) => Err(TileError::IndexUsizeParseFail(i_str.to_string())),
            },
            // If index is an inclusive range
            [from_str, to_str] => {
                let from = match from_str.trim().parse::<usize>() {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(TileError::IndexUsizeParseFail(from_str.to_string()))
                    }
                };
                let to = match to_str.trim().parse::<usize>() {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(TileError::IndexUsizeParseFail(to_str.to_string()))
                    }
                };
                if from > to || from == 0 {
                    return Err(TileError::InvalidIndexRange(index.to_owned(), from, to));
                }
                Ok((from - 1)..=(to - 1))
            }
            _ => Err(TileError::InvalidIndexFormat(index.to_owned())),
        }
    }

    fn parse_tile_expressions(&self, expressions: &str) -> Result<TilePreset, TileError> {
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
                        return Err(TileError::InvalidExpressionFormat(expr.to_owned()));
                    }
                }
                [_, _] => (),
                _ => return Err(TileError::InvalidExpressionFormat(expr.to_owned())),
            }
            let parameter = operands[0].trim();
            let value = operands[1].trim();

            // Identify the parameter and act accordingly
            match parameter {
                // If the parameter is one of these, the value should be a *texture name*
                "bottomT" | "topT" | "groundT" | "ceilingT" => {
                    let Some(&texture) = self.texture_map.get(value) else {
                        return Err(TileError::UnknownTexture(value.to_owned()));
                    };
                    match parameter {
                        "bottomT" => preset.bottom_pillar_tex = Some(texture),
                        "topT" => preset.top_pillar_tex = Some(texture),
                        "groundT" => preset.ground_tex = Some(texture),
                        "ceilingT" => preset.ceiling_tex = Some(texture),
                        _ => unreachable!(),
                    }
                }
                // If the parameter is one of these, the value should be a *number*
                "bottomL" | "groundL" | "ceilingL" | "topL" => {
                    let Ok(parsed) = value.parse::<f32>() else {
                        return Err(TileError::FloatParseFail(value.to_string()));
                    };

                    match parameter {
                        "bottomL" => preset.bottom_level = Some(parsed),
                        "groundL" => preset.ground_level = Some(parsed),
                        "ceilingL" => preset.ceiling_level = Some(parsed),
                        "topL" => preset.top_level = Some(parsed),
                        _ => unreachable!(),
                    }
                }
                "portalDir" => {
                    let parsed = value.parse::<PortalDirection>()?;
                    preset.portal_dir = Some(parsed);
                }
                _ => return Err(TileError::UnknownParameter(parameter.to_owned())),
            }
        }
        Ok(preset)
    }
}

#[derive(Debug, Default, Clone)]
struct TilePreset {
    bottom_pillar_tex: Option<Texture>,
    top_pillar_tex: Option<Texture>,
    ground_tex: Option<Texture>,
    ceiling_tex: Option<Texture>,
    bottom_level: Option<f32>,
    ground_level: Option<f32>,
    ceiling_level: Option<f32>,
    top_level: Option<f32>,
    portal_dir: Option<PortalDirection>,
}

impl TilePreset {
    /// Overwrites all old values with new ones.
    /// Doesn't replace if the new value is `None`.
    fn overwrite_with(&mut self, other: &Self) {
        if let Some(bottom_pillar_tex) = other.bottom_pillar_tex {
            self.bottom_pillar_tex.replace(bottom_pillar_tex);
        }
        if let Some(top_pillar_tex) = other.top_pillar_tex {
            self.top_pillar_tex.replace(top_pillar_tex);
        }
        if let Some(ground_tex) = other.ground_tex {
            self.ground_tex.replace(ground_tex);
        }
        if let Some(ceiling_tex) = other.ceiling_tex {
            self.ceiling_tex.replace(ceiling_tex);
        }
        if let Some(bottom_level) = other.bottom_level {
            self.bottom_level.replace(bottom_level);
        }
        if let Some(ground_level) = other.ground_level {
            self.ground_level.replace(ground_level);
        }
        if let Some(ceiling_level) = other.ceiling_level {
            self.ceiling_level.replace(ceiling_level);
        }
        if let Some(top_level) = other.top_level {
            self.top_level.replace(top_level);
        }
        if let Some(portal_dir) = other.portal_dir {
            self.portal_dir.replace(portal_dir);
        }
    }
}

impl FromStr for PortalDirection {
    type Err = TileError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "N" => Ok(PortalDirection::North),
            "S" => Ok(PortalDirection::South),
            "E" => Ok(PortalDirection::East),
            "W" => Ok(PortalDirection::West),
            _ => Err(TileError::BoolParseFail(s.to_owned())),
        }
    }
}
