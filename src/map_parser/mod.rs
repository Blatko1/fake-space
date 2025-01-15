// TODO maybe add tests for the parser
mod blueprint;

use std::path::PathBuf;

use hashbrown::HashMap;
use image::{EncodableLayout, ImageReader};
use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_until, take_while};
use nom::character::complete::{alphanumeric1, char};
use nom::combinator::{cut, fail, value};
use nom::error::{
    context, convert_error, ContextError, ParseError as NomParseError, VerboseError,
};
use nom::multi::separated_list0;
use nom::number::complete::double;
use nom::sequence::{preceded, terminated, Tuple};
use nom::{Finish, IResult, Parser};

use crate::map::blueprint::{Blueprint, BlueprintID, SkyboxTextureIDs, Tile};
use crate::textures::TextureData;

use super::models::{ModelData, ModelID};
use super::textures::TextureID;

use self::blueprint::SegmentParser;

#[derive(Debug)]
struct ParsedTexture {
    name: String,
    texture: TextureData,
}

pub struct MapParser<'a> {
    input: &'a str,
    parent_path: PathBuf,

    settings: Settings,
    textures: Vec<TextureData>,
    texture_map: HashMap<String, TextureID>,
    texture_count: usize,
    models: Vec<ModelData>,
    model_map: HashMap<String, ModelID>,
    model_count: usize,
    blueprints: Vec<Blueprint>,
}

impl<'a> MapParser<'a> {
    pub fn new<P: Into<PathBuf>>(
        input: &'a str,
        parent_path: P,
    ) -> std::io::Result<Self> {
        Ok(Self {
            input,
            parent_path: parent_path.into(),

            settings: Settings::default(),
            textures: Vec::new(),
            texture_map: HashMap::new(),
            texture_count: 2,
            models: Vec::new(),
            model_map: HashMap::new(),
            model_count: 0,
            blueprints: Vec::new(),
        })
    }

    pub fn parse(
        mut self,
    ) -> IResult<
        &'a str,
        (Vec<Blueprint>, Vec<TextureData>, Vec<ModelData>),
        VerboseError<&'a str>,
    > {
        let (_, expressions) = separate_expressions(self.input)?;
        for expr in expressions {
            let (i, key) = line_key(expr)?;
            match key {
                "#" => {
                    let (_, texture) = parse_texture(i, self.parent_path.clone())?;
                    if self.texture_map.contains_key(&texture.name) {
                        return context("Texture with same name already exists", fail)(i);
                    }
                    self.textures.push(texture.texture);
                    self.texture_map
                        .insert(texture.name, TextureID(self.texture_count));
                    self.texture_count += 1;
                }
                "~" => {
                    let (_, (model_name, model)) =
                        parse_vox_file(i, self.parent_path.clone())?;
                    if self.model_map.contains_key(&model_name) {
                        return context("Model with same name already exists", fail)(i);
                    }
                    self.models.push(model);
                    self.model_map.insert(model_name, ModelID(self.model_count));
                    self.model_count += 1;
                }
                "*" => {
                    let (_, (setting_type, value)) = parse_setting(i, &self.texture_map)?;
                    self.settings.update(setting_type, value)
                }
                "!" => {
                    let (_, (name, dimensions, tiles, skybox, repeatable, ambient_light)) =
                        parse_segment(
                            i,
                            self.parent_path.clone(),
                            &self.settings,
                            &self.texture_map,
                        )?;
                    let blueprint = Blueprint::new(
                        BlueprintID(self.blueprints.len()),
                        dimensions,
                        tiles,
                        skybox,
                        repeatable,
                        ambient_light,
                    );
                    self.blueprints.push(blueprint);
                }
                _ => return context("Unknown line key", fail)(key),
            };
        }
        Ok(("", (self.blueprints, self.textures, self.models)))
    }
}

