use crate::player::render::PointXZ;
use hashbrown::HashMap;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{char, u64};
use nom::combinator::{fail, opt};
use nom::error::{context, ContextError, ParseError as NomParseError, VerboseError};
use nom::number::complete::double;
use nom::sequence::{delimited, preceded, terminated, Tuple};
use nom::{IResult, Parser};

use crate::world::portal::{DummyPortal, PortalDirection, PortalID};
use crate::world::textures::TextureID;
use crate::world::Tile;

use super::Settings;
use super::{
    boolean, empty_or_fail, line_key, separate_expression_fields, separate_expressions,
    space, take_all,
};

#[derive(Debug)]
pub(super) struct SegmentParser<'a> {
    dimensions: (u64, u64),
    input: &'a str,
    settings: &'a Settings,

    preset_map: HashMap<String, TilePreset>,
    texture_map: &'a HashMap<String, TextureID>,
}

impl<'a> SegmentParser<'a> {
    pub(super) fn new(
        input: &'a str,
        settings: &'a Settings,
        texture_map: &'a HashMap<String, TextureID>,
    ) -> Self {
        Self {
            dimensions: (0, 0),
            input,
            settings,

            preset_map: HashMap::new(),
            texture_map,
        }
    }
    
    #[allow(clippy::type_complexity)]
    pub(super) fn parse(
        mut self,
    ) -> IResult<&'a str, ((u64, u64), Vec<Tile>), VerboseError<&'a str>> {
        let (input, (width, _, height)) = preceded(space, parse_dimensions)(self.input)?;
        if width == 0 || height == 0 {
            return context("Invalid dimensions", fail)(self.input);
        }
        self.dimensions = (width, height);

        let (inp, expressions) = separate_expressions(input)?;
        for expr in expressions {
            let (i, key) = line_key(expr)?;
            match key {
                "$" => {
                    let (_, (id, preset)) =
                        parse_preset(i, self.texture_map, &self.preset_map)?;
                    self.preset_map.insert(id, preset);
                }
                _ => {
                    let (_, tiles) = parse_tiles(
                        expr.trim(),
                        &self.preset_map,
                        (self.dimensions.0 as usize, self.dimensions.1 as usize),
                    )?;

                    let mut processed_tiles = Vec::with_capacity(tiles.len());
                    let mut portal_id = 0;
                    for (i, tile) in tiles.into_iter().enumerate() {
                        // Replace None values with Default values, then compare levels
                        // to find errors (lvl1 <= lvl2 < lvl3 <= lvl4)
                        let bottom_level =
                            tile.bottom_level.unwrap_or(self.settings.bottom_level);
                        let ground_level =
                            tile.ground_level.unwrap_or(self.settings.ground_level);
                        let ceiling_level =
                            tile.ceiling_level.unwrap_or(self.settings.ceiling_level);
                        let top_level = tile.top_level.unwrap_or(self.settings.top_level);
                        assert!(
                            bottom_level < ground_level
                                && ground_level < ceiling_level
                                && ceiling_level < top_level,
                            "Invalid level heights for tile {}:\n\t- bottomL: {}\n\t- groundL: {}\
                            \n\t- ceilingL: {}\n\t- topL: {}",
                            i + 1,
                            bottom_level,
                            ground_level,
                            ceiling_level,
                            top_level
                        );

                        let portal = match tile.portal_dir {
                            Some(direction) => {
                                let dummy = DummyPortal {
                                    id: PortalID(portal_id),
                                    direction,
                                };
                                portal_id += 1;
                                Some(dummy)
                            }
                            None => None,
                        };
                        let allow_voxels = tile.allow_voxels.unwrap_or(false);
                        let position = PointXZ::new(
                            i as u64 % self.dimensions.0,
                            i as u64 / self.dimensions.0,
                        );
                        let t = Tile {
                            position,
                            bottom_wall_tex: tile.bottom_wall_tex.unwrap_or_default(),
                            top_wall_tex: tile.top_wall_tex.unwrap_or_default(),
                            ground_tex: tile.ground_tex.unwrap_or_default(),
                            ceiling_tex: tile.ceiling_tex.unwrap_or_default(),
                            bottom_level,
                            ground_level,
                            ceiling_level,
                            top_level,
                            portal,
                            allow_voxels,
                            // The voxel models will be randomly generated after all tiles were loaded
                            voxel_model: None,
                        };

                        processed_tiles.push(t);
                    }
                    if portal_id == 0 {
                        return context("No portals specified!", fail)("");
                    }

                    return Ok((inp, (self.dimensions, processed_tiles)));
                }
            };
        }
        context("Segment parser error", fail)(input)
    }
}

fn parse_dimensions<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, (u64, &str, u64), E> {
    let dimensions = |i: &'a str| -> IResult<&'a str, (u64, &str, u64), E> {
        (u64, delimited(space, tag("x"), space), u64).parse(i)
    };
    context("dimensions", dimensions)(i)
}

