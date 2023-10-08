use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq, Eq)]
pub enum MapParseError {
    IllegalCharacter(char, usize),

    MissingDimensions,
    InvalidDimensionsFormat(usize),

    MissingDirectiveWord,
    InvalidDirectiveWord(usize),
    UnknownDirectiveWord(usize),
}

impl Display for MapParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for MapParseError {}
