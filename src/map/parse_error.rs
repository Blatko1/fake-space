use std::{error::Error, fmt::Display, io};

#[derive(Debug, PartialEq)]
pub enum MapParseError {
    Dimensions(DimensionsError),
    Texture(TextureError),
    Directive(DirectiveError),
    Tile(TileError),

    FileErr(io::ErrorKind),
    UndefinedExpression(usize, String),
    DimensionsAndTileCountNotMatching(usize, usize),
}

#[derive(Debug, PartialEq)]
pub enum DimensionsError {
    IllegalCharacter(usize),
    MissingDimensions,
    InvalidSeparatorFormat(usize),
    InvalidDimensionValue(usize),
}

#[derive(Debug)]
pub enum TextureError {
    InvalidSeparatorFormat(usize),
    TextureNameContainsWhitespace(usize, String),
    TextureNameAlreadyTaken(usize, String),

    InvalidOperandSeparatorFormat(usize),
    UnknownParameter(usize, String),
    InvalidTexturePath(usize, String),
    FailedToOpenTexture(io::ErrorKind),
    FailedToReadTexture(image::ImageError),
    FailedToParseBoolValue(usize, String),
    TextureSrcNotSpecified(usize),
    TextureTransparencyNotSpecified(usize),
}

#[derive(Debug, PartialEq)]
pub enum DirectiveError {
    MultipleSameDirectives,
    InvalidDirective(usize, String),
    UnknownDirective(usize, String),
}

#[derive(Debug, PartialEq)]
pub enum TileError {
    InvalidSeparator(usize),
    InvalidExpression(usize, String),
    UnknownParameter(usize, String),
    FloatParseError(usize, String),
    UnknownTexture(usize, String),

    IllegalTileIndexCharacter(usize, char),
    InvalidTileIndexSeparator(usize),
    FailedToParseTileIndex(usize, String),
    InvalidTileIndexRange(usize, String),
    TileIndexNotContinuous(usize, String),
    InvalidTileIndex(usize),

    InvalidVariableSeparatorFormat(usize),
    InvalidVariableFormat(usize),
    UnknownVariable(usize, String),
    VariableNameAlreadyTaken(usize, String),
}

impl PartialEq for TextureError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::InvalidSeparatorFormat(l0),
                Self::InvalidSeparatorFormat(r0),
            ) => l0 == r0,
            (
                Self::TextureNameContainsWhitespace(l0, l1),
                Self::TextureNameContainsWhitespace(r0, r1),
            ) => l0 == r0 && l1 == r1,
            (
                Self::TextureNameAlreadyTaken(l0, l1),
                Self::TextureNameAlreadyTaken(r0, r1),
            ) => l0 == r0 && l1 == r1,
            (
                Self::InvalidOperandSeparatorFormat(l0),
                Self::InvalidOperandSeparatorFormat(r0),
            ) => l0 == r0,
            (
                Self::UnknownParameter(l0, l1),
                Self::UnknownParameter(r0, r1),
            ) => l0 == r0 && l1 == r1,
            (
                Self::InvalidTexturePath(l0, l1),
                Self::InvalidTexturePath(r0, r1),
            ) => l0 == r0 && l1 == r1,
            (Self::FailedToOpenTexture(l0), Self::FailedToOpenTexture(r0)) => {
                l0 == r0
            }
            (Self::FailedToReadTexture(_), Self::FailedToReadTexture(_)) => {
                todo!()
            }
            _ => false,
        }
    }
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

impl From<io::Error> for TextureError {
    fn from(value: io::Error) -> Self {
        Self::FailedToOpenTexture(value.kind())
    }
}

impl From<image::ImageError> for TextureError {
    fn from(value: image::ImageError) -> Self {
        Self::FailedToReadTexture(value)
    }
}
