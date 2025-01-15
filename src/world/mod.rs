pub mod model;
mod parser;
pub mod portal;
pub mod textures;
pub mod map;

use nom::error::convert_error;
use nom::Finish;
use rand::Rng;
use rand::{rngs::ThreadRng, seq::SliceRandom};
use std::path::PathBuf;

use crate::player::render::PointXZ;
use crate::player::Player;
use crate::world::portal::{DummyPortal, Portal, PortalID};
use parser::WorldParser;
use textures::{TextureData, TextureID, TextureManager};

use self::model::{ModelData, ModelDataRef, ModelID, ModelManager};
use self::parser::cleanup_input;
use self::textures::TextureDataRef;

pub struct World {
    blueprints: Vec<blueprint>,
    texture_manager: TextureManager,
    model_manager: ModelManager,

    rng: ThreadRng,
    // Each room has index which is the position in this Vec
    rooms: Vec<Room>,
}

impl World {
    pub fn update(&mut self, player: &mut Player) {
        let current_room_id = player.current_room_id();
        let current_tile_pos = player.current_tile_pos();
        self.fully_generate_room(current_room_id);
        if let Some(tile) = self
            .get_room_data(current_room_id)
            .blueprint
            .get_tile_checked(current_tile_pos.0, current_tile_pos.1)
        {
            if let Some(object) = tile.object {
                let maybe_exists = self.rooms[current_room_id.0]
                    .objects
                    .get_mut(object.0)
                    .unwrap();
                if maybe_exists.is_some() {
                    player.increase_score(1);
                    maybe_exists.take();
                }
            }
        }
        // The current room is now fully generated
        let current_room = &mut self.rooms[current_room_id.0];
        // Clone needed for the borrow checker. Is there a better solution???
        current_room
            .portals
            .clone()
            .iter()
            .for_each(|portal| self.fully_generate_room(portal.link.unwrap().0));
    }

    fn fully_generate_room(&mut self, room_id: RoomID) {
        let mut next_id = self.rooms.len();
        let current_room = &mut self.rooms[room_id.0];
        // Check if current room has all adjacent rooms generated
        if !current_room.is_fully_generated {
            // Generated adjacent rooms where needed
            let mut adjacent_rooms: Vec<Room> = current_room
                .portals
                .iter_mut()
                .filter(|portal| portal.link.is_none())
                .map(|portal| {
                    // Skip the first (two) rooms since they appear only once
                    let rand_segment = self.blueprints[1..].choose(&mut self.rng).unwrap();
                    let mut new_room = Room::new_with_rng_objects(
                        RoomID(next_id),
                        rand_segment,
                        &mut self.rng,
                        self.model_manager.model_list(),
                    );
                    let room_rand_portal =
                        new_room.portals.choose_mut(&mut self.rng).unwrap();

                    // Connect the two portals:
                    // Connect the starting room with new random room
                    portal.link = Some((new_room.id, room_rand_portal.id));
                    // Connect the new random room with the starting room
                    room_rand_portal.link = Some((current_room.id, portal.id));
                    next_id += 1;
                    new_room
                })
                .collect();
            current_room.is_fully_generated = true;
            self.rooms.append(&mut adjacent_rooms);
        }
    }

    pub fn get_room_data(&self, index: RoomID) -> RoomRef {
        let room = &self.rooms[index.0];
        RoomRef {
            blueprint: &self.blueprints[room.segment_id.0],
            data: room,
        }
    }

    pub fn get_texture(&self, id: TextureID) -> TextureDataRef {
        self.texture_manager.get_texture_data(id)
    }

    pub fn get_model(&self, id: ModelID) -> ModelDataRef {
        self.model_manager.get_model_data(id)
    }

    pub fn collect_dbg_data(&self) -> WorldDebugData {
        WorldDebugData {
            room_count: self.rooms.len() as u64,
        }
    }
}

#[derive(Debug)]
pub struct WorldDebugData {
    pub room_count: u64,
}
