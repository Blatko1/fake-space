use std::{error::Error, fmt::Display, io};

#[derive(Debug, PartialEq, Eq)]
pub enum MapParseError {
    Dimensions(DimensionsError),
    Texture(TextureError),
    Directive(DirectiveError),
    Tile(TileError),

    FileErr(io::ErrorKind),
    Undefined(usize, String),
    DimensionsAndTileCountNotMatching(usize, usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DimensionsError {
    IllegalCharacter(usize),
    MissingDimensions,
    InvalidSeparatorFormat(usize),
    InvalidDimensionValue(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TextureError {
    InvalidSeparatorFormat(usize),
    TextureSymbolContainsWhiteSpaces(usize, String),
    TextureNameAlreadyTaken(usize, String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DirectiveError {
    MultipleSameDirectives,
    InvalidDirective(usize, String),
    UnknownDirective(usize, String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TileError {
    InvalidSeparator(usize),
    MissingTileDefinitions(usize),
    InvalidExpression(usize, String),
    UnknownLeftOperand(usize, String),
    InvalidValueType(usize),
    UnknownTextureKey(usize, String),
    MissingTileNumber(usize),

    IllegalTileIndexCharacter(usize, char),
    InvalidTileIndexSeparator(usize),
    FailedToParseTileIndex(usize, String),
    InvalidTileIndexRange(usize, String),
    TileIndexNotContinuous(usize, String),
    InvalidTileIndex(usize),
    TileIndexContainsWhiteSpaces(usize),

    InvalidVariableFormat(usize),
    UnknownVariable(usize, String),
    VariableNameAlreadyTaken(usize, String),
}

impl Display for MapParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for MapParseError {}

impl From<DimensionsError> for MapParseError {
    fn from(value: DimensionsError) -> Self {
        Self::Dimensions(value)
    }
}

impl From<TextureError> for MapParseError {
    fn from(value: TextureError) -> Self {
        Self::Texture(value)
    }
}

impl From<DirectiveError> for MapParseError {
    fn from(value: DirectiveError) -> Self {
        Self::Directive(value)
    }
}

impl From<TileError> for MapParseError {
    fn from(value: TileError) -> Self {
        Self::Tile(value)
    }
}

impl From<io::Error> for MapParseError {
    fn from(value: io::Error) -> Self {
        Self::FileErr(value.kind())
    }
}