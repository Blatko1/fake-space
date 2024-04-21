// TODO maybe add tests for the parser
pub mod error;
mod segment;

use std::path::PathBuf;

use hashbrown::HashMap;
use image::{io::Reader as ImageReader, EncodableLayout};
use nom::branch::alt;
use nom::bytes::complete::{escaped, tag, take, take_till, take_until, take_while};
use nom::character::complete::{alpha1, alphanumeric1, char, one_of};
use nom::combinator::{cut, fail, map, opt, value};
use nom::error::{context, ContextError, ParseError as NomParseError, VerboseError};
use nom::multi::separated_list0;
use nom::number::complete::double;
use nom::sequence::{delimited, preceded, terminated, Tuple};
use nom::{Finish, IResult, Parser};
use rand::rngs::ThreadRng;
use strum::IntoEnumIterator;
use wgpu::hal::ExposedAdapter;

use super::textures::TextureID;
use super::{SkyboxTextureIDs, TextureData, Tile};

use self::segment::SegmentParser;

use super::{Segment, SegmentID, World};

#[derive(Debug)]
struct ParsedTexture {
    name: String,
    texture: TextureData,
}

pub struct WorldParser2<'a> {
    input: &'a str,
    parent_path: PathBuf,
    rng: ThreadRng,

    settings: Settings,
    textures: Vec<TextureData>,
    texture_map: HashMap<String, TextureID>,
    texture_counter: usize,
    segments: Vec<Segment>,
}

impl<'a> WorldParser2<'a> {
    pub fn new<P: Into<PathBuf>>(
        input: &'a str,
        parent_path: P,
    ) -> std::io::Result<Self> {
        Ok(Self {
            input,
            parent_path: parent_path.into(),
            rng: rand::thread_rng(),

            settings: Settings::default(),
            textures: Vec::new(),
            texture_map: HashMap::new(),
            texture_counter: 2,
            segments: Vec::new(),
        })
    }

    pub fn parse(mut self) -> IResult<&'a str, World, VerboseError<&'a str>> {
        // TODO Remove comments and empty lines
        //let lines = data
        //    .lines()
        //    .enumerate()
        //    .map(|(i, line)| (1 + i as u64, line.split("//").next().unwrap().trim()))
        //    .filter(|(_, line)| !line.is_empty());

        // TODO doesnt need to be reference
        //let mut input = self.input;
        let (_, expressions) = separated_list0(tag(";"), take_while(|c| c!= ';'))(self.input)?;
        for expr in expressions {
            let expr = expr.trim();
            if expr.is_empty() {
                break;
            }
            match line_key(expr)? {
                (i, k) => {
                    match k {
                        "#" => {
                            let (_, texture) =
                                parse_texture(i, self.parent_path.clone())?;
                            self.textures.push(texture.texture);
                            self.texture_map
                                .insert(texture.name, TextureID(self.texture_counter));
                            self.texture_counter += 1;
                        }
                        "*" => {
                            let (_, (setting_type, value)) = parse_setting(i, &self.texture_map)?;
                            self.settings.update(setting_type, value)
                        }
                        "!" => {
                            let (_, (name, dimensions, tiles, skybox, repeatable, ambient_light)) = parse_segment(i, self.parent_path.clone(), &self.settings, &self.texture_map)?;
                            let segment = Segment::generate_rand(
                                SegmentID(self.segments.len()),
                                name.to_owned(),
                                dimensions,
                                tiles,
                                skybox,
                                repeatable,
                                ambient_light,
                                &mut self.rng,
                                &[]
                            );
                            self.segments.push(segment);
                        },
                        _ => return context("Unknown line key", fail)(i),
                    };
                }
            }
        }
        Ok(("", World::new(self.segments, self.textures)))
    }
}

