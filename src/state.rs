use std::path::PathBuf;

use nom::{error::convert_error, Finish};

use crate::{dbg::DebugData, map::{room::RoomID, Map}, map_parser::{cleanup_input, MapParser}, models::ModelArray, player::Player, raycaster::camera::Camera, textures::TextureArray, CANVAS_HEIGHT, CANVAS_WIDTH};

pub struct GameState {
    camera: Camera,

    map: Map,
    textures: TextureArray,
    models: ModelArray,

    player: Player,
}

impl GameState {
    pub fn new<P: Into<PathBuf>>(data_path: P) -> Self {
                // TODO remove 'unwrap()'s
                let path: PathBuf = data_path.into().canonicalize().unwrap();
                let parent_path = path.parent().unwrap().to_path_buf();
                let input = cleanup_input(std::fs::read_to_string(path).unwrap());
                let (segments, textures, models) = 
                    match MapParser::new(&input, parent_path).unwrap().parse().finish() {
                    Ok((_, data)) => data,
                    Err(e) => {
                        println!("verbose errors: \n{}", convert_error(input.as_str(), e));
                        panic!()
                    }
                };

        let camera = Camera::new(
            10.5,
            1.0,
            14.5,
            90f32.to_radians(),
            CANVAS_WIDTH,
            CANVAS_HEIGHT,
        );

        Self {
            camera,
            
            map: Map::new(segments),
            textures: TextureArray::new(textures),
            models: ModelArray::new(models),

            player: Player::new(RoomID(0)),
        }
    }

    pub fn collect_dbg_data(&self) -> DebugData {
        let player_dbg_data = self.player.collect_dbg_data();
        //let world_dbg_data = WorldDebugData {
        //    room_count: 0,
        //};

        DebugData {
            player_data: player_dbg_data,
            //world_data: world_dbg_data,
        }
    }
}