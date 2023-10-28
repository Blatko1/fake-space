use std::{
    hash::Hash,
    ops::{Range, RangeInclusive},
    path::{Path, PathBuf},
    str::FromStr,
};

use hashbrown::HashMap;
use image::{io::Reader as ImageReader, EncodableLayout};

use crate::textures::{Texture, TextureData};

use super::{
    parse_error::{
        DimensionsError, DirectiveError, MapParseError, TextureError, TileError,
    },
    MapTile
};

pub struct MapParser {
    src_path: PathBuf,
    data: String,
    texture_indices: HashMap<String, Texture>,
    variables: HashMap<String, MapTileVariable>,
}

impl<'a> MapParser {
    pub(super) fn from_path<P: Into<PathBuf>>(
        src_path: P,
    ) -> Result<Self, MapParseError> {
        let src_path: PathBuf = src_path.into().canonicalize()?;
        let data = std::fs::read_to_string(src_path.clone())?;
        Ok(Self {
            src_path: src_path.parent().unwrap().to_path_buf(),
            data,
            texture_indices: HashMap::new(),
            variables: HashMap::new()
        } )
    }
            
    pub(super) fn parse(
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

        let dimensions = match lines.next() {
            Some((i, l)) => Self::parse_dimensions(i, l)?,
            None => return Err(DimensionsError::MissingDimensions)?,
        };

        let map_size = (dimensions.0 * dimensions.1) as usize;
        let content_line_count = lines.clone().count();
        let mut textures = Vec::new();
        let mut tiles = Vec::with_capacity(map_size);

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
                    tiles = self.parse_tiles(expressions, map_size)?;
                }
            }
        }

        if tiles.len() != map_size {
            return Err(MapParseError::DimensionsAndTileCountNotMatching(
                tiles.len(),
                map_size,
            ));
        }

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
        if !is_directive(content) {
            return Err(DirectiveError::InvalidDirective(
                index,
                content.to_string(),
            ));
        }
        let directive = Directive::from_str(index, &content[1..])?;

        Ok(directive)
    }

    // TODO get rid of asserts afterwards maybe
    // TODO make it so index is needed to be given to err only at the main callsite.
    // TODO remember: if there are multiple same definitions for same tile,
    //                the last definition will be taken
    fn parse_tiles<I: Iterator<Item = (usize, &'a str)> + Clone>(
        &self,
        lines: I,
        tile_count: usize,
    ) -> Result<Vec<MapTile>, TileError> {
        let mut tiles = Vec::with_capacity(tile_count);

        for (index, content) in lines {
            let operands: Vec<&str> = content.split('=').collect();
            if operands.len() != 2 {
                return Err(TileError::InvalidSeparator(index));
            }
            let tile_index = operands[0];
            let expressions = operands[1];
            if tile_index.split_whitespace().count() != 1 {
                return Err(TileError::InvalidTileIndex(index));
            }
            let tile_index_str = tile_index.split_whitespace().next().unwrap();
            let tile_index = Self::parse_tile_index(index, tile_index_str)?;
            let first_index = tile_index.clone().next().unwrap();

            if tiles.len() != first_index {
                return Err(TileError::TileIndexNotContinuous(
                    index,
                    tile_index_str.to_string(),
                ));
            }

            let mut tile = MapTileVariable::default();
            for expr in expressions.split_whitespace() {
                let operands: Vec<&str> = expr.split(':').collect();
                match operands[..] {
                    [expr] => {
                        if expr.starts_with('$') {
                            let variable = &expr[1..];
                            match self.variables.get(variable) {
                                Some(variable) => {
                                    tile.update(variable);
                                    continue;
                                }
                                None => {
                                    return Err(TileError::UnknownVariable(
                                        index,
                                        variable.to_string(),
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
                    // Top object part height value:
                    "toph" => {
                        tile.obj_top_height = Some(parse_float(index, value)?)
                    }
                    // Bottom object part height value:
                    "both" => {
                        tile.obj_bottom_height =
                            Some(parse_float(index, value)?)
                    }
                    // Object type definition:
                    "obj" => {
                        tile.object = Some(self.get_texture_id(index, value)?)
                    }
                    // Object top side type definition:
                    "top" => {
                        tile.object_top =
                            Some(self.get_texture_id(index, value)?)
                    }
                    // Object bottom side type definition:
                    "bot" => {
                        tile.object_bottom =
                            Some(self.get_texture_id(index, value)?)
                    }
                    // Floor type definition:
                    "flr" => {
                        tile.floor = Some(self.get_texture_id(index, value)?)
                    }
                    // Ceiling type definition:
                    "clg" => {
                        tile.ceiling = Some(self.get_texture_id(index, value)?)
                    }
                    _ => {
                        return Err(TileError::UnknownParameter(
                            index,
                            parameter.to_string(),
                        ))
                    }
                }
            }
            let tile = tile.to_map_tile(index);
            for _i in tile_index {
                tiles.insert(_i, tile)
                //tiles.push(tile);
            }
        }
        Ok(tiles)
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
                        if expr.starts_with('$') {
                            let variable = &expr[1..];
                            match variables.get(variable) {
                                Some(variable) => {
                                    tile.update(variable);
                                    continue;
                                }
                                None => {
                                    return Err(TileError::UnknownVariable(
                                        index,
                                        variable.to_string(),
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
                    // Top object part height value:
                    "toph" => {
                        tile.obj_top_height = Some(parse_float(index, value)?)
                    }
                    // Bottom object part height value:
                    "both" => {
                        tile.obj_bottom_height =
                            Some(parse_float(index, value)?)
                    }
                    // Object type definition:
                    "obj" => {
                        tile.object = Some(self.get_texture_id(index, value)?)
                    }
                    // Object top side type definition:
                    "top" => {
                        tile.object_top =
                            Some(self.get_texture_id(index, value)?)
                    }
                    // Object bottom side type definition:
                    "bot" => {
                        tile.object_bottom =
                            Some(self.get_texture_id(index, value)?)
                    }
                    // Floor type definition:
                    "flr" => {
                        tile.floor = Some(self.get_texture_id(index, value)?)
                    }
                    // Ceiling type definition:
                    "clg" => {
                        tile.ceiling = Some(self.get_texture_id(index, value)?)
                    }
                    _ => {
                        return Err(TileError::UnknownParameter(
                            index,
                            parameter.to_string(),
                        ))
                    }
                }
            }
            variables.insert(variable_name.to_string(), tile);
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
        I: Iterator<Item = (usize, &'a str)> + Clone,
    >(
        &self,
        line: I,
    ) -> Result<(Vec<TextureData>, HashMap<String, Texture>), TextureError> {
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
                        if !value.starts_with('\"') || !value.ends_with('\"'){                  
                            return Err(TextureError::InvalidTexturePath(
                                real_index,
                                value.to_string(),
                            ));
                        }
                        let path_split: Vec<&str> = value.split('"').collect();
                        if path_split.len() != 3 {
                            return Err(TextureError::InvalidTexturePath(
                                real_index,
                                value.to_string(),
                            ));
                        }
                        let path = path_split[1];
                        let full_path = self.src_path.join(path);
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
            let texture = TextureData {
                data: texture_data.to_rgba8().as_bytes().to_vec(),
                width: texture_data.width(),
                height: texture_data.height(),
                transparency,
            };
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
            return Ok(Texture::Empty)
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

fn parse_float<P: FromStr>(
    index: usize,
    s: &str,
) -> Result<P, TileError> {
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

#[derive(Debug, Default)]
struct MapTileVariable {
    pub object: Option<Texture>,
    pub object_top: Option<Texture>,
    pub object_bottom: Option<Texture>,
    pub floor: Option<Texture>,
    pub ceiling: Option<Texture>,
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
