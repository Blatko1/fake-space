use hashbrown::HashMap;
use nom::{bytes::complete::{is_not, tag}, character::complete::{alphanumeric1, char, digit1}, combinator::{eof, fail, map_res, opt, recognize}, error::{context, VerboseError}, multi::separated_list1, number::complete::{float, u32}, sequence::{delimited, preceded, Tuple}, IResult};

use crate::{map::{blueprint::Tile, portal::{DummyPortal, Orientation, PortalID}}, raycaster::PointXZ, textures::TextureID};

use super::{utils::{blueprint_dimensions, boolean, comma_separator, expression_key, is_empty_or_fail, name_and_expression, name_and_value, separate_expressions, space, take_all}, MapSettings};

// TODO maybe it's better to have this whole thing inside one function?
#[derive(Debug)]
pub struct BlueprintParser<'a> {
    settings: MapSettings,
    presets: HashMap<&'a str, &'a str>,

    portal_count: usize
}

impl<'a> BlueprintParser<'a> {
    pub fn new(
        settings: MapSettings,
    ) -> Self {
        Self {
            settings,
            presets: HashMap::new(),

            portal_count: 0
        }
    }

    pub fn parse(
        mut self,
        input: &'a str,
        textures: &'a HashMap<String, TextureID>,
    ) -> IResult<&'a str, ((u64, u64), Vec<Tile>), VerboseError<&'a str>> {
        let (i, (width, height)) = blueprint_dimensions(input)?;
        if width == 0 || height == 0 {
            return context("Invalid dimensions", fail)(input);
        }
        let dimensions = (width as usize, height as usize);

        let (_, expressions) = separate_expressions(i)?;
        for expr in expressions {
            if let (expr, "$") = expression_key(expr)? {
                let (_, (name, keyword_values)) = name_and_expression(expr)?;
                let name = name.trim();
                if self.presets.contains_key(name) {
                    return context("Preset already exists", fail)(name)
                }
                self.presets.insert(name, keyword_values);
                continue;
            }

                let tile_map: Vec<&str> = expr.split_whitespace().collect();
                if tile_map.len() != dimensions.0 * dimensions.1 {
                    return context("Tile count not matching blueprint size!", fail)(expr);
                }

                let mut object_id = 0;
                let mut tiles = Vec::with_capacity(dimensions.0 * dimensions.1);
                for (y, row) in tile_map.chunks_exact(dimensions.0).rev().enumerate() {
                    for (x, preset_name) in row.iter().enumerate() {
                        // TODO maybe better to use usize rather than u64 ??????
                        let mut tile = Tile {
                            position: PointXZ::new(x as u64, y as u64),
                            bottom_wall_tex: Default::default(),
                            top_wall_tex: Default::default(),
                            ground_tex: Default::default(),
                            ceiling_tex: Default::default(),
                            bottom_level: self.settings.bottom_height,
                            ground_level: self.settings.ground_height,
                            ceiling_level: self.settings.ceiling_height,
                            top_level: self.settings.top_height,
                            portal: None,
                            object: None,
                        };
                        self.parse_preset_recursive(preset_name, &mut tile, textures)?;

                        // Replace None values with Default values, then compare levels
                    // to find errors (lvl1 <= lvl2 < lvl3 <= lvl4)
                    let bottom_level =tile.bottom_level;
                    let ground_level =tile.ground_level;
                    let ceiling_level =tile.ceiling_level;
                    let top_level = tile.top_level;
                    if (bottom_level < ground_level)&& (ground_level < ceiling_level)&& (ceiling_level < top_level){
                        println!("Invalid level heights for tile x: {}, y: {}\n\t- bottomL: {}\n\t- groundL: {}\
                            \n\t- ceilingL: {}\n\t- topL: {}",
                            x, y, bottom_level,ground_level,ceiling_level,top_level);
                        return context("Invalid preset height levels", fail)(preset_name)
                    }

                    /*let object = if tile.object.is_some() {
                        // TODO objects!!!
                        //let object = ObjectID(object_id);
                        //object_id += 1;
                        //Some(object)
                        None
                    } else {
                        None
                    };*/

                    tiles.push(tile);
                    }
                }
            

                    if self.portal_count == 0 {
                        return context("No portals specified!", fail)("");
                    }

                    return Ok((i, ((dimensions.0 as u64, dimensions.1 as u64), tiles)));
        }
        context("blueprint parser error", fail)(input)
    }

    fn parse_preset_recursive(
        &mut self,
        preset_name: &'a str,
        tile: &mut Tile,
        textures: &HashMap<String, TextureID>,
    ) -> IResult<&'a str, (), VerboseError<&'a str>> {
        let preset_expression = match self.presets.get(preset_name) {
            Some(t) => t,
            None => return context("Unknown tile preset", fail)(preset_name),
        };
        let (i, keyword_values) = (separated_list1(comma_separator, is_not(",")))(preset_expression)?;
        for keyword_value in keyword_values {
            let keyword_value = keyword_value.trim();

                // Check if a preset is specified
                let (preset_name, preset_key) = opt(char('$'))(keyword_value)?;
                if preset_key.is_some() {
                    self.parse_preset_recursive(preset_name, tile, textures)?;
                    continue;
                }

            let (_, (keyword, value)) = name_and_value(keyword_value)?;
            let value = value.trim();

            match keyword {
                "textures" => {
                    let (_, (_, _, _, bottom_height, _, ground_height, _, ceiling_height, _, top_height, _, _, _, _)) = 
                    (space, char('('), space, opt(alphanumeric1), comma_separator, 
                    opt(alphanumeric1), comma_separator, opt(alphanumeric1), 
                    comma_separator, opt(alphanumeric1), space, char('('), space, eof).parse(value)?;
                    crate::update_if_texture_exists!(bottom_height, textures, tile.bottom_wall_tex);
                    crate::update_if_texture_exists!(ground_height, textures, tile.ground_tex);
                    crate::update_if_texture_exists!(ceiling_height, textures, tile.ceiling_tex);
                    crate::update_if_texture_exists!(top_height, textures, tile.top_wall_tex);
                }
                "heights" => {
                    let (_, (_, _, _, bottom_height, _, ground_height, _, ceiling_height, _, top_height, _, _, _, _)) = (space, char('('), space, opt(float), comma_separator, 
                    opt(float), comma_separator, opt(float), comma_separator, opt(float), space, char('('), space, eof).parse(value)?;
                    bottom_height.map(|height| tile.bottom_level = height);
                    ground_height.map(|height| tile.ground_level = height);
                    ceiling_height.map(|height| tile.ceiling_level = height);
                    top_height.map(|height| tile.top_level = height);
                }
                "portal" => {
                    let orientation = match value {
                        "N" => Orientation::North,
                        "S" => Orientation::South,
                        "E" => Orientation::East,
                        "W" => Orientation::West,
                        _ => return context("Invalid portal orientation", fail)(&value),
                    };

                    let dummy = DummyPortal {
                        id: PortalID(self.portal_count),
                        orientation,
                    };
                    self.portal_count += 1;
                    tile.portal = Some(dummy);
                }
                // TODO voxels!!!!
                //"allowVoxels" => {
                //    let (rest, allow_voxels) = boolean(value)?;
                //    is_empty_or_fail(rest, "Invalid expression")?;
                //    preset.allow_voxels = Some(allow_voxels);
                //}
                _ => return context("Unknown keyword", fail)(keyword)
            };

        }
        Ok((i, ()))
    }
}
