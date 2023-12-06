#[derive(Debug, PartialEq)]
pub enum ParseError {
    Invalid,
    Dimensions(DimensionError, u32),
    Constant(ConstantError, u32),

    //Texture(TextureError),
    Directive(DirectiveError),
    Tile(TileError),

    FileErr(std::io::ErrorKind),
    UndefinedExpression(usize, String),
    UndefinedTileIndex(usize),
}

#[derive(Debug, PartialEq)]
pub enum DimensionError {
    MissingDimensions,
    InvalidFormat(String),
    ParseError(String),
    IllegalDimensions(u32, u32)
}

#[derive(Debug, PartialEq)]
pub enum ConstantError {
    InvalidFormat(String),
    UnknownVariable(String)
}

#[derive(Debug)]
pub enum TextureError {
    InvalidSeparatorFormat(usize),
    TextureNameContainsWhitespace(usize, String),
    TextureNameAlreadyTaken(usize, String),

    InvalidOperandSeparatorFormat(usize),
    UnknownParameter(usize, String),
    FailedToOpenTexture(std::io::ErrorKind),
    FailedToReadTexture(image::ImageError),
    FailedToParseBoolValue(usize, String),
    TextureSrcNotSpecified(usize),
    TextureTransparencyNotSpecified(usize),
    TextureRepetitionNotSpecified(usize),
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

    InvalidTileIndexSeparator(usize),
    FailedToParseTileIndex(usize, String),
    InvalidTileIndex(usize),
    InvalidLevels(usize, f32, f32, f32),
    TileIndexExceedsLimits(usize),

    InvalidVariableSeparatorFormat(usize),
    InvalidVariableFormat(usize),
    UnknownVariable(usize, String),
    VariableNameAlreadyTaken(usize, String),
}