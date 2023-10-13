use std::{ops::{Range, RangeInclusive}, str::FromStr};

use hashbrown::HashMap;

use crate::map::parse_error::TileDefinitionError;

use super::{
    parse_error::{DimensionsError, DirectiveError, MapParseError},
    MapTile, ObjectType,
};

pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<MapTile>,
}

impl Map {
    pub fn from_file_str(data: &str) -> Result<Self, MapParseError> {
        let ((w, h), tiles) = parse(data)?;
        Ok(Self {
            width: w,
            height: h,
            tiles,
        })
    }
}

pub(super) fn parse(
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
        Some((i, l)) => parse_dimensions(i, l)?,
        None => return Err(DimensionsError::MissingDimensions)?,
    };

    let map_size = (dimensions.0 * dimensions.1) as usize;
    let mut tiles = Vec::with_capacity(map_size);
    let mut variables = HashMap::new();

    match lines
        .clone()
        .map(|(_, l)| l.matches("#variables").count())
        .sum()
    {
        0 | 1 => (),
        _ => return Err(DirectiveError::MultipleSameDirectives)?,
    }
    match lines
        .clone()
        .map(|(_, l)| l.matches("#tiles").count())
        .sum()
    {
        1 => (),
        0 => return Err(DirectiveError::MissingTilesDirective)?,
        _ => return Err(DirectiveError::MultipleSameDirectives)?,
    }
    let mut lines = lines.enumerate();
    while let Some((index, (real_index, line))) = lines.next() {
        if !is_directive(line) {
            return Err(MapParseError::Undefined(real_index, line.to_string()));
        }
        let expressions_count = lines.clone()
            .find(|(_, (_, l))| is_directive(l))
            .map(|(i, (_, _))| i-index-1)
            .unwrap_or(content_line_count - (index + 1));
          
        let expressions = lines
            .clone()
            .take(expressions_count)
            .map(|(_, (i_real, l))| (i_real, l));
        lines.by_ref().take(expressions_count).for_each(drop);

        let directive = parse_directive(real_index, line)?;
        match directive {
            Directive::Variables => {
                variables = parse_variables(expressions)?;
            }
            Directive::Tiles => {
                tiles = parse_tiles(expressions, map_size, &variables)?;
            }
        }
    }

    if tiles.len() != map_size {
        return Err(MapParseError::NotEnoughTiles(tiles.len(), map_size))
    }

    Ok((dimensions, tiles))
}

pub(super) fn parse_dimensions(
    index: usize,
    line: &str,
) -> Result<(usize, usize), DimensionsError> {
    // There should be only one 'x' separator and only one
    // value to the left and one to the right.
    let operands: Vec<&str> = line.split('x').collect();
    if operands.len() != 2 {
        return Err(DimensionsError::InvalidDimensions(index));
    }
    if operands.iter().any(|o| o.split_whitespace().count() != 1) {
        return Err(DimensionsError::InvalidDimensions(index));
    }
    let d1_str = operands.first().unwrap().split_whitespace().next().unwrap();
    let d2_str = operands.last().unwrap().split_whitespace().next().unwrap();
    // Check if there are any illegal characters is the line.
    if d1_str.chars().any(|c| !c.is_ascii_digit())
        || d2_str.chars().any(|c| !c.is_ascii_digit())
    {
        return Err(DimensionsError::IllegalCharacter(index));
    }
    let (d1, d2) = match (d1_str.parse(), d2_str.parse()) {
        (Ok(d1), Ok(d2)) => (d1, d2),
        _ => return Err(DimensionsError::InvalidDimensions(index)),
    };
    Ok((d1, d2))
}

pub(super) fn parse_directive(
    index: usize,
    line: &str,
) -> Result<Directive, DirectiveError> {
    if !is_directive(line) {
        return Err(DirectiveError::InvalidDirective(index));
    }
    let directive: Vec<&str> = line[1..].split_whitespace().collect();
    if directive.len() != 1 {
        return Err(DirectiveError::InvalidDirective(index));
    }
    let directive = match *directive.first().unwrap() {
        "variables" => Directive::Variables,
        "tiles" => Directive::Tiles,
        _ => return Err(DirectiveError::UnknownDirective(index)),
    };
    Ok(directive)
}

