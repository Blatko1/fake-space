use std::io;

#[derive(Debug)]
pub enum ParseError {
    Invalid,
    Dimensions(DimensionError, u32),
    Constant(ConstantError, u32),
    FileErr(std::io::ErrorKind),

    Texture(TextureError),
    Directive(DirectiveError),
    Tile(TileError),

    UndefinedExpression(usize, String),
    UndefinedTileIndex(usize),
}

#[derive(Debug)]
pub enum DimensionError {
    MissingDimensions,
    InvalidFormat(String),
    ParseError(String),
    IllegalDimensions(u32, u32),
}

#[derive(Debug)]
pub enum ConstantError {
    InvalidFormat(String),
    UnknownVariable(String),
    InvalidValue(String),
}

#[derive(Debug)]
pub enum TextureError {
    InvalidFormat(String),
    InvalidExpressionFormat(String),
    UnknownExpressionParameter(String),
    FailedBoolParse(String),
    TextureFileErr(std::io::ErrorKind),
    TextureReadFailed(image::ImageError),
    UnspecifiedTexture,
    UnspecifiedTransparency,

    TextureNameContainsWhitespace(usize, String),
    TextureNameAlreadyTaken(usize, String),
    //InvalidOperandSeparatorFormat(usize),
    //UnknownParameter(usize, String),
    //FailedToOpenTexture(std::io::ErrorKind),
    //FailedToReadTexture(image::ImageError),
    //FailedToParseBoolValue(usize, String),
    //TextureSrcNotSpecified(usize),
    //TextureTransparencyNotSpecified(usize),
    //TextureRepetitionNotSpecified(usize),
}

#[derive(Debug)]
pub enum DirectiveError {
    MultipleSameDirectives,
    InvalidDirective(usize, String),
    UnknownDirective(usize, String),
}

#[derive(Debug)]
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

impl From<io::Error> for ParseError {
    fn from(value: io::Error) -> Self {
        Self::FileErr(value.kind())
    }
}

impl From<io::Error> for TextureError {
    fn from(value: io::Error) -> Self {
        Self::TextureFileErr(value.kind())
    }
}

impl From<image::ImageError> for TextureError {
    fn from(value: image::ImageError) -> Self {
        Self::TextureReadFailed(value)
    }
}