fn parse_preset<'a>(
    input: &'a str,
    textures: &HashMap<String, TextureID>,
    presets: &HashMap<String, TilePreset>,
) -> IResult<&'a str, (String, TilePreset), VerboseError<&'a str>> {
    let (i, (_, name, _, _, expressions_str)) =
        (space, preset_name, space, char('='), take_all).parse(input)?;

    let mut preset = TilePreset::default();
    let (_, expressions) = separate_expression_fields(expressions_str)?;
    for expr in expressions {
        let (i, name) = tile_field_name(expr)?;
        let i = i.trim();
        match name {
            "bottomT" | "topT" | "groundT" | "ceilingT" => {
                let Some(&texture) = textures.get(i) else {
                    return context("Unknown texture", fail)(i);
                };
                match name {
                    "bottomT" => preset.bottom_wall_tex = Some(texture),
                    "topT" => preset.top_wall_tex = Some(texture),
                    "groundT" => preset.ground_tex = Some(texture),
                    "ceilingT" => preset.ceiling_tex = Some(texture),
                    _ => unreachable!(),
                }
            }
            // If the parameter is one of these, the value should be a *number*
            "bottomL" | "groundL" | "ceilingL" | "topL" => {
                let (rest, value) = double(i)?;
                empty_or_fail(rest, "Invalid expression")?;
                match name {
                    "bottomL" => preset.bottom_level = Some(value as f32),
                    "groundL" => preset.ground_level = Some(value as f32),
                    "ceilingL" => preset.ceiling_level = Some(value as f32),
                    "topL" => preset.top_level = Some(value as f32),
                    _ => unreachable!(),
                }
            }
            "portalDir" => {
                let Some(parsed) = portal_dir_from_str(i) else {
                    return context("Unknown texture field", fail)(i);
                };
                preset.portal_dir = Some(parsed);
            }
            "allowVoxels" => {
                let (rest, allow_voxels) = boolean(i)?;
                empty_or_fail(rest, "Invalid expression")?;
                preset.allow_voxels = Some(allow_voxels);
            }
            preset_id => {
                if let Ok((id, _)) = opt(tag::<_, _, VerboseError<&str>>("$"))(preset_id)
                {
                    match presets.get(id) {
                        Some(preset_expr) => preset.overwrite(preset_expr),

                        None => return context("Unknown preset", fail)(id),
                    }
                } else {
                    return context("Unknown texture field", fail)(name);
                }
            }
        };
    }

    Ok((i, (name.to_owned(), preset)))
}

fn parse_tiles<'a>(
    input: &'a str,
    presets: &HashMap<String, TilePreset>,
    dimensions: (usize, usize),
) -> IResult<&'a str, Vec<TilePreset>, VerboseError<&'a str>> {
    let tile_ids = input.split_whitespace();
    if tile_ids.clone().count() != dimensions.0 * dimensions.1 {
        return context("Tile count not matching segment size!", fail)(input);
    }

    let mut tiles = vec![TilePreset::default(); dimensions.0 * dimensions.1];
    for (i, tile) in tile_ids.enumerate() {
        let tile = match presets.get(tile) {
            Some(t) => t,
            None => return context("Unknown tile preset", fail)(tile),
        };
        let x = i % dimensions.0;
        let y = dimensions.1 - i / dimensions.0 - 1;
        let index = y * dimensions.0 + x;
        tiles.get_mut(index).unwrap().overwrite(tile);
    }

    Ok(("", tiles))
}

fn preset_name<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\n\r=";
    context(
        "preset name",
        preceded(space, take_while(|c| !chars.contains(c))),
    )(i)
}

fn tile_field_name<'a, E: NomParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        "field name",
        preceded(
            space,
            terminated(take_while(|c| c != ':'), space.and(opt(char(':')))),
        ),
    )(i)
}

#[derive(Debug, Default, Clone)]
struct TilePreset {
    bottom_wall_tex: Option<TextureID>,
    top_wall_tex: Option<TextureID>,
    ground_tex: Option<TextureID>,
    ceiling_tex: Option<TextureID>,
    bottom_level: Option<f32>,
    ground_level: Option<f32>,
    ceiling_level: Option<f32>,
    top_level: Option<f32>,
    portal_dir: Option<PortalDirection>,
    allow_voxels: Option<bool>,
}

impl TilePreset {
    /// Overwrites all old values with new ones.
    /// Doesn't replace if the new value is `None`.
    fn overwrite(&mut self, src: &Self) {
        if let Some(bottom_wall_tex) = src.bottom_wall_tex {
            self.bottom_wall_tex.replace(bottom_wall_tex);
        }
        if let Some(top_wall_tex) = src.top_wall_tex {
            self.top_wall_tex.replace(top_wall_tex);
        }
        if let Some(ground_tex) = src.ground_tex {
            self.ground_tex.replace(ground_tex);
        }
        if let Some(ceiling_tex) = src.ceiling_tex {
            self.ceiling_tex.replace(ceiling_tex);
        }
        if let Some(bottom_level) = src.bottom_level {
            self.bottom_level.replace(bottom_level);
        }
        if let Some(ground_level) = src.ground_level {
            self.ground_level.replace(ground_level);
        }
        if let Some(ceiling_level) = src.ceiling_level {
            self.ceiling_level.replace(ceiling_level);
        }
        if let Some(top_level) = src.top_level {
            self.top_level.replace(top_level);
        }
        if let Some(portal_dir) = src.portal_dir {
            self.portal_dir.replace(portal_dir);
        }
        if let Some(allow_voxels) = src.allow_voxels {
            self.allow_voxels.replace(allow_voxels);
        }
    }
}

fn portal_dir_from_str(s: &str) -> Option<PortalDirection> {
    match s {
        "N" => Some(PortalDirection::North),
        "S" => Some(PortalDirection::South),
        "E" => Some(PortalDirection::East),
        "W" => Some(PortalDirection::West),
        _ => None,
    }
}
