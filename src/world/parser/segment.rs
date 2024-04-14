use std::str::FromStr;

use crate::player::render::PointXZ;
use crate::voxel::VoxelModelID;
use hashbrown::HashMap;

use crate::world::portal::{DummyPortal, PortalDirection, PortalID};
use crate::world::textures::TextureID;
use crate::world::Tile;

use super::error::RowError;
use super::{
    error::{DimensionError, PresetError, SegmentParseError},
    Settings,
};

#[derive(Debug)]
pub(super) struct SegmentParser<'a> {
    dimensions: (u64, u64),
    data: &'a str,
    settings: &'a Settings,

    preset_map: HashMap<String, TilePreset>,
    texture_map: &'a HashMap<String, TextureID>,
    tiles: Vec<TilePreset>,
    processed_tiles: usize,
}

impl<'a> SegmentParser<'a> {
    pub(super) fn new(
        data: &'a str,
        settings: &'a Settings,
        texture_map: &'a HashMap<String, TextureID>,
    ) -> Self {
        Self {
            dimensions: (0, 0),
            data,
            settings,

            preset_map: HashMap::new(),
            texture_map,
            tiles: Vec::new(),
            processed_tiles: 0,
        }
    }
    pub(super) fn parse(mut self) -> Result<((u64, u64), Vec<Tile>), SegmentParseError> {
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
        self.dimensions = dimensions;
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
                '|' => {
                    if let Err(e) = self.parse_segment_row(line) {
                        return Err(SegmentParseError::RowErr(e, i));
                    }
                }
                _ => return Err(SegmentParseError::UnknownKey(key.to_string(), i)),
            };
        }

        let mut tiles = Vec::with_capacity(self.tiles.len());
        let mut portal_id = 0;
        for (i, tile) in self.tiles.into_iter().enumerate() {
            // Replace None values with Default values, then compare levels
            // to find errors (lvl1 <= lvl2 < lvl3 <= lvl4)
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
            let position = PointXZ {
                x: i as u64 % dimensions.0,
                z: i as u64 / dimensions.0,
            };
            let voxel_model = if rand::random() && rand::random() && rand::random() {
                Some(VoxelModelID::Damaged)
            } else {
                None
            };
            let t = Tile {
                position,
                bottom_wall_tex: tile.bottom_wall_tex.unwrap_or_default(),
                top_wall_tex: tile.top_wall_tex.unwrap_or_default(),
                ground_tex: tile.ground_tex.unwrap_or_default(),
                ceiling_tex: tile.ceiling_tex.unwrap_or_default(),
                bottom_level,
                ground_level,
                ceiling_level,
                top_level,
                portal,
                // TODO this is temporary hard-coded
                voxel_model: None,
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

        let mut preset = TilePreset::default();
        for expr in expressions.split(',') {
            // Split the expression and check for formatting errors
            let operands: Vec<&str> = expr.trim().split(':').collect();
            match operands[..] {
                [""] => continue,
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
                                return Err(PresetError::UnknownPreset(
                                    preset_str.to_owned(),
                                ))
                            }
                        }
                    } else {
                        return Err(PresetError::InvalidExpressionFormat(
                            expr.to_owned(),
                        ));
                    }
                }
                [_, _] => (),
                _ => return Err(PresetError::InvalidExpressionFormat(expr.to_owned())),
            }
            let parameter = operands[0].trim();
            let value = operands[1].trim();

            // Identify the parameter and act accordingly
            match parameter {
                // If the parameter is one of these, the value should be a *texture name*
                "bottomT" | "topT" | "groundT" | "ceilingT" => {
                    let Some(&texture) = self.texture_map.get(value) else {
                        return Err(PresetError::UnknownTexture(value.to_owned()));
                    };
                    match parameter {
                        "bottomT" => preset.bottom_wall_tex = Some(texture),
                        "topT" => preset.top_wall_tex = Some(texture),
                        "groundT" => preset.ground_tex = Some(texture),
                        "ceilingT" => preset.ceiling_tex = Some(texture),
                        _ => unreachable!(),
                    }
                }
                // If the parameter is one of these, the value should be a *number*
                "bottomL" | "groundL" | "ceilingL" | "topL" => {
                    let Ok(parsed) = value.parse::<f32>() else {
                        return Err(PresetError::FloatParseFail(value.to_string()));
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
                _ => return Err(PresetError::UnknownParameter(parameter.to_owned())),
            }
        }

        Ok((identifier.to_owned(), preset))
    }

    fn parse_segment_row(&mut self, line: &str) -> Result<(), RowError> {
        let filtered = line.replace('|', "");
        let tiles = filtered.split_whitespace();
        if self.processed_tiles as u64 >= self.dimensions.1 {
            return Err(RowError::SufficientRow);
        }
        if tiles.clone().count() as u64 != self.dimensions.0 {
            return Err(RowError::RowLengthNotMatchingDimension(
                tiles.count() as u64,
                self.dimensions.0,
            ));
        }

        for (i, tile) in tiles.enumerate() {
            let tile = match self.preset_map.get(tile) {
                Some(t) => t,
                None => return Err(RowError::TilePresetNonExistent(tile.to_owned())),
            };
            let index = (self.dimensions.1 as usize - self.processed_tiles - 1)
                * self.dimensions.0 as usize
                + i;
            let old_tile = self.tiles.get_mut(index).unwrap();
            old_tile.overwrite_with(tile);
        }
        self.processed_tiles += 1;

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
struct TilePreset {
    bottom_wall_tex: Option<TextureID>,
    top_wall_tex: Option<TextureID>,
    ground_tex: Option<TextureID>,
    ceiling_tex: Option<TextureID>,
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
        if let Some(bottom_wall_tex) = other.bottom_wall_tex {
            self.bottom_wall_tex.replace(bottom_wall_tex);
        }
        if let Some(top_wall_tex) = other.top_wall_tex {
            self.top_wall_tex.replace(top_wall_tex);
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
    type Err = PresetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "N" => Ok(PortalDirection::North),
            "S" => Ok(PortalDirection::South),
            "E" => Ok(PortalDirection::East),
            "W" => Ok(PortalDirection::West),
            _ => Err(PresetError::BoolParseFail(s.to_owned())),
        }
    }
}
