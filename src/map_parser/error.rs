use std::io;

#[derive(Debug)]
pub enum ParseError {
    Invalid,
    FileErr(std::io::ErrorKind),

    Dimensions(DimensionError, u32),
    Variable(VariableError, u32),
    Texture(TextureError, u32),
    Preset(PresetError, u32),

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
pub enum VariableError {
    InvalidFormat(String),
    UnknownVariable(String),
    InvalidValue(String),
}

#[derive(Debug)]
pub enum TextureError {
    InvalidFormat(String),
    InvalidExpressionFormat(String),

    TextureAlreadyExists(String),
    UnknownExpressionParameter(String),

    BoolParseFail(String),
    TextureFileErr(std::io::ErrorKind),
    TextureReadFailed(image::ImageError),

    UnspecifiedTexture,
    UnspecifiedTransparency,

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
pub enum PresetError {
    InvalidFormat(String),
    InvalidPreset(TileError)
}

#[derive(Debug)]
pub enum TileError {
    InvalidSeparator(usize),
    InvalidExpressionFormat(String),
    UnknownParameter(String),
    UnknownTexture(String),
    FloatParseFail(String),

    InvalidTileIndexSeparator(usize),
    FailedToParseTileIndex(usize, String),
    InvalidTileIndex(usize),
    TileIndexExceedsLimits(usize),

    InvalidLevels(usize, f32, f32, f32),

    InvalidVariableSeparatorFormat(usize),
    InvalidVariableFormat(usize),
    UnknownVariable(usize, String)
}

impl From<TileError> for PresetError {
    fn from(value: TileError) -> Self {
        Self::InvalidPreset(value)
    }
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