fn parse_texture(
    input: &str,
    parent_path: PathBuf,
) -> IResult<&str, ParsedTexture, VerboseError<&str>> {
    let (i, (_, name, _, _, expressions_str)) =
        (space, texture_name, space, char('='), take_all).parse(input)?;

    let mut data = None;
    let mut transparency = false;

    let (_, expressions) = separate_expression_fields(expressions_str)?;
    for expr in expressions {
        let (i, field_name) = field_name(expr)?;
        match field_name {
            "src" => {
                let (rest, src) = preceded(space, string)(i)?;
                empty_or_fail(rest, "Invalid expression")?;
                let full_path = parent_path.join(src);
                match ImageReader::open(full_path) {
                    Ok(tex) => match tex.decode() {
                        Ok(d) => data = Some(d),
                        Err(_) => return context("Texture decoding error", fail)(i),
                    },
                    Err(_) => return context("Texture not found", fail)(i),
                };
            }
            "transparency" => {
                let (rest, value) = preceded(space, boolean)(i)?;
                empty_or_fail(rest, "Invalid expression")?;
                transparency = value;
            }
            _ => return context("Unknown texture field", fail)(field_name),
        };
    }

    let Some(data) = data else {
        return context("File path not specified", fail)(i);
    };
    let texture = TextureData::new(
        data.to_rgba8().as_bytes().to_vec(),
        data.width() as usize,
        data.height() as usize,
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

fn parse_vox_file(
    input: &str,
    parent_path: PathBuf,
) -> IResult<&str, (String, ModelData), VerboseError<&str>> {
    let (i, (_, name, _, _, expressions_str)) =
        (space, vox_model_name, space, char('='), take_all).parse(input)?;

    let mut data = None;

    let (_, expressions) = separate_expression_fields(expressions_str)?;
    for expr in expressions {
        let (i, name) = field_name(expr)?;
        match name {
            "src" => {
                let (rest, src) = preceded(space, string)(i)?;
                empty_or_fail(rest, "Invalid expression")?;
                let full_path = parent_path.join(src);
                let Some(full_path_str) = full_path.to_str() else {
                    return context("Invalid path unicode", fail)(src);
                };
                match dot_vox::load(full_path_str) {
                    Ok(vox_data) => data = Some(vox_data),
                    Err(_) => {
                        return context("Error while reading '.vox' file", fail)(src)
                    }
                }
            }
            _ => return context("Unknown texture field", fail)(name),
        };
    }

    let Some(data) = data else {
        return context("vox file path not specified", fail)(i);
    };

    if data.models.len() != 1 {
        return context("vox file should contain exactly one model", fail)(i);
    }
    let palette = data.palette;
    let model = data.models.into_iter().next().unwrap();
    if model.size.x != model.size.y
        || model.size.y != model.size.z
        || model.size.x != model.size.z
    {
        return context("vox model doesn't have equal dimensions", fail)(i);
    }
    Ok((
        input,
        (name.to_owned(), ModelData::from_vox_model(model, palette)),
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
            Ok((
                i,
                (SettingType::BottomLevel, SettingValue::F32(level as f32)),
            ))
        }
        "groundL" => {
            let (rest, level) = double(value)?;
            empty_or_fail(rest, "Invalid expression")?;
            Ok((
                i,
                (SettingType::GroundLevel, SettingValue::F32(level as f32)),
            ))
        }
        "ceilingL" => {
            let (rest, level) = double(value)?;
            empty_or_fail(rest, "Invalid expression")?;
            Ok((
                i,
                (SettingType::CeilingLevel, SettingValue::F32(level as f32)),
            ))
        }
        "topL" => {
            let (rest, level) = double(value)?;
            empty_or_fail(rest, "Invalid expression")?;
            Ok((i, (SettingType::TopLevel, SettingValue::F32(level as f32))))
        }
        "skyboxNorth" => {
            let Some(&id) = textures.get(value) else {
                return context("Unknown texture", fail)(value);
            };
            Ok((i, (SettingType::SkyboxNorth, SettingValue::TextureID(id))))
        }
        "skyboxEast" => {
            let Some(&id) = textures.get(value) else {
                return context("Unknown texture", fail)(value);
            };
            Ok((i, (SettingType::SkyboxEast, SettingValue::TextureID(id))))
        }
        "skyboxSouth" => {
            let Some(&id) = textures.get(value) else {
                return context("Unknown texture", fail)(value);
            };
            Ok((i, (SettingType::SkyboxSouth, SettingValue::TextureID(id))))
        }
        "skyboxWest" => {
            let Some(&id) = textures.get(value) else {
                return context("Unknown texture", fail)(value);
            };
            Ok((i, (SettingType::SkyboxWest, SettingValue::TextureID(id))))
        }
        "skyboxTop" => {
            let Some(&id) = textures.get(value) else {
                return context("Unknown texture", fail)(value);
            };
            Ok((i, (SettingType::SkyboxTop, SettingValue::TextureID(id))))
        }
        "skyboxBottom" => {
            let Some(&id) = textures.get(value) else {
                return context("Unknown texture", fail)(value);
            };
            Ok((i, (SettingType::SkyboxBottom, SettingValue::TextureID(id))))
        }
        _ => context("Unknown setting", fail)(name),
    }
}

#[allow(clippy::type_complexity)]
fn parse_segment<'a>(
    input: &'a str,
    parent_dir: PathBuf,
    settings: &Settings,
    textures: &HashMap<String, TextureID>,
) -> IResult<
    &'a str,
    (String, (u64, u64), Vec<Tile>, SkyboxTextureIDs, bool, f32),
    VerboseError<&'a str>,
> {
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

    let (_, expressions) = separate_expression_fields(expressions_str)?;
    for expr in expressions {
        let (i, name) = field_name(expr)?;
        let i = i.trim();
        match name {
            "src" => {
                let full_path = parent_dir.join(i);
                let data = match std::fs::read_to_string(full_path.clone()) {
                    Ok(d) => d,
                    Err(_) => return context("blueprint file not found", fail)(i),
                };
                let tidy_data = clean_input(data);
                let (_, parsed) = match SegmentParser::new(&tidy_data, settings, textures)
                    .parse()
                    .finish()
                {
                    Ok(p) => p,
                    Err(e) => panic!(
                        "blueprint parse error for blueprint {}: {}",
                        full_path.display(),
                        convert_error(tidy_data.as_str(), e)
                    ),
                };
                segment_data = Some(parsed);
            }
            "repeatable" => {
                let (rest, value) = boolean(i)?;
                empty_or_fail(rest, "Invalid expression")?;
                repeatable = value;
            }
            "ambientLight" => {
                let (rest, value) = double(i)?;
                empty_or_fail(rest, "Invalid expression")?;
                if value < 0.0 {
                    return context("Invalid ambient light", fail)(i);
                }
                ambient_light = Some(value as f32);
            }
            "skyboxNorth" => {
                skybox_north = match textures.get(i) {
                    Some(&s) => Some(s),
                    None => return context("Unknown texture", fail)(i),
                }
            }
            "skyboxSouth" => {
                skybox_south = match textures.get(i) {
                    Some(&s) => Some(s),
                    None => return context("Unknown texture", fail)(i),
                }
            }
            "skyboxEast" => {
                skybox_east = match textures.get(i) {
                    Some(&s) => Some(s),
                    None => return context("Unknown texture", fail)(i),
                }
            }
            "skyboxWest" => {
                skybox_west = match textures.get(i) {
                    Some(&s) => Some(s),
                    None => return context("Unknown texture", fail)(i),
                }
            }
            "skyboxTop" => {
                skybox_top = match textures.get(i) {
                    Some(&s) => Some(s),
                    None => return context("Unknown texture", fail)(i),
                }
            }
            "skyboxBottom" => {
                skybox_bottom = match textures.get(i) {
                    Some(&s) => Some(s),
                    None => return context("Unknown texture", fail)(i),
                }
            }
            _ => return context("Unknown blueprint field name", fail)(name),
        }
    }

    // Check if all needed information has been acquired
    let Some((dimensions, tiles)) = segment_data else {
        return context("Unspecified src", fail)(input);
    };
    let Some(ambient_light) = ambient_light else {
        return context("Unspecified ambient light", fail)(input);
    };

    let skybox = SkyboxTextureIDs {
        north: skybox_north.unwrap_or(settings.skybox_north),
        east: skybox_east.unwrap_or(settings.skybox_east),
        south: skybox_south.unwrap_or(settings.skybox_south),
        west: skybox_west.unwrap_or(settings.skybox_west),
        top: skybox_top.unwrap_or(settings.skybox_top),
        bottom: skybox_bottom.unwrap_or(settings.skybox_bottom),
    };

    Ok((
        i,
        (
            name.to_owned(),
            dimensions,
            tiles,
            skybox,
            repeatable,
            ambient_light,
        ),
    ))
}

fn separate_expressions<'a, E: NomParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<&'a str>, E> {
    let (i, expressions) = separated_list0(tag(";"), take_while(|c| c != ';'))(i)?;
    let separated = expressions
        .iter()
        .map(|expr| expr.trim())
        .filter(|expr| !expr.is_empty())
        .collect();
    Ok((i, separated))
}

fn separate_expression_fields<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<&'a str>, E> {
    context(
        "expression fields",
        separated_list0(tag(","), take_while(|c| c != ',')),
    )(i)
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

fn vox_model_name<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("vox model name", preceded(space, alphanumeric1))(i)
}

