use crate::map::{
    map_parser::{parse_dimensions, parse_directive, Directive},
    parse_error::{DimensionsError, DirectiveError, TileError},
};

use super::map_parser::{parse_map, parse_tile_index};

#[test]
fn parse_dimensions_test() {
    let i = 1;
    let line = "10x10";
    assert_eq!(parse_dimensions(i, line), Ok((10, 10)));
    let line = "1x100";
    assert_eq!(parse_dimensions(i, line), Ok((1, 100)));
    let line = "    11x27   ".trim();
    assert_eq!(parse_dimensions(i, line), Ok((11, 27)));
    let line = "    11  x   27   ".trim();
    assert_eq!(parse_dimensions(i, line), Ok((11, 27)));
    let line = "    11  x   27   1".trim();
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensionValue(i))
    );
    let line = "x10";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensionValue(i))
    );
    let line = "10x";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensionValue(i))
    );
    let line = "x";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensionValue(i))
    );
    let line = "1010";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensionValue(i))
    );
    let line = "x10x";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensionValue(i))
    );
    let line = "xxx";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensionValue(i))
    );
    let line = "x1cx";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensionValue(i))
    );
    let line = "11cx27";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::IllegalCharacter(i))
    );
}

#[test]
fn parse_directive_word_test() {
    let i = 1;
    let line = "#variables";
    assert_eq!(parse_directive(i, line), Ok(Directive::Variables));
    let line = "#          tiles";
    assert_eq!(parse_directive(i, line), Ok(Directive::Tiles));
    let line = "vars";
    assert_eq!(
        parse_directive(i, line),
        Err(DirectiveError::InvalidDirective(i, "vars".to_string()))
    );
    let line = "# vari ables";
    assert_eq!(
        parse_directive(i, line),
        Err(DirectiveError::InvalidDirective(i, "# vari ables".to_string()))
    );
    let line = "varst";
    assert_eq!(
        parse_directive(i, line),
        Err(DirectiveError::InvalidDirective(i, "varst".to_string()))
    );
    let line = "#varst";
    assert_eq!(
        parse_directive(i, line),
        Err(DirectiveError::UnknownDirective(i, "varst".to_string()))
    );
    let line = "#tt;";
    assert_eq!(
        parse_directive(i, line),
        Err(DirectiveError::UnknownDirective(i, "tt;".to_string()))
    );
}

#[test]
fn parse_tile_index_test() {
    let index = 1;
    let operand = "1";
    assert_eq!(parse_tile_index(index, operand), Ok(0..=0));
    let operand = "100";
    assert_eq!(parse_tile_index(index, operand), Ok(99..=99));
    let operand = "100-101";
    assert_eq!(parse_tile_index(index, operand), Ok(99..=100));
    let operand = "100-109";
    assert_eq!(parse_tile_index(index, operand), Ok(99..=108));
    let operand = "100-100";
    assert_eq!(parse_tile_index(index, operand), Ok(99..=99));
    let operand = "100-99";
    assert_eq!(
        parse_tile_index(index, operand),
        Err(TileError::InvalidTileIndexRange(index, "100-99".to_string()))
    );
    let operand = "100-98";
    assert_eq!(
        parse_tile_index(index, operand),
        Err(TileError::InvalidTileIndexRange(index, "100-98".to_string()))
    );
    let operand = "100-9-8";
    assert_eq!(
        parse_tile_index(index, operand),
        Err(TileError::InvalidTileIndexSeparator(index))
    );
    let operand = "1-9-9-0";
    assert_eq!(
        parse_tile_index(index, operand),
        Err(TileError::InvalidTileIndexSeparator(index))
    );
}

#[test]
fn map_parser_test() {
    let parsed = parse_map(include_str!("../../maps/map1.txt")).unwrap();
    println!("dimensions: {:?}", parsed.0);
    for t in parsed.1 {
        println!("tile: {:?}", t);
    }
}