fn parse_texture<'a>(
    input: &'a str,
    parent_path: PathBuf,
) -> IResult<&str, ParsedTexture, VerboseError<&str>> {
    let (i, (_, name, _, _, expressions_str)) =
        (space, texture_name, space, char('='), take_all).parse(input)?;

    let mut data = None;
    let mut transparency = false;

    let (_, expressions) = separated_list0(tag(","), take_while(|c| c!= ','))(expressions_str)?;
    for expr in expressions {
        match field_name(expr)? {
            (i, k) => {
                match k.trim() {
                    "src" => {
                        let (rest, src) = string(i)?;
                        empty_or_fail(rest, "Invalid expression")?;
                        let full_path = parent_path.join(src);
                        match ImageReader::open(full_path) {
                            Ok(tex) => match tex.decode() {
                                Ok(d) => data = Some(d),
                                Err(e) => {
                                    return context("Texture decoding error", fail)(i)
                                }
                            },
                            Err(e) => return context("Texture not found", fail)(i),
                        };
                    }
                    "transparency" => {
                        let (rest, value) = boolean(i)?;
                        empty_or_fail(rest, "Invalid expression")?;
                        transparency = value;
                    }
                    _ => return context("Unknown texture field", fail)(i),
                };
            }
        }
    }

    let Some(data) = data else {
        return context("File path not specified", fail)(i);
    };
    let texture = TextureData::new(
        data.to_rgba8().as_bytes().to_vec(),
        data.width(),
        data.height(),
        transparency,
    );

    Ok((
        input,
        ParsedTexture {
            name: name.to_owned(),
            texture,
        },
    ))
}

fn parse_setting<'a>(
    input: &'a str,
    textures: &HashMap<String, TextureID>,
) -> IResult<&'a str, (SettingType, SettingValue), VerboseError<&'a str>> {
    let (i, (_, name, _, _, value)) =
        (space, setting_name, space, char('='), take_all).parse(input)?;
    let value = value.trim();
    match name {
        "bottomL" => {
            let (rest, level) = double(value)?;
            empty_or_fail(rest, "Invalid expression")?;
            return Ok((i, (SettingType::BottomLevel, SettingValue::F32(level as f32))));
        }
        "groundL" => {
            let (rest, level) = double(value)?;
            empty_or_fail(rest, "Invalid expression")?;
            return Ok((i, (SettingType::GroundLevel, SettingValue::F32(level as f32))));
        }
        "ceilingL" => {
            let (rest, level) = double(value)?;
            empty_or_fail(rest, "Invalid expression")?;
            return Ok((i, (SettingType::CeilingLevel, SettingValue::F32(level as f32))));
        }
        "topL" => {
            let (rest, level) = double(value)?;
            empty_or_fail(rest, "Invalid expression")?;
            return Ok((i, (SettingType::TopLevel, SettingValue::F32(level as f32))));
        }
        "skyboxNorth" => {
            let Some(&id) = textures.get(value) else {
                return context("Unknown texture", fail)(value);
            };
            return Ok((i, (SettingType::SkyboxNorth, SettingValue::TextureID(id))));
        }
        "skyboxEast" => {
            let Some(&id) = textures.get(value) else {
                return context("Unknown texture", fail)(value);
            };
            return Ok((i, (SettingType::SkyboxEast, SettingValue::TextureID(id))));
        }
        "skyboxSouth" => {
            let Some(&id) = textures.get(value) else {
                return context("Unknown texture", fail)(value);
            };
            return Ok((i, (SettingType::SkyboxSouth, SettingValue::TextureID(id))));
        }
        "skyboxWest" => {
            let Some(&id) = textures.get(value) else {
                return context("Unknown texture", fail)(value);
            };
            return Ok((i, (SettingType::SkyboxWest, SettingValue::TextureID(id))));
        }
        "skyboxTop" => {
            let Some(&id) = textures.get(value) else {
                return context("Unknown texture", fail)(value);
            };
            return Ok((i, (SettingType::SkyboxTop, SettingValue::TextureID(id))));
        }
        "skyboxBottom" => {
            let Some(&id) = textures.get(value) else {
                return context("Unknown texture", fail)(value);
            };
            return Ok((i, (SettingType::SkyboxBottom, SettingValue::TextureID(id))));
        }
        _ => return context("Unknown setting", fail)(name),
    }
}

