use crate::map::{
    map_parser::{parse_dimensions, parse_directive_word, DirectiveWord},
    parse_error::{DimensionsError, DirectiveError},
};

use super::map_parser::parse;

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
        Err(DimensionsError::InvalidDimensions(i))
    );
    let line = "x10";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensions(i))
    );
    let line = "10x";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensions(i))
    );
    let line = "x";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensions(i))
    );
    let line = "1010";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensions(i))
    );
    let line = "x10x";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensions(i))
    );
    let line = "xxx";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensions(i))
    );
    let line = "x1cx";
    assert_eq!(
        parse_dimensions(i, line),
        Err(DimensionsError::InvalidDimensions(i))
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
    assert_eq!(parse_directive_word(i, line), Ok(DirectiveWord::Variables));
    let line = "#          tiles";
    assert_eq!(parse_directive_word(i, line), Ok(DirectiveWord::Tiles));
    let line = "vars";
    assert_eq!(
        parse_directive_word(i, line),
        Err(DirectiveError::InvalidDirectiveWord(i))
    );
    let line = "# vari ables";
    assert_eq!(
        parse_directive_word(i, line),
        Err(DirectiveError::InvalidDirectiveWord(i))
    );
    let line = "varst";
    assert_eq!(
        parse_directive_word(i, line),
        Err(DirectiveError::InvalidDirectiveWord(i))
    );
    let line = "#varst";
    assert_eq!(
        parse_directive_word(i, line),
        Err(DirectiveError::UnknownDirectiveWord(i))
    );
    let line = "#tt;";
    assert_eq!(
        parse_directive_word(i, line),
        Err(DirectiveError::UnknownDirectiveWord(i))
    );
}

#[test]
fn map_parser_test() {
    parse(include_str!("../../maps/map1.txt")).unwrap();
}