use std::str::FromStr;

use crate::map::parse_error::TileDefinitionError;

use super::{
    parse_error::{DimensionsError, DirectiveError, MapParseError},
    MapTile,
};

struct Map {
    width: usize,
    height: usize,
    tiles: Vec<MapTile>,
}

impl Map {
    fn from_file_str(data: &str) -> Self {
        todo!()
    }
}

pub(super) fn parse(
    data: &str,
) -> Result<((u32, u32), Vec<MapTile>), MapParseError> {
    // Split the input data into lines, remove the lines which
    // only contain comments, remove lines with no text,
    // remove the commented out parts from lines with content.
    let mut lines = data
        .lines()
        .enumerate()
        .map(|(i, line)| (i, line.split("//").next().unwrap().trim()))
        .filter(|(_, line)| !line.is_empty());
    let content_line_count = lines.clone().count();
    // Parse dimensions:
    let dimensions = match lines.next() {
        Some((i, l)) => parse_dimensions(i, l)?,
        None => return Err(DimensionsError::MissingDimensions)?,
    };

    let map_size = (dimensions.0 * dimensions.1) as usize;
    let tiles = Vec::with_capacity(map_size);

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
    while let Some((index, (_real_index, line))) = lines.next() {
        if is_directive_word(line) {
            let lines_temp = lines.clone();
            let expressions_count = lines_temp
                .enumerate()
                .find(|(_, (_, (_, l)))| is_directive_word(l))
                .map(|(i, (_, (_, _)))| i)
                .unwrap_or(content_line_count - (index + 1));
            let expressions = lines
                .clone()
                .take(expressions_count)
                .map(|(_, (i_real, l))| (i_real, l));
            lines.nth(expressions_count - 1);
            
            let directive = parse_directive_word(index, line)?;
            match directive {
                DirectiveWord::Variables => todo!(),
                DirectiveWord::Tiles => {
                    parse_tiles(expressions)?
                }
            }
            
        }
    }

    //if tiles == 0 {
    //    return Err(/*Nema podataka za tile */)
    //}

    Ok((dimensions, tiles))
}

pub(super) fn parse_dimensions(
    index: usize,
    line: &str,
) -> Result<(u32, u32), DimensionsError> {
    // There should be only one 'x' separator and only one
    // value to the left and one to the right.
    let operands: Vec<&str> = line.split('x').collect();
    if operands.len() != 2 {
        return Err(DimensionsError::InvalidDimensions(index));
    }
    if operands
        .iter()
        .any(|o| o.split_whitespace().count() != 1)
    {
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

pub(super) fn parse_directive_word(
    index: usize,
    line: &str,
) -> Result<DirectiveWord, DirectiveError> {
    if !is_directive_word(line) {
        return Err(DirectiveError::InvalidDirectiveWord(index));
    }
    let directive_word: Vec<&str> = line[1..].split_whitespace().collect();
    if directive_word.len() != 1 {
        return Err(DirectiveError::InvalidDirectiveWord(index));
    }
    let directive = match *directive_word.first().unwrap() {
        "variables" => DirectiveWord::Variables,
        "tiles" => DirectiveWord::Tiles,
        _ => return Err(DirectiveError::UnknownDirectiveWord(index)),
    };
    Ok(directive)
}

pub(super) fn parse_tiles<'a, I: Iterator<Item = (usize, &'a str)> + Clone>(
    mut lines: I,
) -> Result<Vec<MapTile>, TileDefinitionError> {
    let mut tiles = Vec::with_capacity(lines.clone().count());
    
    while let Some((index, line)) = lines.next() {
        let number: u32 = match line.chars().next().unwrap().to_digit(10) {
            Some(n) => n,
            None => return Err(TileDefinitionError::MissingTileNumber(index)),
        };

    }
    Ok(tiles)
}
// TODO get rid of asserts afterwards maybe
// TODO make it so index is needed to be given to err only at the main callsite.
pub(super) fn parse_tile(
    line: &str,
    index: usize,
) -> Result<MapTile, TileDefinitionError> {
    let mut object_type = None;
    let mut object_top_type = None;
    let mut object_bottom_type = None;
    let mut floor_type = None;
    let mut ceiling_type = None;
    let mut object_top_height = None;
    let mut object_bottom_height = None;

    // Split the line into multiple words where 'spaces' and 'tabs'
    // are considered separators. Get rid of words with no text.
    let expressions = line
        .split(|c| matches!(c, ' ' | '\t'))
        .filter(|k| !k.trim().is_empty());
    for expr in expressions {
        // Split the expression into operands where as the separator
        // is considered a '=' sign or a ':' sign. (e.g. obj:GRASS or o=BRICK)
        let operands: Vec<&str> =
            expr.split(|c| matches!(c, '=' | ':')).collect();
        assert!(!operands.is_empty());
        match operands.len() {
            1 => {
                if is_directive_word(operands.first().unwrap()) {
                    todo!();
                }
            }
            2 => (),
            _ => return Err(TileDefinitionError::InvalidExpression(index))?,
        }
        let left = operands.first().unwrap();
        let right = operands.last().unwrap();

        match *left {
            // Object type definition:
            "o" | "obj" | "object" => todo!(),
            // Object top side type definition:
            "ot" | "o_top" | "obj_top" | "object_top" => todo!(),
            // Object bottom side type definition:
            "ob" | "o_bot" | "o_bottom" | "obj_bot" | "obj_bottom"
            | "object_bottom" => todo!(),
            // Floor type definition:
            "f" | "floor" => todo!(),
            // Ceiling type definition:
            "c" | "ceiling" => todo!(),
            // Top object part height value:
            "th" | "top_h" | "top_height" | "t_height" => {
                object_top_height = Some(parse_from_str(right, index)?)
            }
            // Bottom object part height value:
            "bh" | "bot_h" | "bottom_h" | "b_height" | "bot_height"
            | "bottom_height" => {
                object_bottom_height = Some(parse_from_str(right, index)?)
            }
            _ => return Err(TileDefinitionError::UnknownLeftOperand(index)),
        }
    }
    if object_type.is_none()
        || object_top_type.is_none()
        || object_bottom_type.is_none()
        || floor_type.is_none()
        || ceiling_type.is_none()
        || object_top_height.is_none()
        || object_bottom_type.is_none()
    {
        return Err(TileDefinitionError::MissingTileDefinitions(index));
    }
    let tile = MapTile {
        object: object_type.unwrap(),
        object_top: object_top_type.unwrap(),
        object_bottom: object_bottom_type.unwrap(),
        floor: floor_type.unwrap(),
        ceiling: ceiling_type.unwrap(),
        obj_top_height: object_top_height.unwrap(),
        obj_bottom_height: object_bottom_height.unwrap(),
    };
    todo!()
}

pub(super) fn parse_from_str<P: FromStr>(
    s: &str,
    index: usize,
) -> Result<P, TileDefinitionError> {
    match s.parse() {
        Ok(p) => Ok(p),
        Err(_) => Err(TileDefinitionError::InvalidValueType(index)),
    }
}

pub(super) fn is_directive_word(line: &str) -> bool {
    line.starts_with('#')
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum DirectiveWord {
    Variables,
    Tiles,
}