fn parse_segment<'a>(
    input: &'a str,
    parent_dir: PathBuf,
    settings: &Settings,
    textures: &HashMap<String, TextureID>
) -> IResult<&'a str, (String, (u64, u64), Vec<Tile>, SkyboxTextureIDs, bool, f32), VerboseError<&'a str>> {
    let (i, (_, name, _, _, expressions_str)) =
        (space, segment_name, space, char('='), take_all).parse(input)?;

        let mut segment_data = None;
        let mut repeatable = false;
        let mut ambient_light = None;
        let mut skybox_north = None;
        let mut skybox_east = None;
        let mut skybox_south = None;
        let mut skybox_west = None;
        let mut skybox_top = None;
        let mut skybox_bottom = None;
        
    let (_, expressions) = separated_list0(tag(","), take_while(|c| c!= ','))(expressions_str)?;
    for expr in expressions {
        match field_name(expr)? {
            (i, name) => {
                let i = i.trim();
                match name {
                "src" => {
                    let full_path = parent_dir.join(i);
                    let data = match std::fs::read_to_string(full_path.clone()) {
                        Ok(d) => d,
                        Err(e) => return context("Segment file not found", fail)(i),
                    };
                    let parsed = match SegmentParser::new(
                        &data,
                        settings,
                        textures,
                    )
                    .parse()
                    {
                        Ok(p) => p,
                        Err(e) => return context("Failed to parse segment", fail)(i)
                    };
                    segment_data = Some(parsed);
                },
                "repeatable" => {
                    let (rest, value) = boolean(i)?;
                        empty_or_fail(rest, "Invalid expression")?;
                        repeatable = value;
                },
                "ambientLight" => {
                    let (rest, value) = double(i)?;
                        empty_or_fail(rest, "Invalid expression")?;
                        if value < 0.0 {
                            return context("Invalid ambient light", fail)(i)
                        }
                        ambient_light = Some(value as f32);
                },
                "skyboxNorth" => skybox_north = match textures.get(i) {
                    Some(&s) => Some(s),
                    None => {
                        return context("Unknown texture", fail)(i)
                    }
                },
                "skyboxSouth" => skybox_south = match textures.get(i) {
                    Some(&s) => Some(s),
                    None => {
                        return context("Unknown texture", fail)(i)
                    }
                },
                "skyboxEast" => skybox_east = match textures.get(i) {
                    Some(&s) => Some(s),
                    None => {
                        return context("Unknown texture", fail)(i)
                    }
                },
                "skyboxWest" => skybox_west = match textures.get(i) {
                    Some(&s) => Some(s),
                    None => {
                        return context("Unknown texture", fail)(i)
                    }
                },
                "skyboxTop" => skybox_top = match textures.get(i) {
                    Some(&s) => Some(s),
                    None => {
                        return context("Unknown texture", fail)(i)
                    }
                },
                "skyboxBottom" => skybox_bottom = match textures.get(i) {
                    Some(&s) => Some(s),
                    None => {
                        return context("Unknown texture", fail)(i)
                    }
                },
                _ => return context("Unknown segment field name", fail)(name)
            }}
        }
    }

    // Check if all needed information has been acquired
    let Some((dimensions, tiles)) = segment_data else {
        return context("Unspecified src", fail)(input)
    };
    let Some(ambient_light) = ambient_light else {
        return context("Unspecified ambient light", fail)(input)
    };

    let skybox = SkyboxTextureIDs {
        north: skybox_north.unwrap_or(settings.skybox_north),
        east: skybox_east.unwrap_or(settings.skybox_east),
        south: skybox_south.unwrap_or(settings.skybox_south),
        west: skybox_west.unwrap_or(settings.skybox_west),
        top: skybox_top.unwrap_or(settings.skybox_top),
        bottom: skybox_bottom.unwrap_or(settings.skybox_bottom),
    };

    Ok((i, (
        name.to_owned(),
        dimensions,
        tiles,
        skybox,
        repeatable,
        ambient_light,
    )))
}

