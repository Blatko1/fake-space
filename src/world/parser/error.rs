use std::io;

#[derive(Debug)]
pub enum ParseError {
    UnknownKey(String, u64),
    FileErr(std::io::ErrorKind),
    SettingErr(SettingError, u64),
    TextureErr(TextureError, u64),
    SegmentErr(SegmentError, u64),

    NotEnoughSegments(usize),
}

#[derive(Debug)]
pub enum SegmentError {
    // Errors for the world file
    FileErr(std::io::ErrorKind),
    InvalidFormat(String),
    UnknownParameter(String),
    BoolParseFail(String),
    UnspecifiedSrc,
    UnspecifiedRepetition,

    // Errors for the segment file
    SegmentParseErr(SegmentParseError, String),
}

#[derive(Debug)]
pub enum SegmentParseError {
    Invalid,
    UnknownKey(String, u64),
    DimensionsErr(DimensionError, u64),
    PresetErr(PresetError, u64),
    TileErr(TileError, u64),

    InvalidLevels(usize, f32, f32, f32, f32),
    NoPortalsSpecified,
}

#[derive(Debug)]
pub enum DimensionError {
    InvalidFormat(String),
    ParseError(String),
    IllegalDimensions(u64, u64),
}

#[derive(Debug)]
pub enum SettingError {
    InvalidFormat(String),
    UnknownSetting(String),
    InvalidF32Value(String),
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

    UnspecifiedSrc,
    UnspecifiedTransparency,
}

#[derive(Debug)]
pub enum PresetError {
    InvalidFormat(String),
    InvalidPreset(TileError),
}

#[derive(Debug)]
pub enum TileError {
    InvalidFormat(String),
    InvalidExpressionFormat(String),

    InvalidIndexFormat(String),
    IndexUsizeParseFail(String),
    IndexIsZero(String),
    InvalidIndexRange(String, usize, usize),
    IndexOutOfRange(String, usize),

    UnknownPreset(String),
    UnknownParameter(String),
    UnknownTexture(String),
    FloatParseFail(String),
    BoolParseFail(String),
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

impl From<io::Error> for SegmentError {
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
