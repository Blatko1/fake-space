use std::io;

#[derive(Debug)]
pub enum ParseError {
    UnknownKey(String, u32),
    FileErr(std::io::ErrorKind),
    SettingErr(SettingError, u32),
    TextureErr(TextureError, u32),
    SegmentErr(SegmentError, u32),
}

#[derive(Debug)]
pub enum SegmentError {
    // Errors for the config file
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
    UnknownKey(String, u32),
    DimensionsErr(DimensionError, u32),
    PresetErr(PresetError, u32),
    TileErr(TileError, u32),

    InvalidLevels(usize, f32, f32, f32, f32),
}

#[derive(Debug)]
pub enum DimensionError {
    InvalidFormat(String),
    ParseError(String),
    IllegalDimensions(u32, u32),
}

#[derive(Debug)]
pub enum SettingError {
    InvalidFormat(String),
    UnknownSetting(String),
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