fn space<'a, E: NomParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";
    take_while(|c| chars.contains(c))(i)
}

fn string<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        "string",
        preceded(char('\"'), cut(terminated(take_until("\""), char('\"')))),
    )(i)
}

fn take_all<'a, E: NomParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    take_while(|_| true)(i)
}

fn texture_name<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("texture name", preceded(space, alphanumeric1))(i)
}

fn setting_name<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("setting name", preceded(space, alphanumeric1))(i)
}

fn segment_name<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("segment name", preceded(space, alphanumeric1))(i)
}

fn field_name<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        "field name",
        preceded(space, terminated(take_until(":"), char(':'))),
    )(i)
}

fn line_key<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("line key", preceded(space, take(1usize)))(i)
}

fn boolean<'a, E: NomParseError<&'a str>>(i: &'a str) -> IResult<&'a str, bool, E> {
    let parse_true = value(true, tag("true"));
    let parse_false = value(false, tag("false"));
    alt((parse_true, parse_false))(i)
}

fn empty_or_fail<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(i: &'a str, err_msg: &'static str) -> IResult<&'a str, &'a str, E> {
    if !i.is_empty() {
        context(err_msg, fail)(i)
    } else {
        Ok(("", ""))
    }
}

#[derive(Debug, strum::EnumIter, PartialEq, Eq, Hash)]
pub enum SettingType {
    BottomLevel,
    GroundLevel,
    CeilingLevel,
    TopLevel,
    SkyboxNorth,
    SkyboxEast,
    SkyboxSouth,
    SkyboxWest,
    SkyboxTop,
    SkyboxBottom,
}

#[derive(Debug)]
pub enum SettingValue {
    F32(f32),
    TextureID(TextureID),
}

#[derive(Debug)]
pub(super) struct Settings {
    pub bottom_level: f32,
    pub ground_level: f32,
    pub ceiling_level: f32,
    pub top_level: f32,
    pub skybox_north: TextureID,
    pub skybox_east: TextureID,
    pub skybox_south: TextureID,
    pub skybox_west: TextureID,
    pub skybox_top: TextureID,
    pub skybox_bottom: TextureID,
}

impl Settings {
    fn update(&mut self, setting_type: SettingType, value: SettingValue) {
        match value {
            SettingValue::F32(f) => match setting_type {
                SettingType::BottomLevel => self.bottom_level = f,
                SettingType::GroundLevel => self.ground_level = f,
                SettingType::CeilingLevel => self.ceiling_level = f,
                SettingType::TopLevel => self.top_level = f,
                _ => unreachable!()
            },
            SettingValue::TextureID(id) => match setting_type {
                SettingType::SkyboxNorth => self.skybox_north = id,
                SettingType::SkyboxEast => self.skybox_east = id,
                SettingType::SkyboxSouth => self.skybox_south = id,
                SettingType::SkyboxWest => self.skybox_west = id,
                SettingType::SkyboxTop => self.skybox_top = id,
                SettingType::SkyboxBottom => self.skybox_bottom = id,
                _ => unreachable!()
            },
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bottom_level: -1.0,
            ground_level: -0.5,
            ceiling_level: 0.5,
            top_level: 1.0,
            skybox_north: TextureID(0),
            skybox_east: TextureID(0),
            skybox_south: TextureID(0),
            skybox_west: TextureID(0),
            skybox_top: TextureID(0),
            skybox_bottom: TextureID(0),
        }
    }
}
/*#[test]
fn parsing() {
    WorldParser::new("maps/world.txt").unwrap().parse().unwrap();
}
*/
