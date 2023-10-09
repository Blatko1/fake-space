use std::{
    iter::Enumerate,
    str::{FromStr, Lines},
};

use crate::map;

use super::{parse_error::MapParseError, MapTile};

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

fn parse(data: &str) -> Result<((u32, u32), Vec<MapTile>), MapParseError> {
    // Split the input data into lines, remove the lines which
    // only contain comments, remove lines with no text,
    // remove the commented out parts from lines with content.
    let mut lines = data
        .lines()
        .enumerate()
        .map(|(i, line)| (i, line.split("//").nth(0).unwrap()))
        .filter(|(_, line)| !line.trim().is_empty());
    // Parse dimensions:
    let dimensions = match lines.next() {
        Some((i, l)) => parse_dimensions(l, i)?,
        None => return Err(MapParseError::MissingDimensions),
    };

    let map_size = (dimensions.0 * dimensions.1) as usize;
    let tiles = Vec::with_capacity(map_size);

    // The next line should contain a directive word like 'vars:' or 'tiles:'.
    let directive = match lines.next() {
        Some((i, l)) => parse_directive_word(l, i)?,
        None => return Err(MapParseError::MissingDirectiveWord),
    };
    let tile_element_count = lines
        .clone()
        .position(
            |(_, line)| {
                if is_directive_word(line) {
                    true
                } else {
                    false
                }
            },
        )
        .unwrap_or(lines.clone().count() - 1)
        + 1;
    let tile_lines_iter = lines.clone().take(tile_element_count);
    let lines = lines.skip(tile_element_count);
    let a = match directive {
        DirectiveWord::Variables => todo!(),
        DirectiveWord::Tiles => parse_tiles(tile_lines_iter)?,
    };

    Ok((dimensions, tiles))
}

fn parse_dimensions(
    line: &str,
    index: usize,
) -> Result<(u32, u32), MapParseError> {
    let mut line = line.to_string();
    line.retain(|c| ![' ', '\t'].contains(&c));
    // The line should have more 3 or more chars (minimal example 9x9)
    if line.chars().count() < 3 {
        return Err(MapParseError::InvalidDimensionsFormat(index));
    }
    // Check if there are any illegal characters is the line.
    if let Some(illegal_char) =
        line.chars().find(|c| !matches!(c, '0'..='9' | 'x'))
    {
        return Err(MapParseError::IllegalCharacter(illegal_char, index));
    }
    // There should be only one 'x' separator.
    if line.matches('x').count() != 1 {
        return Err(MapParseError::InvalidDimensionsFormat(index));
    }
    let dimensions: Vec<&str> = line.split('x').collect();
    assert_eq!(dimensions.len(), 2);
    // Make sure that there is a number
    // left and right from the 'x' separator.
    if dimensions.iter().any(|d| d.is_empty()) {
        return Err(MapParseError::InvalidDimensionsFormat(index));
    }
    Ok((
        dimensions.get(0).unwrap().parse().unwrap(),
        dimensions.get(1).unwrap().parse().unwrap(),
    ))
}

fn parse_directive_word(
    line: &str,
    index: usize,
) -> Result<DirectiveWord, MapParseError> {
    let mut line = line.to_string();
    line.retain(|c| ![' ', '\t'].contains(&c));
    let line = line.trim_end_matches(':');

    if line.chars().count() < 2 {
        return Err(MapParseError::InvalidDirectiveWord(index));
    }
    if line.chars().nth(0).unwrap() != '#' {
        return Err(MapParseError::InvalidDirectiveWord(index));
    }
    let directive = match line.get(1..).unwrap() {
        "v" | "vars" | "variables" => DirectiveWord::Variables,
        "t" | "tiles" => DirectiveWord::Tiles,
        _ => return Err(MapParseError::UnknownDirectiveWord(index)),
    };
    Ok(directive)
}

fn is_directive_word(line: &str) -> bool {
    let mut line = line.to_string();
    line.retain(|c| ![' ', '\t'].contains(&c));
    if line.chars().nth(0).unwrap() == '#' {
        true
    } else {
        false
    }
}

const TILE_DEFINITION_KEYWORDS: &[char] = &['o', 't', 'b', 'f', 'c'];