pub(super) fn parse_tiles<'a, I: Iterator<Item = (usize, &'a str)> + Clone>(
    lines: I,
    tile_count: usize,
    variables: &HashMap<&'a str, MapTileVariable>,
) -> Result<Vec<MapTile>, TileDefinitionError> {
    let mut tiles = Vec::with_capacity(tile_count);

    for (index, line) in lines {
        let operands: Vec<&str> = line.split('=').collect();
        if operands.len() != 2 {
            return Err(TileDefinitionError::InvalidFormat(index));
        }
        let left_operand: Vec<&str> =
            operands.first().unwrap().split_whitespace().collect();
        if left_operand.len() != 1 {
            return Err(TileDefinitionError::InvalidTileIndexFormat(index));
        }
        let tile_index =
            parse_tile_index(index, left_operand.first().unwrap())?;
        let first_index = tile_index.clone().next().unwrap();
        let expressions = operands.last().unwrap();

        if tiles.len() != first_index {
            return Err(TileDefinitionError::TileIndexNotContinuous(index));
        }

        let tile = parse_tile(index, expressions, variables)?.to_map_tile();
        if tile.is_none() {
            return Err(TileDefinitionError::MissingTileDefinitions(index));
        }
        for _i in tile_index {
            tiles.insert(_i, tile.unwrap())
            //tiles.push(tile);
        }
    }
    Ok(tiles)
}
// TODO get rid of asserts afterwards maybe
// TODO make it so index is needed to be given to err only at the main callsite.
// TODO remember: if there are multiple same definitions for same tile,
//                the last definition will be taken
pub(super) fn parse_tile(
    index: usize,
    line: &str,
    variables: &HashMap<&str, MapTileVariable>,
) -> Result<MapTileVariable, TileDefinitionError> {
    let mut tile = MapTileVariable::default();

    // Split the line into multiple words separated by whitespaces.
    for expr in line.split_whitespace() {
        // Split the expression into operands where as the separator
        // is considered a ':' sign. (e.g. obj:GRASS)
        let operands: Vec<&str> = expr.split(':').collect();
        match operands.len() {
            1 => {
                let key = operands.first().unwrap();
                match variables.get(key) {
                    Some(var) => {
                        update_if_some(&mut tile.object, var.object);
                        update_if_some(&mut tile.object_top, var.object_top);
                        update_if_some(
                            &mut tile.object_bottom,
                            var.object_bottom,
                        );
                        update_if_some(&mut tile.floor, var.floor);
                        update_if_some(&mut tile.ceiling, var.ceiling);
                        update_if_some(
                            &mut tile.obj_top_height,
                            var.obj_top_height,
                        );
                        update_if_some(
                            &mut tile.obj_bottom_height,
                            var.obj_bottom_height,
                        );
                    }
                    None => {
                        return Err(TileDefinitionError::UnknownVariable(
                            index,
                            key.to_string(),
                        ))
                    }
                }
                continue;
            }
            2 => (),
            _ => {
                return Err(TileDefinitionError::InvalidExpression(
                    index,
                    expr.to_string(),
                ))?
            }
        }
        let left = operands.first().unwrap();
        let right = operands.last().unwrap();

        match *left {
            // Object type definition:
            "o" | "obj" | "object" => {
                tile.object = Some(parse_object_type(index, right)?)
            }
            // Object top side type definition:
            "ot" | "o_top" | "obj_top" | "object_top" => {
                tile.object_top = Some(parse_object_type(index, right)?)
            }
            // Object bottom side type definition:
            "ob" | "o_bot" | "o_bottom" | "obj_bot" | "obj_bottom"
            | "object_bottom" => {
                tile.object_bottom = Some(parse_object_type(index, right)?)
            }
            // Floor type definition:
            "f" | "floor" => {
                tile.floor = Some(parse_object_type(index, right)?)
            }
            // Ceiling type definition:
            "c" | "ceiling" => {
                tile.ceiling = Some(parse_object_type(index, right)?)
            }
            // Top object part height value:
            "th" | "top_h" | "top_height" | "t_height" => {
                tile.obj_top_height = Some(parse_number(index, right)?)
            }
            // Bottom object part height value:
            "bh" | "bot_h" | "bottom_h" | "b_height" | "bot_height"
            | "bottom_height" => {
                tile.obj_bottom_height = Some(parse_number(index, right)?)
            }
            _ => {
                return Err(TileDefinitionError::UnknownLeftOperand(
                    index,
                    left.to_string(),
                ))
            }
        }
    }
    Ok(tile)
}