fn setting_name<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("setting name", preceded(space, alphanumeric1))(i)
}

fn segment_name<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("blueprint name", preceded(space, alphanumeric1))(i)
}

fn field_name<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        "field name",
        preceded(space, terminated(take_until(":"), space.and(char(':')))),
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

fn empty_or_fail<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
    err_msg: &'static str,
) -> IResult<&'a str, &'a str, E> {
    if !i.is_empty() {
        context(err_msg, fail)(i)
    } else {
        Ok(("", ""))
    }
}

/// Removes comments and empty lines from input.
pub fn clean_input(input: String) -> String {
    input
        .lines()
        .map(|line| {
            let mut line = line.split("//").next().unwrap().trim().to_owned();
            line.push('\n');
            line
        })
        .collect()
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
                _ => unreachable!(),
            },
            SettingValue::TextureID(id) => match setting_type {
                SettingType::SkyboxNorth => self.skybox_north = id,
                SettingType::SkyboxEast => self.skybox_east = id,
                SettingType::SkyboxSouth => self.skybox_south = id,
                SettingType::SkyboxWest => self.skybox_west = id,
                SettingType::SkyboxTop => self.skybox_top = id,
                SettingType::SkyboxBottom => self.skybox_bottom = id,
                _ => unreachable!(),
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