fn parse_tiles<'a, L: Iterator<Item = (usize, &'a str)> + Clone>(
    lines: L,
) -> Result<Vec<MapTile>, MapParseError> {
    let tiles = Vec::with_capacity(lines.clone().count());
    // Iterate over every line:
    for (i, line) in lines {
        let line = line.trim();
        let tile = parse_tile(line, i)?;
    }
    Ok(tiles)
}
// TODO get rid of asserts afterwards maybe
// TODO make it so index is needed to be given to err only at the main callsite.
fn parse_tile(line: &str, index: usize) -> Result<MapTile, MapParseError> {
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
        assert!(operands.len() > 0);
        match operands.len() {
            1 => {
                if is_variable_keyword(operands.first().unwrap()) {
                    todo!();
                }
            }
            2 => (),
            _ => return Err(MapParseError::FalseExpression(index)),
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
            _ => return Err(MapParseError::UnknownLeftOperand(index)),
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
        return Err(MapParseError::InsufficientTileDefinitions(index));
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

fn parse_from_str<P: FromStr>(
    s: &str,
    index: usize,
) -> Result<P, MapParseError> {
    match s.parse() {
        Ok(p) => return Ok(p),
        Err(_) => return Err(MapParseError::InvalidValueType(index)),
    }
}

/// Takes the keyword which only contains text without spaces or similar.
/// Checks if the provided word could be a variable keyword by checking
/// if it has a '#' sign at the beginning.
fn is_variable_keyword(word: &str) -> bool {
    word.chars().nth(0).unwrap() == '#'
}

#[derive(Debug, PartialEq, Eq)]
enum DirectiveWord {
    Variables,
    Tiles,
}

#[test]
fn parse_dimensions_test() {
    let i = 1;
    let line = "10x10";
    assert_eq!(parse_dimensions(line, i), Ok((10, 10)));
    let line = "1x100";
    assert_eq!(parse_dimensions(line, i), Ok((1, 100)));
    let line = "    11x27   ";
    assert_eq!(parse_dimensions(line, i), Ok((11, 27)));
    let line = "    11  x   27   ";
    assert_eq!(parse_dimensions(line, i), Ok((11, 27)));
    let line = "x10";
    assert_eq!(
        parse_dimensions(line, i),
        Err(MapParseError::InvalidDimensionsFormat(i))
    );
    let line = "10x";
    assert_eq!(
        parse_dimensions(line, i),
        Err(MapParseError::InvalidDimensionsFormat(i))
    );
    let line = "x";
    assert_eq!(
        parse_dimensions(line, i),
        Err(MapParseError::InvalidDimensionsFormat(i))
    );
    let line = "1010";
    assert_eq!(
        parse_dimensions(line, i),
        Err(MapParseError::InvalidDimensionsFormat(i))
    );
    let line = "x10x";
    assert_eq!(
        parse_dimensions(line, i),
        Err(MapParseError::InvalidDimensionsFormat(i))
    );
    let line = "xxx";
    assert_eq!(
        parse_dimensions(line, i),
        Err(MapParseError::InvalidDimensionsFormat(i))
    );
    let line = "x1cx";
    assert_eq!(
        parse_dimensions(line, i),
        Err(MapParseError::IllegalCharacter('c', i))
    );
    let line = "11cx27";
    assert_eq!(
        parse_dimensions(line, i),
        Err(MapParseError::IllegalCharacter('c', i))
    );
}

#[test]
fn parse_directive_word_test() {
    let i = 1;
    let line = "#vars:";
    assert_eq!(parse_directive_word(line, i), Ok(DirectiveWord::Variables));
    let line = "#t";
    assert_eq!(parse_directive_word(line, i), Ok(DirectiveWord::Tiles));
    let line = "vars:";
    assert_eq!(
        parse_directive_word(line, i),
        Err(MapParseError::InvalidDirectiveWord(i))
    );
    let line = "varst";
    assert_eq!(
        parse_directive_word(line, i),
        Err(MapParseError::InvalidDirectiveWord(i))
    );
    let line = "#varst";
    assert_eq!(
        parse_directive_word(line, i),
        Err(MapParseError::UnknownDirectiveWord(i))
    );
    let line = "#tt;";
    assert_eq!(
        parse_directive_word(line, i),
        Err(MapParseError::UnknownDirectiveWord(i))
    );
}