pub(super) fn parse_variables<
    'a,
    I: Iterator<Item = (usize, &'a str)> + Clone,
>(
    lines: I,
) -> Result<HashMap<&'a str, MapTileVariable>, TileDefinitionError> {
    let mut variables = HashMap::new();
    for (index, line) in lines {
        let operands: Vec<&str> = line.split('=').collect();
        if operands.len() != 2 {
            return Err(TileDefinitionError::InvalidVariableFormat(index));
        }
        let left_operand: Vec<&str> =
            operands.first().unwrap().split_whitespace().collect();
        if left_operand.len() != 1 {
            return Err(TileDefinitionError::InvalidVariableFormat(index));
        }
        let key = left_operand.first().unwrap();

        let expressions = operands.last().unwrap();
        let parsed_expressions = parse_tile(index, expressions, &variables)?;
        variables.insert(key, parsed_expressions);
    }

    Ok(variables)
}

pub(super) fn parse_tile_index(
    index: usize,
    operand: &str,
) -> Result<RangeInclusive<usize>, TileDefinitionError> {
    if operand.chars().any(|c| !matches!(c, '0'..='9' | '-')) {
        return Err(TileDefinitionError::IllegalTileIndexCharacter(index));
    }
    let tile_index: RangeInclusive<usize> = if operand.contains('-') {
        let values: Vec<&str> = operand.split('-').collect();
        if values.len() != 2 {
            return Err(TileDefinitionError::InvalidTileIndexFormat(index));
        }
        let first = values.first().unwrap();
        let last = values.last().unwrap();
        let from = match first.parse::<usize>() {
            Ok(from) => from,
            Err(_) => return Err(TileDefinitionError::FailedToParseTileIndex(index, first.to_string())),
        };
        let to = match last.parse::<usize>() {
            Ok(to) => to,
            Err(_) => return Err(TileDefinitionError::FailedToParseTileIndex(index, last.to_string())),
        };
        from..=to
    } else {
        match operand.parse::<usize>() {
            Ok(i) => i..=i,
            Err(_) => {
                return Err(TileDefinitionError::FailedToParseTileIndex(index, operand.to_string()))
            }
        }
    };
    let first = match tile_index.clone().next() {
        Some(f) => f,
        None => return Err(TileDefinitionError::InvalidTileIndexRange(index)),
    };
    let last = match tile_index.clone().last() {
        Some(l) => l,
        None => return Err(TileDefinitionError::InvalidTileIndexRange(index)),
    };
    if first > last || first == 0 {
        return Err(TileDefinitionError::InvalidTileIndexRange(index));
    }
    Ok(first.saturating_sub(1)..=last.saturating_sub(1))
}

pub(super) fn parse_number<P: FromStr>(
    index: usize,
    s: &str,
) -> Result<P, TileDefinitionError> {
    match s.parse() {
        Ok(p) => Ok(p),
        Err(_) => Err(TileDefinitionError::InvalidValueType(index)),
    }
}

pub(super) fn parse_object_type(
    index: usize,
    s: &str,
) -> Result<ObjectType, TileDefinitionError> {
    match s {
        "EMPTY" => Ok(ObjectType::Empty),
        "MOSSYSTONE" => Ok(ObjectType::MossyStone),
        "BLUEBRICK" => Ok(ObjectType::BlueBrick),
        "LIGHTPLANK" => Ok(ObjectType::LightPlank),
        "FENCE" => Ok(ObjectType::Fence),
        "BLUEGLASS" => Ok(ObjectType::BlueGlass),

        _ => {
            Err(TileDefinitionError::UnknownObjectType(
                index,
                s.to_string(),
            ))
        }
    }
}

/// Updates the `variable` if the input value is `Some`.
pub(super) fn update_if_some<T>(var: &mut Option<T>, input: Option<T>) {
    if input.is_some() {
        *var = input;
    }
}

pub(super) fn is_directive(line: &str) -> bool {
    line.starts_with('#')
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Directive {
    Variables,
    Tiles,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct MapTileVariable {
    pub object: Option<ObjectType>,
    pub object_top: Option<ObjectType>,
    pub object_bottom: Option<ObjectType>,
    pub floor: Option<ObjectType>,
    pub ceiling: Option<ObjectType>,
    pub obj_top_height: Option<f32>,
    pub obj_bottom_height: Option<f32>,
}

impl MapTileVariable {
    fn to_map_tile(self) -> Option<MapTile> {
        let tile = MapTile {
            object: self.object?,
            object_top: self.object_top?,
            object_bottom: self.object_bottom?,
            floor: self.floor?,
            ceiling: self.ceiling?,
            obj_top_height: self.obj_top_height?,
            obj_bottom_height: self.obj_bottom_height?,
        };
        Some(tile)
    }
}
