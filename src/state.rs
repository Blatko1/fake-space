use std::path::PathBuf;

use nom::{error::convert_error, Finish};
use winit::event::DeviceEvent;

use crate::{control::GameInput, map::{room::RoomID, Map}, map_parser::{cleanup_input, MapParser}, models::ModelArray, player::Player, raycaster::{self, camera::Camera}, textures::TextureArray, CANVAS_HEIGHT, CANVAS_WIDTH};

const PHYSICS_TIMESTEP: f32 = 0.01;

pub struct GameState {
    camera: Camera,

    map: Map,
    textures: TextureArray,
    models: ModelArray,

    player: Player,

    delta_accumulator: f32,
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
            CANVAS_WIDTH,
            CANVAS_HEIGHT,
        );

        Self {
            camera,
            
            map: Map::new(segments),
            textures: TextureArray::new(textures),
            models: ModelArray::new(models),

            player: Player::new(RoomID(0)),

            delta_accumulator: 0.0,
        }
    }

    pub fn update(&mut self, delta: f32) {
        // Update world and player
        self.delta_accumulator += delta;
        while self.delta_accumulator >= PHYSICS_TIMESTEP {
            self.player.update(&self.map, PHYSICS_TIMESTEP);
            self.delta_accumulator -= PHYSICS_TIMESTEP;
        }
        self.camera.follow(self.player.get_camera_target());
        //self.world.update(&mut self.player);
    }

    pub fn render<'a, C>(&mut self, canvas_column_iter: C) where
    C: Iterator<Item = &'a mut [u8]> {
        raycaster::cast_and_draw(&self.camera, &self.player, &self.map, &self.textures, &self.models, canvas_column_iter);
    }

    pub fn handle_game_input(&mut self, input: GameInput, is_pressed: bool) {
        self.player.handle_game_input(input, is_pressed);
    }

    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } =>  self.player.handle_mouse_motion(delta),
            DeviceEvent::MouseWheel { delta } => self.camera.handle_mouse_wheel(delta),
            _ => ()
        }
    }

    /*pub fn collect_dbg_data(&self) -> DebugData {
        let player_dbg_data = self.player.collect_dbg_data();
        //let world_dbg_data = WorldDebugData {
        //    room_count: 0,
        //};

        DebugData {
            player_data: player_dbg_data,
            //world_data: world_dbg_data,
        }
    }*/
}