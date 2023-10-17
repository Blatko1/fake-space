use std::{
    hash::Hash,
    ops::{Range, RangeInclusive},
    path::Path,
    str::FromStr,
};

use hashbrown::HashMap;
use image::io::Reader as ImageReader;

use super::{
    parse_error::{
        DimensionsError, DirectiveError, MapParseError, TextureError, TileError,
    },
    MapTile, TextureID,
};

pub struct MapParser {
    textures: HashMap<String, (Texture, usize)>,
    variables: HashMap<String, MapTileVariable>,
}

impl MapParser {
    pub(super) fn from_path<P: AsRef<Path>>(
        path: P,
    ) -> Result<Self, MapParseError> {
        let data = std::fs::read_to_string(path);
        Ok(Self {
            textures: HashMap::new(),
            variables: HashMap::new(),
        })
    }

    pub(super) fn parse(self) {}

    pub(super) fn parse_map(
        &mut self,
        data: &str,
    ) -> Result<((usize, usize), Vec<MapTile>), MapParseError> {
        // Split the input data into lines, remove the lines which
        // only contain comments, remove lines with no text,
        // remove the commented out parts from lines with content.
        let mut lines = data
            .lines()
            .enumerate()
            .map(|(i, line)| (i, line.split("//").next().unwrap().trim()))
            .filter(|(_, line)| !line.is_empty());

        let content_line_count = lines.clone().count();
        // Parse dimensions from the first content line:
        let dimensions = match lines.next() {
            Some((i, l)) => Self::parse_dimensions(i, l)?,
            None => return Err(DimensionsError::MissingDimensions)?,
        };

        let map_size = (dimensions.0 * dimensions.1) as usize;
        let mut tiles = Vec::with_capacity(map_size);

        match (
            lines
                .clone()
                .map(|(_, l)| l.matches("#variables").count())
                .sum(),
            lines
                .clone()
                .map(|(_, l)| l.matches("#tiles").count())
                .sum(),
        ) {
            (0 | 1, 0 | 1) => (),
            _ => return Err(DirectiveError::MultipleSameDirectives)?,
        }

        let mut lines = lines.enumerate();
        while let Some((index, (real_index, line))) = lines.next() {
            if !Self::is_directive(line) {
                return Err(MapParseError::Undefined(
                    real_index,
                    line.to_string(),
                ));
            }
            let expressions_count = lines
                .clone()
                .find(|(_, (_, l))| Self::is_directive(l))
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
                    let (textures, texture_indices) = Self::parse_textures(expressions)?;
                }
                Directive::Variables => {
                    //let variables = Self::parse_variables(expressions, &texture_indices)?;
                }
                Directive::Tiles => {
                    //let tiles = Self::parse_tiles(
                    //    expressions,
                    //    map_size,
                    //    &variables,
                    //    &texture_indices,
                    //)?;
                }
            }
        }

        if tiles.len() != map_size {
            return Err(MapParseError::DimensionsAndTileCountNotMatching(
                tiles.len(),
                map_size,
            ));
        }

        Ok((dimensions, tiles))
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
        // Check if there are any illegal characters within the line.
        if d1_str.chars().any(|c| !c.is_ascii_digit())
            || d2_str.chars().any(|c| !c.is_ascii_digit())
        {
            return Err(DimensionsError::IllegalCharacter(index));
        }
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
        if !Self::is_directive(content) {
            return Err(DirectiveError::InvalidDirective(
                index,
                content.to_string(),
            ));
        }
        let directive = Directive::from_str(index, &content[1..])?;

        Ok(directive)
    }

    fn parse_tiles<'a, I: Iterator<Item = (usize, &'a str)> + Clone>(
        lines: I,
        tile_count: usize,
        variables: &HashMap<&'a str, MapTileVariable>,
        textures: &HashMap<&str, TextureID>,
    ) -> Result<Vec<MapTile>, TileError> {
        let mut tiles = Vec::with_capacity(tile_count);

        for (index, content) in lines {
            let operands: Vec<&str> = content.split('=').collect();
            if operands.len() != 2 {
                return Err(TileError::InvalidSeparator(index));
            }
            let left_operand: Vec<&str> =
                operands.first().unwrap().split_whitespace().collect();
            if left_operand.len() != 1 {
                return Err(TileError::TileIndexContainsWhiteSpaces(index));
            }
            let tile_index_str = left_operand.first().unwrap();
            let tile_index = Self::parse_tile_index(index, tile_index_str)?;
            let first_index = tile_index.clone().next().unwrap();
            let expressions = operands.last().unwrap();

            if tiles.len() != first_index {
                return Err(TileError::TileIndexNotContinuous(
                    index,
                    tile_index_str.to_string(),
                ));
            }

            let tile =
            Self::parse_tile_variable(index, expressions, variables, textures)?
                    .to_map_tile(index);
            for _i in tile_index {
                tiles.insert(_i, tile)
                //tiles.push(tile);
            }
        }
        Ok(tiles)
    }
    // TODO get rid of asserts afterwards maybe
    // TODO make it so index is needed to be given to err only at the main callsite.
    // TODO remember: if there are multiple same definitions for same tile,
    //                the last definition will be taken
    fn parse_tile_variable(
        index: usize,
        content: &str,
        variables: &HashMap<&str, MapTileVariable>,
        textures: &HashMap<&str, TextureID>,
    ) -> Result<MapTileVariable, TileError> {
        let mut tile = MapTileVariable::default();

        for expr in content.split_whitespace() {
            let operands: Vec<&str> = expr.split(':').collect();
            match operands[..] {
                [key] => {
                    match variables.get(key) {
                        Some(variable) => tile.update(variable),
                        None => {
                            return Err(TileError::UnknownVariable(
                                index,
                                key.to_string(),
                            ))
                        }
                    }
                    continue;
                }
                [_, _] => (),
                _ => {
                    return Err(TileError::InvalidExpression(
                        index,
                        expr.to_string(),
                    ))?
                }
            }
            let left = operands.first().unwrap();
            let right = operands.last().unwrap();

            match *left {
                // Top object part height value:
                "th" | "top_h" | "top_height" | "t_height" => {
                    tile.obj_top_height = Some(Self::parse_float(index, right)?)
                }
                // Bottom object part height value:
                "bh" | "bot_h" | "bottom_h" | "b_height" | "bot_height"
                | "bottom_height" => {
                    tile.obj_bottom_height = Some(Self::parse_float(index, right)?)
                }
                requires_str => {
                    let texture_id = match textures.get(right) {
                        Some(t) => *t,
                        None => {
                            return Err(TileError::UnknownTextureKey(
                                index,
                                right.to_string(),
                            ))
                        }
                    };
                    match requires_str {
                        // Object type definition:
                        "o" | "obj" | "object" => {
                            tile.object = Some(texture_id)
                        }
                        // Object top side type definition:
                        "ot" | "o_top" | "obj_top" | "object_top" => {
                            tile.object_top = Some(texture_id)
                        }
                        // Object bottom side type definition:
                        "ob" | "o_bot" | "o_bottom" | "obj_bot"
                        | "obj_bottom" | "object_bottom" => {
                            tile.object_bottom = Some(texture_id)
                        }
                        // Floor type definition:
                        "f" | "floor" => tile.floor = Some(texture_id),
                        // Ceiling type definition:
                        "c" | "ceiling" => tile.ceiling = Some(texture_id),
                        _ => {
                            return Err(TileError::UnknownLeftOperand(
                                index,
                                left.to_string(),
                            ))
                        }
                    }
                }
            }
        }
        Ok(tile)
    }

    fn parse_variables<'a, I: Iterator<Item = (usize, &'a str)> + Clone>(
        lines: I,
        textures: &HashMap<&str, TextureID>,
    ) -> Result<HashMap<&'a str, MapTileVariable>, TileError> {
        let mut variables = HashMap::new();
        for (index, content) in lines {
            let operands: Vec<&str> = content.split('=').collect();
            if operands.len() != 2 {
                return Err(TileError::InvalidVariableFormat(index));
            }
            let left_operand: Vec<&str> =
                operands.first().unwrap().split_whitespace().collect();
            if left_operand.len() != 1 {
                return Err(TileError::InvalidVariableFormat(index));
            }
            let key = left_operand.first().unwrap();
            if variables.contains_key(key) {
                return Err(TileError::VariableNameAlreadyTaken(
                    index,
                    key.to_string(),
                ));
            }

            let expressions = operands.last().unwrap();
            let parsed_expressions =
            Self::parse_tile_variable(index, expressions, &variables, textures)?;
            variables.insert(key, parsed_expressions);
        }

        Ok(variables)
    }

    pub(super) fn parse_tile_index(
        index: usize,
        operand: &str,
    ) -> Result<RangeInclusive<usize>, TileError> {
        if let Some(invalid_char) =
            operand.chars().find(|c| !matches!(c, '0'..='9' | '-'))
        {
            return Err(TileError::IllegalTileIndexCharacter(
                index,
                invalid_char,
            ));
        }
        let tile_index: RangeInclusive<usize> = if operand.contains('-') {
            let values: Vec<&str> = operand.split('-').collect();
            if values.len() != 2 {
                return Err(TileError::InvalidTileIndexSeparator(index));
            }
            let first = values.first().unwrap();
            let last = values.last().unwrap();
            let from = match first.parse::<usize>() {
                Ok(from) => from,
                Err(_) => {
                    return Err(TileError::FailedToParseTileIndex(
                        index,
                        first.to_string(),
                    ))
                }
            };
            let to = match last.parse::<usize>() {
                Ok(to) => to,
                Err(_) => {
                    return Err(TileError::FailedToParseTileIndex(
                        index,
                        last.to_string(),
                    ))
                }
            };
            from..=to
        } else {
            match operand.parse::<usize>() {
                Ok(i) => i..=i,
                Err(_) => {
                    return Err(TileError::FailedToParseTileIndex(
                        index,
                        operand.to_string(),
                    ))
                }
            }
        };
        let first = match tile_index.clone().next() {
            Some(f) => f,
            None => {
                return Err(TileError::InvalidTileIndexRange(
                    index,
                    operand.to_string(),
                ))
            }
        };
        let last = match tile_index.clone().last() {
            Some(l) => l,
            None => {
                return Err(TileError::InvalidTileIndexRange(
                    index,
                    operand.to_string(),
                ))
            }
        };
        if first > last || first == 0 {
            return Err(TileError::InvalidTileIndexRange(
                index,
                operand.to_string(),
            ));
        }
        Ok(first.saturating_sub(1)..=last.saturating_sub(1))
    }

    pub(super) fn parse_textures<
        'a,
        I: Iterator<Item = (usize, &'a str)> + Clone,
    >(
        lines: I,
    ) -> Result<(Vec<Texture>, HashMap<&'a str, TextureID>), TextureError> {
        let textures = Vec::with_capacity(lines.clone().count());
        let texture_indices = HashMap::new();

        for (texture_index, (real_index, content)) in lines.enumerate() {
            let operands: Vec<&str> = content.split('=').collect();
            if operands.len() != 2 {
                return Err(TextureError::InvalidSeparatorFormat(real_index));
            }
            let left_operand: Vec<&str> =
                operands.first().unwrap().split_whitespace().collect();
            if left_operand.len() != 1 {
                return Err(TextureError::TextureSymbolContainsWhiteSpaces(
                    real_index,
                    operands.first().unwrap().to_string(),
                ));
            }
            let key = left_operand.first().unwrap();
            if texture_indices.contains_key(key) {
                return Err(TextureError::TextureNameAlreadyTaken(
                    real_index,
                    key.to_string(),
                ));
            }
        }

        Ok((textures, texture_indices))
    }

    fn parse_texture(path: &str) {
        let img = ImageReader::open(path);
    }

    pub(super) fn parse_float<P: FromStr>(
        index: usize,
        s: &str,
    ) -> Result<P, TileError> {
        match s.parse() {
            Ok(p) => Ok(p),
            Err(_) => Err(TileError::InvalidValueType(index)),
        }
    }

    pub(super) fn is_directive(line: &str) -> bool {
        line.starts_with('#')
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

#[derive(Debug)]
pub struct Texture {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
    has_transparency: bool,
}

#[derive(Debug)]
pub struct TextureRef<'a> {
    rgba: &'a [u8],
    width: u32,
    height: u32,
    has_transparency: bool,
}

#[derive(Debug, Default)]
struct MapTileVariable {
    pub object: Option<TextureID>,
    pub object_top: Option<TextureID>,
    pub object_bottom: Option<TextureID>,
    pub floor: Option<TextureID>,
    pub ceiling: Option<TextureID>,
    pub obj_top_height: Option<f32>,
    pub obj_bottom_height: Option<f32>,
}

impl MapTileVariable {
    fn to_map_tile(self, index: usize) -> MapTile {
        MapTile {
            object: self.object.unwrap_or_default(),
            object_top: self.object_top.unwrap_or_default(),
            object_bottom: self.object_bottom.unwrap_or_default(),
            floor: self.floor.unwrap_or_default(),
            ceiling: self.ceiling.unwrap_or_default(),
            obj_top_height: self.obj_top_height.unwrap_or(1.0),
            obj_bottom_height: self.obj_bottom_height.unwrap_or(1.0),
        }
    }

    fn update(&mut self, var: &MapTileVariable) {
        if let Some(i) = var.object {
            self.object.replace(i);
        }
        if let Some(i) = var.object_top {
            self.object_top.replace(i);
        }
        if let Some(i) = var.object_bottom {
            self.object_bottom.replace(i);
        }
        if let Some(i) = var.floor {
            self.floor.replace(i);
        }
        if let Some(i) = var.ceiling {
            self.ceiling.replace(i);
        }
        if let Some(i) = var.obj_top_height {
            self.obj_top_height.replace(i);
        }
        if let Some(i) = var.obj_bottom_height {
            self.obj_bottom_height.replace(i);
        }
    }
}
