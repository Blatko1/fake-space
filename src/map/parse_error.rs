use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq, Eq)]
pub enum MapParseError {
    Dimensions(DimensionsError),
    Directive(DirectiveError),
    TileDefinition(TileDefinitionError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DimensionsError {
    IllegalCharacter(usize),
    MissingDimensions,
    InvalidDimensions(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DirectiveError {
    MissingTilesDirective,
    MultipleSameDirectives,
    InvalidDirectiveWord(usize),
    UnknownDirectiveWord(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TileDefinitionError {
    MissingTileDefinitions(usize),
    InvalidExpression(usize),
    UnknownLeftOperand(usize),
    InvalidValueType(usize),
    MissingTileNumber(usize)
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

impl From<DirectiveError> for MapParseError {
    fn from(value: DirectiveError) -> Self {
        Self::Directive(value)
    }
}

impl From<TileDefinitionError> for MapParseError {
    fn from(value: TileDefinitionError) -> Self {
        Self::TileDefinition(value)
    }
}
