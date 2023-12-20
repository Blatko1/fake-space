#[cfg(test)]
mod tests;

mod error;
pub mod parse_error;
mod parser;

use std::{ops::RangeInclusive, path::PathBuf, str::FromStr};

use hashbrown::HashMap;
use image::{io::Reader as ImageReader, EncodableLayout};

use crate::textures::{Texture, TextureData};
use crate::world::map::MapTile;

use parse_error::{
    DimensionsError, DirectiveError, MapParseError, TextureError, TileError,
};

const MAP_TILE_LEVEL1_DEFAULT: f32 = -100.0;
const MAP_TILE_LEVEL2_DEFAULT: f32 = -1.0;
const MAP_TILE_LEVEL3_DEFAULT: f32 = f32::MAX;
const MAP_TILE_LEVEL4_DEFAULT: f32 = f32::MAX;

pub struct MapParser {
    src_path: PathBuf,
    data: String,
    texture_indices: HashMap<String, Texture>,
    variables: HashMap<String, MapTileVariable>,
}

impl<'a> MapParser {
    pub fn from_path<P: Into<PathBuf>>(
        src_path: P,
    ) -> Result<Self, MapParseError> {
        let src_path: PathBuf = src_path.into().canonicalize()?;
        let data = std::fs::read_to_string(src_path.clone())?;
        Ok(Self {
            src_path: src_path.parent().unwrap().to_path_buf(),
            data,
            texture_indices: HashMap::new(),
            variables: HashMap::new(),
        })
    }

    pub fn parse(
        mut self,
    ) -> Result<((usize, usize), Vec<MapTile>, Vec<TextureData>), MapParseError>
    {
        let mut lines = self
            .data
            .lines()
            .enumerate()
            .map(|(i, line)| (i, line.split("//").next().unwrap().trim()))
            .filter(|(_, line)| !line.is_empty());

        if are_directives_repeating(lines.clone()) {
            return Err(DirectiveError::MultipleSameDirectives)?;
        }

        // Parse the first line
        let dimensions = match lines.next() {
            Some((i, l)) => Self::parse_dimensions(i, l)?,
            None => return Err(DimensionsError::MissingDimensions)?,
        };

        let map_size = dimensions.0 * dimensions.1;
        let content_line_count = lines.clone().count();
        let mut textures = Vec::new();
        let mut tiles = vec![MapTileInput::Undefined; map_size];

        let mut lines = lines.enumerate();
        while let Some((index, (real_index, line))) = lines.next() {
            if !is_directive(line) {
                return Err(MapParseError::UndefinedExpression(
                    real_index,
                    line.to_string(),
                ));
            }
            let expressions_count = lines
                .clone()
                .find(|(_, (_, l))| is_directive(l))
                .map(|(i, (_, _))| i - (index + 1))
                .unwrap_or(content_line_count - (index + 1));
            let expressions = lines
                .clone()
                .take(expressions_count)
                .map(|(_, (i_real, l))| (i_real, l));
            lines.by_ref().take(expressions_count).for_each(drop);

            let directive = Self::parse_directive(real_index, line)?;
            match directive {
                Directive::Textures => {
                    let (texs, indices) = self.parse_textures(expressions)?;

                    textures = texs;
                    self.texture_indices = indices;
                }
                Directive::Variables => {
                    let variables = self.parse_variables(expressions)?;
                    self.variables = variables;
                }
                Directive::Tiles => {
                    self.parse_tiles(&mut tiles, expressions)?;
                }
            }
        }
        if let Some(undefined_tile_index) = tiles.iter().position(|t| match t {
            MapTileInput::Undefined => true,
            MapTileInput::Tile(_) => false,
        }) {
            return Err(MapParseError::UndefinedTileIndex(
                undefined_tile_index + 1,
            ));
        }
        let tiles = tiles
            .into_iter()
            .map(|tile| match tile {
                MapTileInput::Tile(t) => t,
                _ => unreachable!(),
            })
            .collect();

        Ok((dimensions, tiles, textures))
    }

    pub(super) fn parse_dimensions(
        index: usize,
        content: &str,
    ) -> Result<(usize, usize), DimensionsError> {
        // There should be only one 'x' separator and only one
        // value to the left and one to the right.
        let operands: Vec<&str> = content.split('x').collect();
        if operands.len() != 2 {
            return Err(DimensionsError::InvalidSeparatorFormat(index));
        }
        let d1_str = operands.first().unwrap().trim();
        let d2_str = operands.last().unwrap().trim();

        let (d1, d2) = match (d1_str.parse(), d2_str.parse()) {
            (Ok(d1), Ok(d2)) => (d1, d2),
            _ => return Err(DimensionsError::InvalidDimensionValue(index)),
        };
        if d1 == 0 || d2 == 0 {
            return Err(DimensionsError::InvalidDimensionValue(index));
        }
        Ok((d1, d2))
    }

    pub(super) fn parse_directive(
        index: usize,
        content: &str,
    ) -> Result<Directive, DirectiveError> {
        if !is_directive(content) {
            return Err(DirectiveError::InvalidDirective(
                index,
                content.to_string(),
            ));
        }
        Directive::from_str(index, &content[1..])
    }

    // TODO get rid of asserts afterwards maybe
    // TODO make it so index is needed to be given to err only at the main callsite.
    // TODO remember: if there are multiple same definitions for same tile,
    //                the last definition will be taken
    fn parse_tiles<I: Iterator<Item = (usize, &'a str)> + Clone>(
        &self,
        tiles: &mut [MapTileInput],
        lines: I,
    ) -> Result<(), TileError> {
        for (index, content) in lines {
            let operands: Vec<&str> = content.split('=').collect();
            if operands.len() != 2 {
                return Err(TileError::InvalidSeparator(index));
            }
            let tile_index = operands[0];
            let expressions = operands[1];

            let mut tile = MapTileVariable::default();
            for expr in expressions.split_whitespace() {
                let operands: Vec<&str> = expr.split(':').collect();
                match operands[..] {
                    [expr] => {
                        if let Some(var_str) = expr.strip_prefix('$') {
                            match self.variables.get(var_str) {
                                Some(var) => {
                                    tile.update(var);
                                    continue;
                                }
                                None => {
                                    return Err(TileError::UnknownVariable(
                                        index,
                                        var_str.to_string(),
                                    ))
                                }
                            }
                        } else {
                            return Err(TileError::InvalidExpression(
                                index,
                                expr.to_string(),
                            ));
                        }
                    }

                    [_, _] => (),
                    _ => {
                        return Err(TileError::InvalidExpression(
                            index,
                            expr.to_string(),
                        ))?
                    }
                }
                let parameter = operands[0];
                let value = operands[1];

                match parameter.to_lowercase().as_str() {
                    "pillar1" => {
                        tile.pillar1_tex =
                            Some(self.get_texture_id(index, value)?)
                    }
                    "pillar2" => {
                        tile.pillar2_tex =
                            Some(self.get_texture_id(index, value)?)
                    }
                    "bottom" => {
                        tile.bottom_platform =
                            Some(self.get_texture_id(index, value)?)
                    }
                    "top" => {
                        tile.top_platform =
                            Some(self.get_texture_id(index, value)?)
                    }
                    "lvl1" => tile.level1 = Some(parse_float(index, value)?),
                    "lvl2" => tile.level2 = Some(parse_float(index, value)?),
                    "lvl3" => tile.level3 = Some(parse_float(index, value)?),
                    "lvl4" => tile.level4 = Some(parse_float(index, value)?),
                    _ => {
                        return Err(TileError::UnknownParameter(
                            index,
                            parameter.to_string(),
                        ))
                    }
                }
            }
            let tile = tile.to_map_tile();
            if tile.level1 > tile.level2
                || tile.level2 > tile.level3
                || tile.level3 > tile.level4
            {
                return Err(TileError::InvalidLevels(
                    index,
                    tile.level1,
                    tile.level2,
                    tile.level3,
                ));
            }
            if tile_index.split_whitespace().count() != 1 {
                return Err(TileError::InvalidTileIndex(index));
            }
            let tile_index_str = tile_index.split_whitespace().next().unwrap();
            if tile_index_str == "_" {
                tiles
                    .iter_mut()
                    .filter(|t| matches!(t, MapTileInput::Undefined))
                    .for_each(|t| *t = MapTileInput::Tile(tile));
                return Ok(());
            }
            let tile_index: RangeInclusive<usize> = if tile_index_str
                .contains('-')
            {
                let values: Vec<&str> = tile_index_str.split('-').collect();
                if values.len() != 2 {
                    return Err(TileError::InvalidTileIndexSeparator(index));
                }
                let first = values.first().unwrap();
                let last = values.last().unwrap();
                let Ok(from) = first.parse::<usize>() else {
                    return Err(TileError::FailedToParseTileIndex(
                        index,
                        first.to_string(),
                    ));
                };
                let Ok(to) = last.parse::<usize>() else {
                    return Err(TileError::FailedToParseTileIndex(
                        index,
                        last.to_string(),
                    ));
                };
                from..=to
            } else {
                match tile_index_str.parse::<usize>() {
                    Ok(i) => i..=i,
                    Err(_) => {
                        return Err(TileError::FailedToParseTileIndex(
                            index,
                            tile_index_str.to_string(),
                        ))
                    }
                }
            };
            for i in tile_index {
                match tiles.get_mut(i - 1) {
                    Some(t) => *t = MapTileInput::Tile(tile),
                    None => {
                        return Err(TileError::TileIndexExceedsLimits(index))
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_variables<I: Iterator<Item = (usize, &'a str)> + Clone>(
        &self,
        lines: I,
    ) -> Result<HashMap<String, MapTileVariable>, TileError> {
        let mut variables = HashMap::new();
        for (index, content) in lines {
            let operands: Vec<&str> = content.split('=').collect();
            if operands.len() != 2 {
                return Err(TileError::InvalidVariableSeparatorFormat(index));
            }
            let variable_name = operands[0];
            let expressions = operands[1];
            if variable_name.split_whitespace().count() != 1 {
                return Err(TileError::InvalidVariableFormat(index));
            }
            let variable_name =
                variable_name.split_whitespace().next().unwrap();
            if variables.contains_key(variable_name) {
                return Err(TileError::VariableNameAlreadyTaken(
                    index,
                    variable_name.to_string(),
                ));
            }

            let mut tile = MapTileVariable::default();
            for expr in expressions.split_whitespace() {
                let operands: Vec<&str> = expr.split(':').collect();
                match operands[..] {
                    [expr] => {
                        if let Some(var_str) = expr.strip_prefix('$') {
                            match variables.get(var_str) {
                                Some(var) => {
                                    tile.update(var);
                                    continue;
                                }
                                None => {
                                    return Err(TileError::UnknownVariable(
                                        index,
                                        var_str.to_string(),
                                    ))
                                }
                            }
                        } else {
                            return Err(TileError::InvalidExpression(
                                index,
                                expr.to_string(),
                            ));
                        }
                    }

                    [_, _] => (),
                    _ => {
                        return Err(TileError::InvalidExpression(
                            index,
                            expr.to_string(),
                        ))?
                    }
                }
                let parameter = operands[0];
                let value = operands[1];

                match parameter.to_lowercase().as_str() {
                    "pillar1" => {
                        tile.pillar1_tex =
                            Some(self.get_texture_id(index, value)?)
                    }
                    "pillar2" => {
                        tile.pillar2_tex =
                            Some(self.get_texture_id(index, value)?)
                    }
                    "bottom" => {
                        tile.bottom_platform =
                            Some(self.get_texture_id(index, value)?)
                    }
                    "top" => {
                        tile.top_platform =
                            Some(self.get_texture_id(index, value)?)
                    }
                    "lvl1" => tile.level1 = Some(parse_float(index, value)?),
                    "lvl2" => tile.level2 = Some(parse_float(index, value)?),
                    "lvl3" => tile.level3 = Some(parse_float(index, value)?),
                    "lvl4" => tile.level4 = Some(parse_float(index, value)?),
                    _ => {
                        return Err(TileError::UnknownParameter(
                            index,
                            parameter.to_string(),
                        ))
                    }
                }
            }
            let test = tile.to_map_tile();
            if test.level1 > test.level2
                || test.level2 > test.level3
                || test.level3 > test.level4
            {
                return Err(TileError::InvalidLevels(
                    index,
                    test.level1,
                    test.level2,
                    test.level3,
                ));
            }
            variables.insert(variable_name.to_string(), tile);
        }

        Ok(variables)
    }

    pub(super) fn parse_textures<
        I: Iterator<Item = (usize, &'a str)> + Clone,
    >(
        &self,
        line: I,
    ) -> Result<(Vec<TextureData>, HashMap<String, Texture>), TextureError>
    {
        let mut textures = Vec::with_capacity(line.clone().count());
        let mut texture_indices = HashMap::new();

        for (texture_index, (real_index, content)) in line.enumerate() {
            let operands: Vec<&str> = content.split('=').collect();
            if operands.len() != 2 {
                return Err(TextureError::InvalidSeparatorFormat(real_index));
            }
            let texture_name = operands[0];
            let expressions = operands[1];
            if texture_name.split_whitespace().count() != 1 {
                return Err(TextureError::TextureNameContainsWhitespace(
                    real_index,
                    texture_name.to_string(),
                ));
            }
            let texture_name = texture_name.split_whitespace().next().unwrap();
            if texture_indices.contains_key(texture_name) {
                return Err(TextureError::TextureNameAlreadyTaken(
                    real_index,
                    texture_name.to_string(),
                ));
            }

            let mut texture_data = None;
            let mut transparency = None;
            let mut repeating = None;
            for expr in expressions.split_whitespace() {
                let operands: Vec<&str> = expr.split(':').collect();
                if operands.len() != 2 {
                    return Err(TextureError::InvalidOperandSeparatorFormat(
                        real_index,
                    ));
                }
                let parameter = operands[0];
                let value = operands[1];

                match parameter {
                    "path" => {
                        let full_path = self.src_path.join(value);
                        texture_data =
                            Some(ImageReader::open(full_path)?.decode()?);
                    }
                    "transparency" => {
                        transparency = match value.parse::<bool>() {
                            Ok(b) => Some(b),
                            Err(_) => {
                                return Err(
                                    TextureError::FailedToParseBoolValue(
                                        real_index,
                                        value.to_string(),
                                    ),
                                )
                            }
                        }
                    }
                    "repeating" => {
                        repeating = match value.parse::<bool>() {
                            Ok(b) => Some(b),
                            Err(_) => {
                                return Err(
                                    TextureError::FailedToParseBoolValue(
                                        real_index,
                                        value.to_string(),
                                    ),
                                )
                            }
                        }
                    }
                    _ => {
                        return Err(TextureError::UnknownParameter(
                            real_index,
                            parameter.to_string(),
                        ))
                    }
                }
            }
            let Some(texture_data) = texture_data else {
                return Err(TextureError::TextureSrcNotSpecified(real_index));
            };
            let Some(transparency) = transparency else {
                return Err(TextureError::TextureTransparencyNotSpecified(
                    real_index,
                ));
            };
            let Some(repeating) = repeating else {
                return Err(TextureError::TextureRepetitionNotSpecified(
                    real_index,
                ));
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
        }
        assert_eq!(textures.len(), texture_indices.len());
        Ok((textures, texture_indices))
    }

    fn get_texture_id(
        &self,
        index: usize,
        tex: &str,
    ) -> Result<Texture, TileError> {
        if tex == "0" {
            return Ok(Texture::Empty);
        }
        match self.texture_indices.get(tex) {
            Some(id) => Ok(*id),
            None => Err(TileError::UnknownTexture(index, tex.to_string())),
        }
    }
}

fn is_directive(line: &str) -> bool {
    line.starts_with('#')
}

fn are_directives_repeating<
    'a,
    I: Iterator<Item = (usize, &'a str)> + Clone,
>(
    lines: I,
) -> bool {
    match (
        lines
            .clone()
            .map(|(_, l)| l.matches("#variables").count())
            .sum(),
        lines
            .clone()
            .map(|(_, l)| l.matches("#tiles").count())
            .sum(),
        lines.map(|(_, l)| l.matches("#textures").count()).sum(),
    ) {
        (0 | 1, 0 | 1, 0 | 1) => false,
        _ => true,
    }
}

fn parse_float<P: FromStr>(index: usize, s: &str) -> Result<P, TileError> {
    match s.parse() {
        Ok(p) => Ok(p),
        Err(_) => Err(TileError::FloatParseError(index, s.to_string())),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Directive {
    Textures,
    Variables,
    Tiles,
}

impl Directive {
    fn from_str(index: usize, s: &str) -> Result<Self, DirectiveError> {
        match s {
            "textures" => Ok(Self::Textures),
            "variables" => Ok(Self::Variables),
            "tiles" => Ok(Self::Tiles),
            _ => Err(DirectiveError::UnknownDirective(index, s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum MapTileInput {
    Undefined,
    Tile(MapTile),
}

#[derive(Debug, Default, Clone, Copy)]
struct MapTileVariable {
    pub pillar1_tex: Option<Texture>,
    pub pillar2_tex: Option<Texture>,
    pub bottom_platform: Option<Texture>,
    pub top_platform: Option<Texture>,
    pub level1: Option<f32>,
    pub level2: Option<f32>,
    pub level3: Option<f32>,
    pub level4: Option<f32>,
}
// TODO rename top height and bottom height to top_y and bot_y
impl MapTileVariable {
    fn to_map_tile(self) -> MapTile {
        MapTile {
            pillar1_tex: self.pillar1_tex.unwrap_or_default(),
            pillar2_tex: self.pillar2_tex.unwrap_or_default(),
            bottom_platform_tex: self.bottom_platform.unwrap_or_default(),
            top_platform_tex: self.top_platform.unwrap_or_default(),
            level1: self.level1.unwrap_or(MAP_TILE_LEVEL1_DEFAULT),
            level2: self.level2.unwrap_or(MAP_TILE_LEVEL2_DEFAULT),
            level3: self.level3.unwrap_or(MAP_TILE_LEVEL3_DEFAULT),
            level4: self.level4.unwrap_or(MAP_TILE_LEVEL4_DEFAULT),
        }
    }

    fn update(&mut self, var: &MapTileVariable) {
        if let Some(i) = var.pillar1_tex {
            self.pillar1_tex.replace(i);
        }
        if let Some(i) = var.pillar2_tex {
            self.pillar2_tex.replace(i);
        }
        if let Some(i) = var.bottom_platform {
            self.bottom_platform.replace(i);
        }
        if let Some(i) = var.top_platform {
            self.top_platform.replace(i);
        }
        if let Some(i) = var.level1 {
            self.level1.replace(i);
        }
        if let Some(i) = var.level2 {
            self.level2.replace(i);
        }
        if let Some(i) = var.level3 {
            self.level3.replace(i);
        }
        if let Some(i) = var.level4 {
            self.level4.replace(i);
        }
    }
}
